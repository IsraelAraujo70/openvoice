use crate::modules::auth::application as auth_application;
use crate::modules::live_transcription::infrastructure::db;
use reqwest::blocking::Client;
use serde_json::Value;

/// Codex Responses endpoint — NOT the Cloudflare-blocked `/backend-api/conversation`.
/// Reference: badlogic/pi-mono `openai-codex-responses.ts`.
const CODEX_RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex/responses";

const TITLE_MODEL: &str = "gpt-5.1-codex-mini";

const SYSTEM_INSTRUCTIONS: &str =
    "Summarize this transcription in 1 line (max 80 chars), in the same language as the content. Return ONLY the summary line, nothing else.";

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Generate a 1-line title for a session using the ChatGPT Codex Responses API
/// (OAuth token, NOT the openai_realtime_api_key).
///
/// This is a blocking call meant to run inside `Task::perform`.
/// Returns `(session_id, title)` on success.
pub fn generate_session_title(session_id: i64) -> Result<(i64, String), String> {
    eprintln!("[openvoice][title] generating title for session_id={session_id}");

    let session = auth_application::load_or_refresh_session().map_err(|e| {
        eprintln!("[openvoice][title] auth failed: {e}");
        e
    })?;

    let segments = db::get_session_segments(session_id)?;
    if segments.is_empty() {
        eprintln!("[openvoice][title] session {session_id} has no segments, skipping");
        return Err(String::from("Sessao sem segmentos para gerar titulo."));
    }

    eprintln!(
        "[openvoice][title] session {session_id} has {} segments, building transcript",
        segments.len()
    );

    // Build the transcript text (limit to ~4000 chars to keep the request small)
    let transcript = build_truncated_transcript(&segments, 4000);

    let title = call_codex_responses(
        session.bearer_token(),
        session.account_id.as_deref(),
        &transcript,
    )?;

    // Persist the title to DB
    db::update_session_title(session_id, &title)?;

    eprintln!("[openvoice][title] session {session_id} title saved: {title}");
    Ok((session_id, title))
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn build_truncated_transcript(segments: &[String], max_chars: usize) -> String {
    let mut result = String::with_capacity(max_chars);
    for seg in segments {
        if result.len() + seg.len() + 1 > max_chars {
            let remaining = max_chars.saturating_sub(result.len() + 1);
            if remaining > 0 {
                if !result.is_empty() {
                    result.push(' ');
                }
                result.push_str(&seg[..seg.len().min(remaining)]);
            }
            break;
        }
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(seg);
    }
    result
}

/// Build the request body for the `/backend-api/codex/responses` endpoint.
///
/// Uses the OpenAI Responses API format (not the Chat Completions format).
/// Reference: badlogic/pi-mono `openai-codex-responses.ts` — `buildRequestBody()`.
fn build_codex_body(transcript: &str) -> Value {
    let user_message = format!("{SYSTEM_INSTRUCTIONS}\n\n---\n\n{transcript}");

    serde_json::json!({
        "model": TITLE_MODEL,
        "store": false,
        "stream": true,
        "instructions": SYSTEM_INSTRUCTIONS,
        "input": [{
            "role": "user",
            "content": user_message
        }]
    })
}

/// Call the ChatGPT Codex Responses endpoint (`/backend-api/codex/responses`).
///
/// The response is an SSE stream. We read line by line, looking for
/// `data: {json}` events. The text content arrives in events with type
/// `response.output_text.delta` (field `delta`) and the stream finishes
/// with `response.completed` / `response.done`.
fn call_codex_responses(
    bearer_token: &str,
    account_id: Option<&str>,
    transcript: &str,
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Erro ao criar HTTP client: {e}"))?;

    let body = build_codex_body(transcript);

    let acct_id = account_id
        .ok_or_else(|| String::from("Sem account_id no token OAuth. Faca login novamente."))?;

    let response = client
        .post(CODEX_RESPONSES_URL)
        .header("Authorization", format!("Bearer {bearer_token}"))
        .header("Content-Type", "application/json")
        .header("Accept", "text/event-stream")
        .header("chatgpt-account-id", acct_id)
        .header("OpenAI-Beta", "responses=experimental")
        .header("User-Agent", "openvoice (linux)")
        .json(&body)
        .send()
        .map_err(|e| {
            eprintln!("[openvoice][title] HTTP request failed: {e}");
            format!("Erro ao chamar Codex Responses: {e}")
        })?;

    let status = response.status();
    eprintln!("[openvoice][title] Codex Responses status: {status}");

    if !status.is_success() {
        let body_text = response.text().unwrap_or_default();
        // Truncate error body for logging (Cloudflare pages can be huge)
        let log_body = if body_text.len() > 500 {
            format!("{}...(truncated)", &body_text[..500])
        } else {
            body_text.clone()
        };
        eprintln!("[openvoice][title] error body: {log_body}");
        return Err(format!("Codex Responses retornou status {status}"));
    }

    // Read the full response body and parse the SSE stream
    let body_text = response
        .text()
        .map_err(|e| format!("Erro ao ler corpo da resposta SSE: {e}"))?;

    let title = parse_codex_sse(&body_text)?;

    if title.is_empty() {
        return Err(String::from(
            "Codex Responses retornou resposta vazia para o titulo.",
        ));
    }

    // Enforce max 80 chars
    let title = if title.len() > 80 {
        title[..80].to_string()
    } else {
        title
    };

    Ok(title)
}

/// Parse the SSE response body from `/backend-api/codex/responses`.
///
/// The Responses API streams events like:
/// - `response.output_text.delta` with `{ "delta": "partial text" }`
/// - `response.output_text.done` with `{ "text": "full text" }`
/// - `response.completed` / `response.done` marks the end
/// - `error` events
///
/// We accumulate `delta` values from `response.output_text.delta` events,
/// or take the full `text` from `response.output_text.done` if available.
fn parse_codex_sse(body: &str) -> Result<String, String> {
    let mut accumulated_text = String::new();
    let mut full_text: Option<String> = None;

    for line in body.lines() {
        let line = line.trim();

        if !line.starts_with("data:") {
            continue;
        }

        let data = line["data:".len()..].trim();

        if data == "[DONE]" {
            break;
        }

        // Try to parse as JSON
        let parsed: Value = match serde_json::from_str(data) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let event_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");

        // Check for error events
        if event_type == "error" || event_type == "response.failed" {
            let err_msg = parsed
                .pointer("/response/error/message")
                .or_else(|| parsed.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown error");
            return Err(format!("Codex SSE error: {err_msg}"));
        }

        // Accumulate text deltas
        if event_type == "response.output_text.delta" {
            if let Some(delta) = parsed.get("delta").and_then(|v| v.as_str()) {
                accumulated_text.push_str(delta);
            }
        }

        // Full text on done event (preferred if available)
        if event_type == "response.output_text.done" {
            if let Some(text) = parsed.get("text").and_then(|v| v.as_str()) {
                full_text = Some(text.trim().to_string());
            }
        }
    }

    // Prefer the full text from done event, fall back to accumulated deltas
    let result = full_text.unwrap_or(accumulated_text);
    Ok(result.trim().to_string())
}

use crate::modules::auth::application as auth_application;
use crate::modules::live_transcription::infrastructure::db;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

const CHATGPT_BACKEND_RESPONSES_URL: &str = "https://chatgpt.com/backend-api/v1/responses";

const TITLE_MODEL: &str = "gpt-4o-mini";

const SYSTEM_INSTRUCTIONS: &str =
    "Summarize this transcription in 1 line (max 80 chars), in the same language as the content. Return ONLY the summary line, nothing else.";

// ---------------------------------------------------------------------------
// Request / Response types (OpenAI Responses API)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct ResponsesRequest {
    model: String,
    instructions: String,
    input: String,
}

#[derive(Debug, Deserialize)]
struct ResponsesResponse {
    #[serde(default)]
    output: Vec<ResponsesOutput>,
    #[serde(default)]
    error: Option<ResponsesError>,
}

#[derive(Debug, Deserialize)]
struct ResponsesOutput {
    #[serde(default)]
    content: Vec<ResponsesContent>,
}

#[derive(Debug, Deserialize)]
struct ResponsesContent {
    #[serde(default)]
    text: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ResponsesError {
    message: String,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Generate a 1-line title for a session using the ChatGPT backend API
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

    let title = call_chatgpt_backend(
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

fn call_chatgpt_backend(
    bearer_token: &str,
    account_id: Option<&str>,
    transcript: &str,
) -> Result<String, String> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Erro ao criar HTTP client: {e}"))?;

    let body = ResponsesRequest {
        model: TITLE_MODEL.to_string(),
        instructions: SYSTEM_INSTRUCTIONS.to_string(),
        input: transcript.to_string(),
    };

    let mut request = client
        .post(CHATGPT_BACKEND_RESPONSES_URL)
        .header("Authorization", format!("Bearer {bearer_token}"))
        .header("User-Agent", "OpenVoice")
        .header("Accept", "application/json")
        .json(&body);

    if let Some(acct) = account_id {
        request = request.header("ChatGPT-Account-Id", acct);
    }

    let response = request.send().map_err(|e| {
        eprintln!("[openvoice][title] HTTP request failed: {e}");
        format!("Erro ao chamar ChatGPT backend: {e}")
    })?;

    let status = response.status();
    eprintln!("[openvoice][title] ChatGPT backend response status: {status}");
    if !status.is_success() {
        let body_text = response.text().unwrap_or_default();
        return Err(format!(
            "ChatGPT backend retornou status {status}: {body_text}"
        ));
    }

    let parsed: ResponsesResponse = response
        .json()
        .map_err(|e| format!("Erro ao parsear resposta do ChatGPT backend: {e}"))?;

    if let Some(err) = parsed.error {
        return Err(format!("ChatGPT backend error: {}", err.message));
    }

    // Extract the text from the first output content
    let title = parsed
        .output
        .iter()
        .flat_map(|o| o.content.iter())
        .filter_map(|c| c.text.as_deref())
        .next()
        .unwrap_or("")
        .trim()
        .to_string();

    if title.is_empty() {
        return Err(String::from(
            "ChatGPT backend retornou resposta vazia para o titulo.",
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

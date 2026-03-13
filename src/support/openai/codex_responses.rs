use reqwest::blocking::Client;
use serde_json::Value;

const CODEX_RESPONSES_URL: &str = "https://chatgpt.com/backend-api/codex/responses";
const OPENVOICE_USER_AGENT: &str = "openvoice (linux)";
const RESPONSES_BETA_HEADER: &str = "responses=experimental";
const REQUEST_TIMEOUT_SECS: u64 = 60;
const MAX_ERROR_BODY_CHARS: usize = 500;

pub struct CodexResponsesClient {
    http: Client,
}

pub struct CodexAuth<'a> {
    pub bearer_token: &'a str,
    pub account_id: &'a str,
}

pub struct CodexTextRequest<'a> {
    pub model: &'a str,
    pub instructions: &'a str,
    pub input: &'a str,
}

impl CodexResponsesClient {
    pub fn new() -> Result<Self, String> {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .map_err(|error| format!("Erro ao criar HTTP client: {error}"))?;

        Ok(Self { http })
    }

    pub fn generate_text(
        &self,
        auth: CodexAuth<'_>,
        request: CodexTextRequest<'_>,
    ) -> Result<String, String> {
        let body = build_request_body(&request);

        let response = self
            .http
            .post(CODEX_RESPONSES_URL)
            .header("Authorization", format!("Bearer {}", auth.bearer_token))
            .header("Content-Type", "application/json")
            .header("Accept", "text/event-stream")
            .header("chatgpt-account-id", auth.account_id)
            .header("OpenAI-Beta", RESPONSES_BETA_HEADER)
            .header("User-Agent", OPENVOICE_USER_AGENT)
            .json(&body)
            .send()
            .map_err(|error| format!("Erro ao chamar Codex Responses: {error}"))?;

        let status = response.status();

        if !status.is_success() {
            let body_text = response.text().unwrap_or_default();
            let log_body = truncate_for_log(&body_text, MAX_ERROR_BODY_CHARS);
            eprintln!("[openvoice][codex] status={status} body={log_body}");
            return Err(format!("Codex Responses retornou status {status}"));
        }

        let body_text = response
            .text()
            .map_err(|error| format!("Erro ao ler corpo da resposta SSE: {error}"))?;

        parse_sse_text(&body_text)
    }
}

fn build_request_body(request: &CodexTextRequest<'_>) -> Value {
    serde_json::json!({
        "model": request.model,
        "store": false,
        "stream": true,
        "instructions": request.instructions,
        "input": [{
            "role": "user",
            "content": request.input
        }]
    })
}

fn parse_sse_text(body: &str) -> Result<String, String> {
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

        let parsed: Value = match serde_json::from_str(data) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let event_type = parsed
            .get("type")
            .and_then(|value| value.as_str())
            .unwrap_or("");

        if event_type == "error" || event_type == "response.failed" {
            let error_message = parsed
                .pointer("/response/error/message")
                .or_else(|| parsed.get("message"))
                .and_then(|value| value.as_str())
                .unwrap_or("unknown error");
            return Err(format!("Codex SSE error: {error_message}"));
        }

        if event_type == "response.output_text.delta" {
            if let Some(delta) = parsed.get("delta").and_then(|value| value.as_str()) {
                accumulated_text.push_str(delta);
            }
        }

        if event_type == "response.output_text.done" {
            if let Some(text) = parsed.get("text").and_then(|value| value.as_str()) {
                full_text = Some(text.trim().to_string());
            }
        }
    }

    let text = full_text.unwrap_or(accumulated_text);
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Err(String::from("Codex Responses retornou texto vazio."));
    }

    Ok(trimmed.to_string())
}

fn truncate_for_log(value: &str, max_chars: usize) -> String {
    let total_chars = value.chars().count();

    if total_chars <= max_chars {
        return value.to_string();
    }

    let truncated: String = value.chars().take(max_chars).collect();
    format!("{truncated}...(truncated)")
}

#[cfg(test)]
mod tests {
    use super::parse_sse_text;

    #[test]
    fn prefers_done_text_over_deltas() {
        let body = r#"
data: {"type":"response.output_text.delta","delta":"Parcial "}
data: {"type":"response.output_text.done","text":"Titulo final"}
data: [DONE]
"#;

        let result = parse_sse_text(body).expect("text");

        assert_eq!(result, "Titulo final");
    }

    #[test]
    fn accumulates_delta_when_done_event_is_missing() {
        let body = r#"
data: {"type":"response.output_text.delta","delta":"Ola "}
data: {"type":"response.output_text.delta","delta":"mundo"}
data: [DONE]
"#;

        let result = parse_sse_text(body).expect("text");

        assert_eq!(result, "Ola mundo");
    }

    #[test]
    fn surfaces_sse_errors() {
        let body = r#"
data: {"type":"response.failed","response":{"error":{"message":"quota"}}}
data: [DONE]
"#;

        let error = parse_sse_text(body).expect_err("error");

        assert!(error.contains("quota"));
    }
}

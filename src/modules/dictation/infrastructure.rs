use crate::modules::{
    audio::domain::CaptureSession,
    dictation::domain::{DictationConfig, DualTranscriptOutput},
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "input_audio")]
    InputAudio { input_audio: InputAudio },
}

#[derive(Debug, Serialize)]
struct InputAudio {
    data: String,
    format: String,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
    #[serde(default)]
    error: Option<ApiError>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Debug, Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ApiError {
    message: String,
}

pub fn transcribe(config: &DictationConfig, wav_base64: &str) -> Result<String, String> {
    let client = Client::new();
    let request = ChatRequest {
        model: config.model.clone(),
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: vec![
                ContentPart::Text {
                    text: config.prompt.clone(),
                },
                ContentPart::InputAudio {
                    input_audio: InputAudio {
                        data: wav_base64.to_owned(),
                        format: String::from("wav"),
                    },
                },
            ],
        }],
    };

    let response = client
        .post(OPENROUTER_API_URL)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .header("Content-Type", "application/json")
        .header("HTTP-Referer", &config.referer)
        .header("X-Title", &config.app_title)
        .json(&request)
        .send()
        .map_err(|error| format!("Falha ao chamar OpenRouter: {error}"))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|error| format!("Falha ao ler resposta do OpenRouter: {error}"))?;

    if !status.is_success() {
        if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(message) = error_response
                .get("error")
                .and_then(|error| error.get("message"))
                .and_then(serde_json::Value::as_str)
            {
                return Err(format!("OpenRouter retornou {}: {}", status, message));
            }
        }

        return Err(format!("OpenRouter retornou {}: {}", status, body));
    }

    let chat_response: ChatResponse = serde_json::from_str(&body)
        .map_err(|error| format!("Falha ao interpretar resposta do OpenRouter: {error}"))?;

    if let Some(error) = chat_response.error {
        return Err(format!("OpenRouter retornou erro: {}", error.message));
    }

    chat_response
        .choices
        .first()
        .map(|choice| choice.message.content.trim().to_owned())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| String::from("OpenRouter nao retornou transcricao."))
}

pub fn save_transcripts(
    session: &CaptureSession,
    output: &DualTranscriptOutput,
) -> Result<PathBuf, String> {
    let path = session.artifacts.session_dir.join("transcripts.json");
    let contents = serde_json::to_string_pretty(output)
        .map_err(|error| format!("Falha ao serializar transcricoes: {error}"))?;

    fs::write(&path, contents).map_err(|error| {
        format!(
            "Falha ao salvar transcricoes em {}: {error}",
            path.display()
        )
    })?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::{ChatMessage, ChatRequest, ContentPart, InputAudio};

    #[test]
    fn serializes_input_audio_request() {
        let request = ChatRequest {
            model: String::from("google/gemini-2.5-flash-lite:nitro"),
            messages: vec![ChatMessage {
                role: String::from("user"),
                content: vec![
                    ContentPart::Text {
                        text: String::from("Transcribe this audio"),
                    },
                    ContentPart::InputAudio {
                        input_audio: InputAudio {
                            data: String::from("base64"),
                            format: String::from("wav"),
                        },
                    },
                ],
            }],
        };

        let json = serde_json::to_string(&request).expect("json");

        assert!(json.contains("input_audio"));
        assert!(json.contains("google/gemini-2.5-flash-lite:nitro"));
    }
}

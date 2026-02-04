//! Transcription module using OpenRouter API
//! Sends audio to the API and receives transcription

use serde::{Deserialize, Serialize};
use reqwest;

const OPENROUTER_API_URL: &str = "https://openrouter.ai/api/v1/chat/completions";
const DEFAULT_MODEL: &str = "google/gemini-2.5-flash";

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: Vec<Content>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum Content {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "input_audio")]
    InputAudio { input_audio: AudioData },
}

#[derive(Debug, Serialize)]
struct AudioData {
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
    #[serde(default)]
    code: Option<String>,
}

/// Transcription client
#[derive(Clone)]
pub struct TranscriptionClient {
    client: reqwest::Client,
}

impl TranscriptionClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Transcribe audio using OpenRouter API
    /// 
    /// # Arguments
    /// * `audio_base64` - Base64 encoded WAV audio data
    /// * `api_key` - OpenRouter API key
    /// * `model` - Model to use (optional, defaults to gemini-2.5-flash)
    pub async fn transcribe(
        &self,
        audio_base64: &str,
        api_key: &str,
        model: Option<&str>,
    ) -> Result<String, String> {
        let model = model.unwrap_or(DEFAULT_MODEL);

        log::info!("Transcribing audio with model: {}", model);

        let request = ChatRequest {
            model: model.to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![
                    Content::Text {
                        text: "Transcribe this audio exactly as spoken. Output only the transcription, nothing else. If the audio is in Portuguese, transcribe in Portuguese. If in English, transcribe in English. Preserve the original language.".to_string(),
                    },
                    Content::InputAudio {
                        input_audio: AudioData {
                            data: audio_base64.to_string(),
                            format: "wav".to_string(),
                        },
                    },
                ],
            }],
        };

        let response = self
            .client
            .post(OPENROUTER_API_URL)
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .header("HTTP-Referer", "https://github.com/openvoice/openvoice")
            .header("X-Title", "OpenVoice")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response body: {}", e))?;

        log::debug!("API response status: {}, body length: {}", status, body.len());

        if !status.is_success() {
            // Try to parse error response
            if let Ok(error_response) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(error) = error_response.get("error") {
                    let message = error
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(format!("API error ({}): {}", status, message));
                }
            }
            return Err(format!("API request failed with status {}: {}", status, body));
        }

        let chat_response: ChatResponse = serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, body))?;

        if let Some(error) = chat_response.error {
            return Err(format!("API error: {}", error.message));
        }

        let transcription = chat_response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "No transcription in response".to_string())?;

        log::info!("Transcription received: {} chars", transcription.len());

        Ok(transcription.trim().to_string())
    }
}

impl Default for TranscriptionClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_serialization() {
        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: vec![
                    Content::Text {
                        text: "Transcribe this".to_string(),
                    },
                    Content::InputAudio {
                        input_audio: AudioData {
                            data: "base64data".to_string(),
                            format: "wav".to_string(),
                        },
                    },
                ],
            }],
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("input_audio"));
        assert!(json.contains("test-model"));
    }
}

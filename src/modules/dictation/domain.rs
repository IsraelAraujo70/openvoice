use crate::modules::audio::domain::CaptureSession;
use serde::{Deserialize, Serialize};

pub const TARGET_SAMPLE_RATE: u32 = 16_000;
const DEFAULT_REFERER: &str = "https://github.com/IsraelAraujo70/openvoice";
const DEFAULT_APP_TITLE: &str = "OpenVoice";

#[derive(Debug, Clone)]
pub struct PreparedAudio {
    pub wav_base64: String,
}

#[derive(Debug, Clone)]
pub struct DictationConfig {
    pub api_key: String,
    pub model: String,
    pub referer: String,
    pub app_title: String,
    pub prompt: String,
}

impl DictationConfig {
    pub fn from_settings(
        settings: &crate::modules::settings::domain::AppSettings,
    ) -> Result<Self, String> {
        if !settings.has_api_key() {
            return Err(String::from(
                "Cadastre uma OpenRouter API key antes de tentar gravar.",
            ));
        }

        Ok(Self {
            api_key: settings.openrouter_api_key.clone(),
            model: settings.openrouter_model.clone(),
            referer: String::from(DEFAULT_REFERER),
            app_title: String::from(DEFAULT_APP_TITLE),
            prompt: String::from(
                "Transcribe this audio exactly as spoken. Output only the transcription, nothing else. Preserve the original language and do not add formatting or commentary.",
            ),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualTranscriptOutput {
    pub session_id: String,
    pub mic_transcript: Option<String>,
    pub system_transcript: Option<String>,
    pub mic_error: Option<String>,
    pub system_error: Option<String>,
    pub duration_seconds: f32,
}

impl DualTranscriptOutput {
    pub fn preview(&self) -> String {
        let preview = self
            .system_transcript
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or(self.mic_transcript.as_deref())
            .unwrap_or("")
            .trim();

        if preview.chars().count() <= 160 {
            return preview.to_owned();
        }

        let mut shortened = preview.chars().take(157).collect::<String>();
        shortened.push_str("...");
        shortened
    }

    pub fn clipboard_text(&self) -> String {
        let mut parts = Vec::new();

        parts.push(format!("Session: {}", self.session_id));

        if let Some(system) = self.system_transcript.as_deref() {
            parts.push(format!("System audio\n{}", system.trim()));
        }

        if let Some(mic) = self.mic_transcript.as_deref() {
            parts.push(format!("My voice\n{}", mic.trim()));
        }

        if let Some(error) = self.system_error.as_deref() {
            parts.push(format!("System audio error\n{error}"));
        }

        if let Some(error) = self.mic_error.as_deref() {
            parts.push(format!("Microphone error\n{error}"));
        }

        parts.join("\n\n")
    }

    pub fn status_hint(&self) -> String {
        match (&self.mic_error, &self.system_error) {
            (None, None) => format!(
                "Duas trilhas transcritas para a sessao {}.",
                self.session_id
            ),
            (Some(_), None) => format!(
                "System audio transcrito; houve falha na trilha do microfone ({})",
                self.session_id
            ),
            (None, Some(_)) => format!(
                "Microfone transcrito; houve falha na trilha do system audio ({})",
                self.session_id
            ),
            (Some(_), Some(_)) => {
                format!("Falha ao transcrever as duas trilhas ({})", self.session_id)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TranscriptionJob {
    pub session: CaptureSession,
}

impl TranscriptionJob {
    pub fn new(session: CaptureSession) -> Self {
        Self { session }
    }
}

#[cfg(test)]
mod tests {
    use super::DualTranscriptOutput;

    #[test]
    fn builds_clipboard_text_with_both_tracks() {
        let output = DualTranscriptOutput {
            session_id: String::from("session-1"),
            mic_transcript: Some(String::from("local note")),
            system_transcript: Some(String::from("meeting note")),
            mic_error: None,
            system_error: None,
            duration_seconds: 12.0,
        };

        let clipboard = output.clipboard_text();

        assert!(clipboard.contains("Session: session-1"));
        assert!(clipboard.contains("System audio"));
        assert!(clipboard.contains("My voice"));
    }

    #[test]
    fn preview_prefers_system_track() {
        let output = DualTranscriptOutput {
            session_id: String::from("session-1"),
            mic_transcript: Some(String::from("local note")),
            system_transcript: Some(String::from("meeting note")),
            mic_error: None,
            system_error: None,
            duration_seconds: 12.0,
        };

        assert_eq!(output.preview(), "meeting note");
    }
}

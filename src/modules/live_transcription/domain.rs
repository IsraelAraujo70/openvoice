#![allow(dead_code)]

use crate::modules::auth::domain::OpenAiCredentials;

pub const DEFAULT_TRANSCRIPTION_MODEL: &str = "gpt-4o-mini-transcribe";

#[derive(Debug, Clone)]
pub struct LiveTranscriptionConfig {
    pub credentials: OpenAiCredentials,
    pub model: String,
    pub prompt: Option<String>,
    pub language: Option<String>,
}

impl LiveTranscriptionConfig {
    pub fn bearer_token(&self) -> &str {
        self.credentials.bearer_token()
    }
}

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    Connected,
    TranscriptDelta { item_id: String, delta: String },
    TranscriptCompleted { item_id: String, transcript: String },
    Warning(String),
    Error(String),
    Stopped,
}

#![allow(dead_code)]

use crate::modules::auth::domain::OpenAiOAuthSession;

pub const DEFAULT_TRANSCRIPTION_MODEL: &str = "gpt-4o-mini-transcribe";

#[derive(Debug, Clone)]
pub struct LiveTranscriptionConfig {
    pub session: OpenAiOAuthSession,
    pub model: String,
    pub prompt: Option<String>,
    pub language: Option<String>,
}

impl LiveTranscriptionConfig {
    pub fn bearer_token(&self) -> &str {
        self.session.bearer_token()
    }

    pub fn account_id(&self) -> Option<&str> {
        self.session.account_id.as_deref()
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

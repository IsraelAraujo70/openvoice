#![allow(dead_code)]

use crate::modules::auth::application as auth_application;
use crate::modules::auth::domain::OpenAiOAuthSession;
use crate::modules::live_transcription::domain::{LiveTranscriptionConfig, RuntimeEvent};
use crate::modules::live_transcription::infrastructure::openai_realtime::{
    self, SessionHandle, SharedReceiver,
};
use crate::modules::settings::domain::AppSettings;

pub struct ActiveLiveTranscription {
    session: SessionHandle,
}

impl ActiveLiveTranscription {
    pub fn receiver(&self) -> SharedReceiver {
        self.session.receiver()
    }

    pub fn stop(self) {
        self.session.stop();
    }
}

pub fn load_runtime_session() -> Result<OpenAiOAuthSession, String> {
    auth_application::load_or_refresh_session()
}

pub fn start_live_transcription(settings: &AppSettings) -> Result<ActiveLiveTranscription, String> {
    let session = load_runtime_session()?;
    let config = LiveTranscriptionConfig {
        session,
        model: settings.openai_realtime_model.clone(),
        prompt: None,
        language: None,
    };

    let session = openai_realtime::start_session(config)?;
    Ok(ActiveLiveTranscription { session })
}

pub fn poll_next_event(receiver: SharedReceiver) -> Option<RuntimeEvent> {
    receiver.lock().ok()?.recv().ok()
}

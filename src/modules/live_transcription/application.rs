#![allow(dead_code)]

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

pub fn start_live_transcription(settings: &AppSettings) -> Result<ActiveLiveTranscription, String> {
    let bearer_token = settings.openai_realtime_api_key.trim();
    if bearer_token.is_empty() {
        return Err(String::from(
            "Cadastre e salve uma OpenAI API key antes de iniciar a transcription realtime.",
        ));
    }

    let config = LiveTranscriptionConfig {
        bearer_token: bearer_token.to_owned(),
        model: settings.openai_realtime_model.clone(),
        prompt: None,
        language: (!settings.openai_realtime_language.trim().is_empty())
            .then(|| settings.openai_realtime_language.clone()),
    };

    let session = openai_realtime::start_session(config)?;
    Ok(ActiveLiveTranscription { session })
}

pub fn poll_next_event(receiver: SharedReceiver) -> Option<RuntimeEvent> {
    receiver.lock().ok()?.recv().ok()
}

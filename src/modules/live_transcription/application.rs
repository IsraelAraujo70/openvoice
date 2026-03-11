#![allow(dead_code)]

use crate::modules::auth::application as auth_application;
use crate::modules::auth::domain::OpenAiCredentials;
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

pub fn load_runtime_credentials() -> Result<OpenAiCredentials, String> {
    auth_application::load_credentials()?
        .map(|stored| stored.credentials)
        .ok_or_else(|| {
            String::from(
                "Nao encontrei credenciais OpenAI. Use OPENAI_API_KEY por enquanto ou salve auth do OpenVoice antes de iniciar o realtime.",
            )
        })
}

pub fn start_live_transcription(settings: &AppSettings) -> Result<ActiveLiveTranscription, String> {
    let credentials = load_runtime_credentials()?;
    let config = LiveTranscriptionConfig {
        credentials,
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

use crate::modules::auth::domain::OpenAiCredentials;
use serde::{Deserialize, Serialize};

pub const DEFAULT_OPENROUTER_MODEL: &str = "google/gemini-2.5-flash-lite:nitro";
pub const DEFAULT_OPENAI_REALTIME_MODEL: &str = "gpt-4o-mini-transcribe";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openai_realtime_model: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openrouter_api_key: String::new(),
            openrouter_model: String::from(DEFAULT_OPENROUTER_MODEL),
            openai_realtime_model: String::from(DEFAULT_OPENAI_REALTIME_MODEL),
        }
    }
}

impl AppSettings {
    pub fn new(
        openrouter_api_key: String,
        openrouter_model: String,
        openai_realtime_model: String,
    ) -> Result<Self, String> {
        if openrouter_api_key.trim().is_empty() {
            return Err(String::from("A OpenRouter API key nao pode ficar vazia."));
        }

        let openrouter_model = if openrouter_model.trim().is_empty() {
            String::from(DEFAULT_OPENROUTER_MODEL)
        } else {
            openrouter_model.trim().to_owned()
        };

        let openai_realtime_model = if openai_realtime_model.trim().is_empty() {
            String::from(DEFAULT_OPENAI_REALTIME_MODEL)
        } else {
            openai_realtime_model.trim().to_owned()
        };

        Ok(Self {
            openrouter_api_key: openrouter_api_key.trim().to_owned(),
            openrouter_model,
            openai_realtime_model,
        })
    }

    pub fn has_api_key(&self) -> bool {
        !self.openrouter_api_key.trim().is_empty()
    }
}

#[derive(Debug, Clone)]
pub struct SettingsForm {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
    pub openai_api_key: String,
    pub openai_realtime_model: String,
}

impl From<&AppSettings> for SettingsForm {
    fn from(settings: &AppSettings) -> Self {
        Self {
            openrouter_api_key: settings.openrouter_api_key.clone(),
            openrouter_model: settings.openrouter_model.clone(),
            openai_api_key: String::new(),
            openai_realtime_model: settings.openai_realtime_model.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SaveSettingsResult {
    pub settings: AppSettings,
    pub has_openai_credentials: bool,
    pub openai_credential_kind: Option<String>,
    pub openai_api_key_for_form: String,
}

pub fn form_openai_api_key(credentials: Option<&OpenAiCredentials>) -> String {
    match credentials {
        Some(OpenAiCredentials::ApiKey { api_key }) => api_key.clone(),
        _ => String::new(),
    }
}

pub fn credential_kind_label(credentials: Option<&OpenAiCredentials>) -> Option<String> {
    credentials.map(|credentials| match credentials {
        OpenAiCredentials::ApiKey { .. } => String::from("api_key"),
        OpenAiCredentials::OAuth { .. } => String::from("oauth"),
    })
}

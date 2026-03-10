use serde::{Deserialize, Serialize};

pub const DEFAULT_OPENROUTER_MODEL: &str = "google/gemini-2.5-flash-lite:nitro";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub openrouter_api_key: String,
    pub openrouter_model: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openrouter_api_key: String::new(),
            openrouter_model: String::from(DEFAULT_OPENROUTER_MODEL),
        }
    }
}

impl AppSettings {
    pub fn new(openrouter_api_key: String, openrouter_model: String) -> Result<Self, String> {
        if openrouter_api_key.trim().is_empty() {
            return Err(String::from("A OpenRouter API key nao pode ficar vazia."));
        }

        let model = if openrouter_model.trim().is_empty() {
            String::from(DEFAULT_OPENROUTER_MODEL)
        } else {
            openrouter_model.trim().to_owned()
        };

        Ok(Self {
            openrouter_api_key: openrouter_api_key.trim().to_owned(),
            openrouter_model: model,
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
}

impl From<&AppSettings> for SettingsForm {
    fn from(settings: &AppSettings) -> Self {
        Self {
            openrouter_api_key: settings.openrouter_api_key.clone(),
            openrouter_model: settings.openrouter_model.clone(),
        }
    }
}

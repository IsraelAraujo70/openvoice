use serde::{Deserialize, Serialize};

pub const DEFAULT_OPENROUTER_MODEL: &str = "google/gemini-2.5-flash-lite:nitro";
pub const DEFAULT_OPENAI_REALTIME_MODEL: &str = "gpt-4o-transcribe";
pub const DEFAULT_OPENAI_REALTIME_LANGUAGE: &str = "";
pub const SUPPORTED_OPENAI_REALTIME_MODELS: &[&str] = &[
    "whisper-1",
    "gpt-4o-transcribe",
    "gpt-4o-mini-transcribe",
    "gpt-4o-mini-transcribe-2025-03-20",
    "gpt-4o-mini-transcribe-2025-12-15",
];
pub const SUPPORTED_OPENAI_REALTIME_LANGUAGES: &[&str] =
    &["", "pt", "en", "de", "es", "fr", "it", "ja"];

fn default_openrouter_model() -> String {
    String::from(DEFAULT_OPENROUTER_MODEL)
}

fn default_openai_realtime_model() -> String {
    String::from(DEFAULT_OPENAI_REALTIME_MODEL)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    #[serde(default)]
    pub openrouter_api_key: String,
    #[serde(default)]
    pub openai_realtime_api_key: String,
    #[serde(default = "default_openrouter_model")]
    pub openrouter_model: String,
    #[serde(default = "default_openai_realtime_model")]
    pub openai_realtime_model: String,
    #[serde(default)]
    pub openai_realtime_language: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openrouter_api_key: String::new(),
            openai_realtime_api_key: String::new(),
            openrouter_model: String::from(DEFAULT_OPENROUTER_MODEL),
            openai_realtime_model: String::from(DEFAULT_OPENAI_REALTIME_MODEL),
            openai_realtime_language: String::from(DEFAULT_OPENAI_REALTIME_LANGUAGE),
        }
    }
}

impl AppSettings {
    pub fn new(
        openrouter_api_key: String,
        openai_realtime_api_key: String,
        openrouter_model: String,
        openai_realtime_model: String,
        openai_realtime_language: String,
    ) -> Result<Self, String> {
        if openrouter_api_key.trim().is_empty() {
            return Err(String::from("A OpenRouter API key nao pode ficar vazia."));
        }

        let openrouter_model = if openrouter_model.trim().is_empty() {
            String::from(DEFAULT_OPENROUTER_MODEL)
        } else {
            openrouter_model.trim().to_owned()
        };

        let openai_realtime_model = normalize_openai_realtime_model(&openai_realtime_model);
        let openai_realtime_language =
            normalize_openai_realtime_language(&openai_realtime_language);

        Ok(Self {
            openrouter_api_key: openrouter_api_key.trim().to_owned(),
            openai_realtime_api_key: openai_realtime_api_key.trim().to_owned(),
            openrouter_model,
            openai_realtime_model,
            openai_realtime_language,
        })
    }

    pub fn has_api_key(&self) -> bool {
        !self.openrouter_api_key.trim().is_empty()
    }

    pub fn has_openai_realtime_api_key(&self) -> bool {
        !self.openai_realtime_api_key.trim().is_empty()
    }

    pub fn normalized(mut self) -> Self {
        self.openai_realtime_model = normalize_openai_realtime_model(&self.openai_realtime_model);
        self.openai_realtime_language =
            normalize_openai_realtime_language(&self.openai_realtime_language);
        self
    }
}

#[derive(Debug, Clone)]
pub struct SettingsForm {
    pub openrouter_api_key: String,
    pub openai_realtime_api_key: String,
    pub openrouter_model: String,
    pub openai_realtime_model: String,
    pub openai_realtime_language: String,
}

impl From<&AppSettings> for SettingsForm {
    fn from(settings: &AppSettings) -> Self {
        Self {
            openrouter_api_key: settings.openrouter_api_key.clone(),
            openai_realtime_api_key: settings.openai_realtime_api_key.clone(),
            openrouter_model: settings.openrouter_model.clone(),
            openai_realtime_model: settings.openai_realtime_model.clone(),
            openai_realtime_language: settings.openai_realtime_language.clone(),
        }
    }
}

fn normalize_openai_realtime_model(value: &str) -> String {
    let trimmed = value.trim();

    if SUPPORTED_OPENAI_REALTIME_MODELS.contains(&trimmed) {
        trimmed.to_owned()
    } else {
        String::from(DEFAULT_OPENAI_REALTIME_MODEL)
    }
}

fn normalize_openai_realtime_language(value: &str) -> String {
    let trimmed = value.trim();

    if SUPPORTED_OPENAI_REALTIME_LANGUAGES.contains(&trimmed) {
        trimmed.to_owned()
    } else {
        String::from(DEFAULT_OPENAI_REALTIME_LANGUAGE)
    }
}

use serde::{Deserialize, Serialize};

use crate::modules::copilot::domain::CopilotMode;

pub const DEFAULT_OPENROUTER_MODEL: &str = "google/gemini-2.5-flash-lite:nitro";
pub const DEFAULT_OPENAI_REALTIME_MODEL: &str = "gpt-4o-transcribe";
pub const DEFAULT_OPENAI_REALTIME_LANGUAGE: &str = "";
pub const DEFAULT_OPENAI_REALTIME_PROFILE: &str = "balanced";
pub const DEFAULT_COPILOT_MODEL: &str = "gpt-5.1-codex-mini";
pub const DEFAULT_COPILOT_MODE: &str = "general";
pub const DEFAULT_COPILOT_AUTO_INCLUDE_TRANSCRIPT: bool = true;
pub const DEFAULT_COPILOT_SAVE_HISTORY: bool = true;
pub const SUPPORTED_OPENAI_REALTIME_MODELS: &[&str] = &[
    "whisper-1",
    "gpt-4o-transcribe",
    "gpt-4o-mini-transcribe",
    "gpt-4o-mini-transcribe-2025-03-20",
    "gpt-4o-mini-transcribe-2025-12-15",
];
pub const SUPPORTED_OPENAI_REALTIME_LANGUAGES: &[&str] =
    &["", "pt", "en", "de", "es", "fr", "it", "ja"];
pub const SUPPORTED_OPENAI_REALTIME_PROFILES: &[&str] = &["caption", "balanced", "accuracy"];

fn default_openrouter_model() -> String {
    String::from(DEFAULT_OPENROUTER_MODEL)
}

fn default_openai_realtime_model() -> String {
    String::from(DEFAULT_OPENAI_REALTIME_MODEL)
}

fn default_copilot_model() -> String {
    String::from(DEFAULT_COPILOT_MODEL)
}

fn default_copilot_mode() -> String {
    String::from(DEFAULT_COPILOT_MODE)
}

fn default_copilot_auto_include_transcript() -> bool {
    DEFAULT_COPILOT_AUTO_INCLUDE_TRANSCRIPT
}

fn default_copilot_save_history() -> bool {
    DEFAULT_COPILOT_SAVE_HISTORY
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
    #[serde(default = "default_openai_realtime_profile")]
    pub openai_realtime_profile: String,
    #[serde(default = "default_copilot_model")]
    pub copilot_model: String,
    #[serde(default = "default_copilot_mode")]
    pub copilot_default_mode: String,
    #[serde(default = "default_copilot_auto_include_transcript")]
    pub copilot_auto_include_transcript: bool,
    #[serde(default = "default_copilot_save_history")]
    pub copilot_save_history: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            openrouter_api_key: String::new(),
            openai_realtime_api_key: String::new(),
            openrouter_model: String::from(DEFAULT_OPENROUTER_MODEL),
            openai_realtime_model: String::from(DEFAULT_OPENAI_REALTIME_MODEL),
            openai_realtime_language: String::from(DEFAULT_OPENAI_REALTIME_LANGUAGE),
            openai_realtime_profile: String::from(DEFAULT_OPENAI_REALTIME_PROFILE),
            copilot_model: String::from(DEFAULT_COPILOT_MODEL),
            copilot_default_mode: String::from(DEFAULT_COPILOT_MODE),
            copilot_auto_include_transcript: DEFAULT_COPILOT_AUTO_INCLUDE_TRANSCRIPT,
            copilot_save_history: DEFAULT_COPILOT_SAVE_HISTORY,
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
        openai_realtime_profile: String,
        copilot_model: String,
        copilot_default_mode: String,
        copilot_auto_include_transcript: bool,
        copilot_save_history: bool,
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
        let openai_realtime_profile = normalize_openai_realtime_profile(&openai_realtime_profile);
        let copilot_model = normalize_copilot_model(&copilot_model);
        let copilot_default_mode = normalize_copilot_mode(&copilot_default_mode);

        Ok(Self {
            openrouter_api_key: openrouter_api_key.trim().to_owned(),
            openai_realtime_api_key: openai_realtime_api_key.trim().to_owned(),
            openrouter_model,
            openai_realtime_model,
            openai_realtime_language,
            openai_realtime_profile,
            copilot_model,
            copilot_default_mode,
            copilot_auto_include_transcript,
            copilot_save_history,
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
        self.openai_realtime_profile =
            normalize_openai_realtime_profile(&self.openai_realtime_profile);
        self.copilot_model = normalize_copilot_model(&self.copilot_model);
        self.copilot_default_mode = normalize_copilot_mode(&self.copilot_default_mode);
        self
    }

    pub fn copilot_default_mode(&self) -> CopilotMode {
        CopilotMode::from_code(&self.copilot_default_mode)
    }
}

#[derive(Debug, Clone)]
pub struct SettingsForm {
    pub openrouter_api_key: String,
    pub openai_realtime_api_key: String,
    pub openrouter_model: String,
    pub openai_realtime_model: String,
    pub openai_realtime_language: String,
    pub openai_realtime_profile: String,
    pub copilot_model: String,
    pub copilot_default_mode: String,
    pub copilot_auto_include_transcript: bool,
    pub copilot_save_history: bool,
}

impl From<&AppSettings> for SettingsForm {
    fn from(settings: &AppSettings) -> Self {
        Self {
            openrouter_api_key: settings.openrouter_api_key.clone(),
            openai_realtime_api_key: settings.openai_realtime_api_key.clone(),
            openrouter_model: settings.openrouter_model.clone(),
            openai_realtime_model: settings.openai_realtime_model.clone(),
            openai_realtime_language: settings.openai_realtime_language.clone(),
            openai_realtime_profile: settings.openai_realtime_profile.clone(),
            copilot_model: settings.copilot_model.clone(),
            copilot_default_mode: settings.copilot_default_mode.clone(),
            copilot_auto_include_transcript: settings.copilot_auto_include_transcript,
            copilot_save_history: settings.copilot_save_history,
        }
    }
}

fn default_openai_realtime_profile() -> String {
    String::from(DEFAULT_OPENAI_REALTIME_PROFILE)
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

fn normalize_openai_realtime_profile(value: &str) -> String {
    let trimmed = value.trim();

    if SUPPORTED_OPENAI_REALTIME_PROFILES.contains(&trimmed) {
        trimmed.to_owned()
    } else {
        String::from(DEFAULT_OPENAI_REALTIME_PROFILE)
    }
}

fn normalize_copilot_model(value: &str) -> String {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        String::from(DEFAULT_COPILOT_MODEL)
    } else {
        trimmed.to_owned()
    }
}

fn normalize_copilot_mode(value: &str) -> String {
    CopilotMode::from_code(value).code().to_owned()
}

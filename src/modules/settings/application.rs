use crate::modules::settings::domain::AppSettings;
use crate::modules::settings::infrastructure;

pub fn load_settings() -> Result<AppSettings, String> {
    infrastructure::load_settings()
}

pub fn save_settings(
    openrouter_api_key: String,
    openai_realtime_api_key: String,
    openrouter_model: String,
    openai_realtime_model: String,
    openai_realtime_language: String,
) -> Result<AppSettings, String> {
    let settings = AppSettings::new(
        openrouter_api_key,
        openai_realtime_api_key,
        openrouter_model,
        openai_realtime_model,
        openai_realtime_language,
    )?;
    infrastructure::save_settings(&settings)?;
    Ok(settings)
}

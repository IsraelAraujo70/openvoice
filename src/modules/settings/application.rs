use crate::modules::settings::domain::AppSettings;
use crate::modules::settings::infrastructure;

pub fn load_settings() -> Result<AppSettings, String> {
    infrastructure::load_settings()
}

pub fn save_settings(api_key: String, model: String) -> Result<AppSettings, String> {
    let settings = AppSettings::new(api_key, model)?;
    infrastructure::save_settings(&settings)?;
    Ok(settings)
}

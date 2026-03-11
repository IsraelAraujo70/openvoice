use crate::modules::auth::application as auth_application;
use crate::modules::auth::domain::CredentialStoreStrategy;
use crate::modules::settings::domain::{
    credential_kind_label, form_openai_api_key, AppSettings, SaveSettingsResult,
};
use crate::modules::settings::infrastructure;

pub fn load_settings() -> Result<AppSettings, String> {
    infrastructure::load_settings()
}

pub fn save_settings(
    openrouter_api_key: String,
    openrouter_model: String,
    openai_api_key: String,
    openai_realtime_model: String,
) -> Result<SaveSettingsResult, String> {
    let settings = AppSettings::new(openrouter_api_key, openrouter_model, openai_realtime_model)?;
    infrastructure::save_settings(&settings)?;

    if !openai_api_key.trim().is_empty() {
        auth_application::save_api_key(openai_api_key, CredentialStoreStrategy::Auto)?;
    }

    let stored = auth_application::load_credentials()?;

    Ok(SaveSettingsResult {
        settings,
        has_openai_credentials: stored.is_some(),
        openai_credential_kind: credential_kind_label(
            stored.as_ref().map(|value| &value.credentials),
        ),
        openai_api_key_for_form: form_openai_api_key(
            stored.as_ref().map(|value| &value.credentials),
        ),
    })
}

#![allow(dead_code)]

use crate::modules::auth::domain::{
    CredentialStoreStrategy, OpenAiAuthSnapshot, OpenAiOAuthSession, PendingOpenAiOAuthFlow,
    StoredOpenAiCredentials,
};
use crate::modules::auth::infrastructure;

pub fn load_credentials() -> Result<Option<StoredOpenAiCredentials>, String> {
    infrastructure::load_credentials()
}

pub fn load_auth_snapshot() -> Result<OpenAiAuthSnapshot, String> {
    let stored = load_credentials()?;

    Ok(stored
        .as_ref()
        .map(|stored| OpenAiAuthSnapshot::from_session(&stored.session))
        .unwrap_or_else(OpenAiAuthSnapshot::signed_out))
}

pub fn login_with_chatgpt(strategy: CredentialStoreStrategy) -> Result<OpenAiAuthSnapshot, String> {
    let stored = infrastructure::login_with_chatgpt(strategy)?;
    Ok(OpenAiAuthSnapshot::from_session(&stored.session))
}

pub fn start_login(strategy: CredentialStoreStrategy) -> Result<PendingOpenAiOAuthFlow, String> {
    infrastructure::start_login(strategy)
}

pub fn wait_for_callback(flow_id: String) -> Result<String, String> {
    infrastructure::wait_for_callback(&flow_id)
}

pub fn complete_login(flow_id: String, callback_url: String) -> Result<OpenAiAuthSnapshot, String> {
    let stored = infrastructure::complete_login(&flow_id, &callback_url)?;
    Ok(OpenAiAuthSnapshot::from_session(&stored.session))
}

pub fn logout(strategy: CredentialStoreStrategy) -> Result<(), String> {
    infrastructure::clear_credentials(strategy)
}

pub fn load_or_refresh_session() -> Result<OpenAiOAuthSession, String> {
    let Some(stored) = infrastructure::load_credentials()? else {
        return Err(String::from(
            "Nao encontrei sessao OpenAI. Entre com ChatGPT nas settings antes de iniciar a transcription realtime.",
        ));
    };

    let now_unix_ms = infrastructure::now_unix_ms();

    if stored.session.expires_soon(now_unix_ms) {
        let refreshed = infrastructure::refresh_session(&stored)?;
        return Ok(refreshed.session);
    }

    Ok(stored.session)
}

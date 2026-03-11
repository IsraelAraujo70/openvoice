#![allow(dead_code)]

use crate::modules::auth::domain::{
    CredentialStoreStrategy, OpenAiCredentials, StoredOpenAiCredentials,
};
use crate::modules::auth::infrastructure;

pub fn load_credentials() -> Result<Option<StoredOpenAiCredentials>, String> {
    infrastructure::load_credentials()
}

pub fn save_api_key(
    api_key: String,
    strategy: CredentialStoreStrategy,
) -> Result<StoredOpenAiCredentials, String> {
    let trimmed = api_key.trim();

    if trimmed.is_empty() {
        return Err(String::from("A OpenAI credential nao pode ficar vazia."));
    }

    let stored = StoredOpenAiCredentials {
        strategy,
        credentials: OpenAiCredentials::ApiKey {
            api_key: trimmed.to_owned(),
        },
    };

    infrastructure::save_credentials(&stored)?;
    Ok(stored)
}

pub fn clear_credentials(strategy: CredentialStoreStrategy) -> Result<(), String> {
    infrastructure::clear_credentials(strategy)
}

#![allow(dead_code)]

use crate::modules::auth::domain::{
    CredentialStoreStrategy, OpenAiCredentials, StoredOpenAiCredentials, OPENVOICE_AUTH_ACCOUNT,
    OPENVOICE_AUTH_SERVICE,
};
use keyring::Entry;
use std::fs;
use std::path::PathBuf;

const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";

pub fn load_credentials() -> Result<Option<StoredOpenAiCredentials>, String> {
    if let Some(credentials) = load_from_env() {
        return Ok(Some(StoredOpenAiCredentials {
            strategy: CredentialStoreStrategy::Auto,
            credentials,
        }));
    }

    if let Some(credentials) = load_from_keyring()? {
        return Ok(Some(StoredOpenAiCredentials {
            strategy: CredentialStoreStrategy::Keyring,
            credentials,
        }));
    }

    load_from_file().map(|credentials| {
        credentials.map(|credentials| StoredOpenAiCredentials {
            strategy: CredentialStoreStrategy::File,
            credentials,
        })
    })
}

pub fn save_credentials(stored: &StoredOpenAiCredentials) -> Result<(), String> {
    match stored.strategy {
        CredentialStoreStrategy::Auto | CredentialStoreStrategy::Keyring => {
            if save_to_keyring(&stored.credentials).is_ok() {
                let _ = clear_file();
                return Ok(());
            }

            save_to_file(&stored.credentials)
        }
        CredentialStoreStrategy::File => save_to_file(&stored.credentials),
    }
}

pub fn clear_credentials(strategy: CredentialStoreStrategy) -> Result<(), String> {
    match strategy {
        CredentialStoreStrategy::Auto => {
            let keyring_result = clear_keyring();
            let file_result = clear_file();

            keyring_result.or(file_result)
        }
        CredentialStoreStrategy::Keyring => clear_keyring(),
        CredentialStoreStrategy::File => clear_file(),
    }
}

fn load_from_env() -> Option<OpenAiCredentials> {
    std::env::var(OPENAI_API_KEY_ENV)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .map(|api_key| OpenAiCredentials::ApiKey { api_key })
}

fn entry() -> Result<Entry, String> {
    Entry::new(OPENVOICE_AUTH_SERVICE, OPENVOICE_AUTH_ACCOUNT)
        .map_err(|error| format!("Falha ao preparar o keyring do OpenVoice: {error}"))
}

fn load_from_keyring() -> Result<Option<OpenAiCredentials>, String> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(raw) => parse_credentials(&raw).map(Some),
        Err(error) => {
            let message = error.to_string().to_lowercase();

            if message.contains("no entry") || message.contains("not found") {
                Ok(None)
            } else {
                Err(format!("Falha ao ler credenciais do keyring: {error}"))
            }
        }
    }
}

fn save_to_keyring(credentials: &OpenAiCredentials) -> Result<(), String> {
    let entry = entry()?;
    let serialized = serde_json::to_string(credentials)
        .map_err(|error| format!("Falha ao serializar credenciais OpenAI: {error}"))?;

    entry
        .set_password(&serialized)
        .map_err(|error| format!("Falha ao salvar credenciais no keyring: {error}"))
}

fn clear_keyring() -> Result<(), String> {
    let entry = entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(error) => {
            let message = error.to_string().to_lowercase();

            if message.contains("no entry") || message.contains("not found") {
                Ok(())
            } else {
                Err(format!("Falha ao limpar credenciais do keyring: {error}"))
            }
        }
    }
}

fn load_from_file() -> Result<Option<OpenAiCredentials>, String> {
    let path = auth_file_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Falha ao ler auth em {}: {error}", path.display()))?;

    parse_credentials(&raw).map(Some)
}

fn save_to_file(credentials: &OpenAiCredentials) -> Result<(), String> {
    let path = auth_file_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao criar pasta de auth: {error}"))?;
    }

    let raw = serde_json::to_string_pretty(credentials)
        .map_err(|error| format!("Falha ao serializar auth OpenAI: {error}"))?;

    fs::write(&path, raw)
        .map_err(|error| format!("Falha ao salvar auth em {}: {error}", path.display()))
}

fn clear_file() -> Result<(), String> {
    let path = auth_file_path()?;

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path)
        .map_err(|error| format!("Falha ao remover auth em {}: {error}", path.display()))
}

fn parse_credentials(raw: &str) -> Result<OpenAiCredentials, String> {
    serde_json::from_str(raw)
        .map_err(|error| format!("Falha ao interpretar credenciais OpenAI: {error}"))
}

fn auth_file_path() -> Result<PathBuf, String> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .ok_or_else(|| {
            String::from("Nao consegui descobrir a pasta de configuracao do usuario.")
        })?;

    Ok(base.join("openvoice").join("auth.json"))
}

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const DEFAULT_OPENAI_REALTIME_MODEL: &str = "gpt-4o-mini-transcribe";
pub const OPENVOICE_AUTH_SERVICE: &str = "openvoice";
pub const OPENVOICE_AUTH_ACCOUNT: &str = "openai";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CredentialStoreStrategy {
    #[default]
    Auto,
    Keyring,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OpenAiCredentials {
    ApiKey {
        api_key: String,
    },
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at_unix_ms: Option<u128>,
    },
}

impl OpenAiCredentials {
    pub fn bearer_token(&self) -> &str {
        match self {
            Self::ApiKey { api_key } => api_key,
            Self::OAuth { access_token, .. } => access_token,
        }
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::ApiKey { .. } => "api_key",
            Self::OAuth { .. } => "oauth",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredOpenAiCredentials {
    pub strategy: CredentialStoreStrategy,
    pub credentials: OpenAiCredentials,
}

#[derive(Debug, Clone)]
pub struct RealtimeAuthConfig {
    pub credentials: OpenAiCredentials,
    pub model: String,
}

impl RealtimeAuthConfig {
    pub fn default_model() -> String {
        String::from(DEFAULT_OPENAI_REALTIME_MODEL)
    }
}

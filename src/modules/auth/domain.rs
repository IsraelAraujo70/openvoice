#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const DEFAULT_OPENAI_REALTIME_MODEL: &str = "gpt-4o-mini-transcribe";
pub const OPENVOICE_AUTH_SERVICE: &str = "openvoice";
pub const OPENVOICE_AUTH_ACCOUNT: &str = "openai";
pub const OPENAI_OAUTH_CLIENT_ID: &str = "app_EMoamEEZ73f0CkXaXp7hrann";
pub const OPENAI_OAUTH_ISSUER: &str = "https://auth.openai.com";
pub const OPENAI_OAUTH_PORT: u16 = 1455;
pub const OPENAI_OAUTH_TIMEOUT_SECS: u64 = 300;
pub const OPENAI_OAUTH_SCOPE: &str = "openid profile email offline_access";
pub const OPENAI_OAUTH_ORIGINATOR: &str = "openvoice";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CredentialStoreStrategy {
    #[default]
    Auto,
    Keyring,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAiOAuthSession {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_at_unix_ms: u128,
    pub id_token: Option<String>,
    pub account_id: Option<String>,
    pub email: Option<String>,
}

impl OpenAiOAuthSession {
    pub fn bearer_token(&self) -> &str {
        &self.access_token
    }

    pub fn expires_soon(&self, now_unix_ms: u128) -> bool {
        self.expires_at_unix_ms <= now_unix_ms + 60_000
    }

    pub fn account_label(&self) -> Option<String> {
        self.email.clone().or_else(|| self.account_id.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredOpenAiCredentials {
    pub strategy: CredentialStoreStrategy,
    pub session: OpenAiOAuthSession,
}

#[derive(Debug, Clone)]
pub struct OpenAiAuthSnapshot {
    pub is_authenticated: bool,
    pub account_label: Option<String>,
    pub expires_at_unix_ms: Option<u128>,
}

#[derive(Debug, Clone)]
pub struct PendingOpenAiOAuthFlow {
    pub flow_id: String,
    pub redirect_uri: String,
}

impl OpenAiAuthSnapshot {
    pub fn signed_out() -> Self {
        Self {
            is_authenticated: false,
            account_label: None,
            expires_at_unix_ms: None,
        }
    }

    pub fn from_session(session: &OpenAiOAuthSession) -> Self {
        Self {
            is_authenticated: true,
            account_label: session.account_label(),
            expires_at_unix_ms: Some(session.expires_at_unix_ms),
        }
    }
}

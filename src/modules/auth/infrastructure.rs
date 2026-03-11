#![allow(dead_code)]

use crate::modules::auth::domain::{
    CredentialStoreStrategy, OPENAI_OAUTH_CLIENT_ID, OPENAI_OAUTH_ISSUER, OPENAI_OAUTH_ORIGINATOR,
    OPENAI_OAUTH_PORT, OPENAI_OAUTH_SCOPE, OPENAI_OAUTH_TIMEOUT_SECS, OPENVOICE_AUTH_ACCOUNT,
    OPENVOICE_AUTH_SERVICE, OpenAiOAuthSession, StoredOpenAiCredentials,
};
use base64::Engine;
use keyring::Entry;
use rand::RngCore;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{LazyLock, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
struct PendingOAuthContext {
    strategy: CredentialStoreStrategy,
    redirect_uri: String,
    verifier: String,
    state: String,
}

static PENDING_OAUTH_CONTEXTS: LazyLock<Mutex<HashMap<String, PendingOAuthContext>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));
static PENDING_OAUTH_LISTENERS: LazyLock<Mutex<HashMap<String, TcpListener>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: Option<u64>,
    id_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JwtClaims {
    chatgpt_account_id: Option<String>,
    email: Option<String>,
    organizations: Option<Vec<OrganizationClaim>>,
    #[serde(rename = "https://api.openai.com/auth")]
    openai_auth: Option<OpenAiAuthClaim>,
}

#[derive(Debug, Deserialize)]
struct OrganizationClaim {
    id: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiAuthClaim {
    chatgpt_account_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuthCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

pub fn now_unix_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

pub fn load_credentials() -> Result<Option<StoredOpenAiCredentials>, String> {
    eprintln!("[openvoice][auth] loading credentials");
    if let Some(credentials) = load_from_keyring()? {
        eprintln!("[openvoice][auth] credentials loaded from keyring");
        return Ok(Some(StoredOpenAiCredentials {
            strategy: CredentialStoreStrategy::Keyring,
            session: credentials,
        }));
    }

    eprintln!("[openvoice][auth] keyring unavailable or empty, trying auth file");
    load_from_file().map(|session| {
        session.map(|session| StoredOpenAiCredentials {
            strategy: CredentialStoreStrategy::File,
            session,
        })
    })
}

pub fn login_with_chatgpt(
    strategy: CredentialStoreStrategy,
) -> Result<StoredOpenAiCredentials, String> {
    let flow = start_login(strategy)?;
    let callback_url = wait_for_callback(&flow.flow_id)?;
    complete_login(&flow.flow_id, &callback_url)
}

pub fn refresh_session(
    stored: &StoredOpenAiCredentials,
) -> Result<StoredOpenAiCredentials, String> {
    let session = refresh_access_token(&stored.session.refresh_token)?;
    let refreshed = StoredOpenAiCredentials {
        strategy: stored.strategy,
        session,
    };

    save_credentials(&refreshed)?;
    Ok(refreshed)
}

pub fn save_credentials(stored: &StoredOpenAiCredentials) -> Result<(), String> {
    match stored.strategy {
        CredentialStoreStrategy::Auto => {
            let keyring_result = save_to_keyring(&stored.session);
            let file_result = save_to_file(&stored.session);

            match (keyring_result, file_result) {
                (Ok(()), Ok(())) => Ok(()),
                (Ok(()), Err(error)) => {
                    eprintln!(
                        "[openvoice][auth] auth file fallback save failed but keyring save succeeded error={}",
                        error
                    );
                    Ok(())
                }
                (Err(error), Ok(())) => {
                    eprintln!(
                        "[openvoice][auth] keyring save failed but auth file fallback succeeded error={}",
                        error
                    );
                    Ok(())
                }
                (Err(keyring_error), Err(file_error)) => Err(format!(
                    "Falha ao salvar credenciais no keyring ({keyring_error}) e no arquivo ({file_error})."
                )),
            }
        }
        CredentialStoreStrategy::Keyring => save_to_keyring(&stored.session),
        CredentialStoreStrategy::File => save_to_file(&stored.session),
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

pub fn start_login(
    strategy: CredentialStoreStrategy,
) -> Result<crate::modules::auth::domain::PendingOpenAiOAuthFlow, String> {
    let pkce = generate_pkce();
    let state = generate_state();
    let redirect_uri = oauth_redirect_uri();
    let auth_url = build_authorize_url(&redirect_uri, &pkce.challenge, &state);
    let listener = bind_callback_listener()?;
    let flow_id = random_urlsafe_string(24);

    store_pending_oauth_context(
        flow_id.clone(),
        PendingOAuthContext {
            strategy,
            redirect_uri: redirect_uri.clone(),
            verifier: pkce.verifier,
            state,
        },
    )?;
    store_pending_oauth_listener(flow_id.clone(), listener)?;

    if let Err(error) = open_browser(&auth_url) {
        clear_pending_oauth_flow(&flow_id);
        return Err(error);
    }

    Ok(crate::modules::auth::domain::PendingOpenAiOAuthFlow {
        flow_id,
        redirect_uri,
    })
}

pub fn wait_for_callback(flow_id: &str) -> Result<String, String> {
    let listener = take_pending_oauth_listener(flow_id)?;
    wait_for_callback_on_listener(listener)
}

pub fn complete_login(
    flow_id: &str,
    callback_url: &str,
) -> Result<StoredOpenAiCredentials, String> {
    let context = get_pending_oauth_context(flow_id)?;
    eprintln!(
        "[openvoice][auth] completing oauth flow_id={} redirect_uri={} callback_url={}",
        flow_id, context.redirect_uri, callback_url
    );
    let callback = parse_callback_url(callback_url)?;

    let code = callback
        .code
        .ok_or_else(|| String::from("O callback do OpenAI nao retornou authorization code."))?;
    let returned_state = callback
        .state
        .ok_or_else(|| String::from("O callback do OpenAI nao retornou state."))?;

    if returned_state != context.state {
        return Err(String::from(
            "O state do callback OAuth nao confere. Fluxo cancelado por seguranca.",
        ));
    }

    eprintln!(
        "[openvoice][auth] state validated flow_id={} starting token exchange",
        flow_id
    );
    let session = exchange_code_for_tokens(&code, &context.redirect_uri, &context.verifier)?;
    eprintln!(
        "[openvoice][auth] token exchange succeeded flow_id={} account={:?} expires_at_unix_ms={}",
        flow_id, session.email, session.expires_at_unix_ms
    );
    let stored = StoredOpenAiCredentials {
        strategy: context.strategy,
        session,
    };

    eprintln!(
        "[openvoice][auth] saving credentials flow_id={} strategy={:?}",
        flow_id, stored.strategy
    );
    save_credentials(&stored)?;
    eprintln!("[openvoice][auth] credentials saved flow_id={}", flow_id);
    clear_pending_oauth_flow(flow_id);
    Ok(stored)
}

fn entry() -> Result<Entry, String> {
    Entry::new(OPENVOICE_AUTH_SERVICE, OPENVOICE_AUTH_ACCOUNT)
        .map_err(|error| format!("Falha ao preparar o keyring do OpenVoice: {error}"))
}

fn load_from_keyring() -> Result<Option<OpenAiOAuthSession>, String> {
    let entry = entry()?;
    match entry.get_password() {
        Ok(raw) => parse_session(&raw).map(Some),
        Err(error) if is_missing_keyring_entry(&error.to_string()) => {
            eprintln!(
                "[openvoice][auth] keyring read returned empty/unavailable error={}",
                error
            );
            Ok(None)
        }
        Err(error) => Err(format!("Falha ao ler credenciais do keyring: {error}")),
    }
}

fn save_to_keyring(session: &OpenAiOAuthSession) -> Result<(), String> {
    let entry = entry()?;
    let serialized = serde_json::to_string(session)
        .map_err(|error| format!("Falha ao serializar sessao OpenAI: {error}"))?;

    eprintln!("[openvoice][auth] attempting keyring save");
    entry.set_password(&serialized).map_err(|error| {
        eprintln!("[openvoice][auth] keyring save failed error={}", error);
        format!("Falha ao salvar credenciais no keyring: {error}")
    })
}

fn clear_keyring() -> Result<(), String> {
    let entry = entry()?;
    match entry.delete_credential() {
        Ok(()) => Ok(()),
        Err(error) if is_missing_keyring_entry(&error.to_string()) => Ok(()),
        Err(error) => Err(format!("Falha ao limpar credenciais do keyring: {error}")),
    }
}

fn is_missing_keyring_entry(message: &str) -> bool {
    let message = message.to_lowercase();

    message.contains("no entry")
        || message.contains("not found")
        || message.contains("no matching entry")
        || message.contains("platform secure storage failure")
}

fn load_from_file() -> Result<Option<OpenAiOAuthSession>, String> {
    let path = auth_file_path()?;

    if !path.exists() {
        return Ok(None);
    }

    let raw = fs::read_to_string(&path)
        .map_err(|error| format!("Falha ao ler auth em {}: {error}", path.display()))?;

    parse_session(&raw).map(Some)
}

fn save_to_file(session: &OpenAiOAuthSession) -> Result<(), String> {
    let path = auth_file_path()?;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Falha ao criar pasta de auth: {error}"))?;
    }

    let raw = serde_json::to_string_pretty(session)
        .map_err(|error| format!("Falha ao serializar auth OpenAI: {error}"))?;

    eprintln!(
        "[openvoice][auth] writing auth file path={}",
        path.display()
    );
    fs::write(&path, raw).map_err(|error| {
        eprintln!(
            "[openvoice][auth] auth file write failed path={} error={}",
            path.display(),
            error
        );
        format!("Falha ao salvar auth em {}: {error}", path.display())
    })
}

fn clear_file() -> Result<(), String> {
    let path = auth_file_path()?;

    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(&path)
        .map_err(|error| format!("Falha ao remover auth em {}: {error}", path.display()))
}

fn parse_session(raw: &str) -> Result<OpenAiOAuthSession, String> {
    serde_json::from_str(raw)
        .map_err(|error| format!("Falha ao interpretar sessao OpenAI: {error}"))
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

fn oauth_redirect_uri() -> String {
    format!("http://localhost:{OPENAI_OAUTH_PORT}/auth/callback")
}

fn store_pending_oauth_context(
    flow_id: String,
    context: PendingOAuthContext,
) -> Result<(), String> {
    PENDING_OAUTH_CONTEXTS
        .lock()
        .map_err(|_| String::from("Falha ao bloquear estado OAuth pendente."))?
        .insert(flow_id, context);
    Ok(())
}

fn store_pending_oauth_listener(flow_id: String, listener: TcpListener) -> Result<(), String> {
    PENDING_OAUTH_LISTENERS
        .lock()
        .map_err(|_| String::from("Falha ao bloquear listener OAuth pendente."))?
        .insert(flow_id, listener);
    Ok(())
}

fn get_pending_oauth_context(flow_id: &str) -> Result<PendingOAuthContext, String> {
    PENDING_OAUTH_CONTEXTS
        .lock()
        .map_err(|_| String::from("Falha ao ler contexto OAuth pendente."))?
        .get(flow_id)
        .cloned()
        .ok_or_else(|| String::from("Nao encontrei um fluxo OAuth pendente para concluir o login."))
}

fn take_pending_oauth_listener(flow_id: &str) -> Result<TcpListener, String> {
    PENDING_OAUTH_LISTENERS
        .lock()
        .map_err(|_| String::from("Falha ao ler listener OAuth pendente."))?
        .remove(flow_id)
        .ok_or_else(|| {
            String::from("Nao encontrei listener OAuth pendente para aguardar callback.")
        })
}

fn clear_pending_oauth_flow(flow_id: &str) {
    if let Ok(mut contexts) = PENDING_OAUTH_CONTEXTS.lock() {
        contexts.remove(flow_id);
    }

    if let Ok(mut listeners) = PENDING_OAUTH_LISTENERS.lock() {
        listeners.remove(flow_id);
    }
}

struct PkceCodes {
    verifier: String,
    challenge: String,
}

fn generate_pkce() -> PkceCodes {
    let verifier = random_urlsafe_string(64);
    let challenge_bytes = Sha256::digest(verifier.as_bytes());
    let challenge = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(challenge_bytes);

    PkceCodes {
        verifier,
        challenge,
    }
}

fn generate_state() -> String {
    random_urlsafe_string(32)
}

fn random_urlsafe_string(length: usize) -> String {
    let mut bytes = vec![0_u8; length];
    rand::thread_rng().fill_bytes(&mut bytes);
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

fn build_authorize_url(redirect_uri: &str, challenge: &str, state: &str) -> String {
    let mut url = reqwest::Url::parse(&format!("{OPENAI_OAUTH_ISSUER}/oauth/authorize"))
        .expect("valid oauth authorize url");

    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", OPENAI_OAUTH_CLIENT_ID)
        .append_pair("redirect_uri", redirect_uri)
        .append_pair("scope", OPENAI_OAUTH_SCOPE)
        .append_pair("code_challenge", challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("id_token_add_organizations", "true")
        .append_pair("codex_cli_simplified_flow", "true")
        .append_pair("state", state)
        .append_pair("originator", OPENAI_OAUTH_ORIGINATOR);

    url.to_string()
}

fn bind_callback_listener() -> Result<TcpListener, String> {
    let listener = TcpListener::bind(("127.0.0.1", OPENAI_OAUTH_PORT)).map_err(|error| {
        format!("Nao consegui abrir o callback OAuth em localhost:{OPENAI_OAUTH_PORT}: {error}")
    })?;

    listener
        .set_nonblocking(false)
        .map_err(|error| format!("Falha ao configurar o callback OAuth: {error}"))?;

    Ok(listener)
}

fn open_browser(url: &str) -> Result<(), String> {
    Command::new("xdg-open")
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            format!("Nao consegui abrir o navegador automaticamente com xdg-open: {error}")
        })
}

fn wait_for_callback_on_listener(listener: TcpListener) -> Result<String, String> {
    let (stream, _) = listener
        .accept()
        .map_err(|error| format!("Falha ao aceitar callback OAuth: {error}"))?;

    handle_callback_connection(stream)
}

fn handle_callback_connection(mut stream: TcpStream) -> Result<String, String> {
    stream
        .set_read_timeout(Some(Duration::from_secs(OPENAI_OAUTH_TIMEOUT_SECS)))
        .map_err(|error| format!("Falha ao configurar timeout do callback OAuth: {error}"))?;

    let mut buffer = [0_u8; 8192];
    let read = stream
        .read(&mut buffer)
        .map_err(|error| format!("Falha ao ler callback OAuth: {error}"))?;

    let request = String::from_utf8_lossy(&buffer[..read]);
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| String::from("Recebi um callback OAuth invalido."))?;
    let path = first_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| String::from("Recebi um callback OAuth sem path."))?;

    let url = format!("http://localhost:{OPENAI_OAUTH_PORT}{path}");
    eprintln!("[openvoice][auth] oauth callback received url={}", url);
    let query = parse_callback_url(&url)?;

    if let Some(error) = query.error.as_deref() {
        let description = query
            .error_description
            .clone()
            .unwrap_or_else(|| error.to_owned());
        let _ = write_http_response(&mut stream, 400, oauth_error_html(&description));
        return Err(format!(
            "O OpenAI retornou erro no callback OAuth: {description}"
        ));
    }

    write_http_response(&mut stream, 200, oauth_success_html())?;
    Ok(url)
}

fn parse_callback_url(input: &str) -> Result<OAuthCallbackQuery, String> {
    let trimmed = input.trim();
    let url = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_owned()
    } else if trimmed.starts_with("/auth/callback") {
        format!("http://localhost:{OPENAI_OAUTH_PORT}{trimmed}")
    } else {
        return Err(String::from(
            "Cole a URL completa do callback ou o path iniciado por /auth/callback.",
        ));
    };

    let parsed = reqwest::Url::parse(&url)
        .map_err(|error| format!("Falha ao interpretar callback OAuth: {error}"))?;

    if parsed.path() != "/auth/callback" {
        return Err(String::from(
            "A callback URL precisa apontar para /auth/callback.",
        ));
    }

    Ok(OAuthCallbackQuery {
        code: parsed
            .query_pairs()
            .find(|(key, _)| key == "code")
            .map(|(_, value)| value.into_owned()),
        state: parsed
            .query_pairs()
            .find(|(key, _)| key == "state")
            .map(|(_, value)| value.into_owned()),
        error: parsed
            .query_pairs()
            .find(|(key, _)| key == "error")
            .map(|(_, value)| value.into_owned()),
        error_description: parsed
            .query_pairs()
            .find(|(key, _)| key == "error_description")
            .map(|(_, value)| value.into_owned()),
    })
}

fn write_http_response(stream: &mut TcpStream, status: u16, body: String) -> Result<(), String> {
    let reason = if status == 200 { "OK" } else { "Bad Request" };
    let response = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    stream
        .write_all(response.as_bytes())
        .map_err(|error| format!("Falha ao responder callback OAuth: {error}"))
}

fn oauth_success_html() -> String {
    String::from(
        "<!doctype html><html><body style=\"font-family:sans-serif;background:#091017;color:#ecfeff;display:flex;align-items:center;justify-content:center;height:100vh\"><div><h1>OpenVoice autorizado</h1><p>Voce pode fechar esta aba e voltar ao app.</p></div></body></html>",
    )
}

fn oauth_error_html(error: &str) -> String {
    format!(
        "<!doctype html><html><body style=\"font-family:sans-serif;background:#1f1013;color:#fee2e2;display:flex;align-items:center;justify-content:center;height:100vh\"><div><h1>Falha na autorizacao</h1><p>{}</p></div></body></html>",
        html_escape(error)
    )
}

fn exchange_code_for_tokens(
    code: &str,
    redirect_uri: &str,
    verifier: &str,
) -> Result<OpenAiOAuthSession, String> {
    let client = oauth_http_client()?;
    eprintln!(
        "[openvoice][auth] posting oauth/token grant_type=authorization_code redirect_uri={}",
        redirect_uri
    );
    let response = client
        .post(format!("{OPENAI_OAUTH_ISSUER}/oauth/token"))
        .form(&[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", redirect_uri),
            ("client_id", OPENAI_OAUTH_CLIENT_ID),
            ("code_verifier", verifier),
        ])
        .send()
        .map_err(|error| {
            eprintln!(
                "[openvoice][auth] oauth/token transport failure grant_type=authorization_code error={}",
                error
            );
            format!("Falha ao trocar authorization code por tokens: {error}")
        })?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| {
            eprintln!(
                "[openvoice][auth] oauth/token body read failure grant_type=authorization_code status={} error={}",
                status, error
            );
            format!("Falha ao ler resposta do token OAuth: {error}")
        })?;

    if !status.is_success() {
        eprintln!(
            "[openvoice][auth] oauth/token authorization_code failure status={} body={}",
            status, body
        );
        return Err(format!(
            "OpenAI recusou a troca do authorization code. Status: {}",
            status
        ));
    }

    let token_response = serde_json::from_str::<TokenResponse>(&body).map_err(|error| {
        eprintln!(
            "[openvoice][auth] oauth/token authorization_code parse failure status={} body={} error={}",
            status, body, error
        );
        format!("Falha ao interpretar resposta do token OAuth: {error}")
    })?;

    session_from_token_response(token_response)
}

fn refresh_access_token(refresh_token: &str) -> Result<OpenAiOAuthSession, String> {
    let client = oauth_http_client()?;
    eprintln!("[openvoice][auth] posting oauth/token grant_type=refresh_token");
    let response = client
        .post(format!("{OPENAI_OAUTH_ISSUER}/oauth/token"))
        .form(&[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", OPENAI_OAUTH_CLIENT_ID),
        ])
        .send()
        .map_err(|error| {
            eprintln!(
                "[openvoice][auth] oauth/token transport failure grant_type=refresh_token error={}",
                error
            );
            format!("Falha ao atualizar sessao OpenAI: {error}")
        })?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|error| {
            eprintln!(
                "[openvoice][auth] oauth/token body read failure grant_type=refresh_token status={} error={}",
                status, error
            );
            format!("Falha ao ler resposta do refresh OAuth: {error}")
        })?;

    if !status.is_success() {
        eprintln!(
            "[openvoice][auth] oauth/token refresh_token failure status={} body={}",
            status, body
        );
        return Err(format!(
            "OpenAI recusou o refresh da sessao. Status: {}",
            status
        ));
    }

    let token_response = serde_json::from_str::<TokenResponse>(&body).map_err(|error| {
        eprintln!(
            "[openvoice][auth] oauth/token refresh_token parse failure status={} body={} error={}",
            status, body, error
        );
        format!("Falha ao interpretar refresh OAuth: {error}")
    })?;

    session_from_token_response(token_response)
}

fn oauth_http_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| format!("Falha ao criar cliente HTTP do OAuth: {error}"))
}

fn session_from_token_response(
    token_response: TokenResponse,
) -> Result<OpenAiOAuthSession, String> {
    let account_id = token_response
        .id_token
        .as_deref()
        .and_then(parse_jwt_claims)
        .and_then(extract_account_id)
        .or_else(|| parse_jwt_claims(&token_response.access_token).and_then(extract_account_id));
    let email = token_response
        .id_token
        .as_deref()
        .and_then(parse_jwt_claims)
        .and_then(|claims| claims.email)
        .or_else(|| parse_jwt_claims(&token_response.access_token).and_then(|claims| claims.email));

    Ok(OpenAiOAuthSession {
        access_token: token_response.access_token,
        refresh_token: token_response.refresh_token,
        expires_at_unix_ms: now_unix_ms()
            + (token_response.expires_in.unwrap_or(3600) as u128 * 1000),
        id_token: token_response.id_token,
        account_id,
        email,
    })
}

fn parse_jwt_claims(token: &str) -> Option<JwtClaims> {
    let mut parts = token.split('.');
    let _header = parts.next()?;
    let claims = parts.next()?;
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(claims)
        .ok()?;
    serde_json::from_slice::<JwtClaims>(&decoded).ok()
}

fn extract_account_id(claims: JwtClaims) -> Option<String> {
    claims
        .chatgpt_account_id
        .or_else(|| claims.openai_auth.and_then(|auth| auth.chatgpt_account_id))
        .or_else(|| {
            claims
                .organizations
                .and_then(|items| items.into_iter().next().map(|item| item.id))
        })
}

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::{extract_account_id, parse_jwt_claims};
    use base64::Engine;

    #[test]
    fn extracts_account_id_from_nested_claims() {
        let claims = serde_json::json!({
            "https://api.openai.com/auth": {
                "chatgpt_account_id": "acct_123"
            }
        });
        let token = format!(
            "a.{}.c",
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string())
        );

        let parsed = parse_jwt_claims(&token).expect("claims");
        assert_eq!(extract_account_id(parsed).as_deref(), Some("acct_123"));
    }
}

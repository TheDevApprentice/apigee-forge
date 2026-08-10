use std::{
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use async_trait::async_trait;
use oauth2::{
    basic::BasicClient, reqwest, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken,
    EndpointNotSet, EndpointSet, PkceCodeChallenge, RedirectUrl, RefreshToken, Scope,
    TokenResponse, TokenUrl,
};
use reqwest::Client;
use serde::Deserialize;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    time::timeout,
};
use url::Url;

use crate::{
    domain::{AuthContext, GoogleIdentity},
    error::AuthError,
    ports::auth_provider::{AccessToken, AuthProvider},
};

const DEFAULT_AUTHORIZATION_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const DEFAULT_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const DEFAULT_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const MAX_CALLBACK_BYTES: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct OAuthDesktopConfig {
    client_id: String,
    client_secret: Option<String>,
    authorization_url: String,
    token_url: String,
    userinfo_url: String,
    redirect_host: IpAddr,
    redirect_port: u16,
    callback_timeout: Duration,
    keyring_service: String,
    keyring_username: String,
}

impl OAuthDesktopConfig {
    pub fn new(client_id: impl Into<String>, keyring_username: impl Into<String>) -> Self {
        Self {
            client_id: client_id.into(),
            client_secret: None,
            authorization_url: DEFAULT_AUTHORIZATION_URL.to_owned(),
            token_url: DEFAULT_TOKEN_URL.to_owned(),
            userinfo_url: DEFAULT_USERINFO_URL.to_owned(),
            redirect_host: IpAddr::V4(Ipv4Addr::LOCALHOST),
            redirect_port: 0,
            callback_timeout: Duration::from_secs(300),
            keyring_service: "apigee-forge".to_owned(),
            keyring_username: keyring_username.into(),
        }
    }

    pub fn with_client_secret(mut self, client_secret: impl Into<String>) -> Self {
        self.client_secret = Some(client_secret.into());
        self
    }

    pub fn with_callback_timeout(mut self, callback_timeout: Duration) -> Self {
        self.callback_timeout = callback_timeout;
        self
    }
}

pub trait BrowserLauncher: Send + Sync {
    fn open(&self, url: &str) -> Result<(), AuthError>;
}

pub trait RefreshTokenStore: Send + Sync {
    fn load(&self) -> Result<Option<String>, AuthError>;
    fn save(&self, refresh_token: &str) -> Result<(), AuthError>;
    fn delete(&self) -> Result<(), AuthError>;
}

struct SystemBrowserLauncher;

impl BrowserLauncher for SystemBrowserLauncher {
    fn open(&self, url: &str) -> Result<(), AuthError> {
        webbrowser::open(url)
            .map(|_| ())
            .map_err(|_| AuthError::BrowserLaunch)
    }
}

struct KeyringRefreshTokenStore {
    service: String,
    username: String,
}

impl KeyringRefreshTokenStore {
    fn entry(&self) -> Result<keyring::Entry, AuthError> {
        keyring::Entry::new(&self.service, &self.username).map_err(|_| AuthError::CredentialStore)
    }
}

impl RefreshTokenStore for KeyringRefreshTokenStore {
    fn load(&self) -> Result<Option<String>, AuthError> {
        match self.entry()?.get_password() {
            Ok(token) => Ok(Some(token)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(_) => Err(AuthError::CredentialStore),
        }
    }

    fn save(&self, refresh_token: &str) -> Result<(), AuthError> {
        self.entry()?
            .set_password(refresh_token)
            .map_err(|_| AuthError::CredentialStore)
    }

    fn delete(&self) -> Result<(), AuthError> {
        match self.entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err(AuthError::CredentialStore),
        }
    }
}

struct CachedAccessToken {
    secret: String,
    expires_at: SystemTime,
}

pub struct OAuthDesktopAuthProvider {
    config: OAuthDesktopConfig,
    browser: Arc<dyn BrowserLauncher>,
    refresh_tokens: Arc<dyn RefreshTokenStore>,
    http_client: Client,
    access_token: Mutex<Option<CachedAccessToken>>,
    identity: Mutex<Option<GoogleIdentity>>,
}

impl OAuthDesktopAuthProvider {
    pub fn from_config(config: OAuthDesktopConfig) -> Result<Self, AuthError> {
        let refresh_tokens = KeyringRefreshTokenStore {
            service: config.keyring_service.clone(),
            username: config.keyring_username.clone(),
        };
        let http_client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| AuthError::OAuthConfiguration)?;

        Ok(Self::new(
            config,
            Arc::new(SystemBrowserLauncher),
            Arc::new(refresh_tokens),
            http_client,
        ))
    }

    pub fn new(
        config: OAuthDesktopConfig,
        browser: Arc<dyn BrowserLauncher>,
        refresh_tokens: Arc<dyn RefreshTokenStore>,
        http_client: Client,
    ) -> Self {
        Self {
            config,
            browser,
            refresh_tokens,
            http_client,
            access_token: Mutex::new(None),
            identity: Mutex::new(None),
        }
    }

    pub fn refresh_token_stored(&self) -> Result<bool, AuthError> {
        Ok(self.refresh_tokens.load()?.is_some())
    }

    pub fn logout(&self) -> Result<(), AuthError> {
        self.refresh_tokens.delete()?;
        self.access_token
            .lock()
            .map_err(|_| AuthError::AuthenticationFailed)?
            .take();
        self.identity
            .lock()
            .map_err(|_| AuthError::AuthenticationFailed)?
            .take();
        Ok(())
    }

    fn oauth_client(
        &self,
        redirect_url: String,
    ) -> Result<
        BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
        AuthError,
    > {
        let client_id = ClientId::new(self.config.client_id.clone());
        let auth_url = AuthUrl::new(self.config.authorization_url.clone())
            .map_err(|_| AuthError::OAuthConfiguration)?;
        let token_url = TokenUrl::new(self.config.token_url.clone())
            .map_err(|_| AuthError::OAuthConfiguration)?;
        let redirect_url =
            RedirectUrl::new(redirect_url).map_err(|_| AuthError::OAuthConfiguration)?;

        let client = match &self.config.client_secret {
            Some(secret) => {
                BasicClient::new(client_id).set_client_secret(ClientSecret::new(secret.clone()))
            }
            None => BasicClient::new(client_id),
        };
        Ok(client
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(redirect_url))
    }

    async fn refresh_access_token(
        &self,
        refresh_token: String,
    ) -> Result<(AccessToken, Option<String>), AuthError> {
        let client = self.oauth_client("http://127.0.0.1/unused".to_owned())?;
        let token = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(&self.http_client)
            .await
            .map_err(|error| {
                if env::var_os("APIGEE_FORGE_DEBUG_OAUTH").is_some() {
                    eprintln!("OAuth token exchange failed: {error}");
                }
                AuthError::TokenExchange
            })?;

        let access_token = access_token_from_response(&token)?;
        let replacement_refresh_token =
            token.refresh_token().map(|token| token.secret().to_owned());
        Ok((access_token, replacement_refresh_token))
    }

    async fn authorize_interactively(&self) -> Result<(AccessToken, GoogleIdentity), AuthError> {
        let listener = TcpListener::bind(SocketAddr::new(
            self.config.redirect_host,
            self.config.redirect_port,
        ))
        .await
        .map_err(|_| AuthError::Callback)?;
        let redirect_address = listener.local_addr().map_err(|_| AuthError::Callback)?;
        let redirect_url = format!("http://{}", redirect_address);
        let client = self.oauth_client(redirect_url)?;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let authorization_request = client
            .authorize_url(CsrfToken::new_random)
            .add_extra_param("access_type", "offline")
            .add_scope(Scope::new("openid".to_owned()))
            .add_scope(Scope::new("email".to_owned()))
            .add_scope(Scope::new("profile".to_owned()))
            .add_scope(Scope::new(
                "https://www.googleapis.com/auth/cloud-platform".to_owned(),
            ))
            .set_pkce_challenge(pkce_challenge);
        let authorization_request = authorization_request.add_extra_param("prompt", "consent");
        let (authorization_url, csrf_token) = authorization_request.url();

        self.browser
            .open(authorization_url.as_str())
            .map_err(|_| AuthError::BrowserLaunch)?;
        let (code, state) = receive_callback(listener, self.config.callback_timeout).await?;
        if state != csrf_token.secret().as_str() {
            return Err(AuthError::Callback);
        }

        let token = client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(&self.http_client)
            .await
            .map_err(|error| {
                if env::var_os("APIGEE_FORGE_DEBUG_OAUTH").is_some() {
                    eprintln!("OAuth token exchange failed: {error}");
                }
                AuthError::TokenExchange
            })?;
        let refresh_token = token
            .refresh_token()
            .map(|token| token.secret().to_owned())
            .ok_or(AuthError::RefreshTokenUnavailable)?;
        if env::var_os("APIGEE_FORGE_DEBUG_OAUTH").is_some() {
            eprintln!("OAuth interactive login: refresh token returned=true");
        }
        self.refresh_tokens.save(&refresh_token)?;
        if env::var_os("APIGEE_FORGE_DEBUG_OAUTH").is_some() {
            eprintln!("OAuth interactive login: refresh token saved to credential store");
        }

        let access_token = access_token_from_response(&token)?;
        let identity = self.lookup_identity(&access_token).await?;
        Ok((access_token, identity))
    }

    async fn lookup_identity(
        &self,
        access_token: &AccessToken,
    ) -> Result<GoogleIdentity, AuthError> {
        let response = self
            .http_client
            .get(&self.config.userinfo_url)
            .bearer_auth(access_token.as_str())
            .send()
            .await
            .map_err(|_| AuthError::IdentityLookup)?;
        if !response.status().is_success() {
            return Err(AuthError::IdentityLookup);
        }

        let identity: GoogleUserInfo = response
            .json()
            .await
            .map_err(|_| AuthError::IdentityLookup)?;
        if identity.email.is_empty() {
            return Err(AuthError::IdentityLookup);
        }

        Ok(GoogleIdentity::with_profile(
            identity.email,
            identity.given_name,
            identity.family_name,
            identity.name,
            identity.picture,
        ))
    }

    fn store_access_token(&self, access_token: &AccessToken) -> Result<(), AuthError> {
        let mut cached = self
            .access_token
            .lock()
            .map_err(|_| AuthError::AuthenticationFailed)?;
        *cached = Some(CachedAccessToken {
            secret: access_token.as_str().to_owned(),
            expires_at: access_token.expires_at(),
        });
        Ok(())
    }

    fn cached_access_token(&self) -> Result<Option<AccessToken>, AuthError> {
        let cached = self
            .access_token
            .lock()
            .map_err(|_| AuthError::AuthenticationFailed)?;
        let Some(cached) = cached.as_ref() else {
            return Ok(None);
        };
        if cached.expires_at <= SystemTime::now() {
            return Ok(None);
        }

        Ok(Some(AccessToken::new(
            cached.secret.clone(),
            cached.expires_at,
        )))
    }
}

#[derive(Debug, Deserialize)]
struct GoogleUserInfo {
    email: String,
    #[serde(default)]
    given_name: Option<String>,
    #[serde(default)]
    family_name: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    picture: Option<String>,
}

fn access_token_from_response<T: TokenResponse>(token: &T) -> Result<AccessToken, AuthError> {
    let expires_in = token.expires_in().ok_or(AuthError::TokenExchange)?;
    Ok(AccessToken::new(
        token.access_token().secret().to_owned(),
        SystemTime::now() + expires_in,
    ))
}

async fn receive_callback(
    listener: TcpListener,
    callback_timeout: Duration,
) -> Result<(String, String), AuthError> {
    let (mut stream, _) = timeout(callback_timeout, listener.accept())
        .await
        .map_err(|_| AuthError::Callback)?
        .map_err(|_| AuthError::Callback)?;
    let mut buffer = vec![0_u8; MAX_CALLBACK_BYTES];
    let bytes_read = stream
        .read(&mut buffer)
        .await
        .map_err(|_| AuthError::Callback)?;
    if bytes_read == buffer.len() {
        return Err(AuthError::Callback);
    }

    let request = std::str::from_utf8(&buffer[..bytes_read]).map_err(|_| AuthError::Callback)?;
    let target = request
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or(AuthError::Callback)?;
    let parsed = parse_callback_target(target)?;

    stream
        .write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\nAuthentication complete. You can close this window.",
        )
        .await
        .map_err(|_| AuthError::Callback)?;
    Ok(parsed)
}

fn parse_callback_target(target: &str) -> Result<(String, String), AuthError> {
    let callback_url =
        Url::parse(&format!("http://localhost{target}")).map_err(|_| AuthError::Callback)?;
    let code = callback_url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
        .ok_or(AuthError::Callback)?;
    let state = callback_url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .ok_or(AuthError::Callback)?;

    Ok((code, state))
}

impl OAuthDesktopAuthProvider {
    pub async fn restore_session(&self) -> Result<Option<AuthContext>, AuthError> {
        let refresh_token = self.refresh_tokens.load()?;
        if env::var_os("APIGEE_FORGE_DEBUG_OAUTH").is_some() {
            eprintln!(
                "OAuth restore: refresh token found={}",
                refresh_token.is_some()
            );
        }
        let Some(refresh_token) = refresh_token else {
            return Ok(None);
        };
        let (access_token, replacement) = self.refresh_access_token(refresh_token).await?;
        if env::var_os("APIGEE_FORGE_DEBUG_OAUTH").is_some() {
            eprintln!("OAuth restore: refresh succeeded");
        }
        if let Some(replacement) = replacement {
            self.refresh_tokens.save(&replacement)?;
        }
        let identity = self.lookup_identity(&access_token).await?;
        self.store_access_token(&access_token)?;
        self.identity
            .lock()
            .map_err(|_| AuthError::AuthenticationFailed)?
            .replace(identity.clone());
        Ok(Some(AuthContext::desktop_authenticated(identity)))
    }
}

#[async_trait]
impl AuthProvider for OAuthDesktopAuthProvider {
    async fn authenticate(&self) -> Result<AuthContext, AuthError> {
        if let Some(context) = self.restore_session().await? {
            return Ok(context);
        }
        let (access_token, identity) = self.authorize_interactively().await?;
        self.store_access_token(&access_token)?;
        self.identity
            .lock()
            .map_err(|_| AuthError::AuthenticationFailed)?
            .replace(identity.clone());
        Ok(AuthContext::desktop_authenticated(identity))
    }

    fn identity(&self) -> Option<GoogleIdentity> {
        self.identity
            .lock()
            .ok()
            .and_then(|identity| identity.clone())
    }

    async fn access_token(&self) -> Result<AccessToken, AuthError> {
        if let Some(access_token) = self.cached_access_token()? {
            return Ok(access_token);
        }
        self.authenticate().await?;
        self.cached_access_token()?
            .ok_or(AuthError::TokenUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
        sync::{Arc, Mutex},
    };

    use serde_json::{json, Value};

    use super::{
        parse_callback_target, BrowserLauncher, OAuthDesktopAuthProvider, OAuthDesktopConfig,
        RefreshTokenStore,
    };
    use reqwest::Client;

    fn write_test_report(report_name: &str, report: &Value) -> Result<PathBuf, Box<dyn Error>> {
        let report_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join("test-results");
        fs::create_dir_all(&report_directory)?;

        let report_path = report_directory.join(format!("{report_name}.json"));
        let report_file = File::create(&report_path)?;
        serde_json::to_writer_pretty(report_file, report)?;

        Ok(report_path)
    }

    struct NoopBrowser;

    impl BrowserLauncher for NoopBrowser {
        fn open(&self, _url: &str) -> Result<(), crate::error::AuthError> {
            Ok(())
        }
    }

    #[derive(Clone, Default)]
    struct MemoryRefreshTokenStore {
        deleted: Arc<Mutex<bool>>,
    }

    impl RefreshTokenStore for MemoryRefreshTokenStore {
        fn load(&self) -> Result<Option<String>, crate::error::AuthError> {
            Ok(None)
        }

        fn save(&self, _refresh_token: &str) -> Result<(), crate::error::AuthError> {
            Ok(())
        }

        fn delete(&self) -> Result<(), crate::error::AuthError> {
            *self
                .deleted
                .lock()
                .map_err(|_| crate::error::AuthError::CredentialStore)? = true;
            Ok(())
        }
    }

    #[test]
    fn injected_doubles_support_logout_without_os_access() -> Result<(), Box<dyn Error>> {
        let store = MemoryRefreshTokenStore::default();
        let deleted = Arc::clone(&store.deleted);
        let provider = OAuthDesktopAuthProvider::new(
            OAuthDesktopConfig::new("client-id", "test-account"),
            Arc::new(NoopBrowser),
            Arc::new(store),
            Client::new(),
        );

        provider.logout()?;
        let was_deleted = *deleted.lock().map_err(|_| "poisoned test lock")?;
        let report = json!({
            "test": "injected_doubles_support_logout_without_os_access",
            "expected": { "refresh_token_deleted": true },
            "actual": { "refresh_token_deleted": was_deleted }
        });
        let report_path = write_test_report("oauth_logout_doubles", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(was_deleted);
        Ok(())
    }

    #[test]
    fn callback_parser_requires_code_and_state() -> Result<(), Box<dyn Error>> {
        let parsed = parse_callback_target("/callback?code=auth-code&state=csrf-state");
        let actual = match parsed.as_ref() {
            Ok((code, state)) => json!({
                "code_present": !code.is_empty(),
                "state_present": !state.is_empty()
            }),
            Err(error) => json!({ "error": format!("{error:?}") }),
        };
        let report = json!({
            "test": "callback_parser_requires_code_and_state",
            "expected": {
                "code_present": true,
                "state_present": true
            },
            "actual": actual
        });
        let report_path = write_test_report("oauth_callback_valid", &report)?;
        eprintln!("test report: {}", report_path.display());

        let (code, state) = parsed?;
        assert_eq!(code, "auth-code");
        assert_eq!(state, "csrf-state");
        Ok(())
    }

    #[test]
    fn callback_parser_rejects_missing_state() -> Result<(), Box<dyn Error>> {
        let parsed = parse_callback_target("/callback?code=auth-code");
        let report = json!({
            "test": "callback_parser_rejects_missing_state",
            "expected_error": "Callback",
            "actual_error": parsed.as_ref().err().map(|error| format!("{error:?}"))
        });
        let report_path = write_test_report("oauth_callback_missing_state", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(parsed.is_err());
        Ok(())
    }
}

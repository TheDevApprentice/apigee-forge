mod commands;

use std::{
    env,
    sync::{Arc, Mutex},
};

use apigee_forge_core::{
    domain::{AuthContext, SessionState},
    error::AuthError,
    infra::oauth_desktop_auth_provider::{OAuthDesktopAuthProvider, OAuthDesktopConfig},
    infra::ReqwestApigeeGateway,
    ports::{ApigeeGateway, AuthProvider},
};
use async_trait::async_trait;
use serde::Serialize;
use url::Url;

const OAUTH_CLIENT_ID: &str = "APIGEE_FORGE_OAUTH_CLIENT_ID";
const OAUTH_USERNAME: &str = "APIGEE_FORGE_OAUTH_USERNAME";

#[async_trait]
pub trait GuiAuthProvider: Send + Sync {
    async fn authenticate(&self) -> Result<AuthContext, AuthError>;
    fn logout(&self) -> Result<(), AuthError>;
}

struct DesktopGuiAuthProvider {
    provider: Arc<OAuthDesktopAuthProvider>,
}

#[async_trait]
impl GuiAuthProvider for DesktopGuiAuthProvider {
    async fn authenticate(&self) -> Result<AuthContext, AuthError> {
        self.provider.authenticate().await
    }

    fn logout(&self) -> Result<(), AuthError> {
        self.provider.logout()
    }
}

pub struct GuiState {
    pub auth_provider: Option<Arc<dyn GuiAuthProvider>>,
    pub gateway: Option<Arc<dyn ApigeeGateway>>,
    pub auth_context: Mutex<Option<AuthContext>>,
    pub session: Mutex<SessionState>,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            auth_provider: None,
            gateway: None,
            auth_context: Mutex::new(None),
            session: Mutex::new(SessionState::cloud()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GuiError {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn build_state() -> GuiState {
    let Ok(client_id) = env::var(OAUTH_CLIENT_ID) else {
        return GuiState::default();
    };
    let Ok(username) = env::var(OAUTH_USERNAME) else {
        return GuiState::default();
    };
    let Ok(provider) =
        OAuthDesktopAuthProvider::from_config(OAuthDesktopConfig::new(client_id, username))
    else {
        return GuiState::default();
    };
    let provider = Arc::new(provider);
    let Ok(base_url) = Url::parse("https://apigee.googleapis.com/v1/") else {
        return GuiState::default();
    };
    let auth_provider: Arc<dyn AuthProvider> = provider.clone();
    let Ok(gateway) = ReqwestApigeeGateway::new(base_url, auth_provider) else {
        return GuiState::default();
    };
    GuiState {
        auth_provider: Some(Arc::new(DesktopGuiAuthProvider { provider })),
        gateway: Some(Arc::new(gateway)),
        auth_context: Mutex::new(None),
        session: Mutex::new(SessionState::cloud()),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .manage(build_state())
        .invoke_handler(tauri::generate_handler![
            commands::session_status,
            commands::auth_status,
            commands::auth_login,
            commands::auth_logout,
            commands::list_organizations,
            commands::list_environments,
            commands::list_proxies,
        ])
        .run(tauri::generate_context!())
}

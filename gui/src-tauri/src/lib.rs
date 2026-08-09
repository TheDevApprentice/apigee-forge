mod commands;

use std::{
    env,
    sync::{Arc, Mutex},
};

use apigee_forge_core::{
    domain::{AuthContext, SessionState},
    error::AuthError,
    infra::{
        oauth_desktop_auth_provider::{OAuthDesktopAuthProvider, OAuthDesktopConfig},
        InMemoryApigeeGateway, KeyringLocalKeyStore, ReqwestApigeeGateway,
        SqlCipherLocalStateStore,
    },
    ports::{ApigeeGateway, AuthProvider, LocalStateStore},
    use_cases::SessionStatePersistence,
};
use async_trait::async_trait;
use serde::Serialize;
use tauri::Manager;
use url::Url;

const OAUTH_CLIENT_ID: &str = "APIGEE_FORGE_OAUTH_CLIENT_ID";
const OAUTH_USERNAME: &str = "APIGEE_FORGE_OAUTH_USERNAME";

#[async_trait]
pub trait GuiAuthProvider: Send + Sync {
    async fn restore_session(&self) -> Result<Option<AuthContext>, AuthError>;
    async fn authenticate(&self) -> Result<AuthContext, AuthError>;
    fn logout(&self) -> Result<(), AuthError>;
}

struct DesktopGuiAuthProvider {
    provider: Arc<OAuthDesktopAuthProvider>,
}

#[async_trait]
impl GuiAuthProvider for DesktopGuiAuthProvider {
    async fn restore_session(&self) -> Result<Option<AuthContext>, AuthError> {
        self.provider.restore_session().await
    }

    async fn authenticate(&self) -> Result<AuthContext, AuthError> {
        self.provider.authenticate().await
    }
    fn logout(&self) -> Result<(), AuthError> {
        self.provider.logout()
    }
}

pub struct GuiState {
    pub auth_provider: Option<Arc<dyn GuiAuthProvider>>,
    pub gateway: Mutex<Option<Arc<dyn ApigeeGateway>>>,
    pub cloud_gateway: Option<Arc<dyn ApigeeGateway>>,
    pub demo_gateway: Arc<InMemoryApigeeGateway>,
    pub auth_context: Mutex<Option<AuthContext>>,
    pub session: Mutex<SessionState>,
    pub local_store: Mutex<Option<Arc<dyn LocalStateStore>>>,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            auth_provider: None,
            gateway: Mutex::new(None),
            cloud_gateway: None,
            demo_gateway: Arc::new(InMemoryApigeeGateway::new()),
            auth_context: Mutex::new(None),
            session: Mutex::new(SessionState::cloud()),
            local_store: Mutex::new(None),
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
    let username = env::var(OAUTH_USERNAME).unwrap_or_else(|_| "desktop".to_owned());
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
    let cloud_gateway: Arc<dyn ApigeeGateway> = Arc::new(gateway);
    let demo_gateway = Arc::new(InMemoryApigeeGateway::new());
    GuiState {
        auth_provider: Some(Arc::new(DesktopGuiAuthProvider { provider })),
        gateway: Mutex::new(Some(cloud_gateway.clone())),
        cloud_gateway: Some(cloud_gateway),
        demo_gateway,
        auth_context: Mutex::new(None),
        session: Mutex::new(SessionState::cloud()),
        local_store: Mutex::new(None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .manage(build_state())
        .setup(|app| {
            let path = app
                .path()
                .app_data_dir()
                .map_err(|_| "application data directory is unavailable")?
                .join("state.sqlcipher");
            let key_store = KeyringLocalKeyStore::new("apigee-forge", "demo-local-state-key");
            let state = app.state::<GuiState>();
            if let Ok(store) = SqlCipherLocalStateStore::open(path, &key_store) {
                let store = Arc::new(store);
                if let Ok(session) = SessionStatePersistence::new(store.clone()).load() {
                    *state
                        .session
                        .lock()
                        .map_err(|_| "application state is unavailable")? = session;
                    state
                        .local_store
                        .lock()
                        .map_err(|_| "application state is unavailable")?
                        .replace(store);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_app_mode,
            commands::set_app_mode,
            commands::session_status,
            commands::auth_restore,
            commands::auth_status,
            commands::auth_login,
            commands::auth_logout,
            commands::list_organizations,
            commands::get_roles,
            commands::list_environments,
            commands::list_proxies
        ])
        .run(tauri::generate_context!())
}

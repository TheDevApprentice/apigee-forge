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
        FilesystemTemplateRepository, InMemoryApigeeGateway, KeyringLocalKeyStore,
        ReqwestApigeeGateway, SqlCipherLocalStateStore,
    },
    ports::{
        ApigeeDeploymentGateway, ApigeeGateway, ApigeeRevisionGateway, AuthProvider,
        LocalStateStore, TemplateRepository,
    },
    use_cases::SessionStatePersistence,
};
use async_trait::async_trait;
use serde::Serialize;
use tauri::Manager;
use url::Url;

const OAUTH_CLIENT_ID: &str = "APIGEE_FORGE_OAUTH_CLIENT_ID";
const OAUTH_CLIENT_SECRET: &str = "APIGEE_FORGE_OAUTH_CLIENT_SECRET";
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
    pub deployment_gateway: Mutex<Option<Arc<dyn ApigeeDeploymentGateway>>>,
    pub cloud_deployment_gateway: Option<Arc<dyn ApigeeDeploymentGateway>>,
    pub revision_gateway: Mutex<Option<Arc<dyn ApigeeRevisionGateway>>>,
    pub cloud_revision_gateway: Option<Arc<dyn ApigeeRevisionGateway>>,
    pub demo_gateway: Arc<InMemoryApigeeGateway>,
    pub auth_context: Mutex<Option<AuthContext>>,
    pub session: Mutex<SessionState>,
    pub local_store: Mutex<Option<Arc<dyn LocalStateStore>>>,
    pub template_repository: Mutex<Option<Arc<dyn TemplateRepository>>>,
}

impl Default for GuiState {
    fn default() -> Self {
        Self {
            auth_provider: None,
            gateway: Mutex::new(None),
            cloud_gateway: None,
            deployment_gateway: Mutex::new(None),
            cloud_deployment_gateway: None,
            revision_gateway: Mutex::new(None),
            cloud_revision_gateway: None,
            demo_gateway: Arc::new(InMemoryApigeeGateway::new()),
            auth_context: Mutex::new(None),
            session: Mutex::new(SessionState::cloud()),
            local_store: Mutex::new(None),
            template_repository: Mutex::new(None),
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
    let mut oauth_config = OAuthDesktopConfig::new(client_id, username);
    if let Ok(client_secret) = env::var(OAUTH_CLIENT_SECRET) {
        oauth_config = oauth_config.with_client_secret(client_secret);
    }
    let Ok(provider) = OAuthDesktopAuthProvider::from_config(oauth_config) else {
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
    let cloud_deployment_gateway: Arc<dyn ApigeeDeploymentGateway> = Arc::new(
        ReqwestApigeeGateway::new(
            Url::parse("https://apigee.googleapis.com/v1/")
                .map_err(|_| ())
                .unwrap(),
            provider.clone() as Arc<dyn AuthProvider>,
        )
        .unwrap(),
    );
    let cloud_revision_gateway: Arc<dyn ApigeeRevisionGateway> = Arc::new(
        ReqwestApigeeGateway::new(
            Url::parse("https://apigee.googleapis.com/v1/")
                .map_err(|_| ())
                .unwrap(),
            provider.clone() as Arc<dyn AuthProvider>,
        )
        .unwrap(),
    );
    let demo_gateway = Arc::new(InMemoryApigeeGateway::new());
    GuiState {
        auth_provider: Some(Arc::new(DesktopGuiAuthProvider { provider })),
        gateway: Mutex::new(Some(cloud_gateway.clone())),
        cloud_gateway: Some(cloud_gateway),
        deployment_gateway: Mutex::new(Some(cloud_deployment_gateway.clone())),
        cloud_deployment_gateway: Some(cloud_deployment_gateway),
        revision_gateway: Mutex::new(Some(cloud_revision_gateway.clone())),
        cloud_revision_gateway: Some(cloud_revision_gateway),
        demo_gateway,
        auth_context: Mutex::new(None),
        session: Mutex::new(SessionState::cloud()),
        local_store: Mutex::new(None),
        template_repository: Mutex::new(None),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    let env_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(".env");
    let _ = dotenvy::from_path(env_path);
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
            let template_root = app
                .path()
                .app_data_dir()
                .map_err(|_| "application data directory is unavailable")?
                .join("templates");
            state
                .template_repository
                .lock()
                .map_err(|_| "application state is unavailable")?
                .replace(Arc::new(FilesystemTemplateRepository::new(template_root)));
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
            commands::get_revision_detail,
            commands::list_environments,
            commands::list_proxies,
            commands::list_templates,
            commands::get_template,
            commands::create_template,
            commands::update_template,
            commands::delete_template
        ])
        .run(tauri::generate_context!())
}

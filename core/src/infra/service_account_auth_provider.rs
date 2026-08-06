use std::{sync::Arc, time::SystemTime};

use async_trait::async_trait;
use gcp_auth::TokenProvider;
use tokio::sync::OnceCell;

use crate::{
    domain::{AuthContext, ProjectId},
    error::{AuthError, HeadlessAuthConfigError},
    ports::auth_provider::{AccessToken, AuthProvider},
};

use super::HeadlessAuthConfig;

const CLOUD_PLATFORM_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

pub struct ServiceAccountAuthProvider {
    config: HeadlessAuthConfig,
    provider: OnceCell<Arc<dyn TokenProvider>>,
}

impl ServiceAccountAuthProvider {
    pub fn from_environment() -> Result<Self, HeadlessAuthConfigError> {
        Ok(Self::new(HeadlessAuthConfig::from_environment()?))
    }

    pub fn new(config: HeadlessAuthConfig) -> Self {
        Self {
            config,
            provider: OnceCell::const_new(),
        }
    }

    pub fn config(&self) -> &HeadlessAuthConfig {
        &self.config
    }

    async fn token_provider(&self) -> Result<&Arc<dyn TokenProvider>, AuthError> {
        self.provider
            .get_or_try_init(|| async {
                gcp_auth::provider()
                    .await
                    .map_err(|_| AuthError::AuthenticationFailed)
            })
            .await
    }
}

#[async_trait]
impl AuthProvider for ServiceAccountAuthProvider {
    async fn authenticate(&self) -> Result<AuthContext, AuthError> {
        let project_id = self
            .token_provider()
            .await?
            .project_id()
            .await
            .map_err(|_| AuthError::AuthenticationFailed)?;

        Ok(AuthContext::headless(ProjectId::new(project_id.as_ref())))
    }

    async fn access_token(&self) -> Result<AccessToken, AuthError> {
        let token = self
            .token_provider()
            .await?
            .token(&[CLOUD_PLATFORM_SCOPE])
            .await
            .map_err(|_| AuthError::TokenUnavailable)?;

        Ok(AccessToken::new(
            token.as_str().to_owned(),
            SystemTime::from(token.expires_at()),
        ))
    }
}

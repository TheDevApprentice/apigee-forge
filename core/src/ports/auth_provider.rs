use async_trait::async_trait;

use crate::error::AuthError;

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self) -> Result<(), AuthError>;
    async fn access_token(&self) -> Result<String, AuthError>;
}

use async_trait::async_trait;
use serde_json::Value;

use crate::error::GatewayError;

#[async_trait]
pub trait ApigeeRevisionGateway: Send + Sync {
    async fn get_revision(
        &self,
        organization: &str,
        proxy_name: &str,
        revision: u32,
    ) -> Result<Value, GatewayError>;
}

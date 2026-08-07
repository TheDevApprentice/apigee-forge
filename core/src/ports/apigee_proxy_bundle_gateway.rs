use async_trait::async_trait;

use crate::{domain::ProxyRevision, error::GatewayError};

#[async_trait]
pub trait ApigeeProxyBundleGateway: Send + Sync {
    async fn import_bundle(
        &self,
        org: &str,
        proxy_name: &str,
        bundle: Vec<u8>,
    ) -> Result<ProxyRevision, GatewayError>;
}

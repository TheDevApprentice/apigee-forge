use async_trait::async_trait;

use crate::{domain::Deployment, error::GatewayError};

#[async_trait]
pub trait ApigeeDeploymentGateway: Send + Sync {
    async fn deploy(
        &self,
        org: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
        override_existing: bool,
    ) -> Result<Deployment, GatewayError>;

    async fn get_deployment_status(
        &self,
        org: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
    ) -> Result<Deployment, GatewayError>;
}

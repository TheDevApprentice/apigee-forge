use async_trait::async_trait;

use crate::{
    domain::{ApigeeRole, Deployment, Environment, Organization, Proxy},
    error::GatewayError,
};

#[async_trait]
pub trait ApigeeGateway: Send + Sync {
    async fn list_organizations(&self) -> Result<Vec<Organization>, GatewayError>;
    async fn list_environments(&self, org: &str) -> Result<Vec<Environment>, GatewayError>;
    async fn list_proxies(&self, org: &str) -> Result<Vec<Proxy>, GatewayError>;
    async fn deploy(
        &self,
        org: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
        override_existing: bool,
    ) -> Result<Deployment, GatewayError>;
    async fn get_deployment_status(&self, deployment_id: &str) -> Result<Deployment, GatewayError>;
    async fn get_role(&self, org: &str) -> Result<ApigeeRole, GatewayError>;
}

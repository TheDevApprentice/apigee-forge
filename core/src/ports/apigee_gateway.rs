use async_trait::async_trait;

use crate::{
    domain::{ApigeeRole, Environment, Organization, Proxy},
    error::GatewayError,
};

#[async_trait]
pub trait ApigeeGateway: Send + Sync {
    async fn list_organizations(&self) -> Result<Vec<Organization>, GatewayError>;
    async fn list_environments(&self, org: &str) -> Result<Vec<Environment>, GatewayError>;
    async fn list_proxies(&self, org: &str) -> Result<Vec<Proxy>, GatewayError>;
    async fn get_roles(&self, org: &str) -> Result<Vec<ApigeeRole>, GatewayError>;
}

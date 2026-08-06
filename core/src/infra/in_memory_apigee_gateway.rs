use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use async_trait::async_trait;

use crate::{
    domain::{ApigeeRole, Deployment, DeploymentStatus, Proxy},
    error::GatewayError,
    ports::ApigeeGateway,
};

#[derive(Debug, Default)]
struct GatewayState {
    organizations: Vec<String>,
    environments: HashMap<String, Vec<String>>,
    proxies: HashMap<String, Vec<Proxy>>,
    deployments: HashMap<String, Deployment>,
    roles: HashMap<String, ApigeeRole>,
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryApigeeGateway {
    state: Arc<Mutex<GatewayState>>,
}

impl InMemoryApigeeGateway {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_organization(&self, organization: impl Into<String>) -> Result<(), GatewayError> {
        let mut state = self.lock_state()?;
        let organization = organization.into();
        if !state.organizations.contains(&organization) {
            state.organizations.push(organization);
        }
        Ok(())
    }

    pub fn set_environments(
        &self,
        organization: impl Into<String>,
        environments: Vec<String>,
    ) -> Result<(), GatewayError> {
        let mut state = self.lock_state()?;
        state.environments.insert(organization.into(), environments);
        Ok(())
    }

    pub fn add_proxy(
        &self,
        organization: impl Into<String>,
        proxy: Proxy,
    ) -> Result<(), GatewayError> {
        let mut state = self.lock_state()?;
        state
            .proxies
            .entry(organization.into())
            .or_default()
            .push(proxy);
        Ok(())
    }

    pub fn set_role(
        &self,
        organization: impl Into<String>,
        role: ApigeeRole,
    ) -> Result<(), GatewayError> {
        let mut state = self.lock_state()?;
        state.roles.insert(organization.into(), role);
        Ok(())
    }

    fn lock_state(&self) -> Result<MutexGuard<'_, GatewayState>, GatewayError> {
        self.state.lock().map_err(|_| GatewayError::RequestFailed)
    }
}

#[async_trait]
impl ApigeeGateway for InMemoryApigeeGateway {
    async fn list_organizations(&self) -> Result<Vec<String>, GatewayError> {
        Ok(self.lock_state()?.organizations.clone())
    }

    async fn list_environments(&self, org: &str) -> Result<Vec<String>, GatewayError> {
        self.lock_state()?
            .environments
            .get(org)
            .cloned()
            .ok_or(GatewayError::RequestFailed)
    }

    async fn list_proxies(&self, org: &str) -> Result<Vec<Proxy>, GatewayError> {
        self.lock_state()?
            .proxies
            .get(org)
            .cloned()
            .ok_or(GatewayError::RequestFailed)
    }

    async fn deploy(&self, org: &str, bundle: Vec<u8>) -> Result<Deployment, GatewayError> {
        if bundle.is_empty() {
            return Err(GatewayError::InvalidResponse);
        }

        let mut state = self.lock_state()?;
        if !state
            .organizations
            .iter()
            .any(|organization| organization == org)
        {
            return Err(GatewayError::RequestFailed);
        }

        let id = format!("deployment-{}", state.deployments.len() + 1);
        let environment = state
            .environments
            .get(org)
            .and_then(|environments| environments.first())
            .cloned()
            .unwrap_or_else(|| "default".to_owned());
        let deployment = Deployment {
            id: id.clone(),
            proxy_name: "generated-proxy".to_owned(),
            environment,
            revision: 1,
            status: DeploymentStatus::Succeeded,
        };
        state.deployments.insert(id, deployment.clone());

        Ok(deployment)
    }

    async fn get_deployment_status(&self, deployment_id: &str) -> Result<Deployment, GatewayError> {
        self.lock_state()?
            .deployments
            .get(deployment_id)
            .cloned()
            .ok_or(GatewayError::RequestFailed)
    }

    async fn get_role(&self, org: &str) -> Result<ApigeeRole, GatewayError> {
        self.lock_state()?
            .roles
            .get(org)
            .copied()
            .ok_or(GatewayError::RequestFailed)
    }
}

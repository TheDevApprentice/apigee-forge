use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard},
};

use async_trait::async_trait;

use crate::{
    domain::{
        ApigeeRole, Deployment, DeploymentStatus, Environment, Organization, OrganizationId,
        ProjectId, Proxy,
    },
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
    async fn list_organizations(&self) -> Result<Vec<Organization>, GatewayError> {
        Ok(self
            .lock_state()?
            .organizations
            .iter()
            .map(|organization| Organization {
                id: OrganizationId::new(organization.clone()),
                project_id: ProjectId::new(organization.clone()),
                location: None,
            })
            .collect())
    }

    async fn list_environments(&self, org: &str) -> Result<Vec<Environment>, GatewayError> {
        self.lock_state()?
            .environments
            .get(org)
            .cloned()
            .map(|environments| {
                environments
                    .into_iter()
                    .map(|name| Environment { name })
                    .collect()
            })
            .ok_or(GatewayError::RequestFailed)
    }

    async fn list_proxies(&self, org: &str) -> Result<Vec<Proxy>, GatewayError> {
        self.lock_state()?
            .proxies
            .get(org)
            .cloned()
            .ok_or(GatewayError::RequestFailed)
    }

    async fn deploy(
        &self,
        org: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
        _override_existing: bool,
    ) -> Result<Deployment, GatewayError> {
        if proxy_name.is_empty() || revision == 0 {
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

        if !state
            .environments
            .get(org)
            .is_some_and(|environments| environments.iter().any(|item| item == environment))
        {
            return Err(GatewayError::RequestFailed);
        }

        let id = format!("deployment-{}", state.deployments.len() + 1);
        let deployment = Deployment {
            id: id.clone(),
            proxy_name: proxy_name.to_owned(),
            environment: environment.to_owned(),
            revision,
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

    async fn get_roles(&self, org: &str) -> Result<Vec<ApigeeRole>, GatewayError> {
        self.lock_state()?
            .roles
            .get(org)
            .copied()
            .map(|role| vec![role])
            .ok_or(GatewayError::RequestFailed)
    }
}

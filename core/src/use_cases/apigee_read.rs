use std::sync::Arc;

use crate::{
    domain::{ApigeeRole, Environment, Organization, Proxy},
    error::GatewayError,
    ports::ApigeeGateway,
};

pub struct ListOrganizationsUseCase {
    gateway: Arc<dyn ApigeeGateway>,
}

impl ListOrganizationsUseCase {
    pub fn new(gateway: Arc<dyn ApigeeGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self) -> Result<Vec<Organization>, GatewayError> {
        self.gateway.list_organizations().await
    }
}

pub struct ListEnvironmentsUseCase {
    gateway: Arc<dyn ApigeeGateway>,
}

impl ListEnvironmentsUseCase {
    pub fn new(gateway: Arc<dyn ApigeeGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, organization: &str) -> Result<Vec<Environment>, GatewayError> {
        self.gateway.list_environments(organization).await
    }
}

pub struct ListProxiesUseCase {
    gateway: Arc<dyn ApigeeGateway>,
}

impl ListProxiesUseCase {
    pub fn new(gateway: Arc<dyn ApigeeGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, organization: &str) -> Result<Vec<Proxy>, GatewayError> {
        self.gateway.list_proxies(organization).await
    }
}

pub struct GetApigeeRolesUseCase {
    gateway: Arc<dyn ApigeeGateway>,
}

impl GetApigeeRolesUseCase {
    pub fn new(gateway: Arc<dyn ApigeeGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, organization: &str) -> Result<Vec<ApigeeRole>, GatewayError> {
        self.gateway.get_roles(organization).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::{
        domain::{ApigeeRole, Environment, Organization, OrganizationId, ProjectId, Proxy},
        error::GatewayError,
        ports::ApigeeGateway,
    };

    use super::{ListEnvironmentsUseCase, ListOrganizationsUseCase, ListProxiesUseCase};

    struct FakeGateway;

    #[async_trait]
    impl ApigeeGateway for FakeGateway {
        async fn list_organizations(&self) -> Result<Vec<Organization>, GatewayError> {
            Ok(vec![Organization {
                id: OrganizationId::new("org-one"),
                project_id: ProjectId::new("project-one"),
                location: Some("us-central1".to_owned()),
            }])
        }

        async fn list_environments(
            &self,
            organization: &str,
        ) -> Result<Vec<Environment>, GatewayError> {
            if organization == "org-one" {
                Ok(vec![Environment {
                    name: "prod".to_owned(),
                }])
            } else {
                Err(GatewayError::NotFound)
            }
        }

        async fn list_proxies(&self, organization: &str) -> Result<Vec<Proxy>, GatewayError> {
            if organization == "org-one" {
                Ok(vec![Proxy {
                    name: "orders".to_owned(),
                    revisions: Vec::new(),
                }])
            } else {
                Err(GatewayError::NotFound)
            }
        }

        async fn get_roles(&self, _organization: &str) -> Result<Vec<ApigeeRole>, GatewayError> {
            Ok(vec![ApigeeRole::ReadOnlyAdmin])
        }
    }

    #[tokio::test]
    async fn delegates_read_operations_to_gateway_double() -> Result<(), GatewayError> {
        let gateway: Arc<dyn ApigeeGateway> = Arc::new(FakeGateway);
        let organizations = ListOrganizationsUseCase::new(gateway.clone())
            .execute()
            .await?;
        let environments = ListEnvironmentsUseCase::new(gateway.clone())
            .execute("org-one")
            .await?;
        let proxies = ListProxiesUseCase::new(gateway).execute("org-one").await?;
        assert_eq!(organizations[0].id.as_str(), "org-one");
        assert_eq!(environments[0].name, "prod");
        assert_eq!(proxies[0].name, "orders");
        Ok(())
    }
}

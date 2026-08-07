use std::sync::Arc;

use crate::{
    domain::{Deployment, ProxyRevision},
    error::GatewayError,
    ports::{ApigeeDeploymentGateway, ApigeeProxyBundleGateway},
};

pub struct ImportProxyBundleUseCase {
    gateway: Arc<dyn ApigeeProxyBundleGateway>,
}

impl ImportProxyBundleUseCase {
    pub fn new(gateway: Arc<dyn ApigeeProxyBundleGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(
        &self,
        organization: &str,
        proxy_name: &str,
        bundle: Vec<u8>,
    ) -> Result<ProxyRevision, GatewayError> {
        self.gateway
            .import_bundle(organization, proxy_name, bundle)
            .await
    }
}

pub struct DeployProxyUseCase {
    gateway: Arc<dyn ApigeeDeploymentGateway>,
}

impl DeployProxyUseCase {
    pub fn new(gateway: Arc<dyn ApigeeDeploymentGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(
        &self,
        organization: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
        override_existing: bool,
    ) -> Result<Deployment, GatewayError> {
        self.gateway
            .deploy(
                organization,
                environment,
                proxy_name,
                revision,
                override_existing,
            )
            .await
    }
}

pub struct GetDeploymentStatusUseCase {
    gateway: Arc<dyn ApigeeDeploymentGateway>,
}

impl GetDeploymentStatusUseCase {
    pub fn new(gateway: Arc<dyn ApigeeDeploymentGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(
        &self,
        organization: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
    ) -> Result<Deployment, GatewayError> {
        self.gateway
            .get_deployment_status(organization, environment, proxy_name, revision)
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;

    use crate::{
        domain::{Deployment, DeploymentStatus, ProxyRevision},
        error::GatewayError,
        ports::{ApigeeDeploymentGateway, ApigeeProxyBundleGateway},
    };

    use super::{DeployProxyUseCase, GetDeploymentStatusUseCase, ImportProxyBundleUseCase};

    struct FakeWriteGateway;

    #[async_trait]
    impl ApigeeProxyBundleGateway for FakeWriteGateway {
        async fn import_bundle(
            &self,
            _organization: &str,
            _proxy_name: &str,
            bundle: Vec<u8>,
        ) -> Result<ProxyRevision, GatewayError> {
            if bundle.is_empty() {
                return Err(GatewayError::InvalidResponse);
            }
            Ok(ProxyRevision {
                number: 3,
                deployed: false,
            })
        }
    }

    #[async_trait]
    impl ApigeeDeploymentGateway for FakeWriteGateway {
        async fn deploy(
            &self,
            _organization: &str,
            environment: &str,
            proxy_name: &str,
            revision: u32,
            _override_existing: bool,
        ) -> Result<Deployment, GatewayError> {
            Ok(Deployment {
                id: "deployment-3".to_owned(),
                proxy_name: proxy_name.to_owned(),
                environment: environment.to_owned(),
                revision,
                status: DeploymentStatus::Succeeded,
            })
        }

        async fn get_deployment_status(
            &self,
            _organization: &str,
            environment: &str,
            proxy_name: &str,
            revision: u32,
        ) -> Result<Deployment, GatewayError> {
            self.deploy("org", environment, proxy_name, revision, false)
                .await
        }
    }

    #[tokio::test]
    async fn delegates_import_deploy_and_status_to_gateway_double() -> Result<(), GatewayError> {
        let gateway = Arc::new(FakeWriteGateway);
        let revision = ImportProxyBundleUseCase::new(gateway.clone())
            .execute("org", "orders", vec![1, 2, 3])
            .await?;
        let deployment = DeployProxyUseCase::new(gateway.clone())
            .execute("org", "prod", "orders", revision.number, false)
            .await?;
        let status = GetDeploymentStatusUseCase::new(gateway)
            .execute("org", "prod", "orders", revision.number)
            .await?;
        assert_eq!(deployment.status, DeploymentStatus::Succeeded);
        assert_eq!(status.revision, 3);
        Ok(())
    }
}

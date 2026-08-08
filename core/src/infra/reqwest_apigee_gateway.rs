use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use reqwest::{
    multipart::{Form, Part},
    Client, Method, StatusCode, Url,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::time::sleep;

use crate::{
    domain::{
        ApigeeRole, Environment, Organization, OrganizationId, ProjectId, Proxy, ProxyRevision,
    },
    error::GatewayError,
    ports::{
        auth_provider::AuthProvider, ApigeeDeploymentGateway, ApigeeGateway,
        ApigeeProxyBundleGateway,
    },
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_BACKOFF: Duration = Duration::from_millis(250);

pub struct ReqwestApigeeGateway {
    client: Client,
    base_url: Url,
    iam_base_url: Url,
    auth: Arc<dyn AuthProvider>,
    max_retries: u32,
    retry_backoff: Duration,
}

impl ReqwestApigeeGateway {
    pub fn new(base_url: Url, auth: Arc<dyn AuthProvider>) -> Result<Self, GatewayError> {
        let mut gateway = Self::with_settings(
            base_url,
            auth,
            DEFAULT_TIMEOUT,
            DEFAULT_MAX_RETRIES,
            DEFAULT_BACKOFF,
        )?;
        gateway.iam_base_url = Url::parse("https://cloudresourcemanager.googleapis.com/v3/")
            .map_err(|_| GatewayError::InvalidResponse)?;
        Ok(gateway)
    }

    pub fn with_settings(
        base_url: Url,
        auth: Arc<dyn AuthProvider>,
        timeout: Duration,
        max_retries: u32,
        retry_backoff: Duration,
    ) -> Result<Self, GatewayError> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|_| GatewayError::RequestFailed)?;
        let iam_base_url = base_url.clone();

        Ok(Self {
            client,
            base_url,
            iam_base_url,
            auth,
            max_retries,
            retry_backoff,
        })
    }

    pub async fn get_json<T>(&self, path: &str) -> Result<T, GatewayError>
    where
        T: DeserializeOwned,
    {
        self.request_json_at(&self.base_url, Method::GET, path, Option::<&()>::None)
            .await
    }

    pub async fn post_json<B, T>(&self, path: &str, body: &B) -> Result<T, GatewayError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request_json_at(&self.base_url, Method::POST, path, Some(body))
            .await
    }

    pub async fn list_organizations(&self) -> Result<Vec<Organization>, GatewayError> {
        let response: OrganizationsResponse = self.get_json("organizations").await?;
        response
            .organizations
            .into_iter()
            .map(OrganizationMapping::try_into_domain)
            .collect()
    }

    pub async fn list_environments(
        &self,
        organization: &str,
    ) -> Result<Vec<Environment>, GatewayError> {
        let path = format!("organizations/{organization}/environments");
        let response: EnvironmentsResponse = self.get_json(&path).await?;
        response.into_domain()
    }

    pub async fn list_proxies(&self, organization: &str) -> Result<Vec<Proxy>, GatewayError> {
        let path = format!("organizations/{organization}/apis?includeRevisions=true");
        let response: ProxiesResponse = self.get_json(&path).await?;
        response
            .proxies
            .into_iter()
            .map(ProxyMapping::try_into_domain)
            .collect()
    }

    pub async fn get_roles(&self, project: &str) -> Result<Vec<ApigeeRole>, GatewayError> {
        let context = self
            .auth
            .authenticate()
            .await
            .map_err(|_| GatewayError::RequestFailed)?;
        let identity = context.identity.ok_or(GatewayError::IdentityUnavailable)?;
        let path = format!("projects/{project}:getIamPolicy");
        let response: IamPolicyResponse = self
            .request_json_at(&self.iam_base_url, Method::POST, &path, Some(&json!({})))
            .await?;
        let principal_user = format!("user:{}", identity.email());
        let principal_service_account = format!("serviceAccount:{}", identity.email());

        let mut roles = Vec::new();
        for binding in response.bindings {
            if binding
                .members
                .iter()
                .any(|member| member == &principal_user || member == &principal_service_account)
            {
                if let Some(role) = ApigeeRole::from_iam_name(&binding.role) {
                    if !roles.contains(&role) {
                        roles.push(role);
                    }
                } else if binding.role.starts_with("roles/apigee.") {
                    return Err(GatewayError::UnknownRole);
                }
            }
        }

        Ok(roles)
    }

    async fn request_multipart<T>(&self, url: Url, bundle: Vec<u8>) -> Result<T, GatewayError>
    where
        T: DeserializeOwned,
    {
        let token = self
            .auth
            .access_token()
            .await
            .map_err(|_| GatewayError::RequestFailed)?;
        for attempt in 0..=self.max_retries {
            let part = Part::bytes(bundle.clone())
                .file_name("proxy-bundle.zip")
                .mime_str("application/zip")
                .map_err(|_| GatewayError::InvalidResponse)?;
            let form = Form::new().part("file", part);
            match self
                .client
                .request(Method::POST, url.clone())
                .bearer_auth(token.as_str())
                .multipart(form)
                .send()
                .await
            {
                Ok(response) => {
                    let status = response.status();
                    if should_retry_status(status) && attempt < self.max_retries {
                        sleep(self.retry_backoff * 2_u32.saturating_pow(attempt)).await;
                        continue;
                    }
                    return parse_response(response).await;
                }
                Err(error) if is_retryable_error(&error) && attempt < self.max_retries => {
                    sleep(self.retry_backoff * 2_u32.saturating_pow(attempt)).await;
                }
                Err(error) if error.is_timeout() => return Err(GatewayError::Timeout),
                Err(_) => return Err(GatewayError::RequestFailed),
            }
        }
        Err(GatewayError::RequestFailed)
    }

    async fn request_json_at<B, T>(
        &self,
        base_url: &Url,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, GatewayError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = base_url
            .join(path)
            .map_err(|_| GatewayError::InvalidResponse)?;
        let token = self
            .auth
            .access_token()
            .await
            .map_err(|_| GatewayError::RequestFailed)?;

        for attempt in 0..=self.max_retries {
            let mut request = self
                .client
                .request(method.clone(), url.clone())
                .bearer_auth(token.as_str());
            if let Some(body) = body {
                request = request.json(body);
            } else if method == Method::POST {
                request = request.header("content-length", "0").body(Vec::new());
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    if should_retry_status(status) && attempt < self.max_retries {
                        sleep(self.retry_backoff * 2_u32.saturating_pow(attempt)).await;
                        continue;
                    }
                    return parse_response(response).await;
                }
                Err(error) if is_retryable_error(&error) && attempt < self.max_retries => {
                    sleep(self.retry_backoff * 2_u32.saturating_pow(attempt)).await;
                }
                Err(error) if error.is_timeout() => return Err(GatewayError::Timeout),
                Err(_) => return Err(GatewayError::RequestFailed),
            }
        }

        Err(GatewayError::RequestFailed)
    }
}

#[async_trait]
impl ApigeeGateway for ReqwestApigeeGateway {
    async fn list_organizations(&self) -> Result<Vec<Organization>, GatewayError> {
        Self::list_organizations(self).await
    }

    async fn list_environments(
        &self,
        organization: &str,
    ) -> Result<Vec<Environment>, GatewayError> {
        Self::list_environments(self, organization).await
    }

    async fn list_proxies(&self, organization: &str) -> Result<Vec<Proxy>, GatewayError> {
        Self::list_proxies(self, organization).await
    }

    async fn get_roles(&self, organization: &str) -> Result<Vec<ApigeeRole>, GatewayError> {
        Self::get_roles(self, organization).await
    }
}

#[async_trait]
impl ApigeeProxyBundleGateway for ReqwestApigeeGateway {
    async fn import_bundle(
        &self,
        organization: &str,
        proxy_name: &str,
        bundle: Vec<u8>,
    ) -> Result<ProxyRevision, GatewayError> {
        validate_segment(organization)?;
        validate_proxy_name(proxy_name)?;
        if bundle.is_empty() {
            return Err(GatewayError::InvalidResponse);
        }
        let mut url = self
            .base_url
            .join(&format!("organizations/{organization}/apis"))
            .map_err(|_| GatewayError::InvalidResponse)?;
        url.query_pairs_mut()
            .append_pair("action", "import")
            .append_pair("name", proxy_name);
        let response: ProxyMapping = self.request_multipart(url, bundle).await?;
        response
            .try_into_domain()?
            .revisions
            .into_iter()
            .max_by_key(|revision| revision.number)
            .ok_or(GatewayError::InvalidResponse)
    }
}

#[async_trait]
impl ApigeeDeploymentGateway for ReqwestApigeeGateway {
    async fn deploy(
        &self,
        organization: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
        override_existing: bool,
    ) -> Result<crate::domain::Deployment, GatewayError> {
        validate_segment(organization)?;
        validate_segment(environment)?;
        validate_proxy_name(proxy_name)?;
        if revision == 0 {
            return Err(GatewayError::InvalidResponse);
        }
        let path = format!(
            "organizations/{organization}/environments/{environment}/apis/{proxy_name}/revisions/{revision}/deployments?override={override_existing}"
        );
        let response: DeploymentMapping = self
            .request_json_at(&self.base_url, Method::POST, &path, Option::<&()>::None)
            .await?;
        response.into_domain(organization, environment, proxy_name, revision)
    }

    async fn get_deployment_status(
        &self,
        organization: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
    ) -> Result<crate::domain::Deployment, GatewayError> {
        validate_segment(organization)?;
        validate_segment(environment)?;
        validate_proxy_name(proxy_name)?;
        if revision == 0 {
            return Err(GatewayError::InvalidResponse);
        }
        let path = format!(
            "organizations/{organization}/environments/{environment}/apis/{proxy_name}/revisions/{revision}/deployments"
        );
        let response: DeploymentMapping = self
            .request_json_at(&self.base_url, Method::GET, &path, Option::<&()>::None)
            .await?;
        response.into_domain(organization, environment, proxy_name, revision)
    }
}

fn validate_segment(value: &str) -> Result<(), GatewayError> {
    if value.is_empty()
        || value.chars().any(|character| {
            character.is_control() || character.is_whitespace() || "/\\?&#".contains(character)
        })
    {
        Err(GatewayError::InvalidResponse)
    } else {
        Ok(())
    }
}

fn validate_proxy_name(value: &str) -> Result<(), GatewayError> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-_".contains(character))
    {
        Err(GatewayError::InvalidResponse)
    } else {
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct DeploymentMapping {
    #[serde(default)]
    environment: String,
    #[serde(rename = "apiProxy", default)]
    api_proxy: String,
    #[serde(default)]
    revision: String,
    #[serde(default)]
    state: Option<String>,
}

impl DeploymentMapping {
    fn into_domain(
        self,
        organization: &str,
        environment: &str,
        proxy_name: &str,
        revision: u32,
    ) -> Result<crate::domain::Deployment, GatewayError> {
        let actual_environment = if self.environment.is_empty() {
            environment.to_owned()
        } else {
            self.environment
        };
        let actual_proxy = if self.api_proxy.is_empty() {
            proxy_name.to_owned()
        } else {
            self.api_proxy
        };
        let actual_revision = if self.revision.is_empty() {
            revision
        } else {
            self.revision
                .parse::<u32>()
                .map_err(|_| GatewayError::InvalidResponse)?
        };
        let status = match self.state.as_deref() {
            Some("ACTIVE") | Some("READY") => crate::domain::DeploymentStatus::Succeeded,
            Some("PROGRESSING") => crate::domain::DeploymentStatus::InProgress,
            Some("ERROR") => crate::domain::DeploymentStatus::Failed,
            Some("INACTIVE") => crate::domain::DeploymentStatus::Pending,
            Some(_) => return Err(GatewayError::InvalidResponse),
            None => crate::domain::DeploymentStatus::Pending,
        };
        Ok(crate::domain::Deployment {
            id: format!(
                "organizations/{organization}/environments/{actual_environment}/apis/{actual_proxy}/revisions/{actual_revision}/deployments"
            ),
            proxy_name: actual_proxy,
            environment: actual_environment,
            revision: actual_revision,
            status,
        })
    }
}

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}

async fn parse_response<T>(response: reqwest::Response) -> Result<T, GatewayError>
where
    T: DeserializeOwned,
{
    match response.status() {
        StatusCode::BAD_REQUEST => Err(GatewayError::BadRequest),
        StatusCode::UNAUTHORIZED => Err(GatewayError::Unauthorized),
        StatusCode::FORBIDDEN => Err(GatewayError::Forbidden),
        StatusCode::NOT_FOUND => Err(GatewayError::NotFound),
        status if status == StatusCode::REQUEST_TIMEOUT => Err(GatewayError::Timeout),
        status if status == StatusCode::TOO_MANY_REQUESTS => Err(GatewayError::RateLimited),
        status if status.is_server_error() => Err(GatewayError::Server),
        status if status.is_success() => response
            .json::<T>()
            .await
            .map_err(|_| GatewayError::InvalidResponse),
        _ => Err(GatewayError::RequestFailed),
    }
}

#[derive(Debug, Deserialize)]
struct OrganizationsResponse {
    #[serde(default)]
    organizations: Vec<OrganizationMapping>,
}

#[derive(Debug, Deserialize)]
struct OrganizationMapping {
    organization: String,
    #[serde(rename = "projectId")]
    project_id: String,
    #[serde(default)]
    location: Option<String>,
}

impl OrganizationMapping {
    fn try_into_domain(self) -> Result<Organization, GatewayError> {
        if self.organization.is_empty() || self.project_id.is_empty() {
            return Err(GatewayError::InvalidResponse);
        }

        Ok(Organization {
            id: OrganizationId::new(self.organization),
            project_id: ProjectId::new(self.project_id),
            location: self.location,
        })
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum EnvironmentsResponse {
    ListValue { values: Vec<StringValue> },
    Named { environments: Vec<String> },
    Array(Vec<String>),
}

#[derive(Debug, Deserialize)]
struct StringValue {
    #[serde(rename = "stringValue")]
    value: String,
}

impl EnvironmentsResponse {
    fn into_domain(self) -> Result<Vec<Environment>, GatewayError> {
        let names = match self {
            Self::ListValue { values } => values.into_iter().map(|value| value.value).collect(),
            Self::Named { environments } => environments,
            Self::Array(environments) => environments,
        };

        if names.iter().any(String::is_empty) {
            return Err(GatewayError::InvalidResponse);
        }
        Ok(names.into_iter().map(|name| Environment { name }).collect())
    }
}

#[derive(Debug, Deserialize)]
struct ProxiesResponse {
    #[serde(default)]
    proxies: Vec<ProxyMapping>,
}

#[derive(Debug, Deserialize)]
struct ProxyMapping {
    name: String,
    #[serde(default)]
    revision: Value,
}

impl ProxyMapping {
    fn try_into_domain(self) -> Result<Proxy, GatewayError> {
        if self.name.is_empty() {
            return Err(GatewayError::InvalidResponse);
        }

        let revision_values = match self.revision {
            Value::Array(values) => values,
            Value::String(value) => vec![Value::String(value)],
            Value::Number(value) => vec![Value::Number(value)],
            Value::Null => Vec::new(),
            _ => return Err(GatewayError::InvalidResponse),
        };
        let revisions = revision_values
            .into_iter()
            .map(|value| {
                let number = match value {
                    Value::String(value) => value
                        .parse::<u32>()
                        .map_err(|_| GatewayError::InvalidResponse)?,
                    Value::Number(value) => value
                        .as_u64()
                        .and_then(|number| u32::try_from(number).ok())
                        .ok_or(GatewayError::InvalidResponse)?,
                    _ => return Err(GatewayError::InvalidResponse),
                };
                Ok(ProxyRevision {
                    number,
                    deployed: false,
                })
            })
            .collect::<Result<Vec<_>, GatewayError>>()?;

        Ok(Proxy {
            name: self.name,
            revisions,
        })
    }
}

#[derive(Debug, Deserialize)]
struct IamPolicyResponse {
    #[serde(default)]
    bindings: Vec<IamBinding>,
}

#[derive(Debug, Deserialize)]
struct IamBinding {
    role: String,
    #[serde(default)]
    members: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
        sync::Arc,
        time::{Duration, SystemTime},
    };

    use async_trait::async_trait;
    use reqwest::Url;
    use serde_json::{json, Value};
    use wiremock::{
        matchers::{header, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use crate::{
        domain::{AuthContext, GoogleIdentity, ProjectId},
        error::AuthError,
        ports::auth_provider::{AccessToken, AuthProvider},
    };

    use super::ReqwestApigeeGateway;

    struct TestAuthProvider {
        identity: Option<GoogleIdentity>,
    }

    #[async_trait]
    impl AuthProvider for TestAuthProvider {
        async fn authenticate(&self) -> Result<AuthContext, AuthError> {
            Ok(match &self.identity {
                Some(identity) => AuthContext::desktop_authenticated(identity.clone()),
                None => AuthContext::headless(ProjectId::new("test-project")),
            })
        }

        async fn access_token(&self) -> Result<AccessToken, AuthError> {
            Ok(AccessToken::new(
                "test-token",
                SystemTime::now() + Duration::from_secs(60),
            ))
        }
    }

    fn write_test_report(report_name: &str, report: &Value) -> Result<PathBuf, Box<dyn Error>> {
        let report_directory = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join("test-results");
        fs::create_dir_all(&report_directory)?;

        let report_path = report_directory.join(format!("{report_name}.json"));
        let report_file = File::create(&report_path)?;
        serde_json::to_writer_pretty(report_file, report)?;
        Ok(report_path)
    }

    async fn gateway(
        server: &MockServer,
        identity: Option<GoogleIdentity>,
        base_path: &str,
    ) -> Result<ReqwestApigeeGateway, Box<dyn Error>> {
        let base_url = Url::parse(&format!("{}{base_path}", server.uri()))?;
        Ok(ReqwestApigeeGateway::with_settings(
            base_url,
            Arc::new(TestAuthProvider { identity }),
            Duration::from_secs(5),
            0,
            Duration::ZERO,
        )?)
    }

    #[tokio::test]
    async fn maps_organizations_list() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/organizations"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "organizations": [{
                    "organization": "org-one",
                    "projectId": "project-one",
                    "location": "us-central1"
                }]
            })))
            .mount(&server)
            .await;

        let organizations = gateway(&server, None, "/v1/")
            .await?
            .list_organizations()
            .await?;
        let report = json!({
            "test": "maps_organizations_list",
            "expected": [{"id": "org-one", "project_id": "project-one", "location": "us-central1"}],
            "actual": organizations
        });
        let report_path = write_test_report("apigee_organizations_mapping", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(organizations.len(), 1);
        assert_eq!(organizations[0].id.as_str(), "org-one");
        assert_eq!(organizations[0].project_id.as_str(), "project-one");
        assert_eq!(organizations[0].location.as_deref(), Some("us-central1"));
        Ok(())
    }

    #[tokio::test]
    async fn maps_environment_list_value() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/organizations/org-one/environments"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "values": [{"stringValue": "dev"}, {"stringValue": "prod"}]
            })))
            .mount(&server)
            .await;

        let environments = gateway(&server, None, "/v1/")
            .await?
            .list_environments("org-one")
            .await?;
        let report = json!({
            "test": "maps_environment_list_value",
            "expected": [{"name": "dev"}, {"name": "prod"}],
            "actual": environments
        });
        let report_path = write_test_report("apigee_environments_mapping", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(
            environments
                .iter()
                .map(|item| item.name.as_str())
                .collect::<Vec<_>>(),
            ["dev", "prod"]
        );
        Ok(())
    }

    #[tokio::test]
    async fn maps_proxy_revisions() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/organizations/org-one/apis"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "proxies": [{"name": "orders", "revision": ["1", "2"]}]
            })))
            .mount(&server)
            .await;

        let proxies = gateway(&server, None, "/v1/")
            .await?
            .list_proxies("org-one")
            .await?;
        let report = json!({
            "test": "maps_proxy_revisions",
            "expected": [{"name": "orders", "revisions": [{"number": 1, "deployed": false}, {"number": 2, "deployed": false}]}],
            "actual": proxies
        });
        let report_path = write_test_report("apigee_proxies_mapping", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(proxies.len(), 1);
        assert_eq!(proxies[0].name, "orders");
        assert_eq!(proxies[0].revisions.len(), 2);
        assert_eq!(proxies[0].revisions[1].number, 2);
        Ok(())
    }

    #[tokio::test]
    async fn maps_multiple_apigee_roles_for_identity() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/projects/project-one:getIamPolicy"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "bindings": [
                    {"role": "roles/apigee.admin", "members": ["user:user@example.com"]},
                    {"role": "roles/apigee.deployer", "members": ["user:user@example.com"]},
                    {"role": "roles/storage.objectViewer", "members": ["user:user@example.com"]}
                ]
            })))
            .mount(&server)
            .await;

        let roles = gateway(
            &server,
            Some(GoogleIdentity::new("user@example.com")),
            "/v3/",
        )
        .await?
        .get_roles("project-one")
        .await?;
        let report = json!({
            "test": "maps_multiple_apigee_roles_for_identity",
            "expected": ["Admin", "Deployer"],
            "actual": roles
        });
        let report_path = write_test_report("apigee_roles_mapping", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(
            roles,
            [
                crate::domain::ApigeeRole::Admin,
                crate::domain::ApigeeRole::Deployer
            ]
        );
        Ok(())
    }

    async fn gateway_with_settings(
        server: &MockServer,
        timeout: Duration,
        max_retries: u32,
    ) -> Result<ReqwestApigeeGateway, Box<dyn Error>> {
        let base_url = Url::parse(&format!("{}/v1/", server.uri()))?;
        Ok(ReqwestApigeeGateway::with_settings(
            base_url,
            Arc::new(TestAuthProvider { identity: None }),
            timeout,
            max_retries,
            Duration::ZERO,
        )?)
    }

    #[tokio::test]
    async fn maps_http_error_statuses_without_recording_bodies() -> Result<(), Box<dyn Error>> {
        let cases = [
            (401, "Unauthorized"),
            (403, "Forbidden"),
            (404, "NotFound"),
            (408, "Timeout"),
            (429, "RateLimited"),
            (500, "Server"),
        ];
        let mut actual = Vec::new();

        for (status, expected) in cases {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .and(path("/v1/organizations"))
                .and(header("authorization", "Bearer test-token"))
                .respond_with(
                    ResponseTemplate::new(status)
                        .set_body_string("authorization body must not be recorded"),
                )
                .mount(&server)
                .await;

            let result = gateway(&server, None, "/v1/")
                .await?
                .get_json::<Value>("organizations")
                .await;
            let error = result.err().map(|error| format!("{error:?}"));
            assert_eq!(error.as_deref(), Some(expected));
            actual.push(json!({ "status": status, "error": error }));
        }

        let report = json!({
            "test": "maps_http_error_statuses_without_recording_bodies",
            "expected_statuses": [401, 403, 404, 408, 429, 500],
            "actual": actual
        });
        let report_path = write_test_report("apigee_http_statuses", &report)?;
        eprintln!("test report: {}", report_path.display());
        Ok(())
    }

    #[tokio::test]
    async fn maps_invalid_json_response() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/organizations"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not-json"))
            .mount(&server)
            .await;

        let result = gateway(&server, None, "/v1/")
            .await?
            .get_json::<Value>("organizations")
            .await;
        let error = result.err().map(|error| format!("{error:?}"));
        let report = json!({
            "test": "maps_invalid_json_response",
            "expected_error": "InvalidResponse",
            "actual_error": error
        });
        let report_path = write_test_report("apigee_invalid_json", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(error.as_deref(), Some("InvalidResponse"));
        Ok(())
    }

    #[tokio::test]
    async fn maps_timeout_without_retrying_non_transiently() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/organizations"))
            .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_millis(100)))
            .mount(&server)
            .await;

        let result = gateway_with_settings(&server, Duration::from_millis(10), 0)
            .await?
            .get_json::<Value>("organizations")
            .await;
        let error = result.err().map(|error| format!("{error:?}"));
        let report = json!({
            "test": "maps_timeout_without_retrying_non_transiently",
            "expected_error": "Timeout",
            "actual_error": error
        });
        let report_path = write_test_report("apigee_timeout", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(error.as_deref(), Some("Timeout"));
        Ok(())
    }

    #[tokio::test]
    async fn bounds_transient_retries() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/v1/organizations"))
            .respond_with(ResponseTemplate::new(503))
            .mount(&server)
            .await;

        let result = gateway_with_settings(&server, Duration::from_secs(5), 2)
            .await?
            .get_json::<Value>("organizations")
            .await;
        let requests = server
            .received_requests()
            .await
            .ok_or_else(|| std::io::Error::other("request recording is disabled"))?;
        let error = result.err().map(|error| format!("{error:?}"));
        let report = json!({
            "test": "bounds_transient_retries",
            "expected": { "error": "Server", "request_count": 3 },
            "actual": { "error": error, "request_count": requests.len() }
        });
        let report_path = write_test_report("apigee_retry_bound", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(error.as_deref(), Some("Server"));
        assert_eq!(requests.len(), 3);
        Ok(())
    }

    #[tokio::test]
    async fn imports_deploys_and_reads_deployment_status() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v1/organizations/org-one/apis"))
            .and(query_param("action", "import"))
            .and(query_param("name", "orders"))
            .and(header("authorization", "Bearer test-token"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "name": "orders",
                "revision": "4"
            })))
            .mount(&server)
            .await;
        Mock::given(method("POST"))
            .and(path(
                "/v1/organizations/org-one/environments/prod/apis/orders/revisions/4/deployments",
            ))
            .and(query_param("override", "false"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "environment": "prod",
                "apiProxy": "orders",
                "revision": "4",
                "state": "READY"
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path(
                "/v1/organizations/org-one/environments/prod/apis/orders/revisions/4/deployments",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "environment": "prod",
                "apiProxy": "orders",
                "revision": "4",
                "state": "PROGRESSING"
            })))
            .mount(&server)
            .await;

        let gateway = gateway(&server, None, "/v1/").await?;
        let imported = crate::ports::ApigeeProxyBundleGateway::import_bundle(
            &gateway,
            "org-one",
            "orders",
            vec![1, 2, 3],
        )
        .await?;
        let deployed = crate::ports::ApigeeDeploymentGateway::deploy(
            &gateway,
            "org-one",
            "prod",
            "orders",
            imported.number,
            false,
        )
        .await?;
        let status = crate::ports::ApigeeDeploymentGateway::get_deployment_status(
            &gateway,
            "org-one",
            "prod",
            "orders",
            imported.number,
        )
        .await?;
        assert_eq!(imported.number, 4);
        assert_eq!(deployed.status, crate::domain::DeploymentStatus::Succeeded);
        assert_eq!(status.status, crate::domain::DeploymentStatus::InProgress);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_unknown_apigee_role() -> Result<(), Box<dyn Error>> {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/v3/projects/project-one:getIamPolicy"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "bindings": [{"role": "roles/apigee.futureRole", "members": ["user:user@example.com"]}]
            })))
            .mount(&server)
            .await;

        let result = gateway(
            &server,
            Some(GoogleIdentity::new("user@example.com")),
            "/v3/",
        )
        .await?
        .get_roles("project-one")
        .await;
        let report = json!({
            "test": "rejects_unknown_apigee_role",
            "expected_error": "UnknownRole",
            "actual_error": result.as_ref().err().map(|error| format!("{error:?}"))
        });
        let report_path = write_test_report("apigee_unknown_role", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(matches!(
            result,
            Err(crate::error::GatewayError::UnknownRole)
        ));
        Ok(())
    }
}

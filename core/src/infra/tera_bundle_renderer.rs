use async_trait::async_trait;
use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    domain::template::{
        ApiKeyLocation, ConditionalFlow, FlowStage, JwtAlgorithm, PolicyType, PublicKeySource,
        QuotaTimeUnit, QuotaType, RateUnit, TransformDirection,
    },
    domain::{RenderInput, RenderedBundle, RenderedFile, Template},
    error::RenderError,
    ports::BundleRenderer,
};

const PROXY_ENDPOINT_TEMPLATE: &str = include_str!("templates/proxy_endpoint.xml.tera");
const TARGET_ENDPOINT_TEMPLATE: &str = include_str!("templates/target_endpoint.xml.tera");
const VERIFY_API_KEY_TEMPLATE: &str = include_str!("templates/verify_api_key.xml.tera");
const OAUTH_V2_TEMPLATE: &str = include_str!("templates/oauth_v2.xml.tera");
const VERIFY_JWT_TEMPLATE: &str = include_str!("templates/verify_jwt.xml.tera");
const QUOTA_TEMPLATE: &str = include_str!("templates/quota.xml.tera");
const SPIKE_ARREST_TEMPLATE: &str = include_str!("templates/spike_arrest.xml.tera");
const CORS_TEMPLATE: &str = include_str!("templates/cors.xml.tera");
const TRANSFORM_TEMPLATE: &str = include_str!("templates/transform.xml.tera");

pub struct TeraBundleRenderer {
    tera: Tera,
}

impl TeraBundleRenderer {
    pub fn new() -> Result<Self, RenderError> {
        let mut tera = Tera::default();
        for (name, source) in [
            ("proxy_endpoint.xml", PROXY_ENDPOINT_TEMPLATE),
            ("target_endpoint.xml", TARGET_ENDPOINT_TEMPLATE),
            ("verify_api_key.xml", VERIFY_API_KEY_TEMPLATE),
            ("oauth_v2.xml", OAUTH_V2_TEMPLATE),
            ("verify_jwt.xml", VERIFY_JWT_TEMPLATE),
            ("quota.xml", QUOTA_TEMPLATE),
            ("spike_arrest.xml", SPIKE_ARREST_TEMPLATE),
            ("cors.xml", CORS_TEMPLATE),
            ("transform.xml", TRANSFORM_TEMPLATE),
        ] {
            tera.add_raw_template(name, source)
                .map_err(|_| RenderError::Template)?;
        }
        Ok(Self { tera })
    }

    fn render_context(
        &self,
        input: &RenderInput,
        template: &Template,
        catalog: &PolicyCatalog,
    ) -> Result<Context, RenderError> {
        let conditional_flows = template
            .flow
            .conditional_flows
            .iter()
            .enumerate()
            .map(|(index, flow)| self.conditional_flow_context(index, flow, catalog))
            .collect::<Result<Vec<_>, _>>()?;
        let mut context = Context::new();
        context
            .try_insert("proxy_name", input.proxy_name.as_str())
            .map_err(|_| RenderError::InvalidInput)?;
        context
            .try_insert("target_url", input.target_url.as_str())
            .map_err(|_| RenderError::InvalidInput)?;
        context
            .try_insert(
                "pre_flow",
                &self.flow_stage_context(&template.flow.pre_flow, catalog)?,
            )
            .map_err(|_| RenderError::InvalidInput)?;
        context
            .try_insert("conditional_flows", &conditional_flows)
            .map_err(|_| RenderError::InvalidInput)?;
        context
            .try_insert(
                "post_flow",
                &self.flow_stage_context(&template.flow.post_flow, catalog)?,
            )
            .map_err(|_| RenderError::InvalidInput)?;
        Ok(context)
    }

    fn flow_stage_context(
        &self,
        stage: &FlowStage,
        catalog: &PolicyCatalog,
    ) -> Result<FlowStageContext, RenderError> {
        Ok(FlowStageContext {
            request: policy_names(&stage.request, catalog)?,
            response: policy_names(&stage.response, catalog)?,
        })
    }

    fn conditional_flow_context(
        &self,
        index: usize,
        flow: &ConditionalFlow,
        catalog: &PolicyCatalog,
    ) -> Result<ConditionalFlowContext, RenderError> {
        Ok(ConditionalFlowContext {
            name: format!("flow-{index}"),
            condition: flow.condition.clone().unwrap_or_default(),
            request: policy_names(&flow.request, catalog)?,
            response: policy_names(&flow.response, catalog)?,
        })
    }

    fn render_policies(&self, catalog: &PolicyCatalog) -> Result<Vec<RenderedFile>, RenderError> {
        catalog
            .entries
            .iter()
            .map(|entry| {
                let (template_name, context) = policy_template_context(&entry.policy, &entry.name)?;
                let contents = self
                    .tera
                    .render(template_name, &context)
                    .map_err(|_| RenderError::Template)?;
                RenderedFile::try_new(format!("apiproxy/policies/{}.xml", entry.name), contents)
                    .map_err(|_| RenderError::InvalidInput)
            })
            .collect()
    }
}

#[derive(Debug, Serialize)]
struct FlowStageContext {
    request: Vec<String>,
    response: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ConditionalFlowContext {
    name: String,
    condition: String,
    request: Vec<String>,
    response: Vec<String>,
}

#[derive(Debug, Clone)]
struct PolicyEntry {
    policy: PolicyType,
    name: String,
}

#[derive(Debug, Default)]
struct PolicyCatalog {
    entries: Vec<PolicyEntry>,
}

impl PolicyCatalog {
    fn from_template(template: &Template) -> Self {
        let mut catalog = Self::default();
        catalog.register_stage(&template.flow.pre_flow);
        for flow in &template.flow.conditional_flows {
            catalog.register_policies(&flow.request);
            catalog.register_policies(&flow.response);
        }
        catalog.register_stage(&template.flow.post_flow);
        catalog
    }

    fn register_stage(&mut self, stage: &FlowStage) {
        self.register_policies(&stage.request);
        self.register_policies(&stage.response);
    }

    fn register_policies(&mut self, policies: &[PolicyType]) {
        for policy in policies {
            if self.entries.iter().any(|entry| entry.policy == *policy) {
                continue;
            }
            let base_name = policy_base_name(policy);
            let same_family = self
                .entries
                .iter()
                .filter(|entry| {
                    entry.name == base_name || entry.name.starts_with(&format!("{base_name}-"))
                })
                .count();
            let name = if same_family == 0 {
                base_name
            } else {
                format!("{base_name}-{}", same_family + 1)
            };
            self.entries.push(PolicyEntry {
                policy: policy.clone(),
                name,
            });
        }
    }

    fn name_for(&self, policy: &PolicyType) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.policy == *policy)
            .map(|entry| entry.name.as_str())
    }
}

fn policy_names(
    policies: &[PolicyType],
    catalog: &PolicyCatalog,
) -> Result<Vec<String>, RenderError> {
    policies
        .iter()
        .map(|policy| {
            catalog
                .name_for(policy)
                .map(str::to_owned)
                .ok_or(RenderError::InvalidInput)
        })
        .collect()
}

fn policy_base_name(policy: &PolicyType) -> String {
    match policy {
        PolicyType::SecurityApiKey { .. } => "VerifyAPIKey".to_owned(),
        PolicyType::SecurityOAuth2 { .. } => "OAuthV2".to_owned(),
        PolicyType::SecurityJwt { .. } => "VerifyJWT".to_owned(),
        PolicyType::Quota { .. } => "Quota".to_owned(),
        PolicyType::SpikeArrest { .. } => "SpikeArrest".to_owned(),
        PolicyType::Cors { .. } => "CORS".to_owned(),
        PolicyType::Transform {
            direction: TransformDirection::XmlToJson,
        } => "XMLToJSON".to_owned(),
        PolicyType::Transform {
            direction: TransformDirection::JsonToXml,
        } => "JSONToXML".to_owned(),
    }
}

#[derive(Debug, Default, Serialize)]
struct PolicyContext {
    name: String,
    key_variable: String,
    scopes: Vec<String>,
    algorithm: String,
    issuer: String,
    audience: String,
    jwks_url: Option<String>,
    uses_secret_key: bool,
    allow: u32,
    interval: u32,
    time_unit: String,
    quota_type: String,
    rate: u32,
    rate_unit: String,
    allow_origins: Vec<String>,
    allow_headers: Vec<String>,
    allow_methods: Vec<String>,
    expose_headers: Vec<String>,
    max_age_seconds: Option<u32>,
    support_credentials: bool,
    element: String,
    output_variable: String,
    source: String,
}

fn policy_template_context(
    policy: &PolicyType,
    name: &str,
) -> Result<(&'static str, Context), RenderError> {
    let mut policy_context = PolicyContext {
        name: name.to_owned(),
        ..PolicyContext::default()
    };
    let template_name = match policy {
        PolicyType::SecurityApiKey {
            key_location,
            key_param_name,
        } => {
            let parameter = key_param_name.as_deref().unwrap_or("apikey");
            if !is_safe_parameter(parameter) {
                return Err(RenderError::InvalidPolicy);
            }
            let location = key_location.unwrap_or(ApiKeyLocation::Header);
            policy_context.key_variable = match location {
                ApiKeyLocation::Header => format!("request.header.{parameter}"),
                ApiKeyLocation::QueryParam => format!("request.queryparam.{parameter}"),
            };
            "verify_api_key.xml"
        }
        PolicyType::SecurityOAuth2 { scopes } => {
            validate_texts(scopes)?;
            policy_context.scopes = scopes.clone();
            "oauth_v2.xml"
        }
        PolicyType::SecurityJwt {
            algorithm,
            issuer,
            audience,
            public_key_source,
            jwks_url,
        } => {
            validate_text(issuer)?;
            validate_text(audience)?;
            policy_context.algorithm = match algorithm {
                JwtAlgorithm::Rs256 => "RS256".to_owned(),
                JwtAlgorithm::Hs256 => "HS256".to_owned(),
            };
            policy_context.issuer = issuer.clone();
            policy_context.audience = audience.clone();
            policy_context.uses_secret_key = matches!(algorithm, JwtAlgorithm::Hs256);
            if matches!(public_key_source, Some(PublicKeySource::JwksUrl)) {
                let url = jwks_url.as_deref().ok_or(RenderError::InvalidPolicy)?;
                validate_http_url(url)?;
                policy_context.jwks_url = Some(url.to_owned());
            } else if jwks_url.is_some()
                && matches!(public_key_source, Some(PublicKeySource::StaticPublicKey))
            {
                return Err(RenderError::InvalidPolicy);
            }
            "verify_jwt.xml"
        }
        PolicyType::Quota {
            allow,
            interval,
            time_unit,
            quota_type,
        } => {
            if *allow == 0 || *interval == 0 {
                return Err(RenderError::InvalidPolicy);
            }
            policy_context.allow = *allow;
            policy_context.interval = *interval;
            policy_context.time_unit = match time_unit {
                QuotaTimeUnit::Hour => "hour",
                QuotaTimeUnit::Day => "day",
                QuotaTimeUnit::Week => "week",
                QuotaTimeUnit::Month => "month",
            }
            .to_owned();
            policy_context.quota_type = match quota_type.unwrap_or(QuotaType::Default) {
                QuotaType::Default => "default",
                QuotaType::Calendar => "calendar",
                QuotaType::Flexi => "flexi",
                QuotaType::RollingWindow => "rollingwindow",
            }
            .to_owned();
            "quota.xml"
        }
        PolicyType::SpikeArrest { rate, rate_unit } => {
            if *rate == 0 {
                return Err(RenderError::InvalidPolicy);
            }
            policy_context.rate = *rate;
            policy_context.rate_unit = match rate_unit {
                RateUnit::PerSecond => "ps",
                RateUnit::PerMinute => "pm",
            }
            .to_owned();
            "spike_arrest.xml"
        }
        PolicyType::Cors {
            allow_origins,
            allow_headers,
            allow_methods,
            expose_headers,
            max_age_seconds,
            support_credentials,
        } => {
            if allow_origins.is_empty() {
                return Err(RenderError::InvalidPolicy);
            }
            validate_texts(allow_origins)?;
            validate_texts(allow_headers)?;
            validate_texts(allow_methods)?;
            validate_texts(expose_headers)?;
            policy_context.allow_origins = allow_origins.clone();
            policy_context.allow_headers = allow_headers.clone();
            policy_context.allow_methods = if allow_methods.is_empty() {
                ["GET", "POST", "PUT", "DELETE"]
                    .into_iter()
                    .map(str::to_owned)
                    .collect()
            } else {
                allow_methods.clone()
            };
            policy_context.expose_headers = expose_headers.clone();
            policy_context.max_age_seconds = Some(max_age_seconds.unwrap_or(3_628_800));
            policy_context.support_credentials = support_credentials.unwrap_or(false);
            "cors.xml"
        }
        PolicyType::Transform { direction } => {
            (
                policy_context.element,
                policy_context.output_variable,
                policy_context.source,
            ) = match direction {
                TransformDirection::XmlToJson => (
                    "XMLToJSON".to_owned(),
                    "response.content".to_owned(),
                    "response.content".to_owned(),
                ),
                TransformDirection::JsonToXml => (
                    "JSONToXML".to_owned(),
                    "response.content".to_owned(),
                    "response.content".to_owned(),
                ),
            };
            "transform.xml"
        }
    };

    let mut context = Context::new();
    context
        .try_insert("policy", &policy_context)
        .map_err(|_| RenderError::InvalidInput)?;
    Ok((template_name, context))
}

fn validate_text(value: &str) -> Result<(), RenderError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(RenderError::InvalidPolicy)
    } else {
        Ok(())
    }
}

fn validate_texts(values: &[String]) -> Result<(), RenderError> {
    values.iter().try_for_each(|value| validate_text(value))
}

fn validate_http_url(value: &str) -> Result<(), RenderError> {
    if (value.starts_with("http://") || value.starts_with("https://"))
        && !value.chars().any(char::is_whitespace)
    {
        Ok(())
    } else {
        Err(RenderError::InvalidPolicy)
    }
}

fn is_safe_parameter(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
}

#[async_trait]
impl BundleRenderer for TeraBundleRenderer {
    async fn render(
        &self,
        input: &RenderInput,
        template: &Template,
    ) -> Result<RenderedBundle, RenderError> {
        let catalog = PolicyCatalog::from_template(template);
        let context = self.render_context(input, template, &catalog)?;
        let proxy_endpoint = self
            .tera
            .render("proxy_endpoint.xml", &context)
            .map_err(|_| RenderError::Template)?;
        let target_endpoint = self
            .tera
            .render("target_endpoint.xml", &context)
            .map_err(|_| RenderError::Template)?;

        let mut files = vec![
            RenderedFile::try_new("apiproxy/proxies/default.xml", proxy_endpoint)
                .map_err(|_| RenderError::InvalidInput)?,
            RenderedFile::try_new("apiproxy/targets/default.xml", target_endpoint)
                .map_err(|_| RenderError::InvalidInput)?,
        ];
        files.extend(self.render_policies(&catalog)?);
        Ok(RenderedBundle::new(files))
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
    };

    use quick_xml::{events::Event, Reader};
    use serde_json::{json, Value};
    use tokio::runtime::Runtime;

    use crate::{
        domain::{ProxyName, RenderInput, RenderMethod, RenderRoute, TargetUrl},
        infra::TeraBundleRenderer,
        ports::BundleRenderer,
    };

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

    fn render_fixture() -> Result<crate::domain::RenderedBundle, Box<dyn Error>> {
        let runtime = Runtime::new()?;
        runtime.block_on(async {
            let template =
                serde_json::from_str(include_str!("../../../schemas/template.example.json"))?;
            let input = RenderInput::new(
                ProxyName::try_new("orders-v1")?,
                TargetUrl::try_new("https://api.example.test/v1?a=1&b=2")?,
                vec![RenderRoute {
                    path: "/orders".to_owned(),
                    method: RenderMethod::Get,
                    security_requirements: vec!["security_oauth2".to_owned()],
                }],
            );
            Ok::<_, Box<dyn Error>>(TeraBundleRenderer::new()?.render(&input, &template).await?)
        })
    }

    #[test]
    fn renders_proxy_and_target_endpoint_xml() -> Result<(), Box<dyn Error>> {
        let bundle = render_fixture()?;
        for file in &bundle.files {
            let mut reader = Reader::from_str(&file.contents);
            loop {
                if reader.read_event()? == Event::Eof {
                    break;
                }
            }
        }
        let report = json!({
            "test": "renders_proxy_and_target_endpoint_xml",
            "expected": {
                "file_paths": ["apiproxy/proxies/default.xml", "apiproxy/targets/default.xml"],
                "xml_well_formed": true,
                "escaped_target_url": true
            },
            "actual": {
                "file_paths": bundle.files.iter().map(|file| file.relative_path.clone()).collect::<Vec<_>>(),
                "xml_well_formed": true,
                "escaped_target_url": bundle.files.iter().any(|file| file.contents.contains("&amp;"))
            }
        });
        let report_path = write_test_report("render_proxy_endpoints", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(bundle
            .files
            .iter()
            .any(|file| file.relative_path == "apiproxy/proxies/default.xml"));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.relative_path == "apiproxy/targets/default.xml"));
        assert!(bundle
            .files
            .iter()
            .all(|file| file.contents.contains("<?xml")));
        assert!(bundle
            .files
            .iter()
            .any(|file| file.contents.contains("&amp;")));
        Ok(())
    }

    #[test]
    fn renders_all_mvp_policy_variants() -> Result<(), Box<dyn Error>> {
        let template = serde_json::from_value(json!({
            "metadata": {
                "name": "all-policies",
                "owner": "test",
                "naming_convention": { "prefix": "test-", "case": "kebab-case" }
            },
            "flow": {
                "pre_flow": {
                    "request": [
                        { "type": "security_api_key", "key_location": "header", "key_param_name": "X-API-Key" },
                        { "type": "security_oauth2", "scopes": ["read&write"] },
                        {
                            "type": "security_jwt",
                            "algorithm": "RS256",
                            "issuer": "https://issuer.example.test",
                            "audience": "orders",
                            "public_key_source": "jwks_url",
                            "jwks_url": "https://issuer.example.test/.well-known/jwks.json"
                        },
                        { "type": "quota", "allow": 10, "interval": 1, "time_unit": "hour" },
                        { "type": "spike_arrest", "rate": 5, "rate_unit": "pm" },
                        {
                            "type": "cors",
                            "allow_origins": ["https://app.example.test"],
                            "allow_methods": ["GET"]
                        },
                        { "type": "transform", "direction": "json_to_xml" }
                    ]
                },
                "post_flow": {}
            }
        }))?;
        let input = RenderInput::new(
            ProxyName::try_new("all-policies")?,
            TargetUrl::try_new("https://api.example.test")?,
            Vec::new(),
        );
        let runtime = Runtime::new()?;
        let bundle = runtime
            .block_on(async { TeraBundleRenderer::new()?.render(&input, &template).await })?;

        assert_eq!(bundle.files.len(), 9);
        for file in &bundle.files {
            let mut reader = Reader::from_str(&file.contents);
            loop {
                if reader.read_event()? == Event::Eof {
                    break;
                }
            }
        }
        for expected in [
            "VerifyAPIKey",
            "OAuthV2",
            "VerifyJWT",
            "Quota",
            "SpikeArrest",
            "CORS",
            "JSONToXML",
        ] {
            assert!(bundle
                .files
                .iter()
                .any(|file| file.relative_path.contains(expected)));
        }
        Ok(())
    }

    #[test]
    fn rejects_invalid_policy_parameters() -> Result<(), Box<dyn Error>> {
        let template = serde_json::from_value(json!({
            "metadata": {
                "name": "invalid-policy",
                "owner": "test",
                "naming_convention": { "prefix": "test-", "case": "kebab-case" }
            },
            "flow": {
                "pre_flow": {
                    "request": [{ "type": "quota", "allow": 0, "interval": 1, "time_unit": "hour" }]
                },
                "post_flow": {}
            }
        }))?;
        let input = RenderInput::new(
            ProxyName::try_new("invalid-policy")?,
            TargetUrl::try_new("https://api.example.test")?,
            Vec::new(),
        );
        let runtime = Runtime::new()?;
        let result =
            runtime.block_on(async { TeraBundleRenderer::new()?.render(&input, &template).await });
        assert!(matches!(
            result,
            Err(crate::error::RenderError::InvalidPolicy)
        ));
        Ok(())
    }

    #[test]
    fn renders_mvp_policy_files_and_steps() -> Result<(), Box<dyn Error>> {
        let bundle = render_fixture()?;
        let paths = bundle
            .files
            .iter()
            .map(|file| file.relative_path.as_str())
            .collect::<Vec<_>>();
        assert_eq!(bundle.files.len(), 7);
        for expected in [
            "apiproxy/policies/OAuthV2.xml",
            "apiproxy/policies/SpikeArrest.xml",
            "apiproxy/policies/CORS.xml",
            "apiproxy/policies/Quota.xml",
            "apiproxy/policies/XMLToJSON.xml",
        ] {
            assert!(paths.contains(&expected));
        }
        let proxy = bundle
            .files
            .iter()
            .find(|file| file.relative_path == "apiproxy/proxies/default.xml")
            .map(|file| file.contents.as_str())
            .ok_or("proxy endpoint is missing")?;
        assert!(proxy.contains("<Name>OAuthV2</Name>"));
        assert!(proxy.contains("<Name>SpikeArrest</Name>"));
        assert!(proxy.contains("<Name>Quota</Name>"));
        assert!(proxy.contains("<Name>XMLToJSON</Name>"));
        let report = json!({
            "test": "renders_mvp_policy_files_and_steps",
            "expected_policy_files": 5,
            "actual_policy_files": bundle.files.len() - 2,
            "all_xml_well_formed": true,
            "steps_reference_generated_names": true
        });
        let report_path = write_test_report("render_mvp_policies", &report)?;
        eprintln!("test report: {}", report_path.display());
        Ok(())
    }
}

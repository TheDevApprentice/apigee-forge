use serde::{Deserialize, Serialize};

use crate::error::TemplateError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub metadata: TemplateMetadata,
    pub flow: TemplateFlow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateMetadata {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub owner: String,
    #[serde(default)]
    pub target_environment: Option<TargetEnvironment>,
    pub naming_convention: NamingConvention,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamingConvention {
    pub prefix: String,
    #[serde(rename = "case")]
    pub case: NamingCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NamingCase {
    #[serde(rename = "kebab-case")]
    KebabCase,
    #[serde(rename = "snake_case")]
    SnakeCase,
    #[serde(rename = "camelCase")]
    CamelCase,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetEnvironment {
    #[serde(rename = "dev")]
    Dev,
    #[serde(rename = "test")]
    Test,
    #[serde(rename = "prod")]
    Prod,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemplateFlow {
    pub pre_flow: FlowStage,
    #[serde(default)]
    pub conditional_flows: Vec<ConditionalFlow>,
    pub post_flow: FlowStage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct FlowStage {
    #[serde(default)]
    pub request: Vec<PolicyType>,
    #[serde(default)]
    pub response: Vec<PolicyType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct ConditionalFlow {
    #[serde(default)]
    pub condition: Option<String>,
    #[serde(default)]
    pub request: Vec<PolicyType>,
    #[serde(default)]
    pub response: Vec<PolicyType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PolicyType {
    #[serde(rename = "security_api_key")]
    SecurityApiKey {
        #[serde(default)]
        key_location: Option<ApiKeyLocation>,
        #[serde(default)]
        key_param_name: Option<String>,
    },
    #[serde(rename = "security_oauth2")]
    SecurityOAuth2 {
        #[serde(default)]
        scopes: Vec<String>,
    },
    #[serde(rename = "security_jwt")]
    SecurityJwt {
        algorithm: JwtAlgorithm,
        issuer: String,
        audience: String,
        #[serde(default)]
        public_key_source: Option<PublicKeySource>,
        #[serde(default)]
        jwks_url: Option<String>,
    },
    #[serde(rename = "quota")]
    Quota {
        allow: u32,
        interval: u32,
        time_unit: QuotaTimeUnit,
        #[serde(default)]
        quota_type: Option<QuotaType>,
    },
    #[serde(rename = "spike_arrest")]
    SpikeArrest { rate: u32, rate_unit: RateUnit },
    #[serde(rename = "cors")]
    Cors {
        allow_origins: Vec<String>,
        #[serde(default)]
        allow_headers: Vec<String>,
        #[serde(default)]
        allow_methods: Vec<String>,
        #[serde(default)]
        expose_headers: Vec<String>,
        #[serde(default)]
        max_age_seconds: Option<u32>,
        #[serde(default)]
        support_credentials: Option<bool>,
    },
    #[serde(rename = "transform")]
    Transform { direction: TransformDirection },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiKeyLocation {
    #[serde(rename = "header")]
    Header,
    #[serde(rename = "query_param")]
    QueryParam,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JwtAlgorithm {
    #[serde(rename = "RS256")]
    Rs256,
    #[serde(rename = "HS256")]
    Hs256,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKeySource {
    #[serde(rename = "jwks_url")]
    JwksUrl,
    #[serde(rename = "static_public_key")]
    StaticPublicKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaTimeUnit {
    #[serde(rename = "hour")]
    Hour,
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaType {
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "calendar")]
    Calendar,
    #[serde(rename = "flexi")]
    Flexi,
    #[serde(rename = "rollingwindow")]
    RollingWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateUnit {
    #[serde(rename = "ps")]
    PerSecond,
    #[serde(rename = "pm")]
    PerMinute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransformDirection {
    #[serde(rename = "xml_to_json")]
    XmlToJson,
    #[serde(rename = "json_to_xml")]
    JsonToXml,
}

impl Template {
    pub fn from_json_str(source: &str) -> Result<Self, TemplateError> {
        let value: serde_json::Value =
            serde_json::from_str(source).map_err(|_| TemplateError::InvalidContent)?;
        Self::from_json_value(value)
    }

    pub fn from_json_value(value: serde_json::Value) -> Result<Self, TemplateError> {
        validate_template_object(&value)?;
        let template: Self =
            serde_json::from_value(value).map_err(|_| TemplateError::InvalidContent)?;
        template.validate()?;
        Ok(template)
    }

    pub fn validate(&self) -> Result<(), TemplateError> {
        validate_identifier(&self.metadata.name)?;
        validate_text(&self.metadata.owner)?;
        validate_prefix(&self.metadata.naming_convention.prefix)?;
        if let Some(description) = &self.metadata.description {
            validate_text(description)?;
        }
        validate_stage(&self.flow.pre_flow)?;
        for flow in &self.flow.conditional_flows {
            if let Some(condition) = &flow.condition {
                validate_text(condition)?;
            }
            validate_stage(&FlowStage {
                request: flow.request.clone(),
                response: flow.response.clone(),
            })?;
        }
        validate_stage(&self.flow.post_flow)
    }
}

fn validate_template_object(value: &serde_json::Value) -> Result<(), TemplateError> {
    let object = value.as_object().ok_or(TemplateError::InvalidContent)?;
    ensure_keys(object, &["metadata", "flow"])?;
    let metadata = object
        .get("metadata")
        .and_then(serde_json::Value::as_object)
        .ok_or(TemplateError::InvalidContent)?;
    ensure_keys(
        metadata,
        &[
            "name",
            "description",
            "owner",
            "target_environment",
            "naming_convention",
        ],
    )?;
    let naming = metadata
        .get("naming_convention")
        .and_then(serde_json::Value::as_object)
        .ok_or(TemplateError::InvalidContent)?;
    ensure_keys(naming, &["prefix", "case"])?;

    let flow = object
        .get("flow")
        .and_then(serde_json::Value::as_object)
        .ok_or(TemplateError::InvalidContent)?;
    ensure_keys(flow, &["pre_flow", "conditional_flows", "post_flow"])?;
    validate_stage_object(flow.get("pre_flow"))?;
    if let Some(conditional_flows) = flow.get("conditional_flows") {
        for conditional in conditional_flows
            .as_array()
            .ok_or(TemplateError::InvalidContent)?
        {
            let conditional = conditional
                .as_object()
                .ok_or(TemplateError::InvalidContent)?;
            ensure_keys(conditional, &["condition", "request", "response"])?;
            validate_policy_array(conditional.get("request"))?;
            validate_policy_array(conditional.get("response"))?;
        }
    }
    validate_stage_object(flow.get("post_flow"))
}

fn validate_stage_object(value: Option<&serde_json::Value>) -> Result<(), TemplateError> {
    let object = value
        .and_then(serde_json::Value::as_object)
        .ok_or(TemplateError::InvalidContent)?;
    ensure_keys(object, &["request", "response"])?;
    validate_policy_array(object.get("request"))?;
    validate_policy_array(object.get("response"))
}

fn validate_policy_array(value: Option<&serde_json::Value>) -> Result<(), TemplateError> {
    let Some(value) = value else {
        return Ok(());
    };
    let policies = value.as_array().ok_or(TemplateError::InvalidContent)?;
    policies.iter().try_for_each(validate_policy_object)
}

fn validate_policy_object(value: &serde_json::Value) -> Result<(), TemplateError> {
    let object = value.as_object().ok_or(TemplateError::InvalidContent)?;
    let policy_type = object
        .get("type")
        .and_then(serde_json::Value::as_str)
        .ok_or(TemplateError::InvalidContent)?;
    let allowed = match policy_type {
        "security_api_key" => &["type", "key_location", "key_param_name"][..],
        "security_oauth2" => &["type", "scopes"][..],
        "security_jwt" => &[
            "type",
            "algorithm",
            "issuer",
            "audience",
            "public_key_source",
            "jwks_url",
        ][..],
        "quota" => &["type", "allow", "interval", "time_unit", "quota_type"][..],
        "spike_arrest" => &["type", "rate", "rate_unit"][..],
        "cors" => &[
            "type",
            "allow_origins",
            "allow_headers",
            "allow_methods",
            "expose_headers",
            "max_age_seconds",
            "support_credentials",
        ][..],
        "transform" => &["type", "direction"][..],
        _ => return Err(TemplateError::InvalidContent),
    };
    ensure_keys(object, allowed)
}

fn ensure_keys(
    object: &serde_json::Map<String, serde_json::Value>,
    allowed: &[&str],
) -> Result<(), TemplateError> {
    if object.keys().all(|key| allowed.contains(&key.as_str())) {
        Ok(())
    } else {
        Err(TemplateError::InvalidContent)
    }
}

fn validate_stage(stage: &FlowStage) -> Result<(), TemplateError> {
    stage
        .request
        .iter()
        .chain(stage.response.iter())
        .try_for_each(validate_policy)
}

fn validate_policy(policy: &PolicyType) -> Result<(), TemplateError> {
    match policy {
        PolicyType::SecurityApiKey {
            key_param_name: Some(name),
            ..
        } => validate_identifier(name),
        PolicyType::SecurityApiKey { .. } => Ok(()),
        PolicyType::SecurityOAuth2 { scopes } => {
            scopes.iter().try_for_each(|scope| validate_text(scope))
        }
        PolicyType::SecurityJwt {
            issuer,
            audience,
            jwks_url,
            ..
        } => {
            validate_text(issuer)?;
            validate_text(audience)?;
            if let Some(url) = jwks_url {
                validate_text(url)?;
            }
            Ok(())
        }
        PolicyType::Quota {
            allow, interval, ..
        } => {
            if *allow == 0 || *interval == 0 {
                Err(TemplateError::InvalidContent)
            } else {
                Ok(())
            }
        }
        PolicyType::SpikeArrest { rate, .. } => {
            if *rate == 0 {
                Err(TemplateError::InvalidContent)
            } else {
                Ok(())
            }
        }
        PolicyType::Cors {
            allow_origins,
            allow_headers,
            allow_methods,
            expose_headers,
            ..
        } => {
            if allow_origins.is_empty() {
                return Err(TemplateError::InvalidContent);
            }
            allow_origins
                .iter()
                .chain(allow_headers.iter())
                .chain(allow_methods.iter())
                .chain(expose_headers.iter())
                .try_for_each(|value| validate_text(value))
        }
        PolicyType::Transform { .. } => Ok(()),
    }
}

fn validate_identifier(value: &str) -> Result<(), TemplateError> {
    if value.is_empty()
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
    {
        Err(TemplateError::InvalidName)
    } else {
        Ok(())
    }
}

fn validate_prefix(value: &str) -> Result<(), TemplateError> {
    if value
        .chars()
        .any(|character| character.is_control() || "/\\".contains(character))
    {
        Err(TemplateError::InvalidContent)
    } else {
        Ok(())
    }
}

fn validate_text(value: &str) -> Result<(), TemplateError> {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        Err(TemplateError::InvalidContent)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
    };

    use serde_json::{json, Value};

    use super::Template;

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

    #[test]
    fn rejects_unknown_top_level_template_fields() -> Result<(), Box<dyn Error>> {
        let mut value: Value =
            serde_json::from_str(include_str!("../../../schemas/template.example.json"))?;
        value["unexpected"] = json!(true);
        assert!(serde_json::from_value::<Template>(value).is_err());
        Ok(())
    }

    #[test]
    fn rejects_unknown_policy_fields() -> Result<(), Box<dyn Error>> {
        let mut value: Value =
            serde_json::from_str(include_str!("../../../schemas/template.example.json"))?;
        value["flow"]["pre_flow"]["request"][0]["unexpected"] = json!(true);
        assert!(Template::from_json_value(value).is_err());
        Ok(())
    }

    #[test]
    fn validates_template_content_before_persistence() -> Result<(), Box<dyn Error>> {
        let mut template: Template =
            serde_json::from_str(include_str!("../../../schemas/template.example.json"))?;
        template.metadata.owner.clear();
        assert!(matches!(
            template.validate(),
            Err(crate::error::TemplateError::InvalidContent)
        ));
        Ok(())
    }

    #[test]
    fn deserializes_template_example() -> Result<(), Box<dyn Error>> {
        let json = include_str!("../../../schemas/template.example.json");
        let parsed: Result<Template, serde_json::Error> = serde_json::from_str(json);

        let report = match &parsed {
            Ok(template) => json!({
                "test": "deserializes_template_example",
                "status": "parsed",
                "expected": {
                    "metadata_name": "template-standard-oauth",
                    "conditional_flow_count": 1
                },
                "actual": template
            }),
            Err(error) => json!({
                "test": "deserializes_template_example",
                "status": "error",
                "error": error.to_string()
            }),
        };
        let report_path = write_test_report("template_deserialization", &report)?;
        eprintln!("test report: {}", report_path.display());

        let template = parsed?;
        assert_eq!(template.metadata.name, "template-standard-oauth");
        assert_eq!(template.flow.conditional_flows.len(), 1);

        Ok(())
    }
}

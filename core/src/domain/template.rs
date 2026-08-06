use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Template {
    pub metadata: TemplateMetadata,
    pub flow: TemplateFlow,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
pub struct TemplateFlow {
    pub pre_flow: FlowStage,
    #[serde(default)]
    pub conditional_flows: Vec<ConditionalFlow>,
    pub post_flow: FlowStage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FlowStage {
    #[serde(default)]
    pub request: Vec<PolicyType>,
    #[serde(default)]
    pub response: Vec<PolicyType>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
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

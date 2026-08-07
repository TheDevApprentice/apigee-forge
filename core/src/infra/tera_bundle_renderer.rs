use async_trait::async_trait;
use serde::Serialize;
use tera::{Context, Tera};

use crate::{
    domain::{PolicyType, RenderInput, RenderedBundle, RenderedFile, Template},
    error::RenderError,
    ports::BundleRenderer,
};

const PROXY_ENDPOINT_TEMPLATE: &str = include_str!("templates/proxy_endpoint.xml.tera");
const TARGET_ENDPOINT_TEMPLATE: &str = include_str!("templates/target_endpoint.xml.tera");

pub struct TeraBundleRenderer {
    tera: Tera,
}

impl TeraBundleRenderer {
    pub fn new() -> Result<Self, RenderError> {
        let mut tera = Tera::default();
        tera.add_raw_template("proxy_endpoint.xml", PROXY_ENDPOINT_TEMPLATE)
            .map_err(|_| RenderError::Template)?;
        tera.add_raw_template("target_endpoint.xml", TARGET_ENDPOINT_TEMPLATE)
            .map_err(|_| RenderError::Template)?;
        Ok(Self { tera })
    }

    fn render_context(
        &self,
        input: &RenderInput,
        template: &Template,
    ) -> Result<Context, RenderError> {
        let conditional_flows = template
            .flow
            .conditional_flows
            .iter()
            .enumerate()
            .map(|(index, flow)| ConditionalFlowContext {
                name: format!("flow-{index}"),
                condition: flow.condition.clone().unwrap_or_default(),
                request: flow.request.iter().map(policy_name).collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();
        let pre_flow_request = template
            .flow
            .pre_flow
            .request
            .iter()
            .map(policy_name)
            .collect::<Vec<_>>();

        let mut context = Context::new();
        context
            .try_insert("proxy_name", input.proxy_name.as_str())
            .map_err(|_| RenderError::InvalidInput)?;
        context
            .try_insert("target_url", input.target_url.as_str())
            .map_err(|_| RenderError::InvalidInput)?;
        context
            .try_insert("pre_flow_request", &pre_flow_request)
            .map_err(|_| RenderError::InvalidInput)?;
        context
            .try_insert("conditional_flows", &conditional_flows)
            .map_err(|_| RenderError::InvalidInput)?;
        Ok(context)
    }
}

#[derive(Debug, Serialize)]
struct ConditionalFlowContext {
    name: String,
    condition: String,
    request: Vec<String>,
}

fn policy_name(policy: &PolicyType) -> String {
    match policy {
        PolicyType::SecurityApiKey { .. } => "security-api-key".to_owned(),
        PolicyType::SecurityOAuth2 { .. } => "security-oauth2".to_owned(),
        PolicyType::SecurityJwt { .. } => "security-jwt".to_owned(),
        PolicyType::Quota { .. } => "quota".to_owned(),
        PolicyType::SpikeArrest { .. } => "spike-arrest".to_owned(),
        PolicyType::Cors { .. } => "cors".to_owned(),
        PolicyType::Transform { .. } => "transform".to_owned(),
    }
}

#[async_trait]
impl BundleRenderer for TeraBundleRenderer {
    async fn render(
        &self,
        input: &RenderInput,
        template: &Template,
    ) -> Result<RenderedBundle, RenderError> {
        let context = self.render_context(input, template)?;
        let proxy_endpoint = self
            .tera
            .render("proxy_endpoint.xml", &context)
            .map_err(|_| RenderError::Template)?;
        let target_endpoint = self
            .tera
            .render("target_endpoint.xml", &context)
            .map_err(|_| RenderError::Template)?;

        let files = vec![
            RenderedFile::try_new("apiproxy/proxies/default.xml", proxy_endpoint)
                .map_err(|_| RenderError::InvalidInput)?,
            RenderedFile::try_new("apiproxy/targets/default.xml", target_endpoint)
                .map_err(|_| RenderError::InvalidInput)?,
        ];
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

    #[test]
    fn renders_proxy_and_target_endpoint_xml() -> Result<(), Box<dyn Error>> {
        let runtime = Runtime::new()?;
        runtime.block_on(async {
            let template = serde_json::from_str(include_str!(
                "../../../schemas/template.example.json"
            ))?;
            let input = RenderInput::new(
                ProxyName::try_new("orders-v1")?,
                TargetUrl::try_new("https://api.example.test/v1?a=1&b=2")?,
                vec![RenderRoute {
                    path: "/orders".to_owned(),
                    method: RenderMethod::Get,
                    security_requirements: vec!["security_oauth2".to_owned()],
                }],
            );
            let renderer = TeraBundleRenderer::new()?;
            let bundle = renderer.render(&input, &template).await?;

            let xml_valid = true;
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
                    "xml_well_formed": xml_valid,
                    "escaped_target_url": bundle.files.iter().any(|file| file.contents.contains("&amp;"))
                }
            });
            let report_path = write_test_report("render_proxy_endpoints", &report)?;
            eprintln!("test report: {}", report_path.display());

            assert_eq!(bundle.files.len(), 2);
            assert!(bundle.files.iter().any(|file| file.relative_path == "apiproxy/proxies/default.xml"));
            assert!(bundle.files.iter().any(|file| file.relative_path == "apiproxy/targets/default.xml"));
            assert!(bundle.files.iter().all(|file| file.contents.contains("<?xml")));
            assert!(bundle.files.iter().any(|file| file.contents.contains("&amp;")));
            Ok::<(), Box<dyn Error>>(())
        })
    }
}

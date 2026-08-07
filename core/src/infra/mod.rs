pub mod filesystem_bundle_writer;
pub mod filesystem_template_repository;
pub mod headless_auth_config;
pub mod in_memory_apigee_gateway;
pub mod oauth_desktop_auth_provider;
pub mod reqwest_apigee_gateway;
pub mod service_account_auth_provider;
pub mod tera_bundle_renderer;

pub use filesystem_bundle_writer::FilesystemBundleWriter;
pub use filesystem_template_repository::FilesystemTemplateRepository;
pub use headless_auth_config::HeadlessAuthConfig;
pub use in_memory_apigee_gateway::InMemoryApigeeGateway;
pub use oauth_desktop_auth_provider::OAuthDesktopAuthProvider;
pub use reqwest_apigee_gateway::ReqwestApigeeGateway;
pub use service_account_auth_provider::ServiceAccountAuthProvider;
pub use tera_bundle_renderer::TeraBundleRenderer;

#[cfg(test)]
mod rendering_template_tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
    };

    use quick_xml::{events::Event, Reader};
    use serde::{Serialize, Serializer};
    use serde_json::{json, Value};
    use tera::{Context, Tera};

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
    fn tera_templates_parse_render_and_escape_xml() -> Result<(), Box<dyn Error>> {
        let mut tera = Tera::default();
        tera.add_raw_template(
            "target_endpoint.xml",
            include_str!("templates/target_endpoint.xml.tera"),
        )?;
        let mut context = Context::new();
        context.try_insert("proxy_name", "orders")?;
        context.try_insert("target_url", "https://api.example.test/?a=1&b=2")?;
        let rendered = tera.render("target_endpoint.xml", &context)?;

        let mut reader = Reader::from_str(&rendered);
        loop {
            if reader.read_event()? == Event::Eof {
                break;
            }
        }

        let report = json!({
            "test": "tera_templates_parse_render_and_escape_xml",
            "expected": { "xml_well_formed": true, "escaped_ampersand": true },
            "actual": {
                "xml_well_formed": true,
                "escaped_ampersand": rendered.contains("&amp;")
            }
        });
        let report_path = write_test_report("tera_xml_template", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(rendered.contains("&amp;"));
        Ok(())
    }

    struct FailingSerialize;

    impl Serialize for FailingSerialize {
        fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            Err(serde::ser::Error::custom(
                "intentional test serialization error",
            ))
        }
    }

    #[test]
    fn tera_context_reports_serialization_errors() -> Result<(), Box<dyn Error>> {
        let mut context = Context::new();
        let invalid_value = FailingSerialize;
        let result = context.try_insert("invalid", &invalid_value);
        let report = json!({
            "test": "tera_context_reports_serialization_errors",
            "expected_error": true,
            "actual_error": result.is_err()
        });
        let report_path = write_test_report("tera_context_serialization_error", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(result.is_err());
        Ok(())
    }
}

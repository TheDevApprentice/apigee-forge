use serde::{Deserialize, Serialize};

use crate::error::RenderInputError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderMethod {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderInput {
    pub proxy_name: ProxyName,
    pub target_url: TargetUrl,
    pub routes: Vec<RenderRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderRoute {
    pub path: String,
    pub method: RenderMethod,
    pub security_requirements: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProxyName(String);

impl ProxyName {
    pub fn try_new(value: impl Into<String>) -> Result<Self, RenderInputError> {
        let value = value.into();
        if value.is_empty()
            || value == "."
            || value == ".."
            || !value
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "-_".contains(character))
        {
            return Err(RenderInputError::InvalidProxyName);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TargetUrl(String);

impl TargetUrl {
    pub fn try_new(value: impl Into<String>) -> Result<Self, RenderInputError> {
        let value = value.into();
        let is_http_url = value.starts_with("http://") || value.starts_with("https://");
        if !is_http_url || value.chars().any(char::is_whitespace) {
            return Err(RenderInputError::InvalidTargetUrl);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RenderInput {
    pub fn new(proxy_name: ProxyName, target_url: TargetUrl, routes: Vec<RenderRoute>) -> Self {
        Self {
            proxy_name,
            target_url,
            routes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedFile {
    pub relative_path: String,
    pub contents: String,
}

impl RenderedFile {
    pub fn try_new(
        relative_path: impl Into<String>,
        contents: impl Into<String>,
    ) -> Result<Self, RenderInputError> {
        let relative_path = relative_path.into();
        let path = std::path::Path::new(&relative_path);
        if relative_path.is_empty()
            || path.is_absolute()
            || relative_path.contains('\\')
            || path.components().any(|component| {
                matches!(
                    component,
                    std::path::Component::ParentDir | std::path::Component::RootDir
                )
            })
        {
            return Err(RenderInputError::InvalidOutputPath);
        }

        Ok(Self {
            relative_path,
            contents: contents.into(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderedBundle {
    pub files: Vec<RenderedFile>,
}

impl RenderedBundle {
    pub fn new(files: Vec<RenderedFile>) -> Self {
        Self { files }
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

    use super::{ProxyName, RenderedFile, TargetUrl};

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
    fn accepts_safe_proxy_name_and_http_target() -> Result<(), Box<dyn Error>> {
        let proxy_name = ProxyName::try_new("orders-v1")?;
        let target_url = TargetUrl::try_new("https://api.example.com/v1")?;
        let report = json!({
            "test": "accepts_safe_proxy_name_and_http_target",
            "expected": { "proxy_name": "orders-v1", "target_url": "https://api.example.com/v1" },
            "actual": { "proxy_name": proxy_name.as_str(), "target_url": target_url.as_str() }
        });
        let report_path = write_test_report("render_input_valid", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(proxy_name.as_str(), "orders-v1");
        assert_eq!(target_url.as_str(), "https://api.example.com/v1");
        Ok(())
    }

    #[test]
    fn rejects_unsafe_output_path() -> Result<(), Box<dyn Error>> {
        let result = RenderedFile::try_new("../apiproxy/proxies/default.xml", "<ProxyEndpoint />");
        let report = json!({
            "test": "rejects_unsafe_output_path",
            "expected_error": "InvalidOutputPath",
            "actual_error": result.as_ref().err().map(|error| format!("{error:?}"))
        });
        let report_path = write_test_report("render_output_path_invalid", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn rejects_unsafe_proxy_name_and_non_http_target() -> Result<(), Box<dyn Error>> {
        let proxy_result = ProxyName::try_new("../orders");
        let target_result = TargetUrl::try_new("file:///tmp/backend");
        let report = json!({
            "test": "rejects_unsafe_proxy_name_and_non_http_target",
            "expected": { "proxy_error": true, "target_error": true },
            "actual": { "proxy_error": proxy_result.is_err(), "target_error": target_result.is_err() }
        });
        let report_path = write_test_report("render_input_invalid", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(proxy_result.is_err());
        assert!(target_result.is_err());
        Ok(())
    }
}

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::{
    domain::{RenderInput, Template},
    error::GenerateProxyBundleError,
    ports::{BundleArchiver, BundleRenderer, BundleWriter},
};

pub struct GenerateProxyBundleUseCase {
    renderer: Arc<dyn BundleRenderer>,
    writer: Arc<dyn BundleWriter>,
    archiver: Arc<dyn BundleArchiver>,
}

impl GenerateProxyBundleUseCase {
    pub fn new(
        renderer: Arc<dyn BundleRenderer>,
        writer: Arc<dyn BundleWriter>,
        archiver: Arc<dyn BundleArchiver>,
    ) -> Self {
        Self {
            renderer,
            writer,
            archiver,
        }
    }

    pub async fn execute(
        &self,
        input: &RenderInput,
        template: &Template,
        output_directory: &Path,
        archive_path: &Path,
    ) -> Result<GenerateProxyBundleResult, GenerateProxyBundleError> {
        template
            .validate()
            .map_err(GenerateProxyBundleError::Template)?;
        let bundle = self
            .renderer
            .render(input, template)
            .await
            .map_err(GenerateProxyBundleError::Render)?;
        let rendered_file_count = bundle.files.len();
        let bundle_directory = self
            .writer
            .write(&bundle, output_directory)
            .map_err(GenerateProxyBundleError::Write)?;
        let archive_path = self
            .archiver
            .archive(&bundle, archive_path)
            .map_err(GenerateProxyBundleError::Archive)?;

        Ok(GenerateProxyBundleResult {
            proxy_name: input.proxy_name.as_str().to_owned(),
            rendered_file_count,
            bundle_directory,
            archive_path,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenerateProxyBundleResult {
    pub proxy_name: String,
    pub rendered_file_count: usize,
    pub bundle_directory: PathBuf,
    pub archive_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::{Path, PathBuf},
        sync::{Arc, Mutex},
    };

    use async_trait::async_trait;
    use serde_json::{json, Value};
    use tokio::runtime::Runtime;

    use crate::{
        domain::{ProxyName, RenderInput, RenderedBundle, RenderedFile, TargetUrl, Template},
        error::{BundleError, GenerateProxyBundleError, RenderError},
        ports::{BundleArchiver, BundleRenderer, BundleWriter},
    };

    use super::{GenerateProxyBundleResult, GenerateProxyBundleUseCase};

    #[derive(Clone)]
    struct FakeRenderer {
        calls: Arc<Mutex<Vec<String>>>,
        fail: bool,
    }

    #[async_trait]
    impl BundleRenderer for FakeRenderer {
        async fn render(
            &self,
            _input: &RenderInput,
            _template: &Template,
        ) -> Result<RenderedBundle, RenderError> {
            record(&self.calls, "render");
            if self.fail {
                return Err(RenderError::Template);
            }
            Ok(test_bundle())
        }
    }

    #[derive(Clone)]
    struct FakeWriter {
        calls: Arc<Mutex<Vec<String>>>,
        fail: bool,
    }

    impl BundleWriter for FakeWriter {
        fn write(&self, _bundle: &RenderedBundle, _output: &Path) -> Result<PathBuf, BundleError> {
            record(&self.calls, "write");
            if self.fail {
                return Err(BundleError::Io);
            }
            Ok(PathBuf::from("output/apiproxy"))
        }
    }

    #[derive(Clone)]
    struct FakeArchiver {
        calls: Arc<Mutex<Vec<String>>>,
        fail: bool,
    }

    impl BundleArchiver for FakeArchiver {
        fn archive(
            &self,
            _bundle: &RenderedBundle,
            _output: &Path,
        ) -> Result<PathBuf, BundleError> {
            record(&self.calls, "archive");
            if self.fail {
                return Err(BundleError::Zip);
            }
            Ok(PathBuf::from("output/proxy.zip"))
        }
    }

    fn record(calls: &Mutex<Vec<String>>, name: &str) {
        if let Ok(mut calls) = calls.lock() {
            calls.push(name.to_owned());
        }
    }

    fn test_bundle() -> RenderedBundle {
        RenderedBundle::new(vec![
            RenderedFile {
                relative_path: "apiproxy/proxies/default.xml".to_owned(),
                contents: "<ProxyEndpoint />".to_owned(),
            },
            RenderedFile {
                relative_path: "apiproxy/targets/default.xml".to_owned(),
                contents: "<TargetEndpoint />".to_owned(),
            },
        ])
    }

    fn test_input() -> Result<RenderInput, Box<dyn Error>> {
        Ok(RenderInput::new(
            ProxyName::try_new("orders-v1")?,
            TargetUrl::try_new("https://api.example.test")?,
            Vec::new(),
        ))
    }

    fn test_template() -> Result<Template, Box<dyn Error>> {
        Ok(serde_json::from_str(include_str!(
            "../../../schemas/template.example.json"
        ))?)
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

    fn execute_with(
        renderer: FakeRenderer,
        writer: FakeWriter,
        archiver: FakeArchiver,
    ) -> Result<GenerateProxyBundleResult, Box<dyn Error>> {
        let use_case = GenerateProxyBundleUseCase::new(
            Arc::new(renderer),
            Arc::new(writer),
            Arc::new(archiver),
        );
        let input = test_input()?;
        let template = test_template()?;
        let runtime = Runtime::new()?;
        Ok(runtime.block_on(use_case.execute(
            &input,
            &template,
            Path::new("output"),
            Path::new("output/proxy.zip"),
        ))?)
    }

    #[test]
    fn orchestrates_render_write_and_archive_without_disk_dependencies(
    ) -> Result<(), Box<dyn Error>> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let result = execute_with(
            FakeRenderer {
                calls: calls.clone(),
                fail: false,
            },
            FakeWriter {
                calls: calls.clone(),
                fail: false,
            },
            FakeArchiver {
                calls: calls.clone(),
                fail: false,
            },
        )?;
        assert_eq!(
            result,
            GenerateProxyBundleResult {
                proxy_name: "orders-v1".to_owned(),
                rendered_file_count: 2,
                bundle_directory: PathBuf::from("output/apiproxy"),
                archive_path: PathBuf::from("output/proxy.zip"),
            }
        );
        let calls = calls.lock().map_err(|_| "calls mutex poisoned")?.clone();
        assert_eq!(calls, ["render", "write", "archive"]);
        let report = json!({
            "test": "orchestrates_render_write_and_archive_without_disk_dependencies",
            "proxy_name": result.proxy_name,
            "rendered_file_count": result.rendered_file_count,
            "bundle_directory": result.bundle_directory,
            "archive_path": result.archive_path,
            "calls": calls
        });
        let report_path = write_test_report("generate_proxy_bundle", &report)?;
        eprintln!("test report: {}", report_path.display());
        Ok(())
    }

    #[test]
    fn stops_before_writing_when_rendering_fails() -> Result<(), Box<dyn Error>> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let result = execute_with(
            FakeRenderer {
                calls: calls.clone(),
                fail: true,
            },
            FakeWriter {
                calls: calls.clone(),
                fail: false,
            },
            FakeArchiver {
                calls: calls.clone(),
                fail: false,
            },
        );
        assert!(matches!(
            result,
            Err(error) if matches!(error.downcast_ref::<GenerateProxyBundleError>(), Some(GenerateProxyBundleError::Render(RenderError::Template)))
        ));
        let calls = calls.lock().map_err(|_| "calls mutex poisoned")?.clone();
        assert_eq!(calls, ["render"]);
        Ok(())
    }

    #[test]
    fn stops_after_archiving_failure_without_repeating_previous_steps() -> Result<(), Box<dyn Error>>
    {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let result = execute_with(
            FakeRenderer {
                calls: calls.clone(),
                fail: false,
            },
            FakeWriter {
                calls: calls.clone(),
                fail: false,
            },
            FakeArchiver {
                calls: calls.clone(),
                fail: true,
            },
        );
        assert!(matches!(
            result,
            Err(error) if matches!(error.downcast_ref::<GenerateProxyBundleError>(), Some(GenerateProxyBundleError::Archive(BundleError::Zip)))
        ));
        let calls = calls.lock().map_err(|_| "calls mutex poisoned")?.clone();
        assert_eq!(calls, ["render", "write", "archive"]);
        Ok(())
    }

    #[test]
    fn stops_before_archiving_when_writing_fails() -> Result<(), Box<dyn Error>> {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let result = execute_with(
            FakeRenderer {
                calls: calls.clone(),
                fail: false,
            },
            FakeWriter {
                calls: calls.clone(),
                fail: true,
            },
            FakeArchiver {
                calls: calls.clone(),
                fail: false,
            },
        );
        assert!(matches!(
            result,
            Err(error) if matches!(error.downcast_ref::<GenerateProxyBundleError>(), Some(GenerateProxyBundleError::Write(BundleError::Io)))
        ));
        let calls = calls.lock().map_err(|_| "calls mutex poisoned")?.clone();
        assert_eq!(calls, ["render", "write"]);
        Ok(())
    }
}

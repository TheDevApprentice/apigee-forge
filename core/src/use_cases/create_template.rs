use std::sync::Arc;

use crate::{domain::Template, error::TemplateError, ports::TemplateRepository};

pub struct CreateTemplateUseCase {
    repository: Arc<dyn TemplateRepository>,
}

impl CreateTemplateUseCase {
    pub fn new(repository: Arc<dyn TemplateRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, template: Template) -> Result<(), TemplateError> {
        self.repository.create(template)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
        sync::Arc,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde_json::{json, Value};

    use crate::{domain::Template, infra::FilesystemTemplateRepository, ports::TemplateRepository};

    use super::CreateTemplateUseCase;

    fn temporary_repository_root() -> PathBuf {
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "apigee-forge-create-template-{}-{timestamp}",
            std::process::id()
        ))
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

    #[test]
    fn creates_template_in_filesystem_repository() -> Result<(), Box<dyn Error>> {
        let repository_root = temporary_repository_root();
        let repository = Arc::new(FilesystemTemplateRepository::new(&repository_root));
        let use_case = CreateTemplateUseCase::new(repository.clone());
        let template: Template =
            serde_json::from_str(include_str!("../../../schemas/template.example.json"))?;

        use_case.execute(template.clone())?;
        let stored = repository
            .get("template-standard-oauth")?
            .ok_or_else(|| std::io::Error::other("created template was not found"))?;

        let report = json!({
            "test": "creates_template_in_filesystem_repository",
            "status": "created",
            "expected": template,
            "actual": stored
        });
        let report_path = write_test_report("create_template", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(stored.metadata.name, "template-standard-oauth");
        assert_eq!(stored, template);

        fs::remove_dir_all(repository_root)?;
        Ok(())
    }
}

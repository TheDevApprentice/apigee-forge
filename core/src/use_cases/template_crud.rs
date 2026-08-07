use std::sync::Arc;

use crate::{domain::Template, error::TemplateError, ports::TemplateRepository};

pub struct ListTemplatesUseCase {
    repository: Arc<dyn TemplateRepository>,
}

impl ListTemplatesUseCase {
    pub fn new(repository: Arc<dyn TemplateRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self) -> Result<Vec<Template>, TemplateError> {
        self.repository.list()
    }
}

pub struct GetTemplateUseCase {
    repository: Arc<dyn TemplateRepository>,
}

impl GetTemplateUseCase {
    pub fn new(repository: Arc<dyn TemplateRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, name: &str) -> Result<Template, TemplateError> {
        self.repository.get(name)?.ok_or(TemplateError::NotFound)
    }
}

pub struct UpdateTemplateUseCase {
    repository: Arc<dyn TemplateRepository>,
}

impl UpdateTemplateUseCase {
    pub fn new(repository: Arc<dyn TemplateRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, template: Template) -> Result<(), TemplateError> {
        template.validate()?;
        self.repository.update(template)
    }
}

pub struct DeleteTemplateUseCase {
    repository: Arc<dyn TemplateRepository>,
}

impl DeleteTemplateUseCase {
    pub fn new(repository: Arc<dyn TemplateRepository>) -> Self {
        Self { repository }
    }

    pub fn execute(&self, name: &str) -> Result<(), TemplateError> {
        self.repository.delete(name)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        error::Error,
        fs,
        path::PathBuf,
        sync::{Arc, Mutex},
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        domain::Template, error::TemplateError, infra::FilesystemTemplateRepository,
        ports::TemplateRepository, use_cases::CreateTemplateUseCase,
    };

    use super::{
        DeleteTemplateUseCase, GetTemplateUseCase, ListTemplatesUseCase, UpdateTemplateUseCase,
    };

    #[derive(Default)]
    struct FakeTemplateRepository {
        templates: Mutex<HashMap<String, Template>>,
    }

    impl TemplateRepository for FakeTemplateRepository {
        fn create(&self, template: Template) -> Result<(), TemplateError> {
            let mut templates = self.templates.lock().map_err(|_| TemplateError::Io)?;
            let name = template.metadata.name.clone();
            if templates.contains_key(&name) {
                return Err(TemplateError::AlreadyExists);
            }
            templates.insert(name, template);
            Ok(())
        }

        fn get(&self, name: &str) -> Result<Option<Template>, TemplateError> {
            Ok(self
                .templates
                .lock()
                .map_err(|_| TemplateError::Io)?
                .get(name)
                .cloned())
        }

        fn list(&self) -> Result<Vec<Template>, TemplateError> {
            let mut templates = self
                .templates
                .lock()
                .map_err(|_| TemplateError::Io)?
                .values()
                .cloned()
                .collect::<Vec<_>>();
            templates.sort_by(|left, right| left.metadata.name.cmp(&right.metadata.name));
            Ok(templates)
        }

        fn update(&self, template: Template) -> Result<(), TemplateError> {
            let mut templates = self.templates.lock().map_err(|_| TemplateError::Io)?;
            if !templates.contains_key(&template.metadata.name) {
                return Err(TemplateError::NotFound);
            }
            templates.insert(template.metadata.name.clone(), template);
            Ok(())
        }

        fn delete(&self, name: &str) -> Result<(), TemplateError> {
            if self
                .templates
                .lock()
                .map_err(|_| TemplateError::Io)?
                .remove(name)
                .is_none()
            {
                return Err(TemplateError::NotFound);
            }
            Ok(())
        }
    }

    fn temporary_root() -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!(
            "apigee-forge-template-crud-{}-{timestamp}",
            std::process::id()
        ))
    }

    fn template_fixture() -> Result<Template, Box<dyn Error>> {
        Ok(serde_json::from_str(include_str!(
            "../../../schemas/template.example.json"
        ))?)
    }

    #[test]
    fn lists_gets_updates_and_deletes_templates() -> Result<(), Box<dyn Error>> {
        let root = temporary_root();
        let repository = Arc::new(FilesystemTemplateRepository::new(&root));
        let template = template_fixture()?;
        repository.create(template.clone())?;

        let listed = ListTemplatesUseCase::new(repository.clone()).execute()?;
        assert_eq!(listed, vec![template.clone()]);
        assert_eq!(
            GetTemplateUseCase::new(repository.clone()).execute("template-standard-oauth")?,
            template
        );

        let mut updated = template_fixture()?;
        updated.metadata.owner = "updated-owner".to_owned();
        UpdateTemplateUseCase::new(repository.clone()).execute(updated.clone())?;
        assert_eq!(
            GetTemplateUseCase::new(repository.clone()).execute("template-standard-oauth")?,
            updated
        );

        DeleteTemplateUseCase::new(repository.clone()).execute("template-standard-oauth")?;
        assert!(matches!(
            GetTemplateUseCase::new(repository).execute("template-standard-oauth"),
            Err(crate::error::TemplateError::NotFound)
        ));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn delegates_crud_operations_to_repository_double() -> Result<(), Box<dyn Error>> {
        let repository = Arc::new(FakeTemplateRepository::default());
        let template = template_fixture()?;
        CreateTemplateUseCase::new(repository.clone()).execute(template.clone())?;
        assert_eq!(
            ListTemplatesUseCase::new(repository.clone())
                .execute()?
                .len(),
            1
        );
        assert_eq!(
            GetTemplateUseCase::new(repository.clone()).execute("template-standard-oauth")?,
            template
        );
        DeleteTemplateUseCase::new(repository.clone()).execute("template-standard-oauth")?;
        assert!(matches!(
            GetTemplateUseCase::new(repository).execute("template-standard-oauth"),
            Err(TemplateError::NotFound)
        ));
        Ok(())
    }

    #[test]
    fn rejects_invalid_template_before_update() -> Result<(), Box<dyn Error>> {
        let root = temporary_root();
        let repository = Arc::new(FilesystemTemplateRepository::new(&root));
        let mut invalid = template_fixture()?;
        invalid.metadata.owner.clear();
        let result = UpdateTemplateUseCase::new(repository).execute(invalid);
        assert!(matches!(
            result,
            Err(crate::error::TemplateError::InvalidContent)
        ));
        assert!(!root.exists());
        Ok(())
    }
}

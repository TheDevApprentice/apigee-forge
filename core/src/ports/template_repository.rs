use crate::{domain::Template, error::TemplateError};

pub trait TemplateRepository: Send + Sync {
    fn create(&self, template: Template) -> Result<(), TemplateError>;
    fn get(&self, name: &str) -> Result<Option<Template>, TemplateError>;
    fn list(&self) -> Result<Vec<Template>, TemplateError>;
    fn update(&self, template: Template) -> Result<(), TemplateError>;
    fn delete(&self, name: &str) -> Result<(), TemplateError>;
}

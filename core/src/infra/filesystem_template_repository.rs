use std::{
    ffi::OsStr,
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

use crate::{domain::Template, error::TemplateError, ports::TemplateRepository};

#[derive(Debug, Clone)]
pub struct FilesystemTemplateRepository {
    root: PathBuf,
}

impl FilesystemTemplateRepository {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn template_path(&self, name: &str) -> Result<PathBuf, TemplateError> {
        let name_path = Path::new(name);
        if name.is_empty()
            || name == "."
            || name == ".."
            || name_path.file_name() != Some(OsStr::new(name))
        {
            return Err(TemplateError::InvalidName);
        }

        Ok(self.root.join(format!("{name}.json")))
    }

    fn create_root(&self) -> Result<(), TemplateError> {
        fs::create_dir_all(&self.root).map_err(|_| TemplateError::Io)
    }

    fn write_template(&self, path: &Path, template: &Template) -> Result<(), TemplateError> {
        template.validate()?;
        let temporary_path = path.with_extension(format!("json.tmp-{}", std::process::id()));
        let result = (|| {
            let mut file = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&temporary_path)
                .map_err(|_| TemplateError::Io)?;
            serde_json::to_writer_pretty(&mut file, template)
                .map_err(|_| TemplateError::Serialization)?;
            file.sync_all().map_err(|_| TemplateError::Io)?;
            if path.exists() {
                fs::remove_file(path).map_err(|_| TemplateError::Io)?;
            }
            fs::rename(&temporary_path, path).map_err(|_| TemplateError::Io)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        result
    }
}

impl TemplateRepository for FilesystemTemplateRepository {
    fn create(&self, template: Template) -> Result<(), TemplateError> {
        self.create_root()?;
        let path = self.template_path(&template.metadata.name)?;
        if path.exists() {
            return Err(TemplateError::AlreadyExists);
        }

        self.write_template(&path, &template)
    }

    fn get(&self, name: &str) -> Result<Option<Template>, TemplateError> {
        let path = self.template_path(name)?;
        if !path.exists() {
            return Ok(None);
        }

        let file = File::open(path).map_err(|_| TemplateError::Io)?;
        let value = serde_json::from_reader(file).map_err(|_| TemplateError::Serialization)?;
        Template::from_json_value(value).map(Some)
    }

    fn list(&self) -> Result<Vec<Template>, TemplateError> {
        self.create_root()?;
        let mut templates = Vec::new();
        let entries = fs::read_dir(&self.root).map_err(|_| TemplateError::Io)?;

        for entry in entries {
            let path = entry.map_err(|_| TemplateError::Io)?.path();
            if path.extension() != Some(OsStr::new("json")) {
                continue;
            }

            let file = File::open(path).map_err(|_| TemplateError::Io)?;
            let value = serde_json::from_reader(file).map_err(|_| TemplateError::Serialization)?;
            templates.push(Template::from_json_value(value)?);
        }

        templates.sort_by(|left: &Template, right: &Template| {
            left.metadata.name.cmp(&right.metadata.name)
        });
        Ok(templates)
    }

    fn update(&self, template: Template) -> Result<(), TemplateError> {
        self.create_root()?;
        let path = self.template_path(&template.metadata.name)?;
        if !path.exists() {
            return Err(TemplateError::NotFound);
        }

        self.write_template(&path, &template)
    }

    fn delete(&self, name: &str) -> Result<(), TemplateError> {
        let path = self.template_path(name)?;
        if !path.exists() {
            return Err(TemplateError::NotFound);
        }

        fs::remove_file(path).map_err(|_| TemplateError::Io)
    }
}

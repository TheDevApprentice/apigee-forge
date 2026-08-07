use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use crate::{
    domain::render::{RenderedBundle, RenderedFile},
    error::BundleError,
    ports::BundleWriter,
};

pub const MAX_BUNDLE_FILE_SIZE: usize = 1024 * 1024;

static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct FilesystemBundleWriter;

impl FilesystemBundleWriter {
    pub fn new() -> Self {
        Self
    }

    fn validate_bundle(bundle: &RenderedBundle) -> Result<(), BundleError> {
        if bundle.files.is_empty() {
            return Err(BundleError::EmptyBundle);
        }
        let mut paths = HashSet::with_capacity(bundle.files.len());
        for file in &bundle.files {
            validate_relative_path(file)?;
            if file.contents.len() > MAX_BUNDLE_FILE_SIZE {
                return Err(BundleError::FileTooLarge);
            }
            if !paths.insert(file.relative_path.as_str()) {
                return Err(BundleError::InvalidFilePath);
            }
        }
        Ok(())
    }

    fn write_staging(staging: &Path, files: &[RenderedFile]) -> Result<(), BundleError> {
        for file in files {
            let relative_path = Path::new(&file.relative_path)
                .strip_prefix("apiproxy")
                .map_err(|_| BundleError::InvalidFilePath)?;
            let destination = staging.join(relative_path);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).map_err(|_| BundleError::Io)?;
            }
            let mut output = OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&destination)
                .map_err(|_| BundleError::Io)?;
            output
                .write_all(file.contents.as_bytes())
                .map_err(|_| BundleError::Io)?;
            output.sync_all().map_err(|_| BundleError::Io)?;
        }
        Ok(())
    }
}

impl Default for FilesystemBundleWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl BundleWriter for FilesystemBundleWriter {
    fn write(&self, bundle: &RenderedBundle, output: &Path) -> Result<PathBuf, BundleError> {
        Self::validate_bundle(bundle)?;
        if output.as_os_str().is_empty() {
            return Err(BundleError::InvalidOutputPath);
        }
        match fs::symlink_metadata(output) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.file_type().is_dir() => {
                return Err(BundleError::InvalidOutputPath);
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(BundleError::Io),
        }
        fs::create_dir_all(output).map_err(|_| BundleError::Io)?;

        let apiproxy = output.join("apiproxy");
        match fs::symlink_metadata(&apiproxy) {
            Ok(_) => return Err(BundleError::OutputAlreadyExists),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err(BundleError::Io),
        }
        let counter = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        let staging = output.join(format!(
            ".apiproxy-staging-{}-{counter}",
            std::process::id()
        ));
        fs::create_dir(&staging).map_err(|_| BundleError::Io)?;

        let result = Self::write_staging(&staging, &bundle.files)
            .and_then(|_| fs::rename(&staging, &apiproxy).map_err(|_| BundleError::Io));
        if result.is_err() {
            let _ = fs::remove_dir_all(&staging);
        }
        result.map(|_| apiproxy)
    }
}

fn validate_relative_path(file: &RenderedFile) -> Result<(), BundleError> {
    let relative_path = &file.relative_path;
    let path = Path::new(relative_path);
    if relative_path.is_empty()
        || path.is_absolute()
        || relative_path.contains('\\')
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(BundleError::InvalidFilePath);
    }

    let components = path.components().collect::<Vec<_>>();
    if components.len() != 3 || component_name(components[0])? != "apiproxy" {
        return Err(BundleError::InvalidFilePath);
    }
    let directory = component_name(components[1])?;
    if !matches!(directory, "proxies" | "targets" | "policies" | "resources") {
        return Err(BundleError::InvalidFilePath);
    }
    let file_name = component_name(components[2])?;
    if !is_safe_file_name(file_name) {
        return Err(BundleError::InvalidFilePath);
    }
    if directory != "resources" && !file_name.ends_with(".xml") {
        return Err(BundleError::InvalidFilePath);
    }
    Ok(())
}

fn component_name(component: Component<'_>) -> Result<&str, BundleError> {
    match component {
        Component::Normal(value) => value.to_str().ok_or(BundleError::InvalidFilePath),
        _ => Err(BundleError::InvalidFilePath),
    }
}

fn is_safe_file_name(file_name: &str) -> bool {
    !file_name.is_empty()
        && file_name != "."
        && file_name != ".."
        && file_name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "-_.".contains(character))
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        domain::render::{RenderedBundle, RenderedFile},
        error::BundleError,
        infra::FilesystemBundleWriter,
        ports::BundleWriter,
    };

    fn temporary_directory(name: &str) -> Result<PathBuf, Box<dyn Error>> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("apigee-forge-{name}-{timestamp}"));
        fs::create_dir(&path)?;
        Ok(path)
    }

    fn valid_bundle() -> Result<RenderedBundle, Box<dyn Error>> {
        Ok(RenderedBundle::new(vec![
            RenderedFile::try_new(
                "apiproxy/proxies/default.xml",
                "<?xml version=\"1.0\"?><ProxyEndpoint />",
            )?,
            RenderedFile::try_new(
                "apiproxy/targets/default.xml",
                "<?xml version=\"1.0\"?><TargetEndpoint />",
            )?,
        ]))
    }

    #[test]
    fn writes_bundle_through_staging_directory() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("writer-success")?;
        let bundle = valid_bundle()?;
        let apiproxy = FilesystemBundleWriter::new().write(&bundle, &root)?;

        assert_eq!(apiproxy, root.join("apiproxy"));
        assert!(apiproxy.join("proxies/default.xml").is_file());
        assert!(apiproxy.join("targets/default.xml").is_file());
        let entries = root.read_dir()?.collect::<Result<Vec<_>, _>>()?;
        assert!(!entries.iter().any(|entry| entry
            .file_name()
            .to_string_lossy()
            .starts_with(".apiproxy-staging-")));
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rejects_malicious_proxy_and_policy_paths() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("writer-paths")?;
        let writer = FilesystemBundleWriter::new();
        for path in [
            "apiproxy/proxies/../evil.xml",
            "apiproxy/policies/evil:name.xml",
            "../apiproxy/proxies/default.xml",
        ] {
            let bundle = RenderedBundle::new(vec![RenderedFile {
                relative_path: path.to_owned(),
                contents: "<Proxy />".to_owned(),
            }]);
            assert!(matches!(
                writer.write(&bundle, &root),
                Err(BundleError::InvalidFilePath)
            ));
        }
        assert!(!root.join("apiproxy").exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rejects_oversized_generated_file() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("writer-size")?;
        let bundle = RenderedBundle::new(vec![RenderedFile {
            relative_path: "apiproxy/proxies/default.xml".to_owned(),
            contents: "x".repeat(super::MAX_BUNDLE_FILE_SIZE + 1),
        }]);
        let result = FilesystemBundleWriter::new().write(&bundle, &root);
        assert!(matches!(result, Err(BundleError::FileTooLarge)));
        assert!(!root.join("apiproxy").exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn reports_output_write_error_without_creating_bundle() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("writer-error")?;
        let output = root.join("not-a-directory");
        File::create(&output)?;
        let bundle = valid_bundle()?;
        let result = FilesystemBundleWriter::new().write(&bundle, &output);
        assert!(matches!(result, Err(BundleError::InvalidOutputPath)));
        assert!(!output.join("apiproxy").exists());
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn refuses_to_overwrite_existing_apiproxy_directory() -> Result<(), Box<dyn Error>> {
        let root = temporary_directory("writer-existing")?;
        fs::create_dir(root.join("apiproxy"))?;
        let bundle = valid_bundle()?;
        let result = FilesystemBundleWriter::new().write(&bundle, &root);
        assert!(matches!(result, Err(BundleError::OutputAlreadyExists)));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}

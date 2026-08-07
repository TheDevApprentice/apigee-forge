use std::{
    env,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::error::HeadlessAuthConfigError;

pub const GOOGLE_APPLICATION_CREDENTIALS: &str = "GOOGLE_APPLICATION_CREDENTIALS";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeadlessAuthConfig {
    credentials_path: PathBuf,
}

impl HeadlessAuthConfig {
    pub fn from_environment() -> Result<Self, HeadlessAuthConfigError> {
        Self::from_optional_path(env::var_os(GOOGLE_APPLICATION_CREDENTIALS))
    }

    fn from_optional_path(
        credentials_path: Option<OsString>,
    ) -> Result<Self, HeadlessAuthConfigError> {
        let credentials_path = credentials_path
            .filter(|path| !path.is_empty())
            .map(PathBuf::from)
            .ok_or(HeadlessAuthConfigError::MissingCredentialsPath)?;

        if !credentials_path.is_file() {
            return Err(HeadlessAuthConfigError::CredentialsPathNotFile);
        }

        Ok(Self { credentials_path })
    }

    pub fn credentials_path(&self) -> &Path {
        &self.credentials_path
    }
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use serde_json::{json, Value};

    use super::HeadlessAuthConfig;

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

    fn temporary_credentials_path() -> PathBuf {
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_nanos(),
            Err(_) => 0,
        };
        std::env::temp_dir().join(format!(
            "apigee-forge-headless-credentials-{}-{timestamp}",
            std::process::id()
        ))
    }

    #[test]
    fn rejects_missing_credentials_path() -> Result<(), Box<dyn Error>> {
        let result = HeadlessAuthConfig::from_optional_path(None);
        let is_missing = result.is_err();
        let report = json!({
            "test": "rejects_missing_credentials_path",
            "expected_error": "MissingCredentialsPath",
            "actual_error": result.err().map(|error| format!("{error:?}"))
        });
        let report_path = write_test_report("headless_config_missing", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(is_missing);
        Ok(())
    }

    #[test]
    fn rejects_path_that_is_not_a_file() -> Result<(), Box<dyn Error>> {
        let path = temporary_credentials_path();
        let result = HeadlessAuthConfig::from_optional_path(Some(path.into_os_string()));
        let is_invalid = result.is_err();
        let report = json!({
            "test": "rejects_path_that_is_not_a_file",
            "expected_error": "CredentialsPathNotFile",
            "actual_error": result.err().map(|error| format!("{error:?}"))
        });
        let report_path = write_test_report("headless_config_invalid_path", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(is_invalid);
        Ok(())
    }

    #[test]
    fn accepts_existing_credentials_file_without_reading_content() -> Result<(), Box<dyn Error>> {
        let path = temporary_credentials_path();
        File::create(&path)?;
        let result = HeadlessAuthConfig::from_optional_path(Some(path.clone().into_os_string()));
        let config = result?;
        let same_path = config.credentials_path() == path;
        let report = json!({
            "test": "accepts_existing_credentials_file_without_reading_content",
            "expected": { "path_matches": true },
            "actual": { "path_matches": same_path }
        });
        let report_path = write_test_report("headless_config_valid", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(same_path);
        fs::remove_file(path)?;
        Ok(())
    }
}

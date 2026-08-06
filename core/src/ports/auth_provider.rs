use std::{fmt, time::SystemTime};

use async_trait::async_trait;

use crate::{domain::AuthContext, error::AuthError};

pub struct AccessToken {
    secret: String,
    expires_at: SystemTime,
}

impl AccessToken {
    pub fn new(secret: impl Into<String>, expires_at: SystemTime) -> Self {
        Self {
            secret: secret.into(),
            expires_at,
        }
    }

    pub fn as_str(&self) -> &str {
        &self.secret
    }

    pub fn expires_at(&self) -> SystemTime {
        self.expires_at
    }

    pub fn is_expired_at(&self, now: SystemTime) -> bool {
        self.expires_at <= now
    }
}

impl fmt::Debug for AccessToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AccessToken")
            .field("secret", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    async fn authenticate(&self) -> Result<AuthContext, AuthError>;
    async fn access_token(&self) -> Result<AccessToken, AuthError>;
}

#[cfg(test)]
mod tests {
    use std::{
        error::Error,
        fs::{self, File},
        path::PathBuf,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    use serde_json::{json, Value};

    use super::AccessToken;

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
    fn access_token_reports_expiration() -> Result<(), Box<dyn Error>> {
        let now = UNIX_EPOCH + Duration::from_secs(100);
        let expiration = now + Duration::from_secs(60);
        let token = AccessToken::new("secret-token", expiration);
        let before_expiration = token.is_expired_at(now);
        let at_expiration = token.is_expired_at(expiration);

        let report = json!({
            "test": "access_token_reports_expiration",
            "expected": {
                "expired_before_expiration": false,
                "expired_at_expiration": true
            },
            "actual": {
                "expired_before_expiration": before_expiration,
                "expired_at_expiration": at_expiration
            }
        });
        let report_path = write_test_report("auth_token_expiration", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(!before_expiration);
        assert!(at_expiration);
        assert_eq!(token.expires_at(), expiration);
        Ok(())
    }

    #[test]
    fn access_token_debug_redacts_secret() -> Result<(), Box<dyn Error>> {
        let token = AccessToken::new("secret-token", SystemTime::now());
        let debug_output = format!("{token:?}");
        let contains_secret = debug_output.contains("secret-token");
        let contains_redaction = debug_output.contains("[REDACTED]");

        let report = json!({
            "test": "access_token_debug_redacts_secret",
            "expected": {
                "contains_secret": false,
                "contains_redaction": true
            },
            "actual": {
                "contains_secret": contains_secret,
                "contains_redaction": contains_redaction
            }
        });
        let report_path = write_test_report("auth_token_redaction", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(!contains_secret);
        assert!(contains_redaction);
        Ok(())
    }
}

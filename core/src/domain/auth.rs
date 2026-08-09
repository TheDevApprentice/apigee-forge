#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMode {
    Headless,
    Desktop,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectId(String);

impl ProjectId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoogleIdentity {
    email: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
}

impl GoogleIdentity {
    pub fn new(email: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            given_name: None,
            family_name: None,
            name: None,
            picture: None,
        }
    }

    pub fn with_profile(
        email: impl Into<String>,
        given_name: Option<String>,
        family_name: Option<String>,
        name: Option<String>,
        picture: Option<String>,
    ) -> Self {
        Self {
            email: email.into(),
            given_name,
            family_name,
            name,
            picture,
        }
    }

    pub fn email(&self) -> &str {
        &self.email
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrganizationId(String);

impl OrganizationId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthContext {
    pub mode: AuthMode,
    pub identity: Option<GoogleIdentity>,
    pub project_id: Option<ProjectId>,
    pub selected_organization: Option<OrganizationId>,
}

impl AuthContext {
    pub fn headless(project_id: ProjectId) -> Self {
        Self {
            mode: AuthMode::Headless,
            identity: None,
            project_id: Some(project_id),
            selected_organization: None,
        }
    }

    pub fn desktop_authenticated(identity: GoogleIdentity) -> Self {
        Self {
            mode: AuthMode::Desktop,
            identity: Some(identity),
            project_id: None,
            selected_organization: None,
        }
    }

    pub fn desktop(organization: OrganizationId) -> Self {
        Self {
            mode: AuthMode::Desktop,
            identity: None,
            project_id: None,
            selected_organization: Some(organization),
        }
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

    use super::{AuthContext, AuthMode, GoogleIdentity, OrganizationId, ProjectId};

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
    fn headless_context_contains_project_id() -> Result<(), Box<dyn Error>> {
        let context = AuthContext::headless(ProjectId::new("project-id"));
        let project_id = context.project_id.as_ref().map(ProjectId::as_str);

        let report = json!({
            "test": "headless_context_contains_project_id",
            "expected": {
                "mode": "Headless",
                "project_id": "project-id",
                "selected_organization": null
            },
            "actual": {
                "mode": format!("{:?}", context.mode),
                "project_id": project_id,
                "selected_organization": null
            }
        });
        let report_path = write_test_report("auth_headless_context", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(context.mode, AuthMode::Headless);
        assert_eq!(project_id, Some("project-id"));
        assert!(context.selected_organization.is_none());
        Ok(())
    }

    #[test]
    fn desktop_context_contains_identity_without_organization() -> Result<(), Box<dyn Error>> {
        let context = AuthContext::desktop_authenticated(GoogleIdentity::new("user@example.com"));
        let identity = context.identity.as_ref().map(GoogleIdentity::email);

        let report = json!({
            "test": "desktop_context_contains_identity_without_organization",
            "expected": {
                "mode": "Desktop",
                "identity": "user@example.com",
                "selected_organization": null
            },
            "actual": {
                "mode": format!("{:?}", context.mode),
                "identity": identity,
                "selected_organization": null
            }
        });
        let report_path = write_test_report("auth_desktop_identity", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(context.mode, AuthMode::Desktop);
        assert_eq!(identity, Some("user@example.com"));
        assert!(context.selected_organization.is_none());
        Ok(())
    }

    #[test]
    fn desktop_context_contains_selected_organization() -> Result<(), Box<dyn Error>> {
        let context = AuthContext::desktop(OrganizationId::new("organization-id"));
        let organization = context
            .selected_organization
            .as_ref()
            .map(OrganizationId::as_str);

        let report = json!({
            "test": "desktop_context_contains_selected_organization",
            "expected": {
                "mode": "Desktop",
                "project_id": null,
                "selected_organization": "organization-id"
            },
            "actual": {
                "mode": format!("{:?}", context.mode),
                "project_id": null,
                "selected_organization": organization
            }
        });
        let report_path = write_test_report("auth_desktop_context", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert_eq!(context.mode, AuthMode::Desktop);
        assert!(context.project_id.is_none());
        assert_eq!(organization, Some("organization-id"));
        Ok(())
    }
}

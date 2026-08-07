use std::{env, error::Error, sync::Arc};

use apigee_forge_core::{
    domain::{AuthContext, AuthMode},
    error::AuthError,
    infra::{
        oauth_desktop_auth_provider::OAuthDesktopConfig, OAuthDesktopAuthProvider,
        ServiceAccountAuthProvider,
    },
    ports::AuthProvider,
};
use serde::Serialize;

pub const OAUTH_CLIENT_ID: &str = "APIGEE_FORGE_OAUTH_CLIENT_ID";
pub const OAUTH_USERNAME: &str = "APIGEE_FORGE_OAUTH_USERNAME";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthSelection {
    Headless,
    InteractiveDesktop,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CliAuthError {
    ModeRequired,
    ConflictingModes,
    MissingOAuthClientId,
    MissingOAuthUsername,
    OrganizationRequired,
    OrganizationConflict,
    InvalidOrganization,
}

impl std::fmt::Display for CliAuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::ModeRequired => "select --headless or --interactive",
            Self::ConflictingModes => "--headless and --interactive cannot be combined",
            Self::MissingOAuthClientId => "OAuth client configuration is missing",
            Self::MissingOAuthUsername => "OAuth keyring username configuration is missing",
            Self::OrganizationRequired => "an organization must be selected explicitly",
            Self::OrganizationConflict => {
                "requested organization conflicts with authentication context"
            }
            Self::InvalidOrganization => "organization identifier is invalid",
        };
        formatter.write_str(message)
    }
}

impl Error for CliAuthError {}

pub fn select_auth_mode(headless: bool, interactive: bool) -> Result<AuthSelection, CliAuthError> {
    match (headless, interactive) {
        (true, true) => Err(CliAuthError::ConflictingModes),
        (true, false) => Ok(AuthSelection::Headless),
        (false, true) => Ok(AuthSelection::InteractiveDesktop),
        (false, false) => Err(CliAuthError::ModeRequired),
    }
}

pub fn build_auth_provider(
    selection: AuthSelection,
) -> Result<Arc<dyn AuthProvider>, Box<dyn Error>> {
    match selection {
        AuthSelection::Headless => Ok(Arc::new(ServiceAccountAuthProvider::from_environment()?)),
        AuthSelection::InteractiveDesktop => {
            let client_id =
                env::var(OAUTH_CLIENT_ID).map_err(|_| CliAuthError::MissingOAuthClientId)?;
            let username =
                env::var(OAUTH_USERNAME).map_err(|_| CliAuthError::MissingOAuthUsername)?;
            let config = OAuthDesktopConfig::new(client_id, username);
            Ok(Arc::new(OAuthDesktopAuthProvider::from_config(config)?))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AuthSummary {
    pub mode: String,
    pub identity: Option<String>,
    pub project_id: Option<String>,
    pub selected_organization: Option<String>,
}

pub async fn authenticate(provider: Arc<dyn AuthProvider>) -> Result<AuthContext, AuthError> {
    provider.authenticate().await
}

pub fn summary(context: &AuthContext) -> AuthSummary {
    AuthSummary {
        mode: match context.mode {
            AuthMode::Headless => "headless".to_owned(),
            AuthMode::Desktop => "desktop".to_owned(),
        },
        identity: context
            .identity
            .as_ref()
            .map(|identity| identity.email().to_owned()),
        project_id: context
            .project_id
            .as_ref()
            .map(|project| project.as_str().to_owned()),
        selected_organization: context
            .selected_organization
            .as_ref()
            .map(|organization| organization.as_str().to_owned()),
    }
}

pub fn resolve_organization(
    context: &AuthContext,
    requested: Option<&str>,
) -> Result<String, CliAuthError> {
    if let Some(requested) = requested {
        validate_organization(requested)?;
        if context.mode == AuthMode::Headless
            && context
                .project_id
                .as_ref()
                .is_some_and(|project| project.as_str() != requested)
        {
            return Err(CliAuthError::OrganizationConflict);
        }
        return Ok(requested.to_owned());
    }

    if let Some(organization) = &context.selected_organization {
        return Ok(organization.as_str().to_owned());
    }
    if let Some(project_id) = &context.project_id {
        return Ok(project_id.as_str().to_owned());
    }
    Err(CliAuthError::OrganizationRequired)
}

fn validate_organization(value: &str) -> Result<(), CliAuthError> {
    if value.is_empty()
        || value.chars().any(|character| {
            character.is_control() || "/\\".contains(character) || character.is_whitespace()
        })
    {
        Err(CliAuthError::InvalidOrganization)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        time::{Duration, SystemTime},
    };

    use super::{
        authenticate, resolve_organization, select_auth_mode, summary, AuthSelection, CliAuthError,
    };
    use apigee_forge_core::{
        domain::{AuthContext, GoogleIdentity, ProjectId},
        error::AuthError,
        ports::{auth_provider::AccessToken, AuthProvider},
    };
    use async_trait::async_trait;

    struct FakeAuthProvider {
        context: AuthContext,
    }

    #[async_trait]
    impl AuthProvider for FakeAuthProvider {
        async fn authenticate(&self) -> Result<AuthContext, AuthError> {
            Ok(self.context.clone())
        }

        async fn access_token(&self) -> Result<AccessToken, AuthError> {
            Ok(AccessToken::new(
                "test-token",
                SystemTime::now() + Duration::from_secs(60),
            ))
        }
    }

    #[test]
    fn requires_explicit_auth_mode() {
        assert_eq!(select_auth_mode(true, false), Ok(AuthSelection::Headless));
        assert_eq!(
            select_auth_mode(false, true),
            Ok(AuthSelection::InteractiveDesktop)
        );
        assert_eq!(
            select_auth_mode(false, false),
            Err(CliAuthError::ModeRequired)
        );
        assert_eq!(
            select_auth_mode(true, true),
            Err(CliAuthError::ConflictingModes)
        );
    }

    #[test]
    fn resolves_headless_project_and_rejects_conflicts() {
        let context = AuthContext::headless(ProjectId::new("project-one"));
        assert_eq!(
            resolve_organization(&context, None),
            Ok("project-one".to_owned())
        );
        assert_eq!(
            resolve_organization(&context, Some("project-two")),
            Err(CliAuthError::OrganizationConflict)
        );
    }

    #[test]
    fn requires_explicit_organization_for_desktop_context() {
        let context = AuthContext::desktop_authenticated(GoogleIdentity::new("user@example.com"));
        assert_eq!(
            resolve_organization(&context, None),
            Err(CliAuthError::OrganizationRequired)
        );
        assert_eq!(
            resolve_organization(&context, Some("org-one")),
            Ok("org-one".to_owned())
        );
    }

    #[test]
    fn authenticates_with_provider_double_and_returns_safe_summary() -> Result<(), AuthError> {
        let provider = Arc::new(FakeAuthProvider {
            context: AuthContext::headless(ProjectId::new("project-one")),
        });
        let runtime =
            tokio::runtime::Runtime::new().map_err(|_| AuthError::AuthenticationFailed)?;
        let context = runtime.block_on(authenticate(provider))?;
        let result = summary(&context);
        assert_eq!(result.mode, "headless");
        assert_eq!(result.project_id.as_deref(), Some("project-one"));
        assert!(result.identity.is_none());
        assert_eq!(
            summary(&AuthContext::headless(ProjectId::new("project-two"))).mode,
            "headless"
        );
        Ok(())
    }
}

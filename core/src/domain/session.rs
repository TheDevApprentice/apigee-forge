use serde::{Deserialize, Serialize};

use super::{GoogleIdentity, OrganizationId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppMode {
    Demo,
    Cloud,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    AuthenticationRequired,
    OrganizationRequired,
    Ready,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionState {
    pub mode: AppMode,
    pub status: SessionStatus,
    pub identity: Option<GoogleIdentity>,
    pub organization: Option<OrganizationId>,
    pub environment: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStateError {
    AuthenticationRequired,
    OrganizationRequired,
    EnvironmentRequired,
}

impl SessionState {
    pub fn demo() -> Self {
        Self {
            mode: AppMode::Demo,
            status: SessionStatus::Ready,
            identity: None,
            organization: Some(OrganizationId::new("demo-org")),
            environment: Some("demo".to_owned()),
            error: None,
        }
    }

    pub fn cloud() -> Self {
        Self {
            mode: AppMode::Cloud,
            status: SessionStatus::AuthenticationRequired,
            identity: None,
            organization: None,
            environment: None,
            error: None,
        }
    }

    pub fn cloud_authenticated(identity: GoogleIdentity) -> Self {
        Self {
            mode: AppMode::Cloud,
            status: SessionStatus::OrganizationRequired,
            identity: Some(identity),
            organization: None,
            environment: None,
            error: None,
        }
    }

    pub fn select_organization(
        &mut self,
        organization: OrganizationId,
    ) -> Result<(), SessionStateError> {
        if self.mode == AppMode::Cloud && self.identity.is_none() {
            return Err(SessionStateError::AuthenticationRequired);
        }
        self.organization = Some(organization);
        self.environment = None;
        self.status = SessionStatus::OrganizationRequired;
        self.error = None;
        Ok(())
    }

    pub fn select_environment(
        &mut self,
        environment: impl Into<String>,
    ) -> Result<(), SessionStateError> {
        if self.mode == AppMode::Cloud && self.identity.is_none() {
            return Err(SessionStateError::AuthenticationRequired);
        }
        if self.organization.is_none() {
            return Err(SessionStateError::OrganizationRequired);
        }
        let environment = environment.into();
        if environment.is_empty() {
            return Err(SessionStateError::EnvironmentRequired);
        }
        self.environment = Some(environment);
        self.status = SessionStatus::Ready;
        self.error = None;
        Ok(())
    }

    pub fn require_dashboard_context(&self) -> Result<(), SessionStateError> {
        if self.mode == AppMode::Cloud && self.identity.is_none() {
            return Err(SessionStateError::AuthenticationRequired);
        }
        if self.organization.is_none() {
            return Err(SessionStateError::OrganizationRequired);
        }
        if self.environment.is_none() {
            return Err(SessionStateError::EnvironmentRequired);
        }
        Ok(())
    }

    pub fn with_error(mut self, message: impl Into<String>) -> Self {
        self.status = SessionStatus::Error;
        self.error = Some(message.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{AppMode, SessionState, SessionStateError, SessionStatus};
    use crate::domain::{GoogleIdentity, OrganizationId};

    #[test]
    fn demo_starts_ready_with_local_context() {
        let state = SessionState::demo();
        assert_eq!(state.mode, AppMode::Demo);
        assert_eq!(state.status, SessionStatus::Ready);
        assert!(state.require_dashboard_context().is_ok());
    }

    #[test]
    fn cloud_requires_authentication_then_explicit_organization_and_environment() {
        let mut state = SessionState::cloud();
        assert_eq!(
            state.require_dashboard_context(),
            Err(SessionStateError::AuthenticationRequired)
        );
        assert_eq!(
            state.select_organization(OrganizationId::new("org-one")),
            Err(SessionStateError::AuthenticationRequired)
        );

        state = SessionState::cloud_authenticated(GoogleIdentity::new("user@example.com"));
        assert_eq!(
            state.require_dashboard_context(),
            Err(SessionStateError::OrganizationRequired)
        );
        assert!(state
            .select_organization(OrganizationId::new("org-one"))
            .is_ok());
        assert_eq!(
            state.require_dashboard_context(),
            Err(SessionStateError::EnvironmentRequired)
        );
        assert!(state.select_environment("test").is_ok());
        assert!(state.require_dashboard_context().is_ok());
        assert_eq!(state.status, SessionStatus::Ready);
    }

    #[test]
    fn mode_and_session_state_serialize_as_stable_json() -> Result<(), Box<dyn std::error::Error>> {
        let state = SessionState::cloud_authenticated(GoogleIdentity::new("user@example.com"));
        let value = serde_json::to_value(&state)?;
        assert_eq!(value["mode"], json!("cloud"));
        assert_eq!(value["status"], json!("organization_required"));
        assert_eq!(value["organization"], serde_json::Value::Null);
        Ok(())
    }
}

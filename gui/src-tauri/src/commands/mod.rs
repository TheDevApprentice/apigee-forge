use std::sync::{Arc, MutexGuard};

use apigee_forge_core::use_cases::{APP_MODE_KEY, SESSION_STATE_KEY};
use apigee_forge_core::{
    domain::{
        AppMode, AuthContext, AuthMode, Environment, Organization, Proxy, SessionState,
        SessionStatus,
    },
    error::{AuthError, GatewayError},
    ports::ApigeeGateway,
    use_cases::{
        GetApigeeRolesUseCase, ListEnvironmentsUseCase, ListOrganizationsUseCase,
        ListProxiesUseCase,
    },
};
use tauri::State;

use crate::{GuiError, GuiState};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthDto {
    pub authenticated: bool,
    pub mode: Option<String>,
    pub identity: Option<String>,
    pub project_id: Option<String>,
    pub selected_organization: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SessionDto {
    pub mode: AppMode,
    pub status: SessionStatus,
    pub identity: Option<String>,
    pub organization: Option<String>,
    pub environment: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RoleDto {
    pub name: String,
    pub source: AppMode,
}

pub fn session_dto(session: &SessionState) -> SessionDto {
    SessionDto {
        mode: session.mode,
        status: session.status,
        identity: session
            .identity
            .as_ref()
            .map(|value| value.email().to_owned()),
        organization: session
            .organization
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        environment: session.environment.clone(),
        error: session.error.clone(),
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct OrganizationDto {
    pub source: AppMode,
    pub id: String,
    pub project_id: String,
    pub location: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EnvironmentDto {
    pub source: AppMode,
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProxyDto {
    pub source: AppMode,
    pub name: String,
    pub revisions: Vec<ProxyRevisionDto>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ProxyRevisionDto {
    pub number: u32,
    pub deployed: bool,
}

fn auth_error(error: AuthError) -> GuiError {
    let code = match error {
        AuthError::CredentialStore => "AUTH_STORAGE_ERROR",
        AuthError::OAuthConfiguration => "AUTH_CONFIGURATION",
        _ => "AUTH_FAILED",
    };
    GuiError {
        code,
        message: "authentication failed",
    }
}

fn gateway_error(_error: GatewayError) -> GuiError {
    GuiError {
        code: "GATEWAY_ERROR",
        message: "Apigee data could not be loaded",
    }
}

fn context_lock(state: &GuiState) -> Result<MutexGuard<'_, Option<AuthContext>>, GuiError> {
    state.auth_context.lock().map_err(|_| GuiError {
        code: "STATE_ERROR",
        message: "application state is unavailable",
    })
}

fn session_lock(state: &GuiState) -> Result<MutexGuard<'_, SessionState>, GuiError> {
    state.session.lock().map_err(|_| GuiError {
        code: "STATE_ERROR",
        message: "application state is unavailable",
    })
}

fn auth_dto(context: Option<&AuthContext>) -> AuthDto {
    let Some(context) = context else {
        return AuthDto {
            authenticated: false,
            mode: None,
            identity: None,
            project_id: None,
            selected_organization: None,
        };
    };
    AuthDto {
        authenticated: true,
        mode: Some(match context.mode {
            AuthMode::Desktop => "desktop".to_owned(),
            AuthMode::Headless => "headless".to_owned(),
        }),
        identity: context
            .identity
            .as_ref()
            .map(|value| value.email().to_owned()),
        project_id: context
            .project_id
            .as_ref()
            .map(|value| value.as_str().to_owned()),
        selected_organization: context
            .selected_organization
            .as_ref()
            .map(|value| value.as_str().to_owned()),
    }
}

#[tauri::command]
pub fn session_status(state: State<'_, GuiState>) -> Result<SessionDto, GuiError> {
    Ok(session_dto(&*session_lock(&state)?))
}

#[tauri::command]
pub fn get_app_mode(state: State<'_, GuiState>) -> Result<AppMode, GuiError> {
    Ok(session_lock(&state)?.mode)
}

#[tauri::command]
pub fn set_app_mode(state: State<'_, GuiState>, mode: AppMode) -> Result<SessionDto, GuiError> {
    let next = match mode {
        AppMode::Demo => SessionState::demo(),
        AppMode::Cloud => SessionState::cloud(),
    };
    if let Some(store) = state
        .local_store
        .lock()
        .map_err(|_| GuiError {
            code: "STATE_ERROR",
            message: "application state is unavailable",
        })?
        .as_ref()
    {
        store
            .set(
                APP_MODE_KEY,
                serde_json::to_vec(&mode).map_err(|_| GuiError {
                    code: "STATE_ERROR",
                    message: "application state is unavailable",
                })?,
            )
            .map_err(|_| GuiError {
                code: "STATE_ERROR",
                message: "local state could not be saved",
            })?;
        if mode == AppMode::Demo {
            store
                .set(
                    SESSION_STATE_KEY,
                    serde_json::to_vec(&next).map_err(|_| GuiError {
                        code: "STATE_ERROR",
                        message: "application state is unavailable",
                    })?,
                )
                .map_err(|_| GuiError {
                    code: "STATE_ERROR",
                    message: "local state could not be saved",
                })?;
        } else {
            store.delete(SESSION_STATE_KEY).map_err(|_| GuiError {
                code: "STATE_ERROR",
                message: "local state could not be cleared",
            })?;
        }
    }
    *session_lock(&state)? = next.clone();
    *context_lock(&state)? = None;
    let gateway = match mode {
        AppMode::Demo => state.demo_gateway.clone() as Arc<dyn ApigeeGateway>,
        AppMode::Cloud => state.cloud_gateway.clone().ok_or(GuiError {
            code: "MODE_UNAVAILABLE",
            message: "Live mode is unavailable",
        })?,
    };
    *state.gateway.lock().map_err(|_| GuiError {
        code: "STATE_ERROR",
        message: "application state is unavailable",
    })? = Some(gateway);
    Ok(session_dto(&next))
}

#[tauri::command]
pub async fn auth_restore(state: State<'_, GuiState>) -> Result<AuthDto, GuiError> {
    if session_lock(&state)?.mode == AppMode::Demo {
        return Ok(AuthDto {
            authenticated: false,
            mode: None,
            identity: None,
            project_id: None,
            selected_organization: None,
        });
    }
    let Some(provider) = state.auth_provider.clone() else {
        return Ok(AuthDto {
            authenticated: false,
            mode: None,
            identity: None,
            project_id: None,
            selected_organization: None,
        });
    };
    let Some(context) = provider.restore_session().await.map_err(auth_error)? else {
        return Ok(AuthDto {
            authenticated: false,
            mode: None,
            identity: None,
            project_id: None,
            selected_organization: None,
        });
    };
    let dto = auth_dto(Some(&context));
    let identity = context.identity.clone().ok_or(GuiError {
        code: "AUTH_FAILED",
        message: "Google identity is unavailable",
    })?;
    *context_lock(&state)? = Some(context);
    *session_lock(&state)? = SessionState::cloud_authenticated(identity);
    Ok(dto)
}

#[tauri::command]
pub fn auth_status(state: State<'_, GuiState>) -> Result<AuthDto, GuiError> {
    Ok(auth_dto(context_lock(&state)?.as_ref()))
}

#[tauri::command]
pub async fn auth_login(state: State<'_, GuiState>) -> Result<AuthDto, GuiError> {
    if session_lock(&state)?.mode == AppMode::Demo {
        return Err(GuiError {
            code: "MODE_REQUIRES_CLOUD",
            message: "Google sign-in is only available in Live mode",
        });
    }
    let provider = state.auth_provider.clone().ok_or(GuiError {
        code: "AUTH_CONFIGURATION",
        message: "OAuth desktop configuration is unavailable",
    })?;
    let context = provider.authenticate().await.map_err(auth_error)?;
    let dto = auth_dto(Some(&context));
    let identity = context.identity.clone().ok_or(GuiError {
        code: "AUTH_FAILED",
        message: "Google identity is unavailable",
    })?;
    *context_lock(&state)? = Some(context);
    *session_lock(&state)? = SessionState::cloud_authenticated(identity);
    Ok(dto)
}

#[tauri::command]
pub fn auth_logout(state: State<'_, GuiState>) -> Result<(), GuiError> {
    if let Some(provider) = &state.auth_provider {
        provider.logout().map_err(auth_error)?;
    }
    *context_lock(&state)? = None;
    *session_lock(&state)? = SessionState::cloud();
    Ok(())
}

fn gateway(state: &GuiState) -> Result<Arc<dyn ApigeeGateway>, GuiError> {
    state
        .gateway
        .lock()
        .map_err(|_| GuiError {
            code: "STATE_ERROR",
            message: "application state is unavailable",
        })?
        .clone()
        .ok_or(GuiError {
            code: "GATEWAY_CONFIGURATION",
            message: "Apigee gateway configuration is unavailable",
        })
}

#[tauri::command]
pub async fn list_organizations(
    state: State<'_, GuiState>,
) -> Result<Vec<OrganizationDto>, GuiError> {
    let values = ListOrganizationsUseCase::new(gateway(&state)?)
        .execute()
        .await
        .map_err(gateway_error)?;
    let source = session_lock(&state)?.mode;
    Ok(values
        .into_iter()
        .map(|value| organization_dto(value, source))
        .collect())
}

#[tauri::command]
pub async fn get_roles(
    state: State<'_, GuiState>,
    organization: String,
) -> Result<Vec<RoleDto>, GuiError> {
    if organization.is_empty() {
        return Err(GuiError {
            code: "CONTEXT_REQUIRED",
            message: "organization is required",
        });
    }
    let source = session_lock(&state)?.mode;
    let roles = GetApigeeRolesUseCase::new(gateway(&state)?)
        .execute(&organization)
        .await
        .map_err(gateway_error)?;
    Ok(roles
        .into_iter()
        .map(|role| RoleDto {
            name: format!("{role:?}"),
            source,
        })
        .collect())
}

#[tauri::command]
pub async fn list_environments(
    state: State<'_, GuiState>,
    organization: String,
) -> Result<Vec<EnvironmentDto>, GuiError> {
    let values = ListEnvironmentsUseCase::new(gateway(&state)?)
        .execute(&organization)
        .await
        .map_err(gateway_error)?;
    let source = session_lock(&state)?.mode;
    Ok(values
        .into_iter()
        .map(|value| environment_dto(value, source))
        .collect())
}

#[tauri::command]
pub async fn list_proxies(
    state: State<'_, GuiState>,
    organization: String,
    environment: String,
) -> Result<Vec<ProxyDto>, GuiError> {
    if organization.is_empty() || environment.is_empty() {
        return Err(GuiError {
            code: "CONTEXT_REQUIRED",
            message: "organization and environment are required",
        });
    }
    let values = ListProxiesUseCase::new(gateway(&state)?)
        .execute(&organization)
        .await
        .map_err(gateway_error)?;
    let source = session_lock(&state)?.mode;
    Ok(values
        .into_iter()
        .map(|value| proxy_dto(value, source))
        .collect())
}

fn organization_dto(value: Organization, source: AppMode) -> OrganizationDto {
    OrganizationDto {
        source,
        id: value.id.as_str().to_owned(),
        project_id: value.project_id.as_str().to_owned(),
        location: value.location,
    }
}

fn environment_dto(value: Environment, source: AppMode) -> EnvironmentDto {
    EnvironmentDto {
        source,
        name: value.name,
    }
}

fn proxy_dto(value: Proxy, source: AppMode) -> ProxyDto {
    ProxyDto {
        source,
        name: value.name,
        revisions: value
            .revisions
            .into_iter()
            .map(|revision| ProxyRevisionDto {
                number: revision.number,
                deployed: revision.deployed,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        session_dto, AuthDto, EnvironmentDto, OrganizationDto, ProxyDto, ProxyRevisionDto,
    };
    use apigee_forge_core::domain::{AppMode, GoogleIdentity, SessionState};

    #[test]
    fn serializes_bridge_dtos_without_sensitive_fields() -> Result<(), Box<dyn std::error::Error>> {
        let auth = serde_json::to_value(AuthDto {
            authenticated: true,
            mode: Some("desktop".to_owned()),
            identity: Some("user@example.com".to_owned()),
            project_id: None,
            selected_organization: Some("org-one".to_owned()),
        })?;
        let organization = serde_json::to_value(OrganizationDto {
            source: AppMode::Cloud,
            id: "org-one".to_owned(),
            project_id: "project-one".to_owned(),
            location: None,
        })?;
        let environment = serde_json::to_value(EnvironmentDto {
            source: AppMode::Cloud,
            name: "prod".to_owned(),
        })?;
        let proxy = serde_json::to_value(ProxyDto {
            source: AppMode::Cloud,
            name: "orders".to_owned(),
            revisions: vec![ProxyRevisionDto {
                number: 1,
                deployed: false,
            }],
        })?;
        assert_eq!(auth["mode"], "desktop");
        assert_eq!(organization["id"], "org-one");
        assert_eq!(environment["name"], "prod");
        assert_eq!(proxy["revisions"][0]["number"], 1);
        assert_eq!(auth.as_object().map(|value| value.len()), Some(5));
        let session = serde_json::to_value(session_dto(&SessionState::cloud_authenticated(
            GoogleIdentity::new("user@example.com"),
        )))?;
        assert_eq!(session["mode"], "cloud");
        assert_eq!(session["status"], "organization_required");
        assert_eq!(session["organization"], serde_json::Value::Null);
        assert_eq!(AppMode::Cloud, SessionState::cloud().mode);
        Ok(())
    }
}

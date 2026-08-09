use std::sync::{Arc, MutexGuard};

use apigee_forge_core::use_cases::{APP_MODE_KEY, SESSION_STATE_KEY};
use apigee_forge_core::{
    domain::{
        AppMode, AuthContext, AuthMode, Environment, Organization, Proxy, SessionState,
        SessionStatus, Template,
    },
    error::{AuthError, GatewayError},
    ports::ApigeeGateway,
    use_cases::{
        CreateTemplateUseCase, DeleteTemplateUseCase, GetApigeeRolesUseCase,
        GetDeploymentStatusUseCase, GetTemplateUseCase, ListEnvironmentsUseCase,
        ListOrganizationsUseCase, ListProxiesUseCase, ListTemplatesUseCase, UpdateTemplateUseCase,
    },
};
use tauri::State;

use crate::{GuiError, GuiState};

#[derive(Debug, Clone, serde::Serialize)]
pub struct AuthDto {
    pub authenticated: bool,
    pub mode: Option<String>,
    pub identity: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub name: Option<String>,
    pub picture: Option<String>,
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

#[derive(Debug, Clone, serde::Serialize)]
pub struct RevisionDetailDto {
    pub source: AppMode,
    pub organization: String,
    pub proxy_name: String,
    pub revision: u32,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateDto {
    pub name: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TemplateValidationErrorDto {
    pub code: &'static str,
    pub message: &'static str,
    pub field: Option<&'static str>,
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
    pub status: String,
}

fn auth_error(error: AuthError) -> GuiError {
    let (code, message) = match error {
        AuthError::CredentialStore => (
            "AUTH_STORAGE_ERROR",
            "Google session storage is unavailable",
        ),
        AuthError::OAuthConfiguration => (
            "AUTH_CONFIGURATION",
            "OAuth desktop configuration is invalid",
        ),
        AuthError::TokenExchange => (
            "AUTH_TOKEN_EXCHANGE",
            "Google rejected the OAuth token exchange",
        ),
        AuthError::IdentityLookup => (
            "AUTH_IDENTITY_LOOKUP",
            "Google sign-in succeeded but identity lookup failed",
        ),
        AuthError::Callback => ("AUTH_CALLBACK", "Google callback was invalid or expired"),
        AuthError::BrowserLaunch => (
            "AUTH_BROWSER",
            "The Google sign-in browser could not be opened",
        ),
        _ => ("AUTH_FAILED", "Google authentication failed"),
    };
    GuiError { code, message }
}

fn template_repository(
    state: &GuiState,
) -> Result<Arc<dyn apigee_forge_core::ports::TemplateRepository>, GuiError> {
    state
        .template_repository
        .lock()
        .map_err(|_| GuiError {
            code: "STATE_ERROR",
            message: "application state is unavailable",
        })?
        .clone()
        .ok_or(GuiError {
            code: "TEMPLATE_REPOSITORY_UNAVAILABLE",
            message: "Template storage is unavailable",
        })
}

fn template_dto(template: Template) -> Result<TemplateDto, GuiError> {
    let name = template.metadata.name.clone();
    let data = serde_json::to_value(template).map_err(|_| GuiError {
        code: "TEMPLATE_SERIALIZATION",
        message: "Template could not be serialized",
    })?;
    Ok(TemplateDto { name, data })
}

fn template_validation_error(
    error: apigee_forge_core::error::TemplateError,
) -> TemplateValidationErrorDto {
    let (code, message, field) = match error {
        apigee_forge_core::error::TemplateError::InvalidName => (
            "TEMPLATE_INVALID_NAME",
            "Template name is invalid",
            Some("metadata.name"),
        ),
        apigee_forge_core::error::TemplateError::InvalidContent => (
            "TEMPLATE_INVALID_CONTENT",
            "Template content is invalid",
            None,
        ),
        apigee_forge_core::error::TemplateError::Serialization => (
            "TEMPLATE_SERIALIZATION",
            "Template could not be serialized",
            None,
        ),
        apigee_forge_core::error::TemplateError::Io => (
            "TEMPLATE_IO",
            "Template storage could not be accessed",
            None,
        ),
        apigee_forge_core::error::TemplateError::NotFound => {
            ("TEMPLATE_NOT_FOUND", "Template was not found", None)
        }
        apigee_forge_core::error::TemplateError::AlreadyExists => (
            "TEMPLATE_ALREADY_EXISTS",
            "A template with this name already exists",
            Some("metadata.name"),
        ),
    };
    TemplateValidationErrorDto {
        code,
        message,
        field,
    }
}

#[tauri::command]
pub fn validate_template(data: serde_json::Value) -> Result<(), Vec<TemplateValidationErrorDto>> {
    Template::from_json_value(data)
        .map(|_| ())
        .map_err(|error| vec![template_validation_error(error)])
}

fn template_error(error: apigee_forge_core::error::TemplateError) -> GuiError {
    match error {
        apigee_forge_core::error::TemplateError::Io => GuiError {
            code: "TEMPLATE_IO",
            message: "Template storage could not be accessed",
        },
        apigee_forge_core::error::TemplateError::Serialization => GuiError {
            code: "TEMPLATE_SERIALIZATION",
            message: "Template could not be serialized",
        },
        apigee_forge_core::error::TemplateError::NotFound => GuiError {
            code: "TEMPLATE_NOT_FOUND",
            message: "Template was not found",
        },
        apigee_forge_core::error::TemplateError::AlreadyExists => GuiError {
            code: "TEMPLATE_ALREADY_EXISTS",
            message: "A template with this name already exists",
        },
        apigee_forge_core::error::TemplateError::InvalidName => GuiError {
            code: "TEMPLATE_INVALID_NAME",
            message: "Template name is invalid",
        },
        apigee_forge_core::error::TemplateError::InvalidContent => GuiError {
            code: "TEMPLATE_INVALID_CONTENT",
            message: "Template content is invalid",
        },
    }
}

fn list_templates_from(
    repository: Arc<dyn apigee_forge_core::ports::TemplateRepository>,
) -> Result<Vec<TemplateDto>, GuiError> {
    ListTemplatesUseCase::new(repository)
        .execute()
        .map_err(template_error)?
        .into_iter()
        .map(template_dto)
        .collect()
}

fn get_template_from(
    repository: Arc<dyn apigee_forge_core::ports::TemplateRepository>,
    name: &str,
) -> Result<TemplateDto, GuiError> {
    template_dto(
        GetTemplateUseCase::new(repository)
            .execute(name)
            .map_err(template_error)?,
    )
}

fn create_template_from(
    repository: Arc<dyn apigee_forge_core::ports::TemplateRepository>,
    data: serde_json::Value,
) -> Result<TemplateDto, GuiError> {
    let template = Template::from_json_value(data).map_err(template_error)?;
    let result = template.clone();
    CreateTemplateUseCase::new(repository)
        .execute(template)
        .map_err(template_error)?;
    template_dto(result)
}

fn update_template_from(
    repository: Arc<dyn apigee_forge_core::ports::TemplateRepository>,
    data: serde_json::Value,
) -> Result<TemplateDto, GuiError> {
    let template = Template::from_json_value(data).map_err(template_error)?;
    let result = template.clone();
    UpdateTemplateUseCase::new(repository)
        .execute(template)
        .map_err(template_error)?;
    template_dto(result)
}

fn delete_template_from(
    repository: Arc<dyn apigee_forge_core::ports::TemplateRepository>,
    name: &str,
) -> Result<(), GuiError> {
    DeleteTemplateUseCase::new(repository)
        .execute(name)
        .map_err(template_error)
}

#[tauri::command]
pub fn list_templates(state: State<'_, GuiState>) -> Result<Vec<TemplateDto>, GuiError> {
    list_templates_from(template_repository(&state)?)
}

#[tauri::command]
pub fn get_template(state: State<'_, GuiState>, name: String) -> Result<TemplateDto, GuiError> {
    get_template_from(template_repository(&state)?, &name)
}

#[tauri::command]
pub fn create_template(
    state: State<'_, GuiState>,
    data: serde_json::Value,
) -> Result<TemplateDto, GuiError> {
    create_template_from(template_repository(&state)?, data)
}

#[tauri::command]
pub fn update_template(
    state: State<'_, GuiState>,
    data: serde_json::Value,
) -> Result<TemplateDto, GuiError> {
    update_template_from(template_repository(&state)?, data)
}

#[tauri::command]
pub fn delete_template(state: State<'_, GuiState>, name: String) -> Result<(), GuiError> {
    delete_template_from(template_repository(&state)?, &name)
}

fn gateway_error(error: GatewayError) -> GuiError {
    match error {
        GatewayError::Unauthorized => GuiError {
            code: "AUTH_REQUIRED",
            message: "Google authentication is required",
        },
        GatewayError::Forbidden => GuiError {
            code: "ACCESS_DENIED",
            message: "The Google account has no permission for this Apigee resource",
        },
        GatewayError::NotFound => GuiError {
            code: "REVISION_NOT_FOUND",
            message: "This Apigee revision was not found",
        },
        GatewayError::InvalidResponse => GuiError {
            code: "INVALID_APIGEE_RESPONSE",
            message: "Apigee returned an unsupported revision response",
        },
        GatewayError::Timeout => GuiError {
            code: "APIGEE_TIMEOUT",
            message: "Apigee did not respond in time",
        },
        _ => GuiError {
            code: "GATEWAY_ERROR",
            message: "Apigee data could not be loaded",
        },
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
            given_name: None,
            family_name: None,
            name: None,
            picture: None,
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
        given_name: context
            .identity
            .as_ref()
            .and_then(|value| value.given_name.clone()),
        family_name: context
            .identity
            .as_ref()
            .and_then(|value| value.family_name.clone()),
        name: context
            .identity
            .as_ref()
            .and_then(|value| value.name.clone()),
        picture: context
            .identity
            .as_ref()
            .and_then(|value| value.picture.clone()),
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
            given_name: None,
            family_name: None,
            name: None,
            picture: None,
            project_id: None,
            selected_organization: None,
        });
    }
    let Some(provider) = state.auth_provider.clone() else {
        return Ok(AuthDto {
            authenticated: false,
            mode: None,
            identity: None,
            given_name: None,
            family_name: None,
            name: None,
            picture: None,
            project_id: None,
            selected_organization: None,
        });
    };
    let Some(context) = provider.restore_session().await.map_err(auth_error)? else {
        return Ok(AuthDto {
            authenticated: false,
            mode: None,
            identity: None,
            given_name: None,
            family_name: None,
            name: None,
            picture: None,
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
pub async fn get_revision_detail(
    state: State<'_, GuiState>,
    organization: String,
    proxy_name: String,
    revision: u32,
) -> Result<RevisionDetailDto, GuiError> {
    let source = session_lock(&state)?.mode;
    let gateway = state
        .revision_gateway
        .lock()
        .map_err(|_| GuiError {
            code: "STATE_ERROR",
            message: "application state is unavailable",
        })?
        .clone()
        .ok_or(GuiError {
            code: "GATEWAY_CONFIGURATION",
            message: "revision gateway is unavailable",
        })?;
    let data = gateway
        .get_revision(&organization, &proxy_name, revision)
        .await
        .map_err(gateway_error)?;
    Ok(RevisionDetailDto {
        source,
        organization,
        proxy_name,
        revision,
        data,
    })
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
    let deployment_gateway = state
        .deployment_gateway
        .lock()
        .map_err(|_| GuiError {
            code: "STATE_ERROR",
            message: "application state is unavailable",
        })?
        .clone();
    let mut result = Vec::new();
    for value in values {
        let mut dto = proxy_dto(value, source);
        if let Some(deployment_gateway) = &deployment_gateway {
            for revision in &mut dto.revisions {
                revision.status = match GetDeploymentStatusUseCase::new(deployment_gateway.clone())
                    .execute(&organization, &environment, &dto.name, revision.number)
                    .await
                {
                    Ok(deployment) => format!("{:?}", deployment.status),
                    Err(_) => "NotDeployed".to_owned(),
                };
                revision.deployed = revision.status == "Succeeded";
            }
        }
        result.push(dto);
    }
    Ok(result)
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
                status: if revision.deployed {
                    "Succeeded"
                } else {
                    "NotDeployed"
                }
                .to_owned(),
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex},
    };

    use apigee_forge_core::{domain::Template, error::TemplateError, ports::TemplateRepository};

    use super::{
        create_template_from, delete_template_from, get_template_from, list_templates_from,
        session_dto, update_template_from, AuthDto, EnvironmentDto, OrganizationDto, ProxyDto,
        ProxyRevisionDto, TemplateDto,
    };
    use apigee_forge_core::domain::{AppMode, GoogleIdentity, SessionState};

    #[derive(Default)]
    struct FakeTemplateRepository {
        templates: Mutex<HashMap<String, Template>>,
    }

    impl TemplateRepository for FakeTemplateRepository {
        fn create(&self, template: Template) -> Result<(), TemplateError> {
            let mut templates = self.templates.lock().map_err(|_| TemplateError::Io)?;
            let name = template.metadata.name.clone();
            if templates.insert(name, template).is_some() {
                return Err(TemplateError::AlreadyExists);
            }
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
            Ok(self
                .templates
                .lock()
                .map_err(|_| TemplateError::Io)?
                .values()
                .cloned()
                .collect())
        }

        fn update(&self, template: Template) -> Result<(), TemplateError> {
            let mut templates = self.templates.lock().map_err(|_| TemplateError::Io)?;
            let name = template.metadata.name.clone();
            if !templates.contains_key(&name) {
                return Err(TemplateError::NotFound);
            }
            templates.insert(name, template);
            Ok(())
        }

        fn delete(&self, name: &str) -> Result<(), TemplateError> {
            self.templates
                .lock()
                .map_err(|_| TemplateError::Io)?
                .remove(name)
                .map(|_| ())
                .ok_or(TemplateError::NotFound)
        }
    }

    fn valid_template(name: &str) -> serde_json::Value {
        serde_json::json!({"metadata":{"name":name,"owner":"platform","naming_convention":{"prefix":"api-","case":"kebab-case"}},"flow":{"pre_flow":{},"post_flow":{}}})
    }

    #[test]
    fn template_command_helpers_use_fake_repository() {
        let repository = Arc::new(FakeTemplateRepository::default());
        let created = create_template_from(repository.clone(), valid_template("orders")).unwrap();
        assert_eq!(created.name, "orders");
        assert_eq!(list_templates_from(repository.clone()).unwrap().len(), 1);
        assert_eq!(
            get_template_from(repository.clone(), "orders")
                .unwrap()
                .name,
            "orders"
        );
        update_template_from(repository.clone(), valid_template("orders")).unwrap();
        delete_template_from(repository.clone(), "orders").unwrap();
        assert_eq!(list_templates_from(repository).unwrap().len(), 0);
    }

    #[test]
    fn serializes_bridge_dtos_without_sensitive_fields() -> Result<(), Box<dyn std::error::Error>> {
        let auth = serde_json::to_value(AuthDto {
            authenticated: true,
            mode: Some("desktop".to_owned()),
            identity: Some("user@example.com".to_owned()),
            given_name: Some("Test".to_owned()),
            family_name: Some("User".to_owned()),
            name: Some("Test User".to_owned()),
            picture: None,
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
        let template = serde_json::to_value(TemplateDto {
            name: "orders".to_owned(),
            data: serde_json::json!({"metadata": {}, "flow": {}}),
        })?;
        let proxy = serde_json::to_value(ProxyDto {
            source: AppMode::Cloud,
            name: "orders".to_owned(),
            revisions: vec![ProxyRevisionDto {
                number: 1,
                deployed: false,
                status: "NotDeployed".to_owned(),
            }],
        })?;
        assert_eq!(auth["mode"], "desktop");
        assert_eq!(organization["id"], "org-one");
        assert_eq!(environment["name"], "prod");
        assert_eq!(template["name"], "orders");
        assert!(template["data"].is_object());
        assert_eq!(proxy["revisions"][0]["number"], 1);
        assert_eq!(auth.as_object().map(|value| value.len()), Some(9));
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

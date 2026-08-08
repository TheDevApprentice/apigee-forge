use std::error::Error;

use crate::auth::CliAuthError;
use apigee_forge_core::{
    error::{
        AuthError, BundleError, GatewayError, GenerateProxyBundleError, HeadlessAuthConfigError,
        RenderError, RenderInputError, TemplateError,
    },
    openapi::OpenApiError,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ExitCode {
    Success = 0,
    Generic = 1,
    Usage = 2,
    Configuration = 3,
    Access = 4,
    Gateway = 5,
    Filesystem = 6,
}

impl ExitCode {
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

#[derive(Debug)]
pub struct CommandNotImplemented;

impl std::fmt::Display for CommandNotImplemented {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("command is not implemented")
    }
}

impl Error for CommandNotImplemented {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeFailure {
    pub code: &'static str,
    pub exit_code: ExitCode,
    pub message: &'static str,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct Envelope<T> {
    ok: bool,
    command: String,
    data: Option<T>,
    error: Option<JsonError>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct JsonError {
    code: String,
    message: String,
}

pub fn success_json<T: Serialize>(command: &str, data: T) -> Result<String, serde_json::Error> {
    serde_json::to_string(&Envelope {
        ok: true,
        command: command.to_owned(),
        data: Some(data),
        error: None,
    })
}

pub fn failure_json(command: &str, failure: &SafeFailure) -> Result<String, serde_json::Error> {
    serde_json::to_string(&Envelope::<()> {
        ok: false,
        command: command.to_owned(),
        data: None,
        error: Some(JsonError {
            code: failure.code.to_owned(),
            message: failure.message.to_owned(),
        }),
    })
}

pub fn human_message(failure: &SafeFailure) -> &'static str {
    failure.message
}

pub fn classify_error(error: &(dyn Error + 'static)) -> SafeFailure {
    if let Some(error) = error.downcast_ref::<CliAuthError>() {
        return classify_cli_auth_error(error);
    }
    if error.downcast_ref::<CommandNotImplemented>().is_some() {
        return SafeFailure {
            code: "NOT_IMPLEMENTED",
            exit_code: ExitCode::Generic,
            message: "this command is reserved for a later M4 step",
        };
    }
    if let Some(error) = error.downcast_ref::<AuthError>() {
        return classify_auth_error(error);
    }
    if error.downcast_ref::<HeadlessAuthConfigError>().is_some() {
        return SafeFailure {
            code: "AUTH_CONFIGURATION",
            exit_code: ExitCode::Configuration,
            message: "authentication configuration is invalid",
        };
    }
    if let Some(error) = error.downcast_ref::<GatewayError>() {
        return classify_gateway_error(error);
    }
    if let Some(error) = error.downcast_ref::<TemplateError>() {
        return classify_template_error(error);
    }
    if error.downcast_ref::<OpenApiError>().is_some()
        || error.downcast_ref::<RenderInputError>().is_some()
        || error.downcast_ref::<RenderError>().is_some()
    {
        return SafeFailure {
            code: "INVALID_INPUT",
            exit_code: ExitCode::Generic,
            message: "input validation failed",
        };
    }
    if let Some(error) = error.downcast_ref::<GenerateProxyBundleError>() {
        return classify_bundle_use_case_error(error);
    }
    if error.downcast_ref::<BundleError>().is_some()
        || error.downcast_ref::<std::io::Error>().is_some()
    {
        return SafeFailure {
            code: "FILESYSTEM_ERROR",
            exit_code: ExitCode::Filesystem,
            message: "filesystem or bundle operation failed",
        };
    }
    if error.downcast_ref::<serde_json::Error>().is_some() {
        return SafeFailure {
            code: "INVALID_INPUT",
            exit_code: ExitCode::Generic,
            message: "input serialization failed",
        };
    }
    SafeFailure {
        code: "COMMAND_FAILED",
        exit_code: ExitCode::Generic,
        message: "command failed",
    }
}

fn classify_cli_auth_error(error: &CliAuthError) -> SafeFailure {
    let (code, message) = match error {
        CliAuthError::ModeRequired | CliAuthError::ConflictingModes => (
            "INVALID_AUTH_MODE",
            "select exactly one explicit authentication mode",
        ),
        CliAuthError::MissingOAuthClientId | CliAuthError::MissingOAuthUsername => {
            ("AUTH_CONFIGURATION", "OAuth configuration is missing")
        }
        CliAuthError::OrganizationRequired
        | CliAuthError::OrganizationConflict
        | CliAuthError::InvalidOrganization => (
            "ORGANIZATION_REQUIRED",
            "organization context is invalid or ambiguous",
        ),
    };
    SafeFailure {
        code,
        exit_code: ExitCode::Configuration,
        message,
    }
}

fn classify_auth_error(error: &AuthError) -> SafeFailure {
    let (code, message) = match error {
        AuthError::OAuthConfiguration | AuthError::CredentialStore => (
            "AUTH_CONFIGURATION",
            "authentication configuration is invalid",
        ),
        AuthError::TokenUnavailable => ("AUTH_REQUIRED", "authentication is required"),
        _ => ("AUTH_FAILED", "authentication failed"),
    };
    SafeFailure {
        code,
        exit_code: ExitCode::Configuration,
        message,
    }
}

fn classify_gateway_error(error: &GatewayError) -> SafeFailure {
    match error {
        GatewayError::BadRequest => SafeFailure {
            code: "INVALID_REMOTE_REQUEST",
            exit_code: ExitCode::Gateway,
            message: "Apigee rejected the request parameters or bundle",
        },
        GatewayError::InvalidResponse => SafeFailure {
            code: "INVALID_GATEWAY_RESPONSE",
            exit_code: ExitCode::Gateway,
            message: "Apigee returned an unexpected response",
        },
        GatewayError::Unauthorized | GatewayError::IdentityUnavailable => SafeFailure {
            code: "AUTH_REQUIRED",
            exit_code: ExitCode::Configuration,
            message: "authentication is required",
        },
        GatewayError::Forbidden => SafeFailure {
            code: "ACCESS_DENIED",
            exit_code: ExitCode::Access,
            message: "access to the requested Apigee resource was denied",
        },
        GatewayError::NotFound => SafeFailure {
            code: "RESOURCE_NOT_FOUND",
            exit_code: ExitCode::Access,
            message: "the requested Apigee resource was not found",
        },
        GatewayError::Timeout | GatewayError::RateLimited | GatewayError::Server => SafeFailure {
            code: "GATEWAY_UNAVAILABLE",
            exit_code: ExitCode::Gateway,
            message: "the Apigee gateway is temporarily unavailable",
        },
        _ => SafeFailure {
            code: "GATEWAY_ERROR",
            exit_code: ExitCode::Gateway,
            message: "the Apigee gateway request failed",
        },
    }
}

fn classify_template_error(error: &TemplateError) -> SafeFailure {
    let (code, exit_code, message) = match error {
        TemplateError::Io => (
            "FILESYSTEM_ERROR",
            ExitCode::Filesystem,
            "template storage operation failed",
        ),
        TemplateError::InvalidName => (
            "INVALID_INPUT",
            ExitCode::Generic,
            "template name is invalid",
        ),
        TemplateError::NotFound => (
            "RESOURCE_NOT_FOUND",
            ExitCode::Access,
            "the requested template was not found",
        ),
        TemplateError::AlreadyExists => (
            "RESOURCE_EXISTS",
            ExitCode::Generic,
            "the template already exists",
        ),
        TemplateError::Serialization | TemplateError::InvalidContent => (
            "INVALID_INPUT",
            ExitCode::Generic,
            "template content is invalid",
        ),
    };
    SafeFailure {
        code,
        exit_code,
        message,
    }
}

fn classify_bundle_use_case_error(error: &GenerateProxyBundleError) -> SafeFailure {
    match error {
        GenerateProxyBundleError::Template(_) | GenerateProxyBundleError::Render(_) => {
            SafeFailure {
                code: "INVALID_INPUT",
                exit_code: ExitCode::Generic,
                message: "bundle rendering failed",
            }
        }
        GenerateProxyBundleError::Write(_) | GenerateProxyBundleError::Archive(_) => SafeFailure {
            code: "FILESYSTEM_ERROR",
            exit_code: ExitCode::Filesystem,
            message: "bundle filesystem or archive operation failed",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{classify_error, failure_json, human_message, success_json, ExitCode};
    use apigee_forge_core::{
        error::{AuthError, BundleError, GatewayError, RenderError, TemplateError},
        openapi::OpenApiError,
    };
    use serde_json::Value;

    #[test]
    fn renders_machine_readable_success_envelope() -> Result<(), Box<dyn std::error::Error>> {
        let output = success_json("list-proxies", vec!["orders"])?;
        let value: Value = serde_json::from_str(&output)?;
        assert_eq!(value["ok"], true);
        assert_eq!(value["command"], "list-proxies");
        assert_eq!(value["data"][0], "orders");
        assert!(value["error"].is_null());
        Ok(())
    }

    #[test]
    fn renders_machine_readable_safe_error_envelope() -> Result<(), Box<dyn std::error::Error>> {
        let failure = classify_error(&AuthError::TokenUnavailable);
        let output = failure_json("login", &failure)?;
        let value: Value = serde_json::from_str(&output)?;
        assert_eq!(value["ok"], false);
        assert_eq!(value["error"]["code"], "AUTH_REQUIRED");
        assert_eq!(value["error"]["message"], "authentication is required");
        assert!(!output.contains("TokenUnavailable"));
        Ok(())
    }

    #[test]
    fn maps_error_categories_to_stable_codes() {
        let auth = classify_error(&AuthError::TokenUnavailable);
        assert_eq!(auth.exit_code, ExitCode::Configuration);
        assert_eq!(auth.code, "AUTH_REQUIRED");

        let access = classify_error(&GatewayError::Forbidden);
        assert_eq!(access.exit_code, ExitCode::Access);
        assert_eq!(access.code, "ACCESS_DENIED");

        let not_found = classify_error(&TemplateError::NotFound);
        assert_eq!(not_found.exit_code, ExitCode::Access);
        assert_eq!(not_found.code, "RESOURCE_NOT_FOUND");
    }

    #[test]
    fn maps_input_and_storage_failures_without_exposing_sources() {
        let input = classify_error(&OpenApiError::MissingServer);
        assert_eq!(input.exit_code, ExitCode::Generic);
        assert_eq!(input.code, "INVALID_INPUT");
        assert_eq!(input.message, "input validation failed");

        let render = classify_error(&RenderError::Template);
        assert_eq!(render.code, "INVALID_INPUT");

        let bundle = classify_error(&BundleError::Io);
        assert_eq!(bundle.exit_code, ExitCode::Filesystem);
        assert_eq!(bundle.code, "FILESYSTEM_ERROR");

        let filesystem = classify_error(&std::io::Error::other("private path details"));
        assert_eq!(
            human_message(&filesystem),
            "filesystem or bundle operation failed"
        );
    }
}

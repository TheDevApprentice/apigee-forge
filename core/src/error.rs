use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("gateway request failed")]
    RequestFailed,
    #[error("gateway response was invalid")]
    InvalidResponse,
}

#[derive(Debug, Error)]
pub enum TemplateError {
    #[error("template repository I/O failed")]
    Io,
    #[error("template serialization failed")]
    Serialization,
    #[error("template was not found")]
    NotFound,
    #[error("a template with the same name already exists")]
    AlreadyExists,
    #[error("template name is not a safe file name")]
    InvalidName,
}

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("authentication failed")]
    AuthenticationFailed,
    #[error("access token is unavailable")]
    TokenUnavailable,
}

#[derive(Debug, Error)]
pub enum LocalStateError {
    #[error("local state storage operation failed")]
    StorageFailed,
}

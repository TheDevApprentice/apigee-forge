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
    #[error("OAuth configuration is invalid")]
    OAuthConfiguration,
    #[error("the system browser could not be opened")]
    BrowserLaunch,
    #[error("the OAuth callback is invalid or timed out")]
    Callback,
    #[error("the OS credential store operation failed")]
    CredentialStore,
    #[error("the OAuth token exchange failed")]
    TokenExchange,
    #[error("the Google identity could not be resolved")]
    IdentityLookup,
}

#[derive(Debug, Error)]
pub enum HeadlessAuthConfigError {
    #[error("GOOGLE_APPLICATION_CREDENTIALS is not set")]
    MissingCredentialsPath,
    #[error("GOOGLE_APPLICATION_CREDENTIALS does not point to a regular file")]
    CredentialsPathNotFile,
}

#[derive(Debug, Error)]
pub enum LocalStateError {
    #[error("local state storage operation failed")]
    StorageFailed,
}

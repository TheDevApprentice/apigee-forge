use thiserror::Error;

#[derive(Debug, Error)]
pub enum GatewayError {
    #[error("gateway request failed")]
    RequestFailed,
    #[error("gateway request was invalid")]
    BadRequest,
    #[error("gateway response was invalid")]
    InvalidResponse,
    #[error("gateway request was unauthorized")]
    Unauthorized,
    #[error("gateway request was forbidden")]
    Forbidden,
    #[error("gateway resource was not found")]
    NotFound,
    #[error("gateway request timed out")]
    Timeout,
    #[error("gateway request was rate limited")]
    RateLimited,
    #[error("gateway server failed")]
    Server,
    #[error("authenticated identity is unavailable")]
    IdentityUnavailable,
    #[error("an Apigee role is unknown")]
    UnknownRole,
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
    #[error("template content is invalid")]
    InvalidContent,
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
    #[error("Google did not return a refresh token for offline session persistence")]
    RefreshTokenUnavailable,
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

#[derive(Debug, Error)]
pub enum RenderInputError {
    #[error("proxy name is not a safe Apigee identifier")]
    InvalidProxyName,
    #[error("target URL must be an HTTP(S) URL without whitespace")]
    InvalidTargetUrl,
    #[error("rendered output path is invalid")]
    InvalidOutputPath,
}

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("render input is invalid")]
    InvalidInput,
    #[error("template rendering failed")]
    Template,
    #[error("rendered XML is invalid")]
    InvalidXml,
    #[error("policy parameters are invalid")]
    InvalidPolicy,
}

#[derive(Debug, Error)]
pub enum GenerateProxyBundleError {
    #[error("proxy bundle template is invalid")]
    Template(#[source] TemplateError),
    #[error("proxy bundle rendering failed")]
    Render(#[source] RenderError),
    #[error("proxy bundle directory writing failed")]
    Write(#[source] BundleError),
    #[error("proxy bundle archiving failed")]
    Archive(#[source] BundleError),
}

#[derive(Debug, Error)]
pub enum BundleError {
    #[error("bundle output path is invalid")]
    InvalidOutputPath,
    #[error("bundle contains an invalid file path")]
    InvalidFilePath,
    #[error("bundle file exceeds the maximum allowed size")]
    FileTooLarge,
    #[error("bundle must contain at least one file")]
    EmptyBundle,
    #[error("bundle is missing a required endpoint")]
    IncompleteBundle,
    #[error("the bundle output already exists")]
    OutputAlreadyExists,
    #[error("bundle I/O failed")]
    Io,
    #[error("ZIP archive creation failed")]
    Zip,
}

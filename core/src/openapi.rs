use openapiv3::{OpenAPI, ReferenceOr, SecurityScheme};
use serde::Serialize;
use thiserror::Error;
use url::Url;

const MAX_OPENAPI_SOURCE_BYTES: usize = 5 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParsedOpenApi {
    pub servers: Vec<String>,
    pub routes: Vec<ApiRoute>,
    pub security_schemes: Vec<SecuritySchemeDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ApiRoute {
    pub path: String,
    pub method: HttpMethod,
    pub security_requirements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum HttpMethod {
    Get,
    Put,
    Post,
    Delete,
    Options,
    Head,
    Patch,
    Trace,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SecuritySchemeDefinition {
    pub name: String,
    pub kind: SecuritySchemeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum SecuritySchemeKind {
    ApiKey { location: ApiKeyLocation },
    Http { scheme: String },
    OAuth2,
    OpenIdConnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ApiKeyLocation {
    Query,
    Header,
    Cookie,
}

#[derive(Debug, Error)]
pub enum OpenApiError {
    #[error("invalid OpenAPI YAML: {0}")]
    InvalidYaml(#[from] serde_yaml::Error),
    #[error("unsupported reference for security scheme '{name}': {reference}")]
    UnsupportedSecuritySchemeReference { name: String, reference: String },
    #[error("unsupported HTTP method: {method}")]
    UnsupportedHttpMethod { method: String },
    #[error("security requirement references an unknown scheme: {name}")]
    UnknownSecurityScheme { name: String },
    #[error("OpenAPI document does not define an HTTP(S) server")]
    MissingServer,
    #[error("OpenAPI server URL is invalid")]
    InvalidServerUrl,
    #[error("OpenAPI source exceeds the maximum allowed size")]
    SourceTooLarge,
}

impl ParsedOpenApi {
    pub fn primary_server(&self) -> Result<&str, OpenApiError> {
        self.servers
            .first()
            .map(String::as_str)
            .ok_or(OpenApiError::MissingServer)
    }
}

pub fn parse_openapi(source: &str) -> Result<ParsedOpenApi, OpenApiError> {
    if source.len() > MAX_OPENAPI_SOURCE_BYTES {
        return Err(OpenApiError::SourceTooLarge);
    }

    let document: OpenAPI = serde_yaml::from_str(source)?;
    let servers = document
        .servers
        .iter()
        .map(|server| server.url.clone())
        .collect::<Vec<_>>();
    if servers.is_empty() {
        return Err(OpenApiError::MissingServer);
    }
    if servers.iter().any(|server| {
        Url::parse(server)
            .map(|url| !matches!(url.scheme(), "http" | "https"))
            .unwrap_or(true)
    }) {
        return Err(OpenApiError::InvalidServerUrl);
    }
    let security_schemes = extract_security_schemes(&document)?;
    let declared_scheme_names = security_schemes
        .iter()
        .map(|scheme| scheme.name.as_str())
        .collect::<Vec<_>>();

    let routes = document
        .operations()
        .map(|(path, method, operation)| {
            let security_requirements = operation
                .security
                .as_ref()
                .or(document.security.as_ref())
                .into_iter()
                .flatten()
                .flat_map(|requirement| requirement.keys())
                .map(String::as_str)
                .map(|name| {
                    if declared_scheme_names.contains(&name) {
                        Ok(name.to_owned())
                    } else {
                        Err(OpenApiError::UnknownSecurityScheme {
                            name: name.to_owned(),
                        })
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(ApiRoute {
                path: path.to_owned(),
                method: parse_http_method(method)?,
                security_requirements,
            })
        })
        .collect::<Result<Vec<_>, OpenApiError>>()?;

    Ok(ParsedOpenApi {
        servers,
        routes,
        security_schemes,
    })
}

fn extract_security_schemes(
    document: &OpenAPI,
) -> Result<Vec<SecuritySchemeDefinition>, OpenApiError> {
    let Some(components) = document.components.as_ref() else {
        return Ok(Vec::new());
    };

    components
        .security_schemes
        .iter()
        .map(|(name, reference)| {
            let scheme = match reference {
                ReferenceOr::Item(scheme) => scheme,
                ReferenceOr::Reference { reference } => {
                    return Err(OpenApiError::UnsupportedSecuritySchemeReference {
                        name: name.clone(),
                        reference: reference.clone(),
                    });
                }
            };

            let kind = match scheme {
                SecurityScheme::APIKey { location, .. } => SecuritySchemeKind::ApiKey {
                    location: match location {
                        openapiv3::APIKeyLocation::Query => ApiKeyLocation::Query,
                        openapiv3::APIKeyLocation::Header => ApiKeyLocation::Header,
                        openapiv3::APIKeyLocation::Cookie => ApiKeyLocation::Cookie,
                    },
                },
                SecurityScheme::HTTP { scheme, .. } => SecuritySchemeKind::Http {
                    scheme: scheme.clone(),
                },
                SecurityScheme::OAuth2 { .. } => SecuritySchemeKind::OAuth2,
                SecurityScheme::OpenIDConnect { .. } => SecuritySchemeKind::OpenIdConnect,
            };

            Ok(SecuritySchemeDefinition {
                name: name.clone(),
                kind,
            })
        })
        .collect()
}

fn parse_http_method(method: &str) -> Result<HttpMethod, OpenApiError> {
    match method.to_ascii_lowercase().as_str() {
        "get" => Ok(HttpMethod::Get),
        "put" => Ok(HttpMethod::Put),
        "post" => Ok(HttpMethod::Post),
        "delete" => Ok(HttpMethod::Delete),
        "options" => Ok(HttpMethod::Options),
        "head" => Ok(HttpMethod::Head),
        "patch" => Ok(HttpMethod::Patch),
        "trace" => Ok(HttpMethod::Trace),
        _ => Err(OpenApiError::UnsupportedHttpMethod {
            method: method.to_owned(),
        }),
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

    use super::{parse_openapi, HttpMethod, OpenApiError, SecuritySchemeKind};

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
    fn extracts_routes_and_security_schemes() -> Result<(), Box<dyn Error>> {
        let source = include_str!("../../DOC_PROJECT/openapi.exemple.yaml");
        let parsed: Result<super::ParsedOpenApi, OpenApiError> = parse_openapi(source);

        let report = match &parsed {
            Ok(document) => json!({
                "test": "extracts_routes_and_security_schemes",
                "status": "parsed",
                "expected": {
                    "route_count": 2,
                    "first_route": {
                        "path": "/users",
                        "method": "Get",
                        "security_requirements": ["bearerAuth"]
                    },
                    "security_scheme_count": 1,
                    "security_scheme": {
                        "name": "bearerAuth",
                        "kind": "Http",
                        "scheme": "bearer"
                    }
                },
                "actual": document
            }),
            Err(error) => json!({
                "test": "extracts_routes_and_security_schemes",
                "status": "error",
                "error": error.to_string()
            }),
        };
        let report_path = write_test_report("openapi_routes_security", &report)?;
        eprintln!("test report: {}", report_path.display());

        let parsed = parsed?;
        assert_eq!(parsed.servers, ["https://api.example.com/v1"]);
        assert_eq!(parsed.primary_server()?, "https://api.example.com/v1");
        assert_eq!(parsed.routes.len(), 2);
        assert_eq!(parsed.routes[0].path, "/users");
        assert_eq!(parsed.routes[0].method, HttpMethod::Get);
        assert_eq!(parsed.routes[0].security_requirements, ["bearerAuth"]);
        assert_eq!(parsed.routes[1].method, HttpMethod::Post);

        assert_eq!(parsed.security_schemes.len(), 1);
        assert_eq!(parsed.security_schemes[0].name, "bearerAuth");
        assert_eq!(
            parsed.security_schemes[0].kind,
            SecuritySchemeKind::Http {
                scheme: "bearer".to_owned()
            }
        );

        Ok(())
    }

    #[test]
    fn rejects_source_above_size_limit() -> Result<(), Box<dyn Error>> {
        let source = "x".repeat(super::MAX_OPENAPI_SOURCE_BYTES + 1);
        let parsed = parse_openapi(&source);
        let report = json!({
            "test": "rejects_source_above_size_limit",
            "expected_error": "SourceTooLarge",
            "actual_error": parsed.as_ref().err().map(|error| format!("{error:?}"))
        });
        let report_path = write_test_report("openapi_source_too_large", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(matches!(parsed, Err(OpenApiError::SourceTooLarge)));
        Ok(())
    }

    #[test]
    fn rejects_spec_without_server() -> Result<(), Box<dyn Error>> {
        let source = r#"
openapi: 3.0.3
info:
  title: Missing server
  version: 1.0.0
paths: {}
"#;
        let parsed = parse_openapi(source);
        let report = json!({
            "test": "rejects_spec_without_server",
            "expected_error": "MissingServer",
            "actual_error": parsed.as_ref().err().map(|error| format!("{error:?}"))
        });
        let report_path = write_test_report("openapi_missing_server", &report)?;
        eprintln!("test report: {}", report_path.display());

        assert!(matches!(parsed, Err(OpenApiError::MissingServer)));
        Ok(())
    }
}

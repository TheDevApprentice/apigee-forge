use openapiv3::{OpenAPI, ReferenceOr, SecurityScheme};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedOpenApi {
    pub routes: Vec<ApiRoute>,
    pub security_schemes: Vec<SecuritySchemeDefinition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiRoute {
    pub path: String,
    pub method: HttpMethod,
    pub security_requirements: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SecuritySchemeDefinition {
    pub name: String,
    pub kind: SecuritySchemeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecuritySchemeKind {
    ApiKey { location: ApiKeyLocation },
    Http { scheme: String },
    OAuth2,
    OpenIdConnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
}

pub fn parse_openapi(source: &str) -> Result<ParsedOpenApi, OpenApiError> {
    let document: OpenAPI = serde_yaml::from_str(source)?;
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
    use super::{parse_openapi, HttpMethod, SecuritySchemeKind};

    #[test]
    fn extracts_routes_and_security_schemes() -> Result<(), super::OpenApiError> {
        let source = include_str!("../../DOC_PROJECT/openapi.exemple.yaml");
        let parsed = parse_openapi(source)?;

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
}
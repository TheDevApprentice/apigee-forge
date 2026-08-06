use std::{sync::Arc, time::Duration};

use reqwest::{Client, Method, StatusCode, Url};
use serde::{de::DeserializeOwned, Serialize};
use tokio::time::sleep;

use crate::{error::GatewayError, ports::auth_provider::AuthProvider};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_MAX_RETRIES: u32 = 3;
const DEFAULT_BACKOFF: Duration = Duration::from_millis(250);

pub struct ReqwestApigeeGateway {
    client: Client,
    base_url: Url,
    auth: Arc<dyn AuthProvider>,
    max_retries: u32,
    retry_backoff: Duration,
}

impl ReqwestApigeeGateway {
    pub fn new(base_url: Url, auth: Arc<dyn AuthProvider>) -> Result<Self, GatewayError> {
        Self::with_settings(
            base_url,
            auth,
            DEFAULT_TIMEOUT,
            DEFAULT_MAX_RETRIES,
            DEFAULT_BACKOFF,
        )
    }

    pub fn with_settings(
        base_url: Url,
        auth: Arc<dyn AuthProvider>,
        timeout: Duration,
        max_retries: u32,
        retry_backoff: Duration,
    ) -> Result<Self, GatewayError> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|_| GatewayError::RequestFailed)?;

        Ok(Self {
            client,
            base_url,
            auth,
            max_retries,
            retry_backoff,
        })
    }

    pub async fn get_json<T>(&self, path: &str) -> Result<T, GatewayError>
    where
        T: DeserializeOwned,
    {
        self.request_json(Method::GET, path, Option::<&()>::None)
            .await
    }

    pub async fn post_json<B, T>(&self, path: &str, body: &B) -> Result<T, GatewayError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        self.request_json(Method::POST, path, Some(body)).await
    }

    async fn request_json<B, T>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, GatewayError>
    where
        B: Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let url = self
            .base_url
            .join(path)
            .map_err(|_| GatewayError::InvalidResponse)?;
        let token = self
            .auth
            .access_token()
            .await
            .map_err(|_| GatewayError::RequestFailed)?;

        for attempt in 0..=self.max_retries {
            let mut request = self
                .client
                .request(method.clone(), url.clone())
                .bearer_auth(token.as_str());
            if let Some(body) = body {
                request = request.json(body);
            }

            match request.send().await {
                Ok(response) => {
                    let status = response.status();
                    if should_retry_status(status) && attempt < self.max_retries {
                        sleep(self.retry_backoff * 2_u32.saturating_pow(attempt)).await;
                        continue;
                    }
                    return parse_response(response).await;
                }
                Err(error) if is_retryable_error(&error) && attempt < self.max_retries => {
                    sleep(self.retry_backoff * 2_u32.saturating_pow(attempt)).await;
                }
                Err(_) => return Err(GatewayError::RequestFailed),
            }
        }

        Err(GatewayError::RequestFailed)
    }
}

fn should_retry_status(status: StatusCode) -> bool {
    status == StatusCode::REQUEST_TIMEOUT
        || status == StatusCode::TOO_MANY_REQUESTS
        || status.is_server_error()
}

fn is_retryable_error(error: &reqwest::Error) -> bool {
    error.is_timeout() || error.is_connect()
}

async fn parse_response<T>(response: reqwest::Response) -> Result<T, GatewayError>
where
    T: DeserializeOwned,
{
    match response.status() {
        StatusCode::UNAUTHORIZED => Err(GatewayError::Unauthorized),
        StatusCode::FORBIDDEN => Err(GatewayError::Forbidden),
        StatusCode::NOT_FOUND => Err(GatewayError::NotFound),
        status if status == StatusCode::REQUEST_TIMEOUT => Err(GatewayError::Timeout),
        status if status == StatusCode::TOO_MANY_REQUESTS => Err(GatewayError::RateLimited),
        status if status.is_server_error() => Err(GatewayError::Server),
        status if status.is_success() => response
            .json::<T>()
            .await
            .map_err(|_| GatewayError::InvalidResponse),
        _ => Err(GatewayError::RequestFailed),
    }
}

use std::time::Duration;

use reqwest::{Client, Method, RequestBuilder, StatusCode};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{ApiErrorBody, AuthoraError};

#[derive(Debug, Clone)]
pub(crate) struct HttpClient {
    client: Client,
    base_url: String,
    api_key: String,
}

impl HttpClient {
    pub fn new(api_key: &str, base_url: &str, timeout: Duration) -> Result<Self, AuthoraError> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(AuthoraError::Network)?;

        Ok(Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
        })
    }

    fn request(&self, method: Method, path: &str) -> RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        self.client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Accept", "application/json")
    }

    fn request_with_body(&self, method: Method, path: &str) -> RequestBuilder {
        self.request(method, path)
            .header("Content-Type", "application/json")
    }

    pub async fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R, AuthoraError> {
        let resp = self.request(Method::GET, path).send().await;
        self.handle_response(resp).await
    }

    pub async fn get_with_query<Q: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<R, AuthoraError> {
        let resp = self.request(Method::GET, path).query(query).send().await;
        self.handle_response(resp).await
    }

    pub async fn post<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R, AuthoraError> {
        let resp = self.request_with_body(Method::POST, path)
            .json(body)
            .send()
            .await;
        self.handle_response(resp).await
    }

    pub async fn post_empty<R: DeserializeOwned>(&self, path: &str) -> Result<R, AuthoraError> {
        let resp = self.request(Method::POST, path).send().await;
        self.handle_response(resp).await
    }

    pub async fn patch<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R, AuthoraError> {
        let resp = self.request_with_body(Method::PATCH, path)
            .json(body)
            .send()
            .await;
        self.handle_response(resp).await
    }

    pub async fn delete<R: DeserializeOwned>(&self, path: &str) -> Result<R, AuthoraError> {
        let resp = self.request(Method::DELETE, path).send().await;
        self.handle_response(resp).await
    }

    async fn handle_response<R: DeserializeOwned>(
        &self,
        result: Result<reqwest::Response, reqwest::Error>,
    ) -> Result<R, AuthoraError> {
        let resp = result.map_err(|e| {
            if e.is_timeout() {
                AuthoraError::Timeout
            } else {
                AuthoraError::Network(e)
            }
        })?;

        let status = resp.status();

        if status.is_success() {
            let body = resp.text().await.map_err(AuthoraError::Network)?;
            if body.is_empty() || body == "null" {
                return serde_json::from_str("{}").map_err(AuthoraError::Serialization);
            }
            let unwrapped = unwrap_response(&body);
            return serde_json::from_str(&unwrapped).map_err(AuthoraError::Serialization);
        }

        let status_code = status.as_u16();
        let body_text = resp.text().await.unwrap_or_default();

        let error_body: Option<ApiErrorBody> = serde_json::from_str(&body_text).ok();

        let message = error_body
            .as_ref()
            .and_then(|b| b.message.clone().or_else(|| b.error.clone()))
            .unwrap_or_else(|| body_text.clone());

        let code = error_body.as_ref().and_then(|b| b.code.clone());

        match status {
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                Err(AuthoraError::Authentication(message))
            }
            StatusCode::NOT_FOUND => Err(AuthoraError::NotFound(message)),
            StatusCode::TOO_MANY_REQUESTS => Err(AuthoraError::RateLimit),
            _ => Err(AuthoraError::Api {
                status_code,
                message,
                code,
            }),
        }
    }
}

fn unwrap_response(body: &str) -> String {
    let parsed: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return body.to_string(),
    };

    let obj = match parsed.as_object() {
        Some(o) => o,
        None => return body.to_string(),
    };

    let data = match obj.get("data") {
        Some(d) => d,
        None => return body.to_string(),
    };

    let pagination = obj.get("pagination").or_else(|| obj.get("meta"));

    if let serde_json::Value::Array(_) = data {
        if let Some(pg) = pagination {
            let total = pg.get("total").and_then(|v| v.as_u64()).unwrap_or(0);
            let page = pg.get("page").and_then(|v| v.as_u64()).unwrap_or(0);
            let limit = pg.get("limit").and_then(|v| v.as_u64()).unwrap_or(0);
            let mut result = serde_json::Map::new();
            result.insert("items".to_string(), data.clone());
            result.insert("total".to_string(), serde_json::Value::Number(total.into()));
            result.insert("page".to_string(), serde_json::Value::Number(page.into()));
            result.insert("limit".to_string(), serde_json::Value::Number(limit.into()));
            return serde_json::to_string(&serde_json::Value::Object(result))
                .unwrap_or_else(|_| body.to_string());
        }
        let mut result = serde_json::Map::new();
        result.insert("items".to_string(), data.clone());
        return serde_json::to_string(&serde_json::Value::Object(result))
            .unwrap_or_else(|_| body.to_string());
    }

    serde_json::to_string(data).unwrap_or_else(|_| body.to_string())
}

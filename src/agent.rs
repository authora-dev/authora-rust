use std::sync::Arc;
use std::time::{Duration, Instant};

use reqwest::{Client, Method, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::crypto::{self, build_signature_payload, generate_key_pair, KeyPair};
use crate::error::{ApiErrorBody, AuthoraError};
use crate::permissions::match_any_permission;
use crate::types::{
    Agent, AgentVerification, BatchPermissionCheckResult, Delegation, McpProxyResult,
    PermissionCheckResult,
};

const DEFAULT_BASE_URL: &str = "https://api.authora.dev/api/v1";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300);

pub struct AgentOptions {
    pub agent_id: String,
    pub private_key: String,
    pub base_url: Option<String>,
    pub timeout: Option<Duration>,
    pub permissions_cache_ttl: Option<Duration>,
}

pub struct SignedResponse<T> {
    pub data: T,
    pub status: u16,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolCallParams {
    pub tool_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delegation_token: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegationConstraints {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_depth: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub single_use: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_targets: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCheckItem {
    pub resource: String,
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EffectivePermissions {
    pub agent_id: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub deny_permissions: Option<Vec<String>>,
}

struct PermissionsCache {
    allow: Option<Vec<String>>,
    deny: Option<Vec<String>>,
    updated_at: Option<Instant>,
}

pub struct AgentRuntime {
    agent_id: String,
    private_key: String,
    public_key: String,
    base_url: String,
    client: Client,
    cache_ttl: Duration,
    cache: Arc<RwLock<PermissionsCache>>,
}

impl AgentRuntime {
    pub fn new(opts: AgentOptions) -> Result<Self, AuthoraError> {
        let public_key = crypto::get_public_key(&opts.private_key)?;
        let timeout = opts.timeout.unwrap_or(DEFAULT_TIMEOUT);
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(AuthoraError::Network)?;
        let base_url = opts
            .base_url
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string())
            .trim_end_matches('/')
            .to_string();

        Ok(Self {
            agent_id: opts.agent_id,
            private_key: opts.private_key,
            public_key,
            base_url,
            client,
            cache_ttl: opts.permissions_cache_ttl.unwrap_or(DEFAULT_CACHE_TTL),
            cache: Arc::new(RwLock::new(PermissionsCache {
                allow: None,
                deny: None,
                updated_at: None,
            })),
        })
    }

    pub async fn signed_request<T: DeserializeOwned>(
        &self,
        method: &str,
        path: &str,
        body: Option<&impl Serialize>,
    ) -> Result<SignedResponse<T>, AuthoraError> {
        let method_upper = method.to_uppercase();
        let url = format!("{}{}", self.base_url, path);
        let body_str = body.map(|b| serde_json::to_string(b)).transpose()?;
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let payload = build_signature_payload(
            &method_upper,
            path,
            &timestamp,
            body_str.as_deref(),
        );
        let signature = crypto::sign(&payload, &self.private_key)?;

        let http_method: Method = method_upper.parse().map_err(|_| {
            AuthoraError::Api {
                status_code: 0,
                message: format!("invalid HTTP method: {method_upper}"),
                code: None,
            }
        })?;

        let mut req = self
            .client
            .request(http_method, &url)
            .header("Accept", "application/json")
            .header("x-authora-agent-id", &self.agent_id)
            .header("x-authora-timestamp", &timestamp)
            .header("x-authora-signature", &signature);

        if let Some(ref b) = body_str {
            req = req.header("Content-Type", "application/json").body(b.clone());
        }

        let resp = req.send().await.map_err(|e| {
            if e.is_timeout() {
                AuthoraError::Timeout
            } else {
                AuthoraError::Network(e)
            }
        })?;

        let status = resp.status();
        let status_code = status.as_u16();
        let body_text = resp.text().await.map_err(AuthoraError::Network)?;

        if status.is_success() {
            let unwrapped = unwrap_response(&body_text);
            let data: T = serde_json::from_str(&unwrapped)?;
            return Ok(SignedResponse {
                data,
                status: status_code,
            });
        }

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

    pub async fn check_permission(
        &self,
        resource: &str,
        action: &str,
        context: Option<serde_json::Value>,
    ) -> Result<PermissionCheckResult, AuthoraError> {
        let body = serde_json::json!({
            "agentId": self.agent_id,
            "resource": resource,
            "action": action,
            "context": context,
        });
        let resp: SignedResponse<PermissionCheckResult> =
            self.signed_request("POST", "/permissions/check", Some(&body)).await?;
        Ok(resp.data)
    }

    pub async fn check_permissions(
        &self,
        checks: Vec<BatchCheckItem>,
    ) -> Result<Vec<PermissionCheckResult>, AuthoraError> {
        let body = serde_json::json!({
            "agentId": self.agent_id,
            "checks": checks,
        });
        let resp: SignedResponse<BatchPermissionCheckResult> =
            self.signed_request("POST", "/permissions/check-batch", Some(&body)).await?;
        Ok(resp.data.results.unwrap_or_default())
    }

    pub async fn fetch_permissions(&self) -> Result<EffectivePermissions, AuthoraError> {
        let path = format!("/agents/{}/permissions", self.agent_id);
        let resp: SignedResponse<EffectivePermissions> =
            self.signed_request::<EffectivePermissions>("GET", &path, None::<&()>.as_ref()).await?;
        let mut cache = self.cache.write().await;
        cache.allow = resp.data.permissions.clone();
        cache.deny = resp.data.deny_permissions.clone();
        cache.updated_at = Some(Instant::now());
        Ok(resp.data)
    }

    pub async fn has_permission(&self, resource: &str) -> Result<bool, AuthoraError> {
        {
            let cache = self.cache.read().await;
            let stale = cache
                .updated_at
                .map(|t| t.elapsed() > self.cache_ttl)
                .unwrap_or(true);
            if !stale {
                if let Some(ref deny) = cache.deny {
                    if match_any_permission(deny, resource) {
                        return Ok(false);
                    }
                }
                if let Some(ref allow) = cache.allow {
                    return Ok(match_any_permission(allow, resource));
                }
            }
        }
        self.fetch_permissions().await?;
        let cache = self.cache.read().await;
        if let Some(ref deny) = cache.deny {
            if match_any_permission(deny, resource) {
                return Ok(false);
            }
        }
        if let Some(ref allow) = cache.allow {
            return Ok(match_any_permission(allow, resource));
        }
        Ok(false)
    }

    pub fn invalidate_permissions_cache(&self) {
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let mut c = cache.write().await;
            c.allow = None;
            c.deny = None;
            c.updated_at = None;
        });
    }

    pub async fn delegate(
        &self,
        target_agent_id: &str,
        permissions: Vec<String>,
        constraints: Option<DelegationConstraints>,
    ) -> Result<Delegation, AuthoraError> {
        let body = serde_json::json!({
            "issuerAgentId": self.agent_id,
            "targetAgentId": target_agent_id,
            "permissions": permissions,
            "constraints": constraints,
        });
        let resp: SignedResponse<Delegation> =
            self.signed_request("POST", "/delegations", Some(&body)).await?;
        Ok(resp.data)
    }

    pub async fn call_tool(&self, params: ToolCallParams) -> Result<McpProxyResult, AuthoraError> {
        let timestamp = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true);
        let inner_payload = build_signature_payload("POST", "/mcp/proxy", &timestamp, None);
        let inner_sig = crypto::sign(&inner_payload, &self.private_key)?;

        let id = params
            .id
            .unwrap_or_else(|| serde_json::json!(format!("{}-{}", self.agent_id, timestamp)));

        let mut authora_meta = serde_json::json!({
            "agentId": self.agent_id,
            "signature": inner_sig,
            "timestamp": timestamp,
        });
        if let Some(token) = &params.delegation_token {
            authora_meta["delegationToken"] = serde_json::json!(token);
        }

        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": params.method.as_deref().unwrap_or("tools/call"),
            "params": {
                "name": params.tool_name,
                "arguments": params.arguments,
                "_authora": authora_meta,
            },
        });
        let resp: SignedResponse<McpProxyResult> =
            self.signed_request("POST", "/mcp/proxy", Some(&body)).await?;
        Ok(resp.data)
    }

    pub async fn rotate_key(&self) -> Result<(Agent, KeyPair), AuthoraError> {
        let kp = generate_key_pair();
        let body = serde_json::json!({ "publicKey": kp.public_key });
        let path = format!("/agents/{}/rotate-key", self.agent_id);
        let resp: SignedResponse<Agent> =
            self.signed_request("POST", &path, Some(&body)).await?;
        Ok((resp.data, kp))
    }

    pub async fn suspend(&self) -> Result<Agent, AuthoraError> {
        let path = format!("/agents/{}/suspend", self.agent_id);
        let resp: SignedResponse<Agent> =
            self.signed_request::<Agent>("POST", &path, None::<&()>.as_ref()).await?;
        Ok(resp.data)
    }

    pub async fn reactivate(&self) -> Result<(Agent, KeyPair), AuthoraError> {
        let kp = generate_key_pair();
        let body = serde_json::json!({ "publicKey": kp.public_key });
        let path = format!("/agents/{}/activate", self.agent_id);
        let resp: SignedResponse<Agent> =
            self.signed_request("POST", &path, Some(&body)).await?;
        Ok((resp.data, kp))
    }

    pub async fn revoke(&self) -> Result<Agent, AuthoraError> {
        let path = format!("/agents/{}/revoke", self.agent_id);
        let resp: SignedResponse<Agent> =
            self.signed_request::<Agent>("POST", &path, None::<&()>.as_ref()).await?;
        Ok(resp.data)
    }

    pub async fn get_identity_document(&self) -> Result<AgentVerification, AuthoraError> {
        let url = format!("{}/agents/{}/verify", self.base_url, self.agent_id);
        let resp = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AuthoraError::Timeout
                } else {
                    AuthoraError::Network(e)
                }
            })?;

        let status = resp.status();
        let body_text = resp.text().await.map_err(AuthoraError::Network)?;

        if status.is_success() {
            let unwrapped = unwrap_response(&body_text);
            return serde_json::from_str(&unwrapped).map_err(AuthoraError::Serialization);
        }

        let error_body: Option<ApiErrorBody> = serde_json::from_str(&body_text).ok();
        let message = error_body
            .as_ref()
            .and_then(|b| b.message.clone().or_else(|| b.error.clone()))
            .unwrap_or_else(|| body_text.clone());

        match status {
            StatusCode::NOT_FOUND => Err(AuthoraError::NotFound(message)),
            _ => Err(AuthoraError::Api {
                status_code: status.as_u16(),
                message,
                code: None,
            }),
        }
    }

    pub async fn get_profile(&self) -> Result<Agent, AuthoraError> {
        let path = format!("/agents/{}", self.agent_id);
        let resp: SignedResponse<Agent> =
            self.signed_request::<Agent>("GET", &path, None::<&()>.as_ref()).await?;
        Ok(resp.data)
    }

    pub fn get_public_key(&self) -> &str {
        &self.public_key
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

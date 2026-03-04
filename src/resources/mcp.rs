use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    ListMcpServersInput, McpProxyInput, McpProxyResult, McpServer, McpTool, PaginatedResponse,
    RegisterMcpServerInput, RegisterMcpToolInput, UpdateMcpServerInput,
};

/// Operations on MCP server resources.
#[derive(Debug, Clone)]
pub struct McpResource {
    pub(crate) http: Arc<HttpClient>,
}

impl McpResource {
    /// Register a new MCP server.
    pub async fn register_server(
        &self,
        input: RegisterMcpServerInput,
    ) -> Result<McpServer, AuthoraError> {
        self.http.post("/mcp/servers", &input).await
    }

    /// List MCP servers.
    pub async fn list_servers(
        &self,
        input: ListMcpServersInput,
    ) -> Result<PaginatedResponse<McpServer>, AuthoraError> {
        // The API may return a paginated response or a simple {items: [...]} wrapper.
        // PaginatedResponse has optional fields so it should handle both.
        self.http.get_with_query("/mcp/servers", &input).await
    }

    /// Get an MCP server by ID.
    pub async fn get_server(&self, server_id: &str) -> Result<McpServer, AuthoraError> {
        self.http
            .get(&format!("/mcp/servers/{server_id}"))
            .await
    }

    /// Update an MCP server.
    pub async fn update_server(
        &self,
        server_id: &str,
        input: UpdateMcpServerInput,
    ) -> Result<McpServer, AuthoraError> {
        self.http
            .patch(&format!("/mcp/servers/{server_id}"), &input)
            .await
    }

    /// List tools for an MCP server.
    pub async fn list_tools(&self, server_id: &str) -> Result<Vec<McpTool>, AuthoraError> {
        // The API may return {items: [...]} or a raw array after envelope unwrapping.
        // Try as {items: [...]} first, then fall back to raw array.
        #[derive(serde::Deserialize)]
        struct ItemsWrapper {
            items: Vec<McpTool>,
        }
        let result: Result<ItemsWrapper, _> = self
            .http
            .get(&format!("/mcp/servers/{server_id}/tools"))
            .await;
        match result {
            Ok(wrapper) => Ok(wrapper.items),
            Err(_) => {
                // Fallback: try raw array
                self.http
                    .get(&format!("/mcp/servers/{server_id}/tools"))
                    .await
            }
        }
    }

    /// Register a tool on an MCP server.
    pub async fn register_tool(
        &self,
        server_id: &str,
        input: RegisterMcpToolInput,
    ) -> Result<McpTool, AuthoraError> {
        self.http
            .post(&format!("/mcp/servers/{server_id}/tools"), &input)
            .await
    }

    /// Proxy a request through an MCP server.
    ///
    /// Builds a proper JSON-RPC 2.0 request with `_authora` metadata.
    pub async fn proxy(&self, input: McpProxyInput) -> Result<McpProxyResult, AuthoraError> {
        // Build JSON-RPC body with _authora metadata
        let mut params = match input.params {
            Some(serde_json::Value::Object(map)) => map,
            Some(other) => {
                let mut map = serde_json::Map::new();
                map.insert("_value".to_string(), other);
                map
            }
            None => serde_json::Map::new(),
        };

        // Ensure _authora metadata includes server_id and agent_id
        let authora = params
            .entry("_authora")
            .or_insert_with(|| serde_json::json!({}));
        if let serde_json::Value::Object(ref mut am) = authora {
            am.entry("mcpServerId".to_string())
                .or_insert_with(|| serde_json::Value::String(input.server_id.clone()));
            if let Some(ref agent_id) = input.agent_id {
                am.entry("agentId".to_string())
                    .or_insert_with(|| serde_json::Value::String(agent_id.clone()));
            }
        }

        let rpc_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": input.method,
            "id": 1,
            "params": serde_json::Value::Object(params),
        });

        self.http.post("/mcp/proxy", &rpc_body).await
    }
}

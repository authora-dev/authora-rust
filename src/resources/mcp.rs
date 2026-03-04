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
        self.http
            .get(&format!("/mcp/servers/{server_id}/tools"))
            .await
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
    pub async fn proxy(&self, input: McpProxyInput) -> Result<McpProxyResult, AuthoraError> {
        self.http.post("/mcp/proxy", &input).await
    }
}

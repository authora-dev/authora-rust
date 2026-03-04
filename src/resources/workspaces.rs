use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{CreateWorkspaceInput, ListWorkspacesInput, PaginatedResponse, Workspace};

/// Operations on Workspace resources.
#[derive(Debug, Clone)]
pub struct WorkspacesResource {
    pub(crate) http: Arc<HttpClient>,
}

impl WorkspacesResource {
    /// Create a new workspace.
    pub async fn create(&self, input: CreateWorkspaceInput) -> Result<Workspace, AuthoraError> {
        self.http.post("/workspaces", &input).await
    }

    /// Get a workspace by ID.
    pub async fn get(&self, workspace_id: &str) -> Result<Workspace, AuthoraError> {
        self.http
            .get(&format!("/workspaces/{workspace_id}"))
            .await
    }

    /// List workspaces.
    pub async fn list(
        &self,
        input: ListWorkspacesInput,
    ) -> Result<PaginatedResponse<Workspace>, AuthoraError> {
        self.http.get_with_query("/workspaces", &input).await
    }
}

use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    AgentRoleAssignment, AgentRolesResponse, AssignRoleInput, CreateRoleInput, ListRolesInput,
    PaginatedResponse, Role, SuccessResponse, UpdateRoleInput,
};

#[derive(Debug, Clone)]
pub struct RolesResource {
    pub(crate) http: Arc<HttpClient>,
}

impl RolesResource {
    pub async fn create(&self, input: CreateRoleInput) -> Result<Role, AuthoraError> {
        self.http.post("/roles", &input).await
    }

    pub async fn list(&self, input: ListRolesInput) -> Result<PaginatedResponse<Role>, AuthoraError> {
        self.http.get_with_query("/roles", &input).await
    }

    pub async fn get(&self, role_id: &str) -> Result<Role, AuthoraError> {
        self.http.get(&format!("/roles/{role_id}")).await
    }

    pub async fn update(
        &self,
        role_id: &str,
        input: UpdateRoleInput,
    ) -> Result<Role, AuthoraError> {
        self.http.patch(&format!("/roles/{role_id}"), &input).await
    }

    pub async fn delete(&self, role_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http.delete(&format!("/roles/{role_id}")).await
    }

    pub async fn assign(
        &self,
        agent_id: &str,
        input: AssignRoleInput,
    ) -> Result<AgentRoleAssignment, AuthoraError> {
        self.http
            .post(&format!("/agents/{agent_id}/roles"), &input)
            .await
    }

    pub async fn unassign(
        &self,
        agent_id: &str,
        role_id: &str,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .delete(&format!("/agents/{agent_id}/roles/{role_id}"))
            .await
    }

    pub async fn list_for_agent(
        &self,
        agent_id: &str,
    ) -> Result<AgentRolesResponse, AuthoraError> {
        self.http
            .get(&format!("/agents/{agent_id}/roles"))
            .await
    }
}

use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    AddMembersInput, AgentGroup, AgentGroupMember, BulkAssignRoleInput, BulkAssignRoleResult,
    CreateAgentGroupInput, ListAgentGroupsInput, PaginatedResponse, RemoveMembersInput,
    SuccessResponse, UpdateAgentGroupInput,
};

#[derive(Debug, Clone)]
pub struct AgentGroupsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl AgentGroupsResource {
    pub async fn create(&self, input: CreateAgentGroupInput) -> Result<AgentGroup, AuthoraError> {
        self.http.post("/agent-groups", &input).await
    }

    pub async fn list(
        &self,
        input: ListAgentGroupsInput,
    ) -> Result<PaginatedResponse<AgentGroup>, AuthoraError> {
        self.http.get_with_query("/agent-groups", &input).await
    }

    pub async fn get(&self, group_id: &str) -> Result<AgentGroup, AuthoraError> {
        self.http.get(&format!("/agent-groups/{group_id}")).await
    }

    pub async fn update(
        &self,
        group_id: &str,
        input: UpdateAgentGroupInput,
    ) -> Result<AgentGroup, AuthoraError> {
        self.http
            .patch(&format!("/agent-groups/{group_id}"), &input)
            .await
    }

    pub async fn delete(&self, group_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http.delete(&format!("/agent-groups/{group_id}")).await
    }

    pub async fn add_members(
        &self,
        group_id: &str,
        agent_ids: Vec<String>,
    ) -> Result<SuccessResponse, AuthoraError> {
        let input = AddMembersInput { agent_ids };
        self.http
            .post(&format!("/agent-groups/{group_id}/members"), &input)
            .await
    }

    pub async fn remove_members(
        &self,
        group_id: &str,
        agent_ids: Vec<String>,
    ) -> Result<SuccessResponse, AuthoraError> {
        let input = RemoveMembersInput { agent_ids };
        self.http
            .delete_with_body(&format!("/agent-groups/{group_id}/members"), &input)
            .await
    }

    pub async fn list_members(
        &self,
        group_id: &str,
    ) -> Result<Vec<AgentGroupMember>, AuthoraError> {
        self.http
            .get(&format!("/agent-groups/{group_id}/members"))
            .await
    }

    pub async fn list_agent_groups(
        &self,
        agent_id: &str,
    ) -> Result<Vec<AgentGroup>, AuthoraError> {
        self.http
            .get(&format!("/agents/{agent_id}/groups"))
            .await
    }

    pub async fn bulk_assign_role(
        &self,
        input: BulkAssignRoleInput,
    ) -> Result<BulkAssignRoleResult, AuthoraError> {
        self.http.post("/agents/bulk/assign-role", &input).await
    }
}

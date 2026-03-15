use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    AddPermissionInput, AttachPolicyInput, CreatePolicyInput, EvaluatePolicyInput,
    ListAttachmentsInput, ListPoliciesInput, PaginatedResponse, Policy, PolicyAttachment,
    PolicyEvaluationResult, PolicySimulationResult, RemovePermissionInput, SimulatePolicyInput,
    SuccessResponse, UpdatePolicyInput,
};

#[derive(Debug, Clone)]
pub struct PoliciesResource {
    pub(crate) http: Arc<HttpClient>,
}

impl PoliciesResource {
    pub async fn create(&self, input: CreatePolicyInput) -> Result<Policy, AuthoraError> {
        self.http.post("/policies", &input).await
    }

    pub async fn list(
        &self,
        input: ListPoliciesInput,
    ) -> Result<PaginatedResponse<Policy>, AuthoraError> {
        self.http.get_with_query("/policies", &input).await
    }

    pub async fn update(
        &self,
        policy_id: &str,
        input: UpdatePolicyInput,
    ) -> Result<Policy, AuthoraError> {
        self.http
            .patch(&format!("/policies/{policy_id}"), &input)
            .await
    }

    pub async fn delete(&self, policy_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http.delete(&format!("/policies/{policy_id}")).await
    }

    pub async fn simulate(
        &self,
        input: SimulatePolicyInput,
    ) -> Result<PolicySimulationResult, AuthoraError> {
        self.http.post("/policies/simulate", &input).await
    }

    pub async fn evaluate(
        &self,
        input: EvaluatePolicyInput,
    ) -> Result<PolicyEvaluationResult, AuthoraError> {
        self.http.post("/policies/evaluate", &input).await
    }

    /// Attach a policy to an agent or MCP server.
    /// Idempotent -- returns existing attachment if already attached.
    pub async fn attach_to_target(
        &self,
        input: AttachPolicyInput,
    ) -> Result<PolicyAttachment, AuthoraError> {
        self.http.post("/policies/attachments", &input).await
    }

    /// Detach a policy from an agent or MCP server by composite key.
    pub async fn detach_from_target(
        &self,
        input: AttachPolicyInput,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http.post("/policies/detach", &input).await
    }

    /// Detach a policy attachment by its ID.
    pub async fn detach_by_id(
        &self,
        attachment_id: &str,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .delete(&format!("/policies/attachments/{attachment_id}"))
            .await
    }

    /// List all policies attached to a specific agent or MCP server.
    pub async fn list_attachments(
        &self,
        input: ListAttachmentsInput,
    ) -> Result<PaginatedResponse<PolicyAttachment>, AuthoraError> {
        self.http
            .get_with_query("/policies/attachments", &input)
            .await
    }

    /// List all targets (agents and MCP servers) a policy is attached to.
    pub async fn list_policy_targets(
        &self,
        policy_id: &str,
    ) -> Result<PaginatedResponse<PolicyAttachment>, AuthoraError> {
        self.http
            .get(&format!("/policies/{policy_id}/attachments"))
            .await
    }

    /// Add resources and/or actions to an existing policy without replacing current ones.
    pub async fn add_permission(
        &self,
        input: AddPermissionInput,
    ) -> Result<Policy, AuthoraError> {
        self.http.post("/policies/add-permission", &input).await
    }

    /// Remove specific resources and/or actions from an existing policy.
    pub async fn remove_permission(
        &self,
        input: RemovePermissionInput,
    ) -> Result<Policy, AuthoraError> {
        self.http
            .post("/policies/remove-permission", &input)
            .await
    }
}

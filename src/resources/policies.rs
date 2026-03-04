use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    CreatePolicyInput, EvaluatePolicyInput, ListPoliciesInput, PaginatedResponse, Policy,
    PolicyEvaluationResult, PolicySimulationResult, SimulatePolicyInput, SuccessResponse,
    UpdatePolicyInput,
};

/// Operations on Policy resources.
#[derive(Debug, Clone)]
pub struct PoliciesResource {
    pub(crate) http: Arc<HttpClient>,
}

impl PoliciesResource {
    /// Create a new policy.
    pub async fn create(&self, input: CreatePolicyInput) -> Result<Policy, AuthoraError> {
        self.http.post("/policies", &input).await
    }

    /// List policies.
    pub async fn list(
        &self,
        input: ListPoliciesInput,
    ) -> Result<PaginatedResponse<Policy>, AuthoraError> {
        self.http.get_with_query("/policies", &input).await
    }

    /// Update a policy.
    pub async fn update(
        &self,
        policy_id: &str,
        input: UpdatePolicyInput,
    ) -> Result<Policy, AuthoraError> {
        self.http
            .patch(&format!("/policies/{policy_id}"), &input)
            .await
    }

    /// Delete a policy.
    pub async fn delete(&self, policy_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http.delete(&format!("/policies/{policy_id}")).await
    }

    /// Simulate a policy evaluation.
    pub async fn simulate(
        &self,
        input: SimulatePolicyInput,
    ) -> Result<PolicySimulationResult, AuthoraError> {
        self.http.post("/policies/simulate", &input).await
    }

    /// Evaluate policies for a given request.
    pub async fn evaluate(
        &self,
        input: EvaluatePolicyInput,
    ) -> Result<PolicyEvaluationResult, AuthoraError> {
        self.http.post("/policies/evaluate", &input).await
    }
}

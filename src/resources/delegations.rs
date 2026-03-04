use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    CreateDelegationInput, Delegation, DelegationVerification, ListAgentDelegationsInput,
    ListDelegationsInput, PaginatedResponse, SuccessResponse, VerifyDelegationInput,
};

#[derive(Debug, Clone)]
pub struct DelegationsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl DelegationsResource {
    pub async fn create(&self, input: CreateDelegationInput) -> Result<Delegation, AuthoraError> {
        self.http.post("/delegations", &input).await
    }

    pub async fn get(&self, delegation_id: &str) -> Result<Delegation, AuthoraError> {
        self.http
            .get(&format!("/delegations/{delegation_id}"))
            .await
    }

    pub async fn revoke(&self, delegation_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .post_empty(&format!("/delegations/{delegation_id}/revoke"))
            .await
    }

    pub async fn verify(
        &self,
        input: VerifyDelegationInput,
    ) -> Result<DelegationVerification, AuthoraError> {
        self.http.post("/delegations/verify", &input).await
    }

    pub async fn list(
        &self,
        input: ListDelegationsInput,
    ) -> Result<PaginatedResponse<Delegation>, AuthoraError> {
        self.http.get_with_query("/delegations", &input).await
    }

    pub async fn list_by_agent(
        &self,
        agent_id: &str,
        input: ListAgentDelegationsInput,
    ) -> Result<PaginatedResponse<Delegation>, AuthoraError> {
        self.http
            .get_with_query(&format!("/agents/{agent_id}/delegations"), &input)
            .await
    }
}

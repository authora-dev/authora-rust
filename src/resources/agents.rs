use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    ActivateAgentInput, Agent, AgentVerification, CreateAgentInput, ListAgentsInput,
    PaginatedResponse, RotateAgentKeyInput, SuccessResponse,
};

/// Operations on Agent resources.
#[derive(Debug, Clone)]
pub struct AgentsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl AgentsResource {
    /// Create a new agent.
    pub async fn create(&self, input: CreateAgentInput) -> Result<Agent, AuthoraError> {
        self.http.post("/agents", &input).await
    }

    /// List agents with optional filters.
    pub async fn list(&self, input: ListAgentsInput) -> Result<PaginatedResponse<Agent>, AuthoraError> {
        self.http.get_with_query("/agents", &input).await
    }

    /// Get an agent by ID.
    pub async fn get(&self, agent_id: &str) -> Result<Agent, AuthoraError> {
        self.http.get(&format!("/agents/{agent_id}")).await
    }

    /// Verify an agent (public endpoint).
    pub async fn verify(&self, agent_id: &str) -> Result<AgentVerification, AuthoraError> {
        self.http.get(&format!("/agents/{agent_id}/verify")).await
    }

    /// Activate an agent with a public key.
    pub async fn activate(
        &self,
        agent_id: &str,
        input: ActivateAgentInput,
    ) -> Result<Agent, AuthoraError> {
        self.http
            .post(&format!("/agents/{agent_id}/activate"), &input)
            .await
    }

    /// Suspend an agent.
    pub async fn suspend(&self, agent_id: &str) -> Result<Agent, AuthoraError> {
        self.http
            .post_empty(&format!("/agents/{agent_id}/suspend"))
            .await
    }

    /// Revoke an agent.
    pub async fn revoke(&self, agent_id: &str) -> Result<Agent, AuthoraError> {
        self.http
            .post_empty(&format!("/agents/{agent_id}/revoke"))
            .await
    }

    /// Rotate an agent's key.
    pub async fn rotate_key(
        &self,
        agent_id: &str,
        input: RotateAgentKeyInput,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .post(&format!("/agents/{agent_id}/rotate-key"), &input)
            .await
    }
}

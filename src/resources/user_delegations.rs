use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    CreateUserDelegationInput, IssueUserDelegationTokenInput, ListUserDelegationInput,
    ListUserDelegationOrgInput, RefreshUserDelegationTokenInput, RevokeUserDelegationInput,
    UserDelegationGrant, UserDelegationOrgResponse, UserDelegationToken,
    VerifyUserDelegationTokenInput, VerifyUserDelegationTokenResult,
};

#[derive(Debug, Clone)]
pub struct UserDelegationsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl UserDelegationsResource {
    /// Create a user delegation grant.
    pub async fn create(
        &self,
        input: CreateUserDelegationInput,
    ) -> Result<UserDelegationGrant, AuthoraError> {
        self.http.post("/user-delegations", &input).await
    }

    /// Get a specific delegation grant by ID.
    pub async fn get(&self, grant_id: &str) -> Result<UserDelegationGrant, AuthoraError> {
        self.http
            .get(&format!("/user-delegations/{grant_id}"))
            .await
    }

    /// List delegation grants by user.
    pub async fn list_by_user(
        &self,
        user_id: &str,
        input: ListUserDelegationInput,
    ) -> Result<Vec<UserDelegationGrant>, AuthoraError> {
        self.http
            .get_with_query(&format!("/user-delegations/by-user/{user_id}"), &input)
            .await
    }

    /// List delegation grants by agent.
    pub async fn list_by_agent(
        &self,
        agent_id: &str,
        input: ListUserDelegationInput,
    ) -> Result<Vec<UserDelegationGrant>, AuthoraError> {
        self.http
            .get_with_query(&format!("/user-delegations/by-agent/{agent_id}"), &input)
            .await
    }

    /// List delegation grants by organization.
    pub async fn list_by_org(
        &self,
        org_id: &str,
        input: ListUserDelegationOrgInput,
    ) -> Result<UserDelegationOrgResponse, AuthoraError> {
        self.http
            .get_with_query(&format!("/user-delegations/by-org/{org_id}"), &input)
            .await
    }

    /// Revoke a delegation grant.
    pub async fn revoke(
        &self,
        grant_id: &str,
        input: RevokeUserDelegationInput,
    ) -> Result<UserDelegationGrant, AuthoraError> {
        self.http
            .post(&format!("/user-delegations/{grant_id}/revoke"), &input)
            .await
    }

    /// Issue a fresh delegation JWT from a grant.
    pub async fn issue_token(
        &self,
        grant_id: &str,
        input: IssueUserDelegationTokenInput,
    ) -> Result<UserDelegationToken, AuthoraError> {
        self.http
            .post(&format!("/user-delegations/{grant_id}/token"), &input)
            .await
    }

    /// Refresh a delegation JWT.
    pub async fn refresh_token(
        &self,
        grant_id: &str,
        input: RefreshUserDelegationTokenInput,
    ) -> Result<UserDelegationToken, AuthoraError> {
        self.http
            .post(&format!("/user-delegations/{grant_id}/refresh"), &input)
            .await
    }

    /// Verify a delegation JWT.
    pub async fn verify_token(
        &self,
        input: VerifyUserDelegationTokenInput,
    ) -> Result<VerifyUserDelegationTokenResult, AuthoraError> {
        self.http
            .post("/user-delegations/tokens/verify", &input)
            .await
    }
}

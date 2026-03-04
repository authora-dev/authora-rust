use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    BatchCheckPermissionInput, BatchPermissionCheckResult, CheckPermissionInput,
    EffectivePermission, PermissionCheckResult,
};

#[derive(Debug, Clone)]
pub struct PermissionsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl PermissionsResource {
    pub async fn check(
        &self,
        input: CheckPermissionInput,
    ) -> Result<PermissionCheckResult, AuthoraError> {
        self.http.post("/permissions/check", &input).await
    }

    pub async fn check_batch(
        &self,
        input: BatchCheckPermissionInput,
    ) -> Result<BatchPermissionCheckResult, AuthoraError> {
        self.http.post("/permissions/check-batch", &input).await
    }

    pub async fn effective(
        &self,
        agent_id: &str,
    ) -> Result<Vec<EffectivePermission>, AuthoraError> {
        self.http
            .get(&format!("/agents/{agent_id}/permissions"))
            .await
    }
}

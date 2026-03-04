use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{ApiKey, CreateApiKeyInput, ListApiKeysInput, PaginatedResponse, SuccessResponse};

#[derive(Debug, Clone)]
pub struct ApiKeysResource {
    pub(crate) http: Arc<HttpClient>,
}

impl ApiKeysResource {
    pub async fn create(&self, input: CreateApiKeyInput) -> Result<ApiKey, AuthoraError> {
        self.http.post("/api-keys", &input).await
    }

    pub async fn list(
        &self,
        input: ListApiKeysInput,
    ) -> Result<PaginatedResponse<ApiKey>, AuthoraError> {
        self.http.get_with_query("/api-keys", &input).await
    }

    pub async fn revoke(&self, key_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http.delete(&format!("/api-keys/{key_id}")).await
    }
}

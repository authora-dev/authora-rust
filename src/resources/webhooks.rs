use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    CreateWebhookInput, ListWebhooksInput, PaginatedResponse, SuccessResponse, UpdateWebhookInput,
    Webhook,
};

/// Operations on Webhook resources.
#[derive(Debug, Clone)]
pub struct WebhooksResource {
    pub(crate) http: Arc<HttpClient>,
}

impl WebhooksResource {
    /// Create a new webhook.
    pub async fn create(&self, input: CreateWebhookInput) -> Result<Webhook, AuthoraError> {
        self.http.post("/webhooks", &input).await
    }

    /// List webhooks.
    pub async fn list(
        &self,
        input: ListWebhooksInput,
    ) -> Result<PaginatedResponse<Webhook>, AuthoraError> {
        self.http.get_with_query("/webhooks", &input).await
    }

    /// Update a webhook.
    pub async fn update(
        &self,
        webhook_id: &str,
        input: UpdateWebhookInput,
    ) -> Result<Webhook, AuthoraError> {
        self.http
            .patch(&format!("/webhooks/{webhook_id}"), &input)
            .await
    }

    /// Delete a webhook.
    pub async fn delete(&self, webhook_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http.delete(&format!("/webhooks/{webhook_id}")).await
    }
}

use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    ListNotificationsInput, MarkAllReadInput, Notification, PaginatedResponse, SuccessResponse,
    UnreadCount, UnreadCountInput,
};

/// Operations on Notification resources.
#[derive(Debug, Clone)]
pub struct NotificationsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl NotificationsResource {
    /// List notifications.
    pub async fn list(
        &self,
        input: ListNotificationsInput,
    ) -> Result<PaginatedResponse<Notification>, AuthoraError> {
        self.http.get_with_query("/notifications", &input).await
    }

    /// Get unread notification count.
    pub async fn unread_count(
        &self,
        input: UnreadCountInput,
    ) -> Result<UnreadCount, AuthoraError> {
        self.http
            .get_with_query("/notifications/unread-count", &input)
            .await
    }

    /// Mark a single notification as read.
    pub async fn mark_read(
        &self,
        notification_id: &str,
    ) -> Result<SuccessResponse, AuthoraError> {
        let empty: serde_json::Value = serde_json::json!({});
        self.http
            .patch(&format!("/notifications/{notification_id}/read"), &empty)
            .await
    }

    /// Mark all notifications as read.
    pub async fn mark_all_read(
        &self,
        input: MarkAllReadInput,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http.patch("/notifications/read-all", &input).await
    }
}

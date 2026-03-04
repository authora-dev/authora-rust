use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    Alert, CreateAlertInput, ListAlertsInput, PaginatedResponse, SuccessResponse, UpdateAlertInput,
};

#[derive(Debug, Clone)]
pub struct AlertsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl AlertsResource {
    pub async fn create(&self, input: CreateAlertInput) -> Result<Alert, AuthoraError> {
        self.http.post("/alerts", &input).await
    }

    pub async fn list(
        &self,
        input: ListAlertsInput,
    ) -> Result<PaginatedResponse<Alert>, AuthoraError> {
        self.http.get_with_query("/alerts", &input).await
    }

    pub async fn update(
        &self,
        alert_id: &str,
        input: UpdateAlertInput,
    ) -> Result<Alert, AuthoraError> {
        self.http
            .patch(&format!("/alerts/{alert_id}"), &input)
            .await
    }

    pub async fn delete(&self, alert_id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http.delete(&format!("/alerts/{alert_id}")).await
    }
}

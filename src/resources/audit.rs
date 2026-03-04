use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    AuditEvent, AuditMetrics, AuditMetricsInput, AuditReport, CreateAuditReportInput,
    ListAuditEventsInput, PaginatedResponse,
};

/// Operations on Audit resources.
#[derive(Debug, Clone)]
pub struct AuditResource {
    pub(crate) http: Arc<HttpClient>,
}

impl AuditResource {
    /// List audit events.
    pub async fn list_events(
        &self,
        input: ListAuditEventsInput,
    ) -> Result<PaginatedResponse<AuditEvent>, AuthoraError> {
        self.http.get_with_query("/audit/events", &input).await
    }

    /// Get a single audit event.
    pub async fn get_event(&self, event_id: &str) -> Result<AuditEvent, AuthoraError> {
        self.http
            .get(&format!("/audit/events/{event_id}"))
            .await
    }

    /// Generate an audit report.
    pub async fn create_report(
        &self,
        input: CreateAuditReportInput,
    ) -> Result<AuditReport, AuthoraError> {
        self.http.post("/audit/reports", &input).await
    }

    /// Get audit metrics.
    pub async fn metrics(&self, input: AuditMetricsInput) -> Result<AuditMetrics, AuthoraError> {
        self.http.get_with_query("/audit/metrics", &input).await
    }
}

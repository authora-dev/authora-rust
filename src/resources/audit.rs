use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    AuditEvent, AuditMetrics, AuditMetricsInput, AuditReport, CreateAuditReportInput,
    ListAuditEventsInput, PaginatedResponse,
};

#[derive(Debug, Clone)]
pub struct AuditResource {
    pub(crate) http: Arc<HttpClient>,
}

impl AuditResource {
    pub async fn list_events(
        &self,
        input: ListAuditEventsInput,
    ) -> Result<PaginatedResponse<AuditEvent>, AuthoraError> {
        self.http.get_with_query("/audit/events", &input).await
    }

    pub async fn get_event(&self, event_id: &str) -> Result<AuditEvent, AuthoraError> {
        self.http
            .get(&format!("/audit/events/{event_id}"))
            .await
    }

    pub async fn create_report(
        &self,
        input: CreateAuditReportInput,
    ) -> Result<AuditReport, AuthoraError> {
        self.http.post("/audit/reports", &input).await
    }

    pub async fn metrics(&self, input: AuditMetricsInput) -> Result<AuditMetrics, AuthoraError> {
        self.http.get_with_query("/audit/metrics", &input).await
    }

    pub async fn stream_events<F>(&self, mut on_event: F) -> Result<(), AuthoraError>
    where
        F: FnMut(AuditEvent),
    {
        let url = self.http.stream_url("/audit/stream");
        let client = reqwest::Client::new();
        let resp = client
            .get(&url)
            .header("Accept", "text/event-stream")
            .header("Authorization", format!("Bearer {}", self.http.get_api_key()))
            .send()
            .await
            .map_err(AuthoraError::Network)?;

        if !resp.status().is_success() {
            return Err(AuthoraError::Api {
                status_code: resp.status().as_u16(),
                message: format!("SSE connection failed with status {}", resp.status()),
                code: None,
            });
        }

        let mut stream = resp.bytes_stream();
        use futures_util::StreamExt;
        let mut buffer = String::new();
        let mut event_type = String::new();
        let mut data = String::new();

        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(AuthoraError::Network)?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].to_string();
                buffer = buffer[pos + 1..].to_string();

                if line.starts_with("event: ") {
                    event_type = line[7..].trim().to_string();
                } else if line.starts_with("data: ") {
                    data = line[6..].to_string();
                } else if line.is_empty() && !data.is_empty() {
                    if event_type == "audit" {
                        if let Ok(ev) = serde_json::from_str::<AuditEvent>(&data) {
                            on_event(ev);
                        }
                    }
                    event_type.clear();
                    data.clear();
                }
            }
        }

        Ok(())
    }
}

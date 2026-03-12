use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    ApprovalChallenge, ApprovalPattern, ApprovalSettings, ApprovalStats, ApprovalWebhook,
    BulkDecideInput, BulkDecideResult, CreateApprovalInput, CreateApprovalWebhookInput,
    CreateEscalationRuleInput, DecideApprovalInput, EscalationRule, ListApprovalsInput,
    ListPatternsInput, PaginatedResponse, PermissionSuggestion, PushSubscribeInput,
    SuccessResponse, TestAiInput, TestAiResult, UnsubscribePushInput,
    UpdateApprovalSettingsInput, UpdateApprovalWebhookInput, UpdateEscalationRuleInput,
    VapidKeyResponse,
};

#[derive(Debug, Clone)]
pub struct ApprovalsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl ApprovalsResource {
    pub async fn create(
        &self,
        input: CreateApprovalInput,
    ) -> Result<ApprovalChallenge, AuthoraError> {
        self.http.post("/approvals", &input).await
    }

    pub async fn list(
        &self,
        input: ListApprovalsInput,
    ) -> Result<PaginatedResponse<ApprovalChallenge>, AuthoraError> {
        self.http.get_with_query("/approvals", &input).await
    }

    pub async fn get(&self, id: &str) -> Result<ApprovalChallenge, AuthoraError> {
        self.http.get(&format!("/approvals/{id}")).await
    }

    pub async fn get_status(&self, id: &str) -> Result<ApprovalStatusResponse, AuthoraError> {
        self.http.get(&format!("/approvals/{id}/status")).await
    }

    pub async fn stats(&self) -> Result<ApprovalStats, AuthoraError> {
        self.http.get("/approvals/stats").await
    }

    pub async fn decide(
        &self,
        id: &str,
        input: DecideApprovalInput,
    ) -> Result<ApprovalChallenge, AuthoraError> {
        self.http
            .post(&format!("/approvals/{id}/decide"), &input)
            .await
    }

    pub async fn bulk_decide(
        &self,
        input: BulkDecideInput,
    ) -> Result<BulkDecideResult, AuthoraError> {
        self.http.post("/approvals/bulk-decide", &input).await
    }

    pub async fn suggestions(
        &self,
        id: &str,
    ) -> Result<Vec<PermissionSuggestion>, AuthoraError> {
        self.http
            .post_empty(&format!("/approvals/{id}/suggestions"))
            .await
    }

    pub async fn get_settings(&self) -> Result<ApprovalSettings, AuthoraError> {
        self.http.get("/approvals/settings").await
    }

    pub async fn update_settings(
        &self,
        input: UpdateApprovalSettingsInput,
    ) -> Result<ApprovalSettings, AuthoraError> {
        self.http.patch("/approvals/settings", &input).await
    }

    pub async fn test_ai(&self, input: TestAiInput) -> Result<TestAiResult, AuthoraError> {
        self.http
            .post("/approvals/settings/test-ai", &input)
            .await
    }

    pub async fn list_patterns(
        &self,
        input: ListPatternsInput,
    ) -> Result<Vec<ApprovalPattern>, AuthoraError> {
        self.http
            .get_with_query("/approvals/patterns", &input)
            .await
    }

    pub async fn dismiss_pattern(&self, id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .post_empty(&format!("/approvals/patterns/{id}/dismiss"))
            .await
    }

    pub async fn create_policy_from_pattern(
        &self,
        id: &str,
    ) -> Result<serde_json::Value, AuthoraError> {
        self.http
            .post_empty(&format!("/approvals/patterns/{id}/create-policy"))
            .await
    }

    pub async fn list_escalation_rules(&self) -> Result<Vec<EscalationRule>, AuthoraError> {
        self.http.get("/approvals/escalation-rules").await
    }

    pub async fn get_escalation_rule(&self, id: &str) -> Result<EscalationRule, AuthoraError> {
        self.http
            .get(&format!("/approvals/escalation-rules/{id}"))
            .await
    }

    pub async fn create_escalation_rule(
        &self,
        input: CreateEscalationRuleInput,
    ) -> Result<EscalationRule, AuthoraError> {
        self.http
            .post("/approvals/escalation-rules", &input)
            .await
    }

    pub async fn update_escalation_rule(
        &self,
        id: &str,
        input: UpdateEscalationRuleInput,
    ) -> Result<EscalationRule, AuthoraError> {
        self.http
            .patch(&format!("/approvals/escalation-rules/{id}"), &input)
            .await
    }

    pub async fn delete_escalation_rule(
        &self,
        id: &str,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .delete(&format!("/approvals/escalation-rules/{id}"))
            .await
    }

    pub async fn get_vapid_key(&self) -> Result<VapidKeyResponse, AuthoraError> {
        self.http.get("/approvals/push/vapid-key").await
    }

    pub async fn subscribe_push(
        &self,
        input: PushSubscribeInput,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http.post("/approvals/push/subscribe", &input).await
    }

    pub async fn unsubscribe_push(
        &self,
        endpoint: &str,
    ) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .post(
                "/approvals/push/unsubscribe",
                &UnsubscribePushInput {
                    endpoint: endpoint.to_string(),
                },
            )
            .await
    }

    pub async fn list_webhooks(&self) -> Result<Vec<ApprovalWebhook>, AuthoraError> {
        self.http.get("/approvals/webhooks").await
    }

    pub async fn create_webhook(
        &self,
        input: CreateApprovalWebhookInput,
    ) -> Result<ApprovalWebhook, AuthoraError> {
        self.http.post("/approvals/webhooks", &input).await
    }

    pub async fn update_webhook(
        &self,
        id: &str,
        input: UpdateApprovalWebhookInput,
    ) -> Result<ApprovalWebhook, AuthoraError> {
        self.http
            .patch(&format!("/approvals/webhooks/{id}"), &input)
            .await
    }

    pub async fn delete_webhook(&self, id: &str) -> Result<SuccessResponse, AuthoraError> {
        self.http
            .delete(&format!("/approvals/webhooks/{id}"))
            .await
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalStatusResponse {
    pub status: Option<String>,
}

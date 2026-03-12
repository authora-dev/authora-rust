use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    CreditBalance, CreditCheckoutInput, CreditCheckoutResult, CreditTransaction,
    ListCreditTransactionsInput, PaginatedResponse,
};

#[derive(Debug, Clone)]
pub struct CreditsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl CreditsResource {
    pub async fn balance(&self) -> Result<CreditBalance, AuthoraError> {
        self.http.get("/credits").await
    }

    pub async fn transactions(
        &self,
        input: ListCreditTransactionsInput,
    ) -> Result<PaginatedResponse<CreditTransaction>, AuthoraError> {
        self.http
            .get_with_query("/credits/transactions", &input)
            .await
    }

    pub async fn checkout(&self, pack: &str) -> Result<CreditCheckoutResult, AuthoraError> {
        self.http
            .post("/credits/checkout", &CreditCheckoutInput { pack: pack.to_string() })
            .await
    }
}

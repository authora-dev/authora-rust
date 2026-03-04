use std::sync::Arc;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::types::{
    CreateOrganizationInput, ListOrganizationsInput, Organization, PaginatedResponse,
};

#[derive(Debug, Clone)]
pub struct OrganizationsResource {
    pub(crate) http: Arc<HttpClient>,
}

impl OrganizationsResource {
    pub async fn create(
        &self,
        input: CreateOrganizationInput,
    ) -> Result<Organization, AuthoraError> {
        self.http.post("/organizations", &input).await
    }

    pub async fn get(&self, org_id: &str) -> Result<Organization, AuthoraError> {
        self.http.get(&format!("/organizations/{org_id}")).await
    }

    pub async fn list(
        &self,
        input: ListOrganizationsInput,
    ) -> Result<PaginatedResponse<Organization>, AuthoraError> {
        self.http.get_with_query("/organizations", &input).await
    }
}

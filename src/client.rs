use std::sync::Arc;
use std::time::Duration;

use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::resources::agents::AgentsResource;
use crate::resources::alerts::AlertsResource;
use crate::resources::api_keys::ApiKeysResource;
use crate::resources::audit::AuditResource;
use crate::resources::delegations::DelegationsResource;
use crate::resources::mcp::McpResource;
use crate::resources::notifications::NotificationsResource;
use crate::resources::organizations::OrganizationsResource;
use crate::resources::permissions::PermissionsResource;
use crate::resources::policies::PoliciesResource;
use crate::resources::roles::RolesResource;
use crate::resources::webhooks::WebhooksResource;
use crate::resources::workspaces::WorkspacesResource;

const DEFAULT_BASE_URL: &str = "https://api.authora.dev/api/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// The main client for interacting with the Authora API.
///
/// Create one via [`AuthoraClient::new`] (defaults) or [`AuthoraClient::builder`]
/// (custom configuration).
#[derive(Debug, Clone)]
pub struct AuthoraClient {
    http: Arc<HttpClient>,
}

impl AuthoraClient {
    /// Create a client with default settings.
    ///
    /// Uses `https://api.authora.dev/api/v1` as the base URL and a 30-second
    /// request timeout.
    pub fn new(api_key: &str) -> Result<Self, AuthoraError> {
        Self::builder(api_key).build()
    }

    /// Return a builder for fine-grained configuration.
    #[must_use]
    pub fn builder(api_key: &str) -> AuthoraClientBuilder {
        AuthoraClientBuilder {
            api_key: api_key.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    // -- resource accessors ---------------------------------------------------

    /// Access agent operations.
    pub fn agents(&self) -> AgentsResource {
        AgentsResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access role operations.
    pub fn roles(&self) -> RolesResource {
        RolesResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access permission-check operations.
    pub fn permissions(&self) -> PermissionsResource {
        PermissionsResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access delegation operations.
    pub fn delegations(&self) -> DelegationsResource {
        DelegationsResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access policy operations.
    pub fn policies(&self) -> PoliciesResource {
        PoliciesResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access MCP operations.
    pub fn mcp(&self) -> McpResource {
        McpResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access audit operations.
    pub fn audit(&self) -> AuditResource {
        AuditResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access notification operations.
    pub fn notifications(&self) -> NotificationsResource {
        NotificationsResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access webhook operations.
    pub fn webhooks(&self) -> WebhooksResource {
        WebhooksResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access alert operations.
    pub fn alerts(&self) -> AlertsResource {
        AlertsResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access API key operations.
    pub fn api_keys(&self) -> ApiKeysResource {
        ApiKeysResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access organization operations.
    pub fn organizations(&self) -> OrganizationsResource {
        OrganizationsResource {
            http: Arc::clone(&self.http),
        }
    }

    /// Access workspace operations.
    pub fn workspaces(&self) -> WorkspacesResource {
        WorkspacesResource {
            http: Arc::clone(&self.http),
        }
    }
}

/// Builder for [`AuthoraClient`].
#[must_use]
#[derive(Debug)]
pub struct AuthoraClientBuilder {
    api_key: String,
    base_url: String,
    timeout: Duration,
}

impl AuthoraClientBuilder {
    /// Override the base URL (default: `https://api.authora.dev/api/v1`).
    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    /// Override the request timeout (default: 30 s).
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<AuthoraClient, AuthoraError> {
        let http = HttpClient::new(&self.api_key, &self.base_url, self.timeout)?;
        Ok(AuthoraClient {
            http: Arc::new(http),
        })
    }
}

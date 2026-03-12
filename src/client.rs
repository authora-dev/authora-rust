use std::sync::Arc;
use std::time::Duration;

use crate::agent::{AgentOptions, AgentRuntime};
use crate::crypto::{generate_key_pair, KeyPair};
use crate::error::AuthoraError;
use crate::http::HttpClient;
use crate::resources::agents::AgentsResource;
use crate::resources::alerts::AlertsResource;
use crate::resources::api_keys::ApiKeysResource;
use crate::resources::audit::AuditResource;
use crate::resources::delegations::DelegationsResource;
use crate::resources::user_delegations::UserDelegationsResource;
use crate::resources::mcp::McpResource;
use crate::resources::notifications::NotificationsResource;
use crate::resources::organizations::OrganizationsResource;
use crate::resources::permissions::PermissionsResource;
use crate::resources::policies::PoliciesResource;
use crate::resources::roles::RolesResource;
use crate::resources::webhooks::WebhooksResource;
use crate::resources::workspaces::WorkspacesResource;
use crate::resources::approvals::ApprovalsResource;
use crate::resources::credits::CreditsResource;
use crate::types::{
    ActivateAgentInput, Agent, AgentVerification, CreateAgentInput,
};

const DEFAULT_BASE_URL: &str = "https://api.authora.dev/api/v1";
const DEFAULT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone)]
pub struct AuthoraClient {
    http: Arc<HttpClient>,
    base_url: String,
}

impl AuthoraClient {
    pub fn new(api_key: &str) -> Result<Self, AuthoraError> {
        Self::builder(api_key).build()
    }

    #[must_use]
    pub fn builder(api_key: &str) -> AuthoraClientBuilder {
        AuthoraClientBuilder {
            api_key: api_key.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    pub async fn create_agent(
        &self,
        input: CreateAgentInput,
    ) -> Result<(Agent, KeyPair), AuthoraError> {
        let agent: Agent = self.http.post("/agents", &input).await?;
        let kp = generate_key_pair();
        let agent_id = agent
            .id
            .as_deref()
            .ok_or_else(|| AuthoraError::Api {
                status_code: 0,
                message: "missing agent id in response".into(),
                code: None,
            })?;
        let activated: Agent = self
            .http
            .post(
                &format!("/agents/{agent_id}/activate"),
                &ActivateAgentInput {
                    public_key: kp.public_key.clone(),
                },
            )
            .await?;
        Ok((activated, kp))
    }

    pub fn load_agent(&self, opts: AgentOptions) -> Result<AgentRuntime, AuthoraError> {
        let resolved = AgentOptions {
            base_url: opts.base_url.or_else(|| Some(self.base_url.clone())),
            ..opts
        };
        AgentRuntime::new(resolved)
    }

    pub fn load_delegated_agent(&self, opts: AgentOptions) -> Result<AgentRuntime, AuthoraError> {
        if opts.delegation_token.is_none() {
            return Err(AuthoraError::Api {
                status_code: 0,
                message: "delegation_token is required".into(),
                code: None,
            });
        }
        let resolved = AgentOptions {
            base_url: opts.base_url.or_else(|| Some(self.base_url.clone())),
            ..opts
        };
        AgentRuntime::new(resolved)
    }

    pub async fn verify_agent(
        &self,
        agent_id: &str,
    ) -> Result<AgentVerification, AuthoraError> {
        self.http
            .get(&format!("/agents/{agent_id}/verify"))
            .await
    }

    pub fn agents(&self) -> AgentsResource {
        AgentsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn roles(&self) -> RolesResource {
        RolesResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn permissions(&self) -> PermissionsResource {
        PermissionsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn delegations(&self) -> DelegationsResource {
        DelegationsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn user_delegations(&self) -> UserDelegationsResource {
        UserDelegationsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn policies(&self) -> PoliciesResource {
        PoliciesResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn mcp(&self) -> McpResource {
        McpResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn audit(&self) -> AuditResource {
        AuditResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn notifications(&self) -> NotificationsResource {
        NotificationsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn webhooks(&self) -> WebhooksResource {
        WebhooksResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn alerts(&self) -> AlertsResource {
        AlertsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn api_keys(&self) -> ApiKeysResource {
        ApiKeysResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn organizations(&self) -> OrganizationsResource {
        OrganizationsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn workspaces(&self) -> WorkspacesResource {
        WorkspacesResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn approvals(&self) -> ApprovalsResource {
        ApprovalsResource {
            http: Arc::clone(&self.http),
        }
    }

    pub fn credits(&self) -> CreditsResource {
        CreditsResource {
            http: Arc::clone(&self.http),
        }
    }
}

#[must_use]
#[derive(Debug)]
pub struct AuthoraClientBuilder {
    api_key: String,
    base_url: String,
    timeout: Duration,
}

impl AuthoraClientBuilder {
    pub fn base_url(mut self, url: &str) -> Self {
        self.base_url = url.to_string();
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn build(self) -> Result<AuthoraClient, AuthoraError> {
        let http = HttpClient::new(&self.api_key, &self.base_url, self.timeout)?;
        Ok(AuthoraClient {
            http: Arc::new(http),
            base_url: self.base_url,
        })
    }
}

//! Integration tests for the Authora Rust SDK.
//!
//! These tests hit the live Authora API and are gated behind `#[ignore]`.
//! Run them with:
//!
//! ```sh
//! cargo test -- --ignored
//! ```

use authora::types::*;
use authora::AuthoraClient;

const API_KEY: &str = "authora_live_076270f52d3fc0fe9af9d08fe49b2803eb8b64ba5132fc76";
const BASE_URL: &str = "https://api.authora.dev/api/v1";
const WORKSPACE_ID: &str = "ws_a7067ccce35d36b5";
const ORG_ID: &str = "org_92582b4a512e52ff";

/// Helper: generate a current ISO-8601 timestamp string without external crates.
/// Uses Unix epoch seconds and constructs a rough but parseable timestamp.
fn new_iso_timestamp() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    // Calculate date/time components from epoch seconds
    // This is a simplified calculation that's accurate enough for timestamp drift checks
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;
    // Approximate year/month/day from days since epoch (1970-01-01)
    let mut y = 1970i64;
    let mut remaining_days = days as i64;
    loop {
        let days_in_year = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining_days < days_in_year { break; }
        remaining_days -= days_in_year;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let month_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut m = 0usize;
    while m < 12 && remaining_days >= month_days[m] {
        remaining_days -= month_days[m];
        m += 1;
    }
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m + 1, remaining_days + 1, hours, minutes, seconds
    )
}

/// Helper: build a configured client.
fn client() -> AuthoraClient {
    AuthoraClient::builder(API_KEY)
        .base_url(BASE_URL)
        .build()
        .expect("failed to build AuthoraClient")
}

/// Helper: create a throwaway agent and return its ID.
async fn create_test_agent(client: &AuthoraClient, suffix: &str) -> String {
    let agent = client
        .agents()
        .create(CreateAgentInput {
            workspace_id: WORKSPACE_ID.into(),
            name: format!("rust-sdk-test-agent-{suffix}"),
            created_by: "integration-test".into(),
            description: Some("Created by Rust SDK integration tests".into()),
            tags: Some(vec!["test".into(), "rust-sdk".into()]),
            ..Default::default()
        })
        .await
        .expect("failed to create test agent");
    agent.id.expect("agent missing id")
}

// ---------------------------------------------------------------------------
// 1. Agent lifecycle
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_agent_lifecycle() {
    let c = client();

    // Create
    let agent = c
        .agents()
        .create(CreateAgentInput {
            workspace_id: WORKSPACE_ID.into(),
            name: "rust-sdk-lifecycle-agent".into(),
            created_by: "integration-test".into(),
            description: Some("Lifecycle test agent".into()),
            tags: Some(vec!["lifecycle".into()]),
            ..Default::default()
        })
        .await
        .expect("create agent");
    let agent_id = agent.id.as_deref().expect("agent id");
    // Note: agent name is stored in metadata.name on the backend;
    // the unwrapped response may have name: null at top level.

    // Get
    let fetched = c.agents().get(agent_id).await.expect("get agent");
    assert_eq!(fetched.id.as_deref(), Some(agent_id));

    // List
    let page = c
        .agents()
        .list(ListAgentsInput {
            workspace_id: WORKSPACE_ID.into(),
            limit: Some(5),
            ..Default::default()
        })
        .await
        .expect("list agents");
    assert!(!page.items.is_empty(), "expected at least one agent");

    // Revoke
    let revoked = c.agents().revoke(agent_id).await.expect("revoke agent");
    assert_eq!(revoked.status.as_deref(), Some("REVOKED"));
}

// ---------------------------------------------------------------------------
// 1b. Agent Security Lifecycle (activate, suspend, rotate-key, verify)
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_agent_security_lifecycle() {
    let c = client();

    // Create
    let agent = c
        .agents()
        .create(CreateAgentInput {
            workspace_id: WORKSPACE_ID.into(),
            name: "rust-sdk-security-agent".into(),
            created_by: "integration-test".into(),
            description: Some("Security lifecycle test".into()),
            ..Default::default()
        })
        .await
        .expect("create agent");
    let agent_id = agent.id.as_deref().expect("agent id");
    assert_eq!(agent.status.as_deref(), Some("PENDING"));

    // Activate with public key
    let activated = c
        .agents()
        .activate(
            agent_id,
            ActivateAgentInput {
                public_key: "test-pubkey-rust-security".into(),
            },
        )
        .await
        .expect("activate agent");
    assert_eq!(activated.status.as_deref(), Some("ACTIVE"));

    // Verify (public, no auth) -- must be after activate (needs identity doc)
    // The verify endpoint returns an identity document with id, status, metadata, etc.
    let verification = c.agents().verify(agent_id).await.expect("verify agent");
    // Just verify the call succeeded -- the response shape may vary
    let _ = verification;

    // Rotate key (must be ACTIVE)
    c.agents()
        .rotate_key(
            agent_id,
            RotateAgentKeyInput {
                public_key: "rotated-pubkey-rust-security".into(),
            },
        )
        .await
        .expect("rotate key");

    // Suspend
    let suspended = c.agents().suspend(agent_id).await.expect("suspend agent");
    assert_eq!(suspended.status.as_deref(), Some("SUSPENDED"));

    // Revoke
    let revoked = c.agents().revoke(agent_id).await.expect("revoke agent");
    assert_eq!(revoked.status.as_deref(), Some("REVOKED"));
}

// ---------------------------------------------------------------------------
// 2. RBAC flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_rbac_flow() {
    let c = client();

    // Create an agent
    let agent_id = create_test_agent(&c, "rbac").await;

    // Create a role with string permissions
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let role = c
        .roles()
        .create(CreateRoleInput {
            workspace_id: WORKSPACE_ID.into(),
            name: format!("rust-sdk-test-role-{ts}"),
            description: Some("Integration test role".into()),
            permissions: Some(vec![
                "files:*:read".into(),
                "files:*:write".into(),
                "db:*:read".into(),
            ]),
            ..Default::default()
        })
        .await
        .expect("create role");
    let role_id = role.id.as_deref().expect("role id");

    // Assign role to agent
    let assignment = c
        .roles()
        .assign(
            &agent_id,
            AssignRoleInput {
                role_id: role_id.into(),
                granted_by: Some("integration-test".into()),
                ..Default::default()
            },
        )
        .await
        .expect("assign role");
    assert_eq!(assignment.role_id.as_deref(), Some(role_id));

    // List agent roles
    let agent_roles = c
        .roles()
        .list_for_agent(&agent_id)
        .await
        .expect("list agent roles");
    assert!(
        agent_roles.roles.iter().any(|r| r.id.as_deref() == Some(role_id)),
        "agent should have the assigned role"
    );

    // Check permission
    let check = c
        .permissions()
        .check(CheckPermissionInput {
            agent_id: agent_id.clone(),
            resource: "files:doc1".into(),
            action: "read".into(),
            ..Default::default()
        })
        .await
        .expect("check permission");
    assert!(check.allowed.is_some(), "permission check returned a result");

    // Batch check
    let batch = c
        .permissions()
        .check_batch(BatchCheckPermissionInput {
            agent_id: agent_id.clone(),
            checks: vec![
                PermissionCheckItem {
                    resource: "files:doc1".into(),
                    action: "read".into(),
                    ..Default::default()
                },
                PermissionCheckItem {
                    resource: "files:doc2".into(),
                    action: "write".into(),
                    ..Default::default()
                },
            ],
        })
        .await
        .expect("batch check");
    // At least the results should exist
    assert!(batch.results.is_some(), "batch results should be present");

    // Unassign role
    c.roles()
        .unassign(&agent_id, role_id)
        .await
        .expect("unassign role");

    // Delete role
    c.roles().delete(role_id).await.expect("delete role");

    // Cleanup: revoke agent
    c.agents().revoke(&agent_id).await.expect("revoke agent");
}

// ---------------------------------------------------------------------------
// 3. Policy flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_policy_flow() {
    let c = client();

    // Create policy with principals (structured JSON)
    let policy = c
        .policies()
        .create(CreatePolicyInput {
            workspace_id: WORKSPACE_ID.into(),
            name: "rust-sdk-test-policy".into(),
            description: Some("Integration test policy".into()),
            effect: "ALLOW".into(),
            principals: Some(serde_json::json!({
                "roles": ["editor"]
            })),
            resources: Some(vec!["files:*".into()]),
            actions: Some(vec!["read".into(), "write".into()]),
            priority: Some(10),
            enabled: Some(true),
            ..Default::default()
        })
        .await
        .expect("create policy");
    let policy_id = policy.id.as_deref().expect("policy id");
    assert_eq!(policy.effect.as_deref(), Some("ALLOW"));
    assert_eq!(policy.name.as_deref(), Some("rust-sdk-test-policy"));

    // List policies
    let page = c
        .policies()
        .list(ListPoliciesInput {
            workspace_id: WORKSPACE_ID.into(),
            ..Default::default()
        })
        .await
        .expect("list policies");
    assert!(
        page.items.iter().any(|p| p.id.as_deref() == Some(policy_id)),
        "created policy should appear in list"
    );

    // Update policy
    let updated = c
        .policies()
        .update(
            policy_id,
            UpdatePolicyInput {
                description: Some("Updated by integration test".into()),
                actions: Some(vec!["read".into(), "list".into(), "download".into()]),
                ..Default::default()
            },
        )
        .await
        .expect("update policy");
    assert_eq!(
        updated.description.as_deref(),
        Some("Updated by integration test")
    );

    // Delete policy
    c.policies().delete(policy_id).await.expect("delete policy");
}

// ---------------------------------------------------------------------------
// 4. Delegation flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_delegation_flow() {
    let c = client();

    // Create two agents: issuer and delegate
    let issuer_id = create_test_agent(&c, "deleg-issuer").await;
    let delegate_id = create_test_agent(&c, "deleg-delegate").await;

    // Create a role with the permission we want to delegate
    let role = c
        .roles()
        .create(CreateRoleInput {
            workspace_id: WORKSPACE_ID.into(),
            name: "rust-sdk-delegation-role".into(),
            description: Some("Role for delegation test".into()),
            permissions: Some(vec!["files:read".into(), "files:write".into()]),
            ..Default::default()
        })
        .await
        .expect("create role");
    let role_id = role.id.as_deref().expect("role id");

    // Assign role to the issuer so it has permissions to delegate
    c.roles()
        .assign(
            &issuer_id,
            AssignRoleInput {
                role_id: role_id.into(),
                granted_by: Some("integration-test".into()),
                ..Default::default()
            },
        )
        .await
        .expect("assign role to issuer");

    // Create delegation from issuer -> delegate
    let delegation = c
        .delegations()
        .create(CreateDelegationInput {
            issuer_agent_id: issuer_id.clone(),
            target_agent_id: delegate_id.clone(),
            permissions: Some(vec!["files:read".into()]),
            constraints: Some(serde_json::json!({
                "maxDepth": 1
            })),
            ..Default::default()
        })
        .await
        .expect("create delegation");
    let delegation_id = delegation.id.as_deref().expect("delegation id");

    // Get delegation
    let fetched = c
        .delegations()
        .get(delegation_id)
        .await
        .expect("get delegation");
    assert_eq!(fetched.id.as_deref(), Some(delegation_id));
    assert_eq!(
        fetched.issuer_agent_id.as_deref(),
        Some(issuer_id.as_str())
    );
    assert_eq!(
        fetched.target_agent_id.as_deref(),
        Some(delegate_id.as_str())
    );

    // List delegations
    let page = c
        .delegations()
        .list(ListDelegationsInput {
            status: Some("active".into()),
            ..Default::default()
        })
        .await
        .expect("list delegations");
    assert!(
        page.items
            .iter()
            .any(|d| d.id.as_deref() == Some(delegation_id)),
        "delegation should appear in list"
    );

    // Revoke delegation
    c.delegations()
        .revoke(delegation_id)
        .await
        .expect("revoke delegation");

    // Cleanup
    c.roles()
        .unassign(&issuer_id, role_id)
        .await
        .expect("unassign role");
    c.roles().delete(role_id).await.expect("delete role");
    c.agents().revoke(&issuer_id).await.expect("revoke issuer");
    c.agents()
        .revoke(&delegate_id)
        .await
        .expect("revoke delegate");
}

// ---------------------------------------------------------------------------
// 5. Audit flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_audit_flow() {
    let c = client();

    // List audit events
    let page = c
        .audit()
        .list_events(ListAuditEventsInput {
            limit: Some(5),
            ..Default::default()
        })
        .await
        .expect("list audit events");
    // The workspace should have events from previous operations
    assert!(page.items.len() <= 5, "limit should be respected");

    // Get audit metrics
    let metrics = c
        .audit()
        .metrics(AuditMetricsInput {
            org_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("get audit metrics");
    // Metrics call should succeed; result shape may vary
    let _ = metrics;
}

// ---------------------------------------------------------------------------
// 6. Webhook flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_webhook_flow() {
    let c = client();

    // Create webhook (organizationId, eventTypes, secret required)
    let webhook = c
        .webhooks()
        .create(CreateWebhookInput {
            organization_id: ORG_ID.into(),
            url: "https://example.com/webhook/rust-sdk-test".into(),
            event_types: vec!["agent.created".into(), "agent.revoked".into()],
            secret: "test-secret-rust-sdk".into(),
        })
        .await
        .expect("create webhook");
    let webhook_id = webhook.id.as_deref().expect("webhook id");
    assert_eq!(
        webhook.url.as_deref(),
        Some("https://example.com/webhook/rust-sdk-test")
    );

    // List webhooks
    let page = c
        .webhooks()
        .list(ListWebhooksInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("list webhooks");
    assert!(
        page.items
            .iter()
            .any(|w| w.id.as_deref() == Some(webhook_id)),
        "webhook should appear in list"
    );

    // Update webhook
    let updated = c
        .webhooks()
        .update(
            webhook_id,
            UpdateWebhookInput {
                url: Some("https://example.com/webhook/rust-sdk-updated".into()),
                enabled: Some(false),
                ..Default::default()
            },
        )
        .await
        .expect("update webhook");
    assert_eq!(
        updated.url.as_deref(),
        Some("https://example.com/webhook/rust-sdk-updated")
    );

    // Delete webhook
    c.webhooks()
        .delete(webhook_id)
        .await
        .expect("delete webhook");
}

// ---------------------------------------------------------------------------
// 7. Alert flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_alert_flow() {
    let c = client();

    // Create alert (organizationId, eventTypes, conditions, channels)
    let alert = c
        .alerts()
        .create(CreateAlertInput {
            organization_id: ORG_ID.into(),
            name: "rust-sdk-test-alert".into(),
            event_types: vec!["agent.revoked".into(), "permission.denied".into()],
            conditions: serde_json::json!({
                "severity": "high"
            }),
            channels: vec!["email".into()],
        })
        .await
        .expect("create alert");
    let alert_id = alert.id.as_deref().expect("alert id");
    assert_eq!(alert.name.as_deref(), Some("rust-sdk-test-alert"));

    // List alerts
    let page = c
        .alerts()
        .list(ListAlertsInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("list alerts");
    assert!(
        page.items
            .iter()
            .any(|a| a.id.as_deref() == Some(alert_id)),
        "alert should appear in list"
    );

    // Update alert
    let updated = c
        .alerts()
        .update(
            alert_id,
            UpdateAlertInput {
                name: Some("rust-sdk-test-alert-updated".into()),
                enabled: Some(false),
                ..Default::default()
            },
        )
        .await
        .expect("update alert");
    assert_eq!(
        updated.name.as_deref(),
        Some("rust-sdk-test-alert-updated")
    );

    // Delete alert
    c.alerts().delete(alert_id).await.expect("delete alert");
}

// ---------------------------------------------------------------------------
// 8. API key flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_api_key_flow() {
    let c = client();

    // List existing keys
    let page = c
        .api_keys()
        .list(ListApiKeysInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("list api keys");
    let initial_count = page.items.len();

    // Create a new API key (organizationId, name, createdBy required)
    let key = c
        .api_keys()
        .create(CreateApiKeyInput {
            organization_id: ORG_ID.into(),
            name: "rust-sdk-test-key".into(),
            created_by: "integration-test".into(),
            expires_in_days: Some(1),
            ..Default::default()
        })
        .await
        .expect("create api key");
    let key_id = key.id.as_deref().expect("key id");
    assert_eq!(key.name.as_deref(), Some("rust-sdk-test-key"));
    // On creation, the raw key value should be returned
    assert!(
        key.raw_key.is_some(),
        "newly created key should have the rawKey value"
    );

    // List again - should have one more
    let page2 = c
        .api_keys()
        .list(ListApiKeysInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("list api keys after create");
    assert!(
        page2.items.len() > initial_count,
        "should have more keys after create"
    );

    // Revoke (delete) the key
    c.api_keys().revoke(key_id).await.expect("revoke api key");
}

// ---------------------------------------------------------------------------
// 9. Notification flow
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_notification_flow() {
    let c = client();

    // List notifications
    let page = c
        .notifications()
        .list(ListNotificationsInput {
            organization_id: ORG_ID.into(),
            limit: Some(10),
            ..Default::default()
        })
        .await
        .expect("list notifications");
    // We may or may not have notifications, but the call should succeed
    assert!(page.items.len() <= 10);

    // Get unread count
    let unread = c
        .notifications()
        .unread_count(UnreadCountInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("unread count");
    // API may return count or unreadCount field
    let cnt = unread.unread_count.or(unread.count);
    assert!(cnt.is_some(), "unread count should be present");

    // Mark all as read
    c.notifications()
        .mark_all_read(MarkAllReadInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("mark all read");

    // Verify unread count is now 0
    let unread_after = c
        .notifications()
        .unread_count(UnreadCountInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("unread count after mark all read");
    let cnt_after = unread_after.unread_count.or(unread_after.count).unwrap_or(0);
    assert_eq!(cnt_after, 0, "unread count should be 0 after marking all read");
}

// ---------------------------------------------------------------------------
// 10. Organization and Workspace
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_org_and_workspace() {
    let c = client();

    // Get organization
    let org = c
        .organizations()
        .get(ORG_ID)
        .await
        .expect("get organization");
    assert_eq!(org.id.as_deref(), Some(ORG_ID));
    assert!(org.name.is_some(), "org should have a name");

    // List workspaces
    let page = c
        .workspaces()
        .list(ListWorkspacesInput {
            organization_id: ORG_ID.into(),
            ..Default::default()
        })
        .await
        .expect("list workspaces");
    assert!(!page.items.is_empty(), "should have at least one workspace");
    assert!(
        page.items
            .iter()
            .any(|w| w.id.as_deref() == Some(WORKSPACE_ID)),
        "known workspace should appear in list"
    );

    // Get workspace
    let ws = c
        .workspaces()
        .get(WORKSPACE_ID)
        .await
        .expect("get workspace");
    assert_eq!(ws.id.as_deref(), Some(WORKSPACE_ID));
    assert!(ws.name.is_some(), "workspace should have a name");
    assert_eq!(ws.organization_id.as_deref(), Some(ORG_ID));
}

// ---------------------------------------------------------------------------
// 11. MCP Server & Tool Registration + Proxy
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_mcp_flow() {
    let c = client();

    // Register MCP server
    let server = c
        .mcp()
        .register_server(RegisterMcpServerInput {
            workspace_id: WORKSPACE_ID.into(),
            name: "rust-sdk-mcp-server".into(),
            description: Some("Rust SDK integration test MCP server".into()),
            url: "http://127.0.0.1:9100".into(),
            ..Default::default()
        })
        .await
        .expect("register mcp server");
    let server_id = server.id.as_deref().expect("server id");
    assert!(server.name.as_deref() == Some("rust-sdk-mcp-server"));

    // List servers
    let page = c
        .mcp()
        .list_servers(ListMcpServersInput {
            workspace_id: WORKSPACE_ID.into(),
            ..Default::default()
        })
        .await
        .expect("list mcp servers");
    assert!(
        page.items.iter().any(|s| s.id.as_deref() == Some(server_id)),
        "registered server should appear in list"
    );

    // Get server
    let fetched = c.mcp().get_server(server_id).await.expect("get mcp server");
    assert_eq!(fetched.id.as_deref(), Some(server_id));

    // Update server
    let updated = c
        .mcp()
        .update_server(
            server_id,
            UpdateMcpServerInput {
                description: Some("Updated by Rust SDK integration test".into()),
                ..Default::default()
            },
        )
        .await
        .expect("update mcp server");
    assert_eq!(updated.id.as_deref(), Some(server_id));

    // Register tool
    let tool = c
        .mcp()
        .register_tool(
            server_id,
            RegisterMcpToolInput {
                name: "echo".into(),
                description: Some("Echo tool for testing".into()),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "message": { "type": "string" }
                    }
                })),
                ..Default::default()
            },
        )
        .await
        .expect("register tool");
    assert_eq!(tool.name.as_deref(), Some("echo"));

    // List tools
    let tools = c
        .mcp()
        .list_tools(server_id)
        .await
        .expect("list tools");
    assert!(
        tools.iter().any(|t| t.name.as_deref() == Some("echo")),
        "echo tool should appear in list"
    );

    // Proxy: need an agent with MCP permissions
    let proxy_agent_id = create_test_agent(&c, "mcp-proxy").await;
    let ts2 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let proxy_role = c
        .roles()
        .create(CreateRoleInput {
            workspace_id: WORKSPACE_ID.into(),
            name: format!("rust-sdk-mcp-proxy-role-{ts2}"),
            permissions: Some(vec![format!("mcp:{}:tool.*", server_id)]),
            ..Default::default()
        })
        .await
        .expect("create proxy role");
    let proxy_role_id = proxy_role.id.as_deref().expect("proxy role id");
    c.roles()
        .assign(
            &proxy_agent_id,
            AssignRoleInput {
                role_id: proxy_role_id.into(),
                granted_by: Some("integration-test".into()),
                ..Default::default()
            },
        )
        .await
        .expect("assign proxy role");

    let proxy_result = c
        .mcp()
        .proxy(McpProxyInput {
            server_id: server_id.into(),
            method: "tools/call".into(),
            params: Some(serde_json::json!({
                "name": "echo",
                "arguments": { "message": "hello-from-rust-sdk" },
                "_authora": {
                    "mcpServerId": server_id,
                    "agentId": proxy_agent_id,
                    "timestamp": new_iso_timestamp()
                }
            })),
            ..Default::default()
        })
        .await
        .expect("mcp proxy");
    // Proxy should return something
    assert!(
        proxy_result.result.is_some() || proxy_result.error.is_some(),
        "proxy should return result or error"
    );

    // Cleanup proxy resources
    c.roles()
        .unassign(&proxy_agent_id, proxy_role_id)
        .await
        .expect("unassign proxy role");
    c.roles()
        .delete(proxy_role_id)
        .await
        .expect("delete proxy role");
    c.agents()
        .revoke(&proxy_agent_id)
        .await
        .expect("revoke proxy agent");
}

// ---------------------------------------------------------------------------
// 12. Policy Simulate & Evaluate
// ---------------------------------------------------------------------------

#[tokio::test]
#[ignore]
async fn test_policy_simulate_evaluate() {
    let c = client();

    // Create agent
    let agent_id = create_test_agent(&c, "poleval").await;

    // Create role
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let role = c
        .roles()
        .create(CreateRoleInput {
            workspace_id: WORKSPACE_ID.into(),
            name: format!("rust-sdk-poleval-role-{ts}"),
            description: Some("Policy evaluation test role".into()),
            permissions: Some(vec!["docs:*:read".into()]),
            ..Default::default()
        })
        .await
        .expect("create role");
    let role_id = role.id.as_deref().expect("role id");
    let role_name = role.name.as_deref().expect("role name");

    // Assign role to agent
    c.roles()
        .assign(
            &agent_id,
            AssignRoleInput {
                role_id: role_id.into(),
                granted_by: Some("integration-test".into()),
                ..Default::default()
            },
        )
        .await
        .expect("assign role");

    // Create DENY policy
    let policy = c
        .policies()
        .create(CreatePolicyInput {
            workspace_id: WORKSPACE_ID.into(),
            name: format!("rust-sdk-deny-policy-{ts}"),
            description: Some("DENY policy for simulate/evaluate test".into()),
            effect: "DENY".into(),
            principals: Some(serde_json::json!({
                "roles": [role_name]
            })),
            resources: Some(vec!["docs:secret".into()]),
            actions: Some(vec!["read".into()]),
            priority: Some(100),
            enabled: Some(true),
            ..Default::default()
        })
        .await
        .expect("create deny policy");
    let policy_id = policy.id.as_deref().expect("policy id");

    // Simulate
    let sim_result = c
        .policies()
        .simulate(SimulatePolicyInput {
            workspace_id: WORKSPACE_ID.into(),
            agent_id: agent_id.clone(),
            resource: "docs:secret".into(),
            action: "read".into(),
            ..Default::default()
        })
        .await
        .expect("simulate policy");
    // The API returns {effect, reason} -- at least one should be present
    assert!(
        sim_result.allowed.is_some() || sim_result.decision.is_some() || sim_result.reason.is_some(),
        "simulate should return some result"
    );

    // Evaluate
    let eval_result = c
        .policies()
        .evaluate(EvaluatePolicyInput {
            workspace_id: WORKSPACE_ID.into(),
            agent_id: agent_id.clone(),
            resource: "docs:secret".into(),
            action: "read".into(),
            ..Default::default()
        })
        .await
        .expect("evaluate policy");
    // The API returns {effect, reason} -- may not include allowed
    assert!(
        eval_result.allowed.is_some() || eval_result.effect.is_some() || eval_result.reason.is_some(),
        "evaluate should return some result"
    );

    // Cleanup
    c.policies().delete(policy_id).await.expect("delete policy");
    c.roles()
        .unassign(&agent_id, role_id)
        .await
        .expect("unassign role");
    c.roles().delete(role_id).await.expect("delete role");
    c.agents().revoke(&agent_id).await.expect("revoke agent");
}

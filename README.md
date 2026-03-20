# Authora Rust SDK

Official Rust client library for the [Authora](https://authora.dev) agent authorization platform.

## Requirements

- Rust 2021 edition (MSRV 1.70)
- Async runtime: [Tokio](https://tokio.rs)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
authora = "0.2"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Find your credentials

Several SDK methods require identifiers that are generated when you sign up:

| Value | Format | Where to find it |
|---|---|---|
| **API Key** | `authora_live_...` | [Account page](https://www.authora.dev/account) > API Keys tab |
| **Workspace ID** | `ws_...` | [Account page](https://www.authora.dev/account) > Profile tab > SDK Quick Start |
| **User ID** | `usr_...` | [Account page](https://www.authora.dev/account) > Profile tab > User ID |
| **Organization ID** | `org_...` | [Account page](https://www.authora.dev/account) > Profile tab > Organization ID |

The `created_by` parameter used when creating agents or API keys is your **User ID** (`usr_...`).

## Quick start

```rust
use authora::{AuthoraClient, types::CreateAgentInput};

#[tokio::main]
async fn main() -> Result<(), authora::AuthoraError> {
    let client = AuthoraClient::new("authora_live_...")?; // from Account > API Keys

    let agent = client.agents().create(CreateAgentInput {
        workspace_id: "ws_...".into(),    // from Account > Profile
        name: "my-agent".into(),
        created_by: "usr_...".into(),     // your User ID
        ..Default::default()
    }).await?;

    println!("Created agent: {:?}", agent.id);
    Ok(())
}
```

## Custom configuration

```rust
use std::time::Duration;
use authora::AuthoraClient;

let client = AuthoraClient::builder("authora_live_...")
    .base_url("https://api.authora.dev/api/v1")
    .timeout(Duration::from_secs(60))
    .build()?;
```

## Edge Endpoints

For high-availability scenarios, Authora provides an edge proxy at `https://edge.authora.dev` powered by Cloudflare Workers. Agent identity verification, JWT validation, and public key lookups are served from globally distributed edge caches with 24-hour survivability if the origin is unreachable. The edge proxy runs in parallel with the primary API -- no client changes required.

## Resource modules

The client exposes one method per resource area. Each returns a lightweight handle
that borrows the underlying HTTP client:

| Method                  | Resource             | Endpoints |
|-------------------------|----------------------|-----------|
| `client.agents()`      | Agents               | 8         |
| `client.roles()`       | Roles + Assignments  | 8         |
| `client.permissions()` | Permission Checks    | 3         |
| `client.delegations()` | Delegations          | 6         |
| `client.policies()`    | Policies             | 12        |
| `client.mcp()`         | MCP Servers / Tools  | 7         |
| `client.audit()`       | Audit Events         | 4         |
| `client.notifications()` | Notifications      | 4         |
| `client.webhooks()`    | Webhooks             | 4         |
| `client.alerts()`      | Alerts               | 4         |
| `client.api_keys()`    | API Keys             | 3         |
| `client.organizations()` | Organizations      | 3         |
| `client.workspaces()`  | Workspaces           | 3         |
| `client.approvals()`  | Approval Challenges  | 13        |
| `client.credits()`    | Credit Balance       | 3         |
| `client.user_delegations()` | User Delegation Grants | 11  |

**Total: 90 endpoints**

## Examples

### Check a permission

```rust
use authora::types::CheckPermissionInput;

let result = client.permissions().check(CheckPermissionInput {
    agent_id: "agt_abc".into(),
    resource: "files:*".into(),
    action: "read".into(),
    ..Default::default()
}).await?;

if result.allowed == Some(true) {
    println!("Access granted");
}
```

### List agents

```rust
use authora::types::ListAgentsInput;

let page = client.agents().list(ListAgentsInput {
    workspace_id: "ws_123".into(),
    ..Default::default()
}).await?;

for agent in &page.data {
    println!("{}: {:?}", agent.id.as_deref().unwrap_or("-"), agent.status);
}
```

### Create a delegation

```rust
use authora::types::{CreateDelegationInput, DelegatedPermission};

let delegation = client.delegations().create(CreateDelegationInput {
    from_agent_id: "agt_1".into(),
    to_agent_id: "agt_2".into(),
    workspace_id: "ws_123".into(),
    permissions: Some(vec![DelegatedPermission {
        resource: "files:reports/*".into(),
        actions: vec!["read".into()],
    }]),
    ..Default::default()
}).await?;
```

### Manage policies

```rust
use authora::types::CreatePolicyInput;

let policy = client.policies().create(CreatePolicyInput {
    workspace_id: "ws_123".into(),
    name: "deny-delete".into(),
    effect: "deny".into(),
    resources: vec!["*".into()],
    actions: vec!["delete".into()],
    ..Default::default()
}).await?;
```

## Error handling

All methods return `Result<T, AuthoraError>`. The error enum covers:

- `AuthoraError::Api` -- Non-success HTTP status with message and optional error code
- `AuthoraError::Authentication` -- 401 / 403 responses
- `AuthoraError::NotFound` -- 404 responses
- `AuthoraError::RateLimit` -- 429 responses
- `AuthoraError::Timeout` -- Request timeout
- `AuthoraError::Network` -- Connection / transport errors (wraps `reqwest::Error`)
- `AuthoraError::Serialization` -- JSON serialization / deserialization failures

## JSON field mapping

Rust structs use `snake_case` fields. Serde automatically maps them to/from `camelCase`
for the Authora API via `#[serde(rename_all = "camelCase")]`.

## Agent Runtime

The `AgentRuntime` provides a full async agent runtime with Ed25519 signed requests, tokio `RwLock`-protected permission caching, delegation, and MCP tool calls.

```rust
use authora::{AuthoraClient, AgentRuntime, generate_key_pair};
use authora::types::{CreateAgentInput, AgentOptions, ToolCallParams, DelegationConstraints};

#[tokio::main]
async fn main() -> Result<(), authora::AuthoraError> {
    let client = AuthoraClient::new("authora_live_...")?;

    // Create + activate an agent (generates Ed25519 keypair locally)
    let (runtime, key_pair) = client.create_agent(CreateAgentInput {
        workspace_id: "ws_...".into(),        // from Account > Profile
        name: "data-processor".into(),
        created_by: "usr_...".into(),         // your User ID
        ..Default::default()
    }).await?;

    // All requests are Ed25519-signed automatically
    let profile = runtime.get_profile().await?;
    let doc = runtime.get_identity_document().await?;

    // Server-side permission check
    let result = runtime.check_permission("files:read", "read", None).await?;

    // Client-side cached check (deny-first, 5-minute TTL, Arc<RwLock>)
    if runtime.has_permission("mcp:server1:tool.query").await? {
        let result = runtime.call_tool(ToolCallParams {
            tool_name: "query".into(),
            arguments: Some(serde_json::json!({"sql": "SELECT 1"})),
            ..Default::default()
        }).await?;
    }

    // Delegate permissions
    let delegation = runtime.delegate(
        "agent_...",
        vec!["files:read".into()],
        Some(DelegationConstraints { expires_in: Some("1h".into()), ..Default::default() }),
    ).await?;

    // Key rotation
    let (updated, new_key_pair) = runtime.rotate_key().await?;

    // Lifecycle
    runtime.suspend().await?;
    let (reactivated, fresh_key_pair) = runtime.reactivate().await?;
    runtime.revoke().await?;

    Ok(())
}
```

## Cryptography

Ed25519 key generation, signing, and verification via `ed25519_dalek`.

```rust
use authora::{generate_key_pair};
use authora::crypto::{sign, verify, build_signature_payload, sha256_hash};

// Generate Ed25519 keypair (base64url encoded)
let key_pair = generate_key_pair();

// Sign and verify
let signature = sign("hello world", &key_pair.private_key)?;
let valid = verify("hello world", &signature, &key_pair.public_key);

// Build canonical signature payload
let payload = build_signature_payload("POST", "/api/v1/agents", &timestamp, Some("{}"));
```

## License

MIT

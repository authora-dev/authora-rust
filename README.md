# Authora Rust SDK

Official Rust client library for the [Authora](https://authora.dev) agent authorization platform.

## Requirements

- Rust 2021 edition (MSRV 1.70)
- Async runtime: [Tokio](https://tokio.rs)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
authora = "0.1.0"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick start

```rust
use authora::{AuthoraClient, types::CreateAgentInput};

#[tokio::main]
async fn main() -> Result<(), authora::AuthoraError> {
    let client = AuthoraClient::new("authora_live_...")?;

    let agent = client.agents().create(CreateAgentInput {
        workspace_id: "ws_123".into(),
        name: "my-agent".into(),
        created_by: "user_456".into(),
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

## Resource modules

The client exposes one method per resource area. Each returns a lightweight handle
that borrows the underlying HTTP client:

| Method                  | Resource             | Endpoints |
|-------------------------|----------------------|-----------|
| `client.agents()`      | Agents               | 8         |
| `client.roles()`       | Roles + Assignments  | 8         |
| `client.permissions()` | Permission Checks    | 3         |
| `client.delegations()` | Delegations          | 6         |
| `client.policies()`    | Policies             | 6         |
| `client.mcp()`         | MCP Servers / Tools  | 7         |
| `client.audit()`       | Audit Events         | 4         |
| `client.notifications()` | Notifications      | 4         |
| `client.webhooks()`    | Webhooks             | 4         |
| `client.alerts()`      | Alerts               | 4         |
| `client.api_keys()`    | API Keys             | 3         |
| `client.organizations()` | Organizations      | 3         |
| `client.workspaces()`  | Workspaces           | 3         |

**Total: 63 endpoints**

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

## License

MIT

//! # Authora SDK for Rust
//!
//! Official Rust client for the [Authora](https://authora.dev) agent
//! authorization platform.
//!
//! ## Quick start
//!
//! ```rust,no_run
//! use authora::{AuthoraClient, types::CreateAgentInput};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), authora::AuthoraError> {
//!     let client = AuthoraClient::new("authora_live_...")?;
//!
//!     let agent = client.agents().create(CreateAgentInput {
//!         workspace_id: "ws_123".into(),
//!         name: "my-agent".into(),
//!         created_by: "user_456".into(),
//!         ..Default::default()
//!     }).await?;
//!
//!     println!("Created agent: {:?}", agent.id);
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod http;
pub mod resources;
pub mod types;

// Re-exports for convenience.
pub use client::{AuthoraClient, AuthoraClientBuilder};
pub use error::AuthoraError;

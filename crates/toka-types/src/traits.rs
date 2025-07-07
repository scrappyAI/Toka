use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;

/// Common parameter map type used by tools.
pub type Params = HashMap<String, String>;

/// Core abstraction for executable tools in the Toka ecosystem.
///
/// The trait is intentionally minimal and lives in `toka-types` so it can be
/// shared without creating cyclic dependencies.  All higher-level crates
/// (`toka-tools`, `toka-runtime`, etc.) should depend on *this* definition.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Canonical registry name (snake_case).
    fn name(&self) -> &str;
    /// Human-readable description.
    fn description(&self) -> &str;
    /// Semantic version string (e.g. "1.2.0").
    fn version(&self) -> &str;

    /// Validate input parameters. Implementations must reject unknown keys and
    /// malformed values to maintain kernel safety.
    async fn validate(&self, params: &Params) -> Result<()>;

    /// Execute the tool.
    async fn execute(&self, params: &Params) -> Result<String>;
}

/// Minimal interface for an agent instance that can receive kernel messages.
#[async_trait]
pub trait Agent: Send + Sync {
    /// Called by the runtime when the agent receives a `Message`.
    async fn handle_message(&self, msg: &crate::Message) -> Result<()>;
}

/// Marker trait for resources that can be granted to agents/tools (e.g. file
/// handles, network sockets, GPU contexts).
pub trait Resource: Send + Sync {
    /// Stable identifier of the resource owner / handle.
    fn id(&self) -> crate::EntityId;
}
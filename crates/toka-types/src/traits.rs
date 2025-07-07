use std::collections::HashMap;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Serialize, Deserialize};

/// Common parameter map type used by tools (key-value string pairs).
pub type Params = HashMap<String, String>;

/// Structured parameter wrapper used by tools and registries. Keeps the
/// original `ToolParams` naming used across the codebase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ToolParams {
    /// Name of the tool for which the parameters are intended (useful for
    /// logging/audit).
    pub name: String,
    /// Arbitrary key-value argument map.
    #[serde(default)]
    pub args: Params,
}

impl ToolParams {
    /// Borrow the inner map for APIs that expect `Params`.
    pub fn as_map(&self) -> &Params {
        &self.args
    }
}

/// Execution metadata returned by every tool run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolMetadata {
    /// Wall-clock execution duration in milliseconds.
    pub execution_time_ms: u64,
    /// Semantic version of the tool implementation.
    pub tool_version: String,
    /// Unix timestamp when the tool finished execution.
    pub timestamp: u64,
}

/// Standard result wrapper for tool execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolResult {
    /// Whether execution was successful.
    pub success: bool,
    /// Tool output payload (format defined by tool implementation).
    pub output: String,
    /// Execution metadata including timing, version etc.
    pub metadata: ToolMetadata,
}

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

    /// Validate input parameters.
    fn validate_params(&self, params: &ToolParams) -> Result<()>;

    /// Execute the tool.
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult>;
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
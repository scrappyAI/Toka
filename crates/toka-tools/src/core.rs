//! Core tooling abstractions – migrated from the former `toka-toolkit-core` crate.
//!
//! The module was moved into `toka-tools` as part of the *crate consolidation*
//! described in `docs/code-clarity-report.md` (July 2025).
//!
//! Downstream crates should `use toka_tools::{Tool, ToolRegistry, …}` which are
//! re-exported at the crate root.

#![allow(clippy::module_name_repetitions)]

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Execution metadata returned by every tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    /// Wall-clock execution duration in milliseconds.
    pub execution_time_ms: u64,
    /// Semver of the tool implementation.
    pub tool_version: String,
    /// Unix timestamp when the tool finished.
    pub timestamp: u64,
}

/// Result wrapper for tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the execution was successful.
    pub success: bool,
    /// Arbitrary string output (tools decide the format).
    pub output: String,
    /// Execution metadata.
    pub metadata: ToolMetadata,
}

/// Parameters passed to a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    /// Name of the target tool (for auditing; redundant but handy).
    pub name: String,
    /// Arbitrary key-value argument map.
    #[serde(default)]
    pub args: HashMap<String, String>,
}

/// Contract every tool must implement.
#[async_trait]
pub trait Tool: Send + Sync {
    /// Canonical name by which the tool is looked up in a registry.
    fn name(&self) -> &str;
    /// Human-readable, short description.
    fn description(&self) -> &str;
    /// Semantic version.
    fn version(&self) -> &str;

    /// Execute the tool with the given parameters.
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult>;

    /// Validate parameters before execution.
    fn validate_params(&self, params: &ToolParams) -> Result<()>;
}

/// Minimal registry – ships empty; higher-level crates register built-ins.
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool + Send + Sync>>>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl ToolRegistry {
    /// Create an *empty* registry.
    pub fn new_empty() -> Self {
        Self::default()
    }

    /// Alias for historical `new()` constructor which used to ship with built-ins.
    pub async fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Register a new tool instance.
    pub async fn register_tool(&self, tool: Arc<dyn Tool + Send + Sync>) -> Result<()> {
        let name = tool.name().to_string();
        let mut map = self.tools.write().await;
        if map.contains_key(&name) {
            anyhow::bail!("Tool already registered: {name}");
        }
        map.insert(name.clone(), tool);
        info!("Registered tool: {name}");
        Ok(())
    }

    /// Fetch a tool by name.
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool + Send + Sync>> {
        let map = self.tools.read().await;
        map.get(name).cloned()
    }

    /// Execute a tool by name.
    pub async fn execute_tool(&self, name: &str, params: &ToolParams) -> Result<ToolResult> {
        let tool = {
            let map = self.tools.read().await;
            map.get(name).cloned()
        }
        .ok_or_else(|| anyhow::anyhow!("Tool not found: {name}"))?;

        tool.validate_params(params)?;
        let start = std::time::Instant::now();
        let mut result = tool
            .execute(params)
            .await
            .with_context(|| format!("Tool {name} failed"))?;
        result.metadata.execution_time_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// List registered tool names.
    pub async fn list_tools(&self) -> Vec<String> {
        let map = self.tools.read().await;
        map.keys().cloned().collect()
    }
}

// Re-export former sub-modules for backwards compatibility. Full implementations
// have been copied one-to-one from the original crate.
#[path = "manifest.rs"]
pub mod manifest;
#[path = "loader.rs"]
pub mod loader;
//! Standard **toolkit** crate – currently empty.
//!
//! This crate provides two things:
//! 1. `ToolRegistry` – thin wrapper around `toka_toolkit_core::ToolRegistry` so
//!    downstream code can keep using `toka_tools::ToolRegistry` while we
//!    rebuild the standard library of tools.
//! 2. Authoring guidelines (`TOOL_GUIDELINES`) embedded as a constant string so
//!    dev tooling can surface them programmatically.

use anyhow::Result;
use std::sync::Arc;
use async_trait::async_trait;

// Re-export canonical types from `toka_toolkit_core` so internal modules can
// simply `use crate::tools::{Tool, ToolParams, …}` without caring about the
// crate boundary.
pub use toka_toolkit_core::{Tool, ToolMetadata, ToolParams, ToolRegistry as CoreRegistry, ToolResult};

/// Thin wrapper that delegates every call to an inner `CoreRegistry`.
/// At the moment **no tools are registered by default** – callers must install
/// their own tool instances.
pub struct ToolRegistry {
    inner: CoreRegistry,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self { inner: CoreRegistry::new() }
    }
}

impl ToolRegistry {
    /// Create an *empty* registry.
    pub fn new_empty() -> Self { Self::default() }

    /// Back-compat helper: historically `ToolRegistry::new()` shipped with
    /// built-ins.  For now it's an alias for `new_empty` while the standard
    /// toolkit is being rewritten.
    pub async fn new() -> Result<Self> { Ok(Self::default()) }

    pub async fn register_tool(&self, t: Arc<dyn Tool + Send + Sync>) -> Result<()> {
        self.inner.register_tool(t).await
    }

    pub async fn execute_tool(&self, name: &str, p: &ToolParams) -> Result<ToolResult> {
        self.inner.execute_tool(name, p).await
    }

    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool + Send + Sync>> {
        self.inner.get_tool(name).await
    }

    pub async fn list_tools(&self) -> Vec<String> { self.inner.list_tools().await }
}

/// Canonical guidelines for building agent tools – markdown formatted.
pub const TOOL_GUIDELINES: &str = include_str!("../../TOOL_DEVELOPMENT.md");

/// Example tool implementation
pub struct EchoTool {
    name: String,
    description: String,
    version: String,
}

impl EchoTool {
    pub fn new() -> Self {
        Self {
            name: "echo".to_string(),
            description: "Echoes back the input".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let message = params
            .args
            .get("message")
            .ok_or_else(|| anyhow::anyhow!("Missing 'message' parameter"))?;

        Ok(ToolResult {
            success: true,
            output: message.clone(),
            metadata: ToolMetadata {
                execution_time_ms: 0,
                tool_version: self.version.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("message") {
            return Err(anyhow::anyhow!("Missing required parameter: message"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_registry() -> Result<()> {
        let registry = ToolRegistry::new().await?;

        // Test listing tools
        let tools = registry.list_tools().await;
        assert!(tools.contains(&"ingestion".to_string()));
        assert!(tools.contains(&"ledger".to_string()));
        assert!(tools.contains(&"scheduling".to_string()));
        assert!(tools.contains(&"reporting".to_string()));
        assert!(tools.contains(&"semantic_index".to_string()));
        assert!(tools.contains(&"coverage-json".to_string()));
        assert!(tools.contains(&"coverage-analyse".to_string()));

        Ok(())
    }
}

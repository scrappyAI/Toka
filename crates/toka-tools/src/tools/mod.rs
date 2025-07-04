//! Standard **toolkit** crate – currently empty.
//!
//! This crate provides two things:
//! 1. `ToolRegistry` – thin wrapper around `toka_toolkit_core::ToolRegistry` so
//!    downstream code can keep using `toka_tools::ToolRegistry` while we
//!    rebuild the standard library of tools.
//! 2. Authoring guidelines (`TOOL_GUIDELINES`) embedded as a constant string so
//!    dev tooling can surface them programmatically.

use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use std::sync::Arc;

// Re-export canonical types from `toka_toolkit_core` so internal modules can
// simply `use crate::tools::{Tool, ToolParams, …}` without caring about the
// crate boundary.
pub use crate::core::{Tool, ToolMetadata, ToolParams, ToolResult};

/// Thin wrapper that delegates every call to an inner `CoreRegistry`.
/// At the moment **no tools are registered by default** – callers must install
/// their own tool instances.
pub type ToolRegistry = crate::core::ToolRegistry;

/// Canonical guidelines for building agent tools – markdown formatted.
pub const TOOL_GUIDELINES: &str = include_str!("../../TOOL_DEVELOPMENT.md");

/// Example tool implementation
pub struct EchoTool {
    name: String,
    description: String,
    version: String,
}

impl EchoTool {
    /// Create a new instance of the echo tool
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

        // Register echo tool
        registry.register_tool(Arc::new(EchoTool::new())).await?;

        // Listing should contain "echo"
        let tools = registry.list_tools().await;
        assert_eq!(tools, vec!["echo".to_string()]);

        Ok(())
    }
}

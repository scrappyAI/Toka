use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod coverage;
mod ingestion;
mod ledger;
mod reporting;
mod scheduling;
mod semantic_index;

pub use coverage::{CoverageAnalysisTool, CoverageJsonTool};
pub use ingestion::IngestionTool;
pub use ledger::LedgerTool;
pub use reporting::ReportingTool;
pub use scheduling::SchedulingTool;
pub use semantic_index::{ItemMetadata, SemanticIndexTool, TaggedItem};

// Re-export canonical types from `toka_toolkit_core` so internal modules can
// simply `use crate::tools::{Tool, ToolParams, …}` without caring about the
// crate boundary.
pub use toka_toolkit_core::{Tool, ToolMetadata, ToolParams, ToolResult};

// Alias for brevity inside this module only
use toka_toolkit_core as core;

/// Concrete `ToolRegistry` that bundles all **standard** tools behind the
/// `toka-tools` crate.  It delegates to the lightweight implementation in
/// `toka_toolkit_core` while adding convenience constructors.
pub struct ToolRegistry {
    inner: core::ToolRegistry,
}

impl ToolRegistry {
    /// Create a new registry and eagerly register all default tools shipped by
    /// this crate.  This mirrors the previous behaviour so existing callers can
    /// keep using `ToolRegistry::new().await?`.
    pub async fn new() -> Result<Self> {
        let registry = Self {
            inner: core::ToolRegistry::new(),
        };

        // Register default tools (feature-gated refactor TBD)
        registry.register_tool(Arc::new(IngestionTool::new())).await?;
        registry.register_tool(Arc::new(LedgerTool::new())).await?;
        registry.register_tool(Arc::new(SchedulingTool::new())).await?;
        registry.register_tool(Arc::new(ReportingTool::new())).await?;
        registry.register_tool(Arc::new(SemanticIndexTool::new())).await?;
        registry.register_tool(Arc::new(CoverageJsonTool::new())).await?;
        registry.register_tool(Arc::new(CoverageAnalysisTool::new())).await?;
        registry.register_tool(Arc::new(EchoTool::new())).await?;

        Ok(registry)
    }

    /// Forwarder for manual registration of additional tools.
    pub async fn register_tool(&self, tool: Arc<dyn Tool + Send + Sync>) -> Result<()> {
        self.inner.register_tool(tool).await
    }

    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool + Send + Sync>> {
        self.inner.get_tool(name).await
    }

    pub async fn list_tools(&self) -> Vec<String> {
        self.inner.list_tools().await
    }

    pub async fn execute_tool(&self, name: &str, params: &ToolParams) -> Result<ToolResult> {
        self.inner.execute_tool(name, params).await
    }
}

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

/// Resolve a URI to a concrete filesystem path.
///
/// Currently supports only the `local://` scheme which maps to
/// `~/.toka/storage/…`.  For backwards-compatibility any string **without** a
/// scheme is treated as an absolute/relative path untouched.
pub fn resolve_uri_to_path(uri: &str) -> PathBuf {
    if let Some(path) = uri.strip_prefix("local://") {
        let root = directories::BaseDirs::new()
            .map(|d| d.home_dir().join(".toka/storage"))
            .unwrap_or_else(|| PathBuf::from(".toka/storage"));
        root.join(path)
    } else {
        PathBuf::from(uri)
    }
}

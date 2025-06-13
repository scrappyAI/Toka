//! Toka Toolkit Core
//!
//! Defines the `Tool` trait, data structures, and a lightweight `ToolRegistry`.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use tracing::info;

/// Execution metadata returned by every tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub execution_time_ms: u64,
    pub tool_version: String,
    pub timestamp: u64,
}

/// Result wrapper for tool execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub metadata: ToolMetadata,
}

/// Parameters passed to a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub name: String,
    #[serde(default)]
    pub args: HashMap<String, String>,
}

/// Contract every tool must implement.
#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn version(&self) -> &str;
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult>;
    fn validate_params(&self, params: &ToolParams) -> Result<()>;
}

/// Minimal registry — ships empty; higher-level crates register built-ins.
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool + Send + Sync>>>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self { tools: Arc::new(RwLock::new(HashMap::new())) }
    }
}

impl ToolRegistry {
    pub fn new() -> Self { Self::default() }

    pub fn new_empty() -> Self { Self::default() }

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

    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool + Send + Sync>> {
        let map = self.tools.read().await;
        map.get(name).cloned()
    }

    pub async fn execute_tool(&self, name: &str, params: &ToolParams) -> Result<ToolResult> {
        let tool = {
            let map = self.tools.read().await;
            map.get(name).cloned()
        }.ok_or_else(|| anyhow::anyhow!("Tool not found: {name}"))?;

        tool.validate_params(params)?;
        let start = std::time::Instant::now();
        let mut result = tool.execute(params).await.with_context(|| format!("Tool {name} failed"))?;
        result.metadata.execution_time_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    pub async fn list_tools(&self) -> Vec<String> {
        let map = self.tools.read().await;
        map.keys().cloned().collect()
    }
} 
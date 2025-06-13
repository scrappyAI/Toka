use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use tracing::info;
use async_trait::async_trait;

mod ingestion;
mod ledger;
mod scheduling;
mod reporting;
mod semantic_index;

pub use ingestion::IngestionTool;
pub use ledger::LedgerTool;
pub use scheduling::SchedulingTool;
pub use reporting::ReportingTool;
pub use semantic_index::SemanticIndexTool;

/// Tool execution result with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub output: String,
    pub metadata: ToolMetadata,
}

/// Metadata for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub execution_time_ms: u64,
    pub tool_version: String,
    pub timestamp: u64,
}

/// Tool execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParams {
    pub name: String,
    pub args: HashMap<String, String>,
}

/// Core trait that all tools must implement
#[async_trait]
pub trait Tool: Send + Sync {
    /// Get the tool's name
    fn name(&self) -> &str;
    
    /// Get the tool's description
    fn description(&self) -> &str;
    
    /// Get the tool's version
    fn version(&self) -> &str;
    
    /// Execute the tool with given parameters
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult>;
    
    /// Validate the tool parameters
    fn validate_params(&self, params: &ToolParams) -> Result<()>;
}

/// Tool registry for managing available tools
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool + Send + Sync>>>>,
}

impl ToolRegistry {
    /// Create a new tool registry with all default tools
    pub async fn new() -> Result<Self> {
        let registry = Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        };

        // Register default tools
        registry.register_tool(Arc::new(IngestionTool::new())).await?;
        registry.register_tool(Arc::new(LedgerTool::new())).await?;
        registry.register_tool(Arc::new(SchedulingTool::new())).await?;
        registry.register_tool(Arc::new(ReportingTool::new())).await?;
        registry.register_tool(Arc::new(SemanticIndexTool::new())).await?;
        registry.register_tool(Arc::new(EchoTool::new())).await?;

        Ok(registry)
    }
    
    /// Register a new tool
    pub async fn register_tool(&self, tool: Arc<dyn Tool + Send + Sync>) -> Result<()> {
        let name = tool.name().to_string();
        let mut tools = self.tools.write().await;
        
        if tools.contains_key(&name) {
            return Err(anyhow::anyhow!("Tool already registered: {}", name));
        }
        
        tools.insert(name.clone(), tool);
        info!("Registered tool: {}", name);
        Ok(())
    }
    
    /// Get a tool by name
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool + Send + Sync>> {
        let tools = self.tools.read().await;
        tools.get(name).cloned()
    }
    
    /// List all registered tools
    pub async fn list_tools(&self) -> Vec<String> {
        let tools = self.tools.read().await;
        tools.keys().cloned().collect()
    }
    
    /// Execute a tool by name with parameters
    pub async fn execute_tool(&self, name: &str, params: &ToolParams) -> Result<ToolResult> {
        let tool = self.get_tool(name).await
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
            
        let start_time = std::time::Instant::now();
        
        // Validate parameters
        tool.validate_params(params)?;
        
        // Execute tool
        let result = tool.execute(params).await
            .with_context(|| format!("Failed to execute tool: {}", name))?;
            
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(ToolResult {
            success: result.success,
            output: result.output,
            metadata: ToolMetadata {
                execution_time_ms: execution_time,
                tool_version: tool.version().to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
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
        let message = params.args.get("message")
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
        
        Ok(())
    }
} 
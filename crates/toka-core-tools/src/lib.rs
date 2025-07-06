//! Toka Core Tools - Kernel-Enforced Tool Implementations
//!
//! This crate provides the core tool implementations that use the toka-kernel
//! enforcement layer for secure and controlled execution. All tools in this
//! crate follow the CoreBaseline guidelines and integrate with the kernel
//! security model.
//!
//! # Architecture
//!
//! Core tools are built around the kernel enforcement pattern:
//!
//! 1. **Tool Registration**: Tools declare their capabilities and requirements
//! 2. **Kernel Integration**: All operations go through kernel enforcement
//! 3. **Security Validation**: File access, network operations, and resource usage are validated
//! 4. **Resource Management**: Memory, CPU, and other resources are tracked and limited
//! 5. **Error Handling**: Rich error context with security violation reporting
//!
//! # Usage
//!
//! ```rust
//! use toka_core_tools::{ToolRegistry, ExecutionRequest};
//! use toka_kernel::{ToolKernel, SecurityLevel};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize kernel and registry
//!     let kernel = ToolKernel::new().await?;
//!     let registry = ToolRegistry::new(kernel).await?;
//!     
//!     // Execute a tool with kernel enforcement
//!     let request = ExecutionRequest {
//!         tool_id: "file_reader".to_string(),
//!         session_id: "user_session".to_string(),
//!         parameters: serde_json::json!({"path": "./README.md"}),
//!         security_level: SecurityLevel::Restricted,
//!     };
//!     
//!     let result = registry.execute_tool(request).await?;
//!     println!("Tool result: {}", result);
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

// Re-export kernel types for convenience
pub use toka_kernel::{
    ToolKernel, ExecutionContext, SecurityLevel, KernelError,
    Capability, CapabilitySet, FileAccess, NetworkAccess,
};

pub mod tools;
pub mod registry;
pub mod execution;
pub mod protocols;

/// Tool execution request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    /// Unique identifier for the tool to execute
    pub tool_id: String,
    /// Session identifier for capability checking
    pub session_id: String,
    /// Tool-specific parameters as JSON
    pub parameters: JsonValue,
    /// Security level for execution
    pub security_level: SecurityLevel,
    /// Optional timeout override
    pub timeout_override: Option<std::time::Duration>,
}

/// Tool execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Whether execution was successful
    pub success: bool,
    /// Tool output data
    pub output: JsonValue,
    /// Execution metadata
    pub metadata: ExecutionMetadata,
    /// Any errors that occurred
    pub error: Option<String>,
}

/// Execution metadata for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Tool identifier
    pub tool_id: String,
    /// Session identifier
    pub session_id: String,
    /// Execution duration
    pub duration: std::time::Duration,
    /// Resource usage during execution
    pub resource_usage: ResourceUsageSnapshot,
    /// Security level used
    pub security_level: SecurityLevel,
    /// Timestamp when execution started
    pub started_at: std::time::SystemTime,
}

/// Snapshot of resource usage during tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    /// Peak memory usage in MB
    pub peak_memory_mb: u64,
    /// Average CPU usage percentage
    pub avg_cpu_percent: f32,
    /// Number of file handles used
    pub file_handles_used: u32,
    /// Number of network connections made
    pub network_connections: u32,
}

/// Tool definition structure for registration
#[derive(Debug, Clone)]
pub struct ToolDefinition {
    /// Unique tool identifier
    pub id: String,
    /// Human-readable tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Version of the tool
    pub version: String,
    /// Required capabilities for execution
    pub required_capabilities: CapabilitySet,
    /// Tool implementation
    pub implementation: Arc<dyn Tool + Send + Sync>,
}

/// Core tool trait that all tools must implement
#[async_trait::async_trait]
pub trait Tool {
    /// Get tool metadata
    fn metadata(&self) -> ToolMetadata;
    
    /// Validate parameters before execution
    async fn validate_parameters(&self, parameters: &JsonValue) -> Result<()>;
    
    /// Execute the tool with kernel enforcement
    async fn execute(
        &self,
        context: &ExecutionContext,
        parameters: &JsonValue,
        kernel: &ToolKernel,
    ) -> Result<JsonValue>;
    
    /// Get tool-specific help information
    fn help(&self) -> String {
        format!("Help for {}", self.metadata().name)
    }
    
    /// Check if tool supports a specific capability
    fn supports_capability(&self, capability: &Capability) -> bool {
        self.metadata().required_capabilities.contains(capability)
    }
}

/// Tool metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub required_capabilities: CapabilitySet,
    pub parameter_schema: JsonValue,
    pub output_schema: JsonValue,
}

/// Main tool registry for managing and executing tools
pub struct ToolRegistry {
    kernel: Arc<ToolKernel>,
    tools: RwLock<HashMap<String, ToolDefinition>>,
    execution_history: RwLock<Vec<ExecutionResult>>,
}

impl ToolRegistry {
    /// Create new tool registry with kernel enforcement
    pub async fn new(kernel: ToolKernel) -> Result<Self> {
        Ok(Self {
            kernel: Arc::new(kernel),
            tools: RwLock::new(HashMap::new()),
            execution_history: RwLock::new(Vec::new()),
        })
    }
    
    /// Register a new tool in the registry
    pub async fn register_tool(&self, definition: ToolDefinition) -> Result<()> {
        // Validate tool definition
        self.validate_tool_definition(&definition).await?;
        
        let mut tools = self.tools.write().await;
        tools.insert(definition.id.clone(), definition);
        
        tracing::info!("Registered tool: {}", definition.id);
        Ok(())
    }
    
    /// Execute a tool with kernel enforcement
    pub async fn execute_tool(&self, request: ExecutionRequest) -> Result<ExecutionResult> {
        let start_time = std::time::Instant::now();
        let started_at = std::time::SystemTime::now();
        
        // Get tool definition
        let tools = self.tools.read().await;
        let tool_def = tools.get(&request.tool_id)
            .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", request.tool_id))?
            .clone();
        drop(tools);
        
        // Validate parameters
        tool_def.implementation.validate_parameters(&request.parameters).await?;
        
        // Create execution context with kernel
        let context = self.kernel.create_execution_context(
            &request.tool_id,
            &request.session_id,
            &tool_def.required_capabilities,
            request.security_level,
        ).await?;
        
        // Execute tool with kernel enforcement
        let result = self.kernel.enforce_execution(&context, async {
            tool_def.implementation.execute(&context, &request.parameters, &self.kernel).await
        }).await;
        
        let duration = start_time.elapsed();
        
        // Create execution result
        let execution_result = match result {
            Ok(output) => ExecutionResult {
                success: true,
                output,
                metadata: ExecutionMetadata {
                    tool_id: request.tool_id.clone(),
                    session_id: request.session_id.clone(),
                    duration,
                    resource_usage: self.get_resource_snapshot(&context).await,
                    security_level: request.security_level,
                    started_at,
                },
                error: None,
            },
            Err(error) => ExecutionResult {
                success: false,
                output: JsonValue::Null,
                metadata: ExecutionMetadata {
                    tool_id: request.tool_id.clone(),
                    session_id: request.session_id.clone(),
                    duration,
                    resource_usage: self.get_resource_snapshot(&context).await,
                    security_level: request.security_level,
                    started_at,
                },
                error: Some(error.to_string()),
            },
        };
        
        // Store execution history
        let mut history = self.execution_history.write().await;
        history.push(execution_result.clone());
        
        // Keep only recent executions (last 1000)
        if history.len() > 1000 {
            history.drain(0..100);
        }
        
        Ok(execution_result)
    }
    
    /// Get list of registered tools
    pub async fn list_tools(&self) -> Vec<ToolMetadata> {
        let tools = self.tools.read().await;
        tools.values()
            .map(|def| def.implementation.metadata())
            .collect()
    }
    
    /// Get tool by ID
    pub async fn get_tool(&self, tool_id: &str) -> Option<ToolDefinition> {
        let tools = self.tools.read().await;
        tools.get(tool_id).cloned()
    }
    
    /// Get execution history
    pub async fn get_execution_history(&self) -> Vec<ExecutionResult> {
        let history = self.execution_history.read().await;
        history.clone()
    }
    
    /// Validate tool definition before registration
    async fn validate_tool_definition(&self, definition: &ToolDefinition) -> Result<()> {
        // Validate tool ID
        toka_kernel::utils::validate_tool_id(&definition.id)?;
        
        // Check for existing tool with same ID
        let tools = self.tools.read().await;
        if tools.contains_key(&definition.id) {
            return Err(anyhow::anyhow!("Tool already registered: {}", definition.id));
        }
        
        // Validate metadata consistency
        let metadata = definition.implementation.metadata();
        if metadata.id != definition.id {
            return Err(anyhow::anyhow!("Tool ID mismatch in metadata"));
        }
        
        Ok(())
    }
    
    /// Get resource usage snapshot for execution metadata
    async fn get_resource_snapshot(&self, _context: &ExecutionContext) -> ResourceUsageSnapshot {
        // In a real implementation, this would get actual resource usage
        // from the kernel's execution monitor
        ResourceUsageSnapshot {
            peak_memory_mb: 0,
            avg_cpu_percent: 0.0,
            file_handles_used: 0,
            network_connections: 0,
        }
    }
}

/// Builder for creating tool registry with common tools
pub struct ToolRegistryBuilder {
    kernel: ToolKernel,
}

impl ToolRegistryBuilder {
    /// Create new builder with kernel
    pub fn new(kernel: ToolKernel) -> Self {
        Self { kernel }
    }
    
    /// Register all core tools
    pub async fn with_core_tools(self) -> Result<ToolRegistry> {
        let registry = ToolRegistry::new(self.kernel).await?;
        
        // Register core tools
        registry.register_tool(tools::file_reader::create_tool_definition()).await?;
        registry.register_tool(tools::file_writer::create_tool_definition()).await?;
        registry.register_tool(tools::command_executor::create_tool_definition()).await?;
        registry.register_tool(tools::http_client::create_tool_definition()).await?;
        
        Ok(registry)
    }
    
    /// Build registry with custom tools only
    pub async fn build(self) -> Result<ToolRegistry> {
        ToolRegistry::new(self.kernel).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_kernel::presets;

    #[tokio::test]
    async fn test_registry_creation() {
        let kernel = presets::testing_kernel().await.unwrap();
        let registry = ToolRegistry::new(kernel).await.unwrap();
        
        let tools = registry.list_tools().await;
        assert_eq!(tools.len(), 0); // No tools registered initially
    }
    
    #[tokio::test]
    async fn test_core_tools_registration() {
        let kernel = presets::development_kernel().await.unwrap();
        let registry = ToolRegistryBuilder::new(kernel)
            .with_core_tools()
            .await
            .unwrap();
        
        let tools = registry.list_tools().await;
        assert!(tools.len() > 0); // Should have core tools registered
        
        // Check for specific core tools
        let tool_ids: Vec<String> = tools.iter().map(|t| t.id.clone()).collect();
        assert!(tool_ids.contains(&"file_reader".to_string()));
        assert!(tool_ids.contains(&"file_writer".to_string()));
    }
} 
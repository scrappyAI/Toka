//! Toka Tools - Unified Tool Execution System
//!
//! This is the main facade crate for the Toka tool execution system. It provides
//! a unified interface to all tool-related functionality while maintaining clean
//! separation of concerns through the underlying kernel enforcement architecture.
//!
//! # Architecture Overview
//!
//! The Toka tools system is built on a layered architecture:
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │            toka-tools (facade)          │
//! ├─────────────────────────────────────────┤
//! │      toka-vector-registry (discovery)   │
//! ├─────────────────────────────────────────┤
//! │      toka-runtime (dynamic execution)   │
//! ├─────────────────────────────────────────┤
//! │      toka-core-tools (implementations)  │
//! ├─────────────────────────────────────────┤
//! │        toka-kernel (enforcement)        │
//! └─────────────────────────────────────────┘
//! ```
//!
//! # Key Features
//!
//! - **Kernel Enforcement**: All tool operations are secured through capability-based access control
//! - **Dynamic Discovery**: Vector-based semantic tool discovery and matching
//! - **Runtime Flexibility**: Support for Python, WASM, JavaScript, and native code execution
//! - **Security-First**: Multi-layered security with sandboxing and resource limits
//! - **Performance**: Efficient caching and resource management
//!
//! # Quick Start
//!
//! ```rust
//! use toka_tools::{ToolSystem, SecurityLevel};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize the complete tool system
//!     let system = ToolSystem::builder()
//!         .with_core_tools()
//!         .with_runtime_engines()
//!         .with_vector_discovery()
//!         .build()
//!         .await?;
//!     
//!     // Grant capabilities to a session
//!     system.grant_session_capabilities(
//!         "user_session",
//!         &CapabilitySet::workspace_files()
//!     ).await?;
//!     
//!     // Discover and execute tools
//!     let tools = system.discover_tools("read a JSON file").await?;
//!     if let Some(tool) = tools.first() {
//!         let result = system.execute_tool(
//!             &tool.tool.tool_id,
//!             "user_session",
//!             serde_json::json!({"path": "./data.json"}),
//!             SecurityLevel::Restricted
//!         ).await?;
//!         
//!         println!("Tool execution result: {}", result.output);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use anyhow::Result;
use serde_json::Value as JsonValue;

// Re-export all public types from underlying crates
pub use toka_kernel::{
    ToolKernel, ExecutionContext, SecurityLevel, ResourceLimits, ResourceUsage,
    ExecutionStats, KernelError, Capability, CapabilitySet, FileAccess, NetworkAccess,
};

// TODO: Uncomment when toka-core-tools has a Cargo.toml file
// pub use toka_core_tools::{
//     ToolRegistry, ToolDefinition, Tool, ToolMetadata, ExecutionRequest, ExecutionResult,
//     ExecutionMetadata, ToolRegistryBuilder,
// };

pub use toka_runtime::{
    RuntimeManager, CodeType, RuntimeBuilder,
    RuntimeMetadata, RuntimeResourceUsage, Artifact,
};

// TODO: Uncomment when toka-vector-registry has a Cargo.toml file
// pub use toka_vector_registry::{
//     VectorRegistry, ToolQuery, ToolDiscoveryResult, ToolRegistration, UsageStatistics,
// };

// TODO: Uncomment when toka-core-tools and toka-vector-registry are available
// /// Unified tool system that integrates all components
// pub struct ToolSystem {
//     kernel: Arc<ToolKernel>,
//     tool_registry: Arc<ToolRegistry>,
//     runtime_manager: Arc<RuntimeManager>,
//     vector_registry: Arc<VectorRegistry>,
// }

// TODO: Uncomment when dependencies are available
// /// Builder for creating a complete tool system
// pub struct ToolSystemBuilder {
//     include_core_tools: bool,
//     include_runtime_engines: bool,
//     include_vector_discovery: bool,
//     security_preset: SecurityPreset,
// }
// 
// /// Security presets for different environments
// #[derive(Debug, Clone)]
// pub enum SecurityPreset {
//     Development,
//     Testing,
//     Production,
//     Custom(ToolKernel),
// }
// 
// /// Unified execution request that can handle both direct tool calls and discovery
// #[derive(Debug, Clone)]
// pub struct UnifiedExecutionRequest {
//     /// Tool identifier or natural language query
//     pub tool_or_query: String,
//     /// Session identifier
//     pub session_id: String,
//     /// Execution parameters
//     pub parameters: JsonValue,
//     /// Security level
//     pub security_level: SecurityLevel,
//     /// Whether to use discovery if tool_id is not found
//     pub allow_discovery: bool,
//     /// Discovery similarity threshold
//     pub discovery_threshold: f32,
// }

// TODO: The following code is commented out until toka-core-tools and toka-vector-registry have Cargo.toml files
/*
impl ToolSystem {
    /// Create a new tool system builder
    pub fn builder() -> ToolSystemBuilder {
        ToolSystemBuilder::new()
    }
    
    /// Create tool system with development preset
    pub async fn development() -> Result<Self> {
        Self::builder()
            .with_core_tools()
            .with_runtime_engines()
            .with_vector_discovery()
            .with_security_preset(SecurityPreset::Development)
            .build()
            .await
    }
    
    /// Create tool system with production preset
    pub async fn production() -> Result<Self> {
        Self::builder()
            .with_core_tools()
            .with_security_preset(SecurityPreset::Production)
            .build()
            .await
    }
    
    /// Grant capabilities to a session
    pub async fn grant_session_capabilities(
        &self,
        session_id: &str,
        capabilities: &CapabilitySet,
    ) -> Result<()> {
        self.kernel.grant_capabilities(session_id, capabilities.clone()).await
            .map_err(|e| anyhow::anyhow!("Failed to grant capabilities: {}", e))
    }
    
    /// Revoke capabilities from a session
    pub async fn revoke_session_capabilities(&self, session_id: &str) -> Result<()> {
        self.kernel.revoke_capabilities(session_id).await
            .map_err(|e| anyhow::anyhow!("Failed to revoke capabilities: {}", e))
    }
    
    /// Execute a tool by ID
    pub async fn execute_tool(
        &self,
        tool_id: &str,
        session_id: &str,
        parameters: JsonValue,
        security_level: SecurityLevel,
    ) -> Result<ExecutionResult> {
        let request = ExecutionRequest {
            tool_id: tool_id.to_string(),
            session_id: session_id.to_string(),
            parameters,
            security_level,
            timeout_override: None,
        };
        
        self.tool_registry.execute_tool(request).await
    }
    
    /// Execute code dynamically
    pub async fn execute_code(
        &self,
        code_type: CodeType,
        code: &str,
        session_id: &str,
        security_level: SecurityLevel,
        inputs: JsonValue,
    ) -> Result<toka_runtime::ExecutionResult> {
        let request = toka_runtime::ExecutionRequest {
            code_type,
            code: code.to_string(),
            session_id: session_id.to_string(),
            security_level,
            inputs,
            timeout_override: None,
            environment: None,
        };
        
        self.runtime_manager.execute_code(request).await
    }
    
    /// Discover tools using natural language query
    pub async fn discover_tools(&self, query: &str) -> Result<Vec<ToolDiscoveryResult>> {
        let tool_query = ToolQuery::new(query);
        self.vector_registry.discover_tools(tool_query).await
    }
    
    /// Execute with automatic discovery fallback
    pub async fn execute_unified(&self, request: UnifiedExecutionRequest) -> Result<ExecutionResult> {
        // Try direct tool execution first
        let direct_result = self.execute_tool(
            &request.tool_or_query,
            &request.session_id,
            request.parameters.clone(),
            request.security_level.clone(),
        ).await;
        
        match direct_result {
            Ok(result) => Ok(result),
            Err(_) if request.allow_discovery => {
                // Tool not found, try discovery
                let discovered = self.discover_tools(&request.tool_or_query).await?;
                
                if let Some(tool_result) = discovered.first() {
                    if tool_result.similarity_score >= request.discovery_threshold {
                        return self.execute_tool(
                            &tool_result.tool.tool_id,
                            &request.session_id,
                            request.parameters,
                            request.security_level,
                        ).await;
                    }
                }
                
                Err(anyhow::anyhow!(
                    "No suitable tool found for query: {}",
                    request.tool_or_query
                ))
            },
            Err(e) => Err(e),
        }
    }
    
    /// List all available tools
    pub async fn list_tools(&self) -> Result<Vec<ToolMetadata>> {
        Ok(self.tool_registry.list_tools().await)
    }
    
    /// Get system statistics
    pub async fn get_system_stats(&self) -> Result<SystemStats> {
        let kernel_stats = self.kernel.get_execution_stats().await;
        let resource_usage = self.kernel.get_resource_usage().await;
        
        Ok(SystemStats {
            total_tools: self.tool_registry.list_tools().await.len(),
            total_executions: kernel_stats.total_executions,
            success_rate: if kernel_stats.total_executions > 0 {
                kernel_stats.successful_executions as f32 / kernel_stats.total_executions as f32
            } else {
                0.0
            },
            avg_execution_time: kernel_stats.average_execution_time,
            current_resource_usage: resource_usage,
            security_violations: kernel_stats.security_violations,
        })
    }
    
    /// Register a new tool
    pub async fn register_tool(&self, definition: ToolDefinition) -> Result<()> {
        // Register in tool registry
        self.tool_registry.register_tool(definition.clone()).await?;
        
        // Register in vector registry for discovery
        self.vector_registry.register_tool_from_description(
            &definition.id,
            &definition.description,
        ).await?;
        
        Ok(())
    }
    
    /// Generate code dynamically
    pub async fn generate_code(
        &self,
        prompt: &str,
        code_type: CodeType,
        session_id: &str,
    ) -> Result<String> {
        self.runtime_manager.generate_code(prompt, code_type, session_id).await
    }
    
    /// Get tool recommendations
    pub async fn get_recommendations(
        &self,
        session_id: &str,
        context: &str,
    ) -> Result<Vec<ToolDiscoveryResult>> {
        self.vector_registry.get_recommendations(session_id, context).await
    }
}

/// System-wide statistics
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub total_tools: usize,
    pub total_executions: u64,
    pub success_rate: f32,
    pub avg_execution_time: std::time::Duration,
    pub current_resource_usage: ResourceUsage,
    pub security_violations: u32,
}

impl ToolSystemBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            include_core_tools: false,
            include_runtime_engines: false,
            include_vector_discovery: false,
            security_preset: SecurityPreset::Development,
        }
    }
    
    /// Include core tools in the system
    pub fn with_core_tools(mut self) -> Self {
        self.include_core_tools = true;
        self
    }
    
    /// Include runtime engines for dynamic execution
    pub fn with_runtime_engines(mut self) -> Self {
        self.include_runtime_engines = true;
        self
    }
    
    /// Include vector-based tool discovery
    pub fn with_vector_discovery(mut self) -> Self {
        self.include_vector_discovery = true;
        self
    }
    
    /// Set security preset
    pub fn with_security_preset(mut self, preset: SecurityPreset) -> Self {
        self.security_preset = preset;
        self
    }
    
    /// Build the complete tool system
    pub async fn build(self) -> Result<ToolSystem> {
        // Initialize kernel based on security preset
        let kernel = match self.security_preset {
            SecurityPreset::Development => toka_kernel::presets::development_kernel().await?,
            SecurityPreset::Testing => toka_kernel::presets::testing_kernel().await?,
            SecurityPreset::Production => toka_kernel::presets::production_kernel().await?,
            SecurityPreset::Custom(kernel) => kernel,
        };
        
        // Initialize tool registry
        let tool_registry = if self.include_core_tools {
            ToolRegistryBuilder::new(kernel.clone())
                .with_core_tools()
                .await?
        } else {
            ToolRegistry::new(kernel.clone()).await?
        };
        
        // Initialize runtime manager
        let runtime_manager = if self.include_runtime_engines {
            RuntimeManager::new(kernel.clone()).await?
        } else {
            RuntimeBuilder::new(kernel.clone()).build().await?
        };
        
        // Initialize vector registry
        let vector_registry = if self.include_vector_discovery {
            VectorRegistry::new(kernel.clone()).await?
        } else {
            // Create minimal registry without embeddings
            VectorRegistry::new(kernel.clone()).await?
        };
        
        Ok(ToolSystem {
            kernel: Arc::new(kernel),
            tool_registry: Arc::new(tool_registry),
            runtime_manager: Arc::new(runtime_manager),
            vector_registry: Arc::new(vector_registry),
        })
    }
}

impl UnifiedExecutionRequest {
    /// Create new unified execution request
    pub fn new(
        tool_or_query: &str,
        session_id: &str,
        parameters: JsonValue,
        security_level: SecurityLevel,
    ) -> Self {
        Self {
            tool_or_query: tool_or_query.to_string(),
            session_id: session_id.to_string(),
            parameters,
            security_level,
            allow_discovery: false,
            discovery_threshold: 0.7,
        }
    }
    
    /// Enable discovery fallback
    pub fn with_discovery(mut self, threshold: f32) -> Self {
        self.allow_discovery = true;
        self.discovery_threshold = threshold;
        self
    }
}

impl Default for ToolSystemBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common use cases
pub mod presets {
    use super::*;
    
    /// Create a full-featured development system
    pub async fn development_system() -> Result<ToolSystem> {
        ToolSystem::development().await
    }
    
    /// Create a production-ready system
    pub async fn production_system() -> Result<ToolSystem> {
        ToolSystem::production().await
    }
    
    /// Create a minimal testing system
    pub async fn testing_system() -> Result<ToolSystem> {
        ToolSystem::builder()
            .with_security_preset(SecurityPreset::Testing)
            .build()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_system_creation() {
        let system = ToolSystem::builder()
            .with_core_tools()
            .with_security_preset(SecurityPreset::Testing)
            .build()
            .await
            .unwrap();
        
        let stats = system.get_system_stats().await.unwrap();
        assert!(stats.total_tools > 0); // Should have core tools
        assert_eq!(stats.total_executions, 0); // No executions yet
    }
    
    #[tokio::test]
    async fn test_development_preset() {
        let system = presets::development_system().await.unwrap();
        let tools = system.list_tools().await.unwrap();
        
        // Development system should include core tools
        assert!(!tools.is_empty());
        
        // Should include file reader tool
        assert!(tools.iter().any(|t| t.id == "file_reader"));
    }
    
    #[tokio::test]
    async fn test_unified_execution_request() {
        let request = UnifiedExecutionRequest::new(
            "file_reader",
            "test_session",
            serde_json::json!({"path": "./test.txt"}),
            SecurityLevel::Sandboxed,
        ).with_discovery(0.8);
        
        assert_eq!(request.tool_or_query, "file_reader");
        assert!(request.allow_discovery);
        assert_eq!(request.discovery_threshold, 0.8);
    }
    
    #[tokio::test]
    async fn test_capability_management() {
        let system = presets::testing_system().await.unwrap();
        
        let capabilities = CapabilitySet::workspace_files();
        
        // Should be able to grant capabilities
        system.grant_session_capabilities("test_session", &capabilities)
            .await
            .unwrap();
        
        // Should be able to revoke capabilities
        system.revoke_session_capabilities("test_session")
            .await
            .unwrap();
    }
}

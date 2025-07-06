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
//! use toka_tools::{ToolRegistry, ToolParams};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize the tool registry
//!     let registry = ToolRegistry::new().await?;
//!     
//!     // Register essential tools
//!     toka_tools::tools::register_essential_tools(&registry).await?;
//!     
//!     // Execute a tool
//!     let mut params = ToolParams {
//!         name: "read_file".to_string(),
//!         args: std::collections::HashMap::new(),
//!     };
//!     params.args.insert("path".to_string(), "README.md".to_string());
//!     
//!     let result = registry.execute_tool("read_file", &params).await?;
//!     println!("Tool execution result: {}", result.output);
//!     
//!     Ok(())
//! }
//! ```

use std::sync::Arc;
use anyhow::Result;

// Re-export all public types from underlying crates
pub use toka_kernel::{Kernel, KernelError};
pub use toka_runtime::{
    RuntimeManager, CodeType, RuntimeBuilder, ToolKernel,
    RuntimeMetadata, RuntimeResourceUsage, Artifact,
    SecurityLevel, Capability, CapabilitySet, ExecutionContext,
};

// Import modules
pub mod core;
pub mod tools;
pub mod wrappers;
pub mod runtime_integration;
pub mod manifest;
pub mod loader;

// Re-export core types
pub use crate::core::{Tool, ToolRegistry, ToolParams, ToolResult, ToolMetadata};

/// Unified tool system that integrates all components
/// 
/// This is a placeholder for the full unified system that will be implemented
/// once all dependencies are available.
pub struct ToolSystem {
    /// Kernel for security enforcement
    pub kernel: Arc<Kernel>,
    /// Tool registry for managing tools
    pub registry: Arc<ToolRegistry>,
}

impl ToolSystem {
    /// Create a new tool system with default configuration
    pub async fn new() -> Result<Self> {
        // Create a minimal kernel for now - in a real implementation,
        // this would be properly initialized with auth and event bus
        let world_state = toka_kernel::WorldState::default();
        
        // Use a simple HS256 validator with a test secret
        let auth = Arc::new(toka_auth::hs256::JwtHs256Validator::new("test-secret"));
        
        // Use the default in-memory event bus
        let bus = Arc::new(toka_bus_core::InMemoryBus::default());
        
        let kernel = Kernel::new(world_state, auth, bus);
        
        let registry = ToolRegistry::new().await?;
        
        Ok(Self {
            kernel: Arc::new(kernel),
            registry: Arc::new(registry),
        })
    }
    
    /// Create a new tool system with development preset
    pub async fn development() -> Result<Self> {
        let system = Self::new().await?;
        
        // Register essential tools for development
        tools::register_essential_tools(&system.registry).await?;
        
        Ok(system)
    }
    
    /// Execute a tool by name
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: &ToolParams,
    ) -> Result<ToolResult> {
        self.registry.execute_tool(tool_name, params).await
    }
    
    /// List all available tools
    pub async fn list_tools(&self) -> Vec<String> {
        self.registry.list_tools().await
    }
    
    /// Register a new tool
    pub async fn register_tool(&self, tool: Arc<dyn Tool + Send + Sync>) -> Result<()> {
        self.registry.register_tool(tool).await
    }
}

/// Builder for creating a complete tool system
pub struct ToolSystemBuilder {
    include_core_tools: bool,
    include_runtime_engines: bool,
    security_level: SecurityLevel,
}

impl ToolSystemBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            include_core_tools: false,
            include_runtime_engines: false,
            security_level: SecurityLevel::Restricted,
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
    
    /// Set security level
    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = level;
        self
    }
    
    /// Build the complete tool system
    pub async fn build(self) -> Result<ToolSystem> {
        let system = ToolSystem::new().await?;
        
        if self.include_core_tools {
            tools::register_essential_tools(&system.registry).await?;
        }
        
        Ok(system)
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
    
    /// Create a minimal testing system
    pub async fn testing_system() -> Result<ToolSystem> {
        ToolSystemBuilder::new()
            .with_core_tools()
            .with_security_level(SecurityLevel::Restricted)
            .build()
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_tool_system_creation() -> Result<()> {
        let system = ToolSystem::new().await?;
        assert_eq!(system.list_tools().await.len(), 0);
        Ok(())
    }
    
    #[tokio::test]
    async fn test_development_system() -> Result<()> {
        let system = ToolSystem::development().await?;
        let tools = system.list_tools().await;
        assert!(!tools.is_empty());
        assert!(tools.contains(&"read_file".to_string()));
        Ok(())
    }
    
    #[tokio::test]
    async fn test_tool_execution() -> Result<()> {
        let system = ToolSystem::development().await?;
        
        let mut params = ToolParams {
            name: "read_file".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("path".to_string(), "Cargo.toml".to_string());
        
        let result = system.execute_tool("read_file", &params).await?;
        assert!(result.success);
        assert!(result.output.contains("toka-tools"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_builder_pattern() -> Result<()> {
        let system = ToolSystemBuilder::new()
            .with_core_tools()
            .with_security_level(SecurityLevel::Restricted)
            .build()
            .await?;
        
        let tools = system.list_tools().await;
        assert!(!tools.is_empty());
        
        Ok(())
    }
}

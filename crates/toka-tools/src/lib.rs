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

// Declare modules
pub mod core;
pub mod errors;
pub mod tools;
pub mod wrappers;
pub mod runtime_integration;
pub mod catalogue;

// Re-export all public types from underlying crates
pub use toka_kernel::{Kernel, KernelError};
pub use toka_runtime::{
    RuntimeManager, CodeType, RuntimeBuilder, ToolKernel,
    RuntimeMetadata, RuntimeResourceUsage, Artifact,
    SecurityLevel, Capability, CapabilitySet, ExecutionContext,
};

// Re-export core types
pub use crate::core::{Tool, ToolRegistry, ToolParams, ToolResult, ToolMetadata};

// Re-export catalogue types
pub use crate::catalogue::{ToolCatalogue, ToolFilter, ToolCategory};
pub use crate::wrappers::SecurityLevel;
pub use crate::manifest::SideEffect;

// Re-export error types
pub use crate::errors::{ToolError, RegistryError, ValidationError, SecurityError};

// Re-export manifest and loader
pub use crate::core::{manifest, loader};

/// Unified tool system that integrates all components
/// 
/// This is a placeholder for the full unified system that will be implemented
/// once all dependencies are available.
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::ToolSystem;
/// 
/// # tokio_test::block_on(async {
/// let system = ToolSystem::new().await?;
/// assert_eq!(system.list_tools().await.len(), 0);
/// # Ok::<(), anyhow::Error>(())
/// # });
/// ```
pub struct ToolSystem {
    /// Kernel for security enforcement
    pub kernel: Arc<Kernel>,
    /// Tool registry for managing tools
    pub registry: Arc<ToolRegistry>,
}

impl ToolSystem {
    /// Create a new tool system with default configuration
    /// 
    /// This creates a minimal tool system with basic kernel initialization
    /// using a test JWT validator and in-memory event bus.
    /// 
    /// # Errors
    /// 
    /// Returns an error if the kernel or registry initialization fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::ToolSystem;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystem::new().await?;
    /// assert_eq!(system.list_tools().await.len(), 0);
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
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
    /// 
    /// This creates a tool system with essential tools pre-registered,
    /// suitable for development and testing scenarios.
    /// 
    /// # Errors
    /// 
    /// Returns an error if system creation or tool registration fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::ToolSystem;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystem::development().await?;
    /// let tools = system.list_tools().await;
    /// assert!(!tools.is_empty());
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn development() -> Result<Self> {
        let system = Self::new().await?;
        
        // Register essential tools for development
        tools::register_essential_tools(&system.registry).await?;
        
        Ok(system)
    }
    
    /// Execute a tool by name
    /// 
    /// # Arguments
    /// 
    /// * `tool_name` - The name of the tool to execute
    /// * `params` - Parameters to pass to the tool
    /// 
    /// # Errors
    /// 
    /// Returns an error if the tool is not found or execution fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::{ToolSystem, ToolParams};
    /// use std::collections::HashMap;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystem::development().await?;
    /// 
    /// let mut params = ToolParams {
    ///     name: "read_file".to_string(),
    ///     args: HashMap::new(),
    /// };
    /// params.args.insert("path".to_string(), "Cargo.toml".to_string());
    /// 
    /// let result = system.execute_tool("read_file", &params).await?;
    /// assert!(result.success);
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn execute_tool(
        &self,
        tool_name: &str,
        params: &ToolParams,
    ) -> Result<ToolResult> {
        self.registry.execute_tool(tool_name, params).await
            .map_err(|e| anyhow::anyhow!(e))
    }
    
    /// List all available tools
    /// 
    /// Returns a vector of tool names that are currently registered
    /// in the system.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::ToolSystem;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystem::development().await?;
    /// let tools = system.list_tools().await;
    /// assert!(tools.contains(&"read_file".to_string()));
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn list_tools(&self) -> Vec<String> {
        self.registry.list_tools().await
    }
    
    /// Register a new tool
    /// 
    /// Adds a new tool implementation to the system registry.
    /// 
    /// # Arguments
    /// 
    /// * `tool` - The tool implementation to register
    /// 
    /// # Errors
    /// 
    /// Returns an error if a tool with the same name is already registered
    /// or if registration fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::{ToolSystem, tools::ReadFileTool};
    /// use std::sync::Arc;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystem::new().await?;
    /// let tool = Arc::new(ReadFileTool::new());
    /// system.register_tool(tool).await?;
    /// 
    /// let tools = system.list_tools().await;
    /// assert!(tools.contains(&"read_file".to_string()));
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn register_tool(&self, tool: Arc<dyn Tool + Send + Sync>) -> Result<()> {
        self.registry.register_tool(tool).await
            .map_err(|e| anyhow::anyhow!(e))
    }
}

/// Builder for creating a complete tool system
/// 
/// Provides a fluent interface for configuring and building tool systems
/// with various capabilities and security levels.
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::{ToolSystemBuilder, SecurityLevel};
/// 
/// # tokio_test::block_on(async {
/// let system = ToolSystemBuilder::new()
///     .with_core_tools()
///     .with_security_level(SecurityLevel::Restricted)
///     .build()
///     .await?;
/// 
/// let tools = system.list_tools().await;
/// assert!(!tools.is_empty());
/// # Ok::<(), anyhow::Error>(())
/// # });
/// ```
pub struct ToolSystemBuilder {
    include_core_tools: bool,
    include_runtime_engines: bool,
    security_level: SecurityLevel,
}

impl ToolSystemBuilder {
    /// Create new builder with default configuration
    /// 
    /// Initializes a builder with no core tools, no runtime engines,
    /// and restricted security level.
    pub fn new() -> Self {
        Self {
            include_core_tools: false,
            include_runtime_engines: false,
            security_level: SecurityLevel::Restricted,
        }
    }
    
    /// Include core tools in the system
    /// 
    /// When enabled, the built system will include essential tools
    /// like file operations, command execution, and HTTP requests.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::ToolSystemBuilder;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystemBuilder::new()
    ///     .with_core_tools()
    ///     .build()
    ///     .await?;
    /// 
    /// let tools = system.list_tools().await;
    /// assert!(tools.contains(&"read_file".to_string()));
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub fn with_core_tools(mut self) -> Self {
        self.include_core_tools = true;
        self
    }
    
    /// Include runtime engines for dynamic execution
    /// 
    /// When enabled, the system will support dynamic code execution
    /// in various languages and environments.
    /// 
    /// # Note
    /// 
    /// This feature is currently a placeholder and will be implemented
    /// in future versions.
    pub fn with_runtime_engines(mut self) -> Self {
        self.include_runtime_engines = true;
        self
    }
    
    /// Set security level for the system
    /// 
    /// Configures the security restrictions and capabilities
    /// for tool execution.
    /// 
    /// # Arguments
    /// 
    /// * `level` - The security level to apply
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::{ToolSystemBuilder, SecurityLevel};
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystemBuilder::new()
    ///     .with_security_level(SecurityLevel::High)
    ///     .build()
    ///     .await?;
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = level;
        self
    }
    
    /// Build the complete tool system
    /// 
    /// Creates and initializes the tool system based on the
    /// configuration specified in this builder.
    /// 
    /// # Errors
    /// 
    /// Returns an error if system initialization or tool registration fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::{ToolSystemBuilder, SecurityLevel};
    /// 
    /// # tokio_test::block_on(async {
    /// let system = ToolSystemBuilder::new()
    ///     .with_core_tools()
    ///     .with_security_level(SecurityLevel::Restricted)
    ///     .build()
    ///     .await?;
    /// 
    /// assert!(!system.list_tools().await.is_empty());
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
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
/// 
/// This module provides pre-configured tool systems for common scenarios,
/// eliminating the need to manually configure the builder for standard setups.
pub mod presets {
    use super::*;
    
    /// Create a full-featured development system
    /// 
    /// Creates a tool system optimized for development work, including
    /// all essential tools and appropriate security settings.
    /// 
    /// # Errors
    /// 
    /// Returns an error if system creation fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::presets;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = presets::development_system().await?;
    /// let tools = system.list_tools().await;
    /// assert!(!tools.is_empty());
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn development_system() -> Result<ToolSystem> {
        ToolSystem::development().await
    }
    
    /// Create a minimal testing system
    /// 
    /// Creates a tool system suitable for testing scenarios with
    /// core tools enabled and sandboxed security.
    /// 
    /// # Errors
    /// 
    /// Returns an error if system creation fails.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::presets;
    /// 
    /// # tokio_test::block_on(async {
    /// let system = presets::testing_system().await?;
    /// let tools = system.list_tools().await;
    /// assert!(!tools.is_empty());
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
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
        assert!(tools.contains(&"file-reader".to_string()));
        Ok(())
    }
    
    #[tokio::test]
    async fn test_tool_execution() -> Result<()> {
        let system = ToolSystem::development().await?;
        
        let mut params = ToolParams {
            name: "file-reader".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("path".to_string(), "Cargo.toml".to_string());
        
        let result = system.execute_tool("file-reader", &params).await?;
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

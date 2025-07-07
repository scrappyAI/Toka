//! Core tooling abstractions – migrated from the former `toka-toolkit-core` crate.
//!
//! The module was moved into `toka-tools` as part of the *crate consolidation*
//! described in `docs/code-clarity-report.md` (July 2025).
//!
//! Downstream crates should `use toka_tools::{Tool, ToolRegistry, …}` which are
//! re-exported at the crate root.

#![allow(clippy::module_name_repetitions)]

use anyhow::Result;
#[allow(unused_imports)]
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::errors::ToolError;

// Re-export so downstream modules can `use crate::core::Tool`.
pub use toka_types::traits::{Tool, ToolParams};

// Re-export metadata/result types from toka-types
pub use toka_types::{ToolMetadata, ToolResult};

/// Thread-safe registry for managing tool instances
/// 
/// Provides centralized tool management with registration, lookup, and execution
/// capabilities. The registry starts empty and tools must be explicitly registered.
/// 
/// # Thread Safety
/// 
/// The registry is fully thread-safe and can be shared across multiple tasks
/// or threads using `Arc<ToolRegistry>`.
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::{ToolRegistry, tools::FileReader};
/// use std::sync::Arc;
/// 
/// # tokio_test::block_on(async {
/// let registry = ToolRegistry::new().await?;
/// 
/// // Register a tool
/// let tool = Arc::new(FileReader::new());
/// registry.register_tool(tool).await?;
/// 
/// // List registered tools
/// let tools = registry.list_tools().await;
/// assert!(tools.contains(&"file-reader".to_string()));
/// # Ok::<(), anyhow::Error>(())
/// # });
/// ```
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
    /// Create an empty registry
    /// 
    /// Creates a new registry with no tools registered. Tools must be
    /// explicitly added using `register_tool`.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::ToolRegistry;
    /// 
    /// let registry = ToolRegistry::new_empty();
    /// # tokio_test::block_on(async {
    /// assert_eq!(registry.list_tools().await.len(), 0);
    /// # });
    /// ```
    pub fn new_empty() -> Self {
        Self::default()
    }

    /// Create a new registry (alias for `new_empty`)
    /// 
    /// This method exists for historical compatibility and creates
    /// an empty registry that requires explicit tool registration.
    /// 
    /// # Errors
    /// 
    /// Currently always succeeds, but returns Result for future compatibility.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::ToolRegistry;
    /// 
    /// # tokio_test::block_on(async {
    /// let registry = ToolRegistry::new().await?;
    /// assert_eq!(registry.list_tools().await.len(), 0);
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn new() -> Result<Self> {
        Ok(Self::default())
    }

    /// Register a new tool instance
    /// 
    /// Adds a tool to the registry, making it available for execution.
    /// Tool names must be unique within the registry.
    /// 
    /// # Arguments
    /// 
    /// * `tool` - The tool implementation to register
    /// 
    /// # Errors
    /// 
    /// Returns an error if a tool with the same name is already registered.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::{ToolRegistry, tools::FileReader};
    /// use std::sync::Arc;
    /// 
    /// # tokio_test::block_on(async {
    /// let registry = ToolRegistry::new().await?;
    /// let tool = Arc::new(FileReader::new());
    /// 
    /// registry.register_tool(tool).await?;
    /// 
    /// let tools = registry.list_tools().await;
    /// assert!(tools.contains(&"file-reader".to_string()));
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn register_tool(&self, tool: Arc<dyn Tool + Send + Sync>) -> Result<(), ToolError> {
        let name = tool.name().to_string();
        let mut map = self.tools.write().await;
        if map.contains_key(&name) {
            return Err(ToolError::ToolAlreadyRegistered { name });
        }
        map.insert(name.clone(), tool);
        info!("Registered tool: {name}");
        Ok(())
    }

    /// Fetch a tool by name
    /// 
    /// Retrieves a tool from the registry by its name. Returns None
    /// if the tool is not registered.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the tool to retrieve
    /// 
    /// # Returns
    /// 
    /// Returns Some(tool) if found, None otherwise.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::{ToolRegistry, tools::FileReader};
    /// use std::sync::Arc;
    /// 
    /// # tokio_test::block_on(async {
    /// let registry = ToolRegistry::new().await?;
    /// let tool = Arc::new(FileReader::new());
    /// registry.register_tool(tool).await?;
    /// 
    /// let retrieved = registry.get_tool("file-reader").await;
    /// assert!(retrieved.is_some());
    /// 
    /// let missing = registry.get_tool("nonexistent").await;
    /// assert!(missing.is_none());
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool + Send + Sync>> {
        let map = self.tools.read().await;
        map.get(name).cloned()
    }

    /// Execute a tool by name
    /// 
    /// Finds and executes a tool with the given parameters. This method
    /// handles parameter validation and execution timing automatically.
    /// 
    /// # Arguments
    /// 
    /// * `name` - The name of the tool to execute
    /// * `params` - Parameters for tool execution
    /// 
    /// # Errors
    /// 
    /// Returns an error if:
    /// - The tool is not found
    /// - Parameter validation fails
    /// - Tool execution fails
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use toka_tools::{ToolRegistry, ToolParams, tools::FileReader};
    /// use std::sync::Arc;
    /// use std::collections::HashMap;
    /// 
    /// # tokio_test::block_on(async {
    /// let registry = ToolRegistry::new().await?;
    /// let tool = Arc::new(FileReader::new());
    /// registry.register_tool(tool).await?;
    /// 
    /// let mut params = ToolParams {
    ///     name: "file-reader".to_string(),
    ///     args: HashMap::new(),
    /// };
    /// params.args.insert("path".to_string(), "Cargo.toml".to_string());
    /// 
    /// let result = registry.execute_tool("file-reader", &params).await?;
    /// assert!(result.success);
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn execute_tool(&self, name: &str, params: &ToolParams) -> Result<ToolResult, ToolError> {
        let tool = {
            let map = self.tools.read().await;
            map.get(name).cloned()
        }
        .ok_or_else(|| ToolError::ToolNotFound { name: name.to_string() })?;

        // Validate parameters first
        tool.validate_params(params)
            .map_err(|e| ToolError::ParameterValidation {
                tool_name: name.to_string(),
                reason: e.to_string(),
            })?;

        let start = std::time::Instant::now();
        let mut result = tool
            .execute(params)
            .await
            .map_err(|e| ToolError::ExecutionFailed {
                tool_name: name.to_string(),
                reason: e.to_string(),
            })?;
        
        result.metadata.execution_time_ms = start.elapsed().as_millis() as u64;
        Ok(result)
    }

    /// List registered tool names
    /// 
    /// Returns a vector of all tool names currently registered in the registry.
    /// The order is not guaranteed.
    /// 
    /// # Examples
    /// 
    /// TODO: Update doctest to match current API
    /// ```ignore
    /// use toka_tools::{ToolRegistry, tools::ReadFileTool};
    /// use std::sync::Arc;
    /// 
    /// # tokio_test::block_on(async {
    /// let registry = ToolRegistry::new().await?;
    /// 
    /// // Initially empty
    /// assert_eq!(registry.list_tools().await.len(), 0);
    /// 
    /// // Register a tool
    /// let tool = Arc::new(ReadFileTool::new());
    /// registry.register_tool(tool).await?;
    /// 
    /// // Now contains the tool
    /// let tools = registry.list_tools().await;
    /// assert_eq!(tools.len(), 1);
    /// assert!(tools.contains(&"read_file".to_string()));
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
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
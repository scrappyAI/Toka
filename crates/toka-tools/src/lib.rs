#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! Toka Tools - Unified tool system for composable, hot-swappable execution
//!
//! This crate provides a comprehensive tool system that enables:
//! - Automatic discovery of Python, Shell, and external tools
//! - Unified YAML manifest format for tool configuration
//! - Sandboxed execution with capability-based security
//! - Hot-swappable tool registration and execution
//! - Integration with the Toka agent runtime system
//!
//! ## Architecture
//!
//! The unified tool system consists of several key components:
//!
//! - **Core**: Basic tool traits, registry, and execution framework
//! - **Wrappers**: Security-focused wrappers for different tool types
//! - **Manifests**: YAML-based tool configuration and metadata
//! - **Runtime Integration**: Connection to Toka agent runtime for hot-swappable execution
//!
//! ## Usage
//!
//! ```no_run
//! use toka_tools::{RuntimeToolRegistry, RuntimeContext};
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Initialize runtime tool registry
//!     let registry = RuntimeToolRegistry::new("tools").await?;
//!     
//!     // Create runtime context
//!     let context = RuntimeContext {
//!         agent_id: "analysis-agent-001".to_string(),
//!         agent_type: "analysis".to_string(),
//!         workstream: Some("code-analysis".to_string()),
//!         execution_environment: "development".to_string(),
//!         capabilities: vec![
//!             "filesystem-read".to_string(),
//!             "code-analysis".to_string(),
//!             "visualization".to_string(),
//!         ],
//!     };
//!     
//!     // Execute a tool
//!     let params = toka_tools::ToolParams {
//!         name: "control-flow-analyzer".to_string(),
//!         args: [
//!             ("output_format", "mermaid"),
//!             ("complexity_analysis", "true"),
//!         ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
//!     };
//!     
//!     let result = registry.execute_tool_runtime(
//!         "control-flow-analyzer",
//!         &params,
//!         &context.capabilities,
//!         &context,
//!     ).await?;
//!     
//!     println!("Tool execution result: {}", result.tool_result.output);
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod tools;
pub mod wrappers;
pub mod manifest;
pub mod loader;
pub mod runtime_integration;

// Re-export core types
pub use core::{Tool, ToolParams, ToolResult, ToolMetadata, ToolRegistry};

// Re-export wrapper types for unified tool system
pub use wrappers::{
    UnifiedToolRegistry, DiscoveredTool, ToolType, ToolExecutionMetrics,
    ToolSecurityClassification, SecurityLevel, SecurityConfig, 
    SandboxConfig, ResourceLimits, CapabilityValidator
};

// Re-export manifest types
pub use manifest::{ToolManifest, Transport, SideEffect, ProtocolMapping};

// Re-export runtime integration types
pub use runtime_integration::{
    RuntimeToolRegistry, RuntimeContext, RuntimeToolResult,
    UnifiedToolManifest, ToolMetadata as RuntimeToolMetadata,
    ToolSpec, ExecutableSpec, CapabilitiesSpec, SecuritySpec,
    SandboxSpec, ParameterSpec, ToolInterface, ProtocolMapping as RuntimeProtocolMapping,
};

/// Version of the unified tool system
pub const TOOLS_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default tools directory relative to workspace root
pub const DEFAULT_TOOLS_DIR: &str = "tools";

/// Default manifests directory within tools directory  
pub const DEFAULT_MANIFESTS_DIR: &str = "manifests";

/// Initialize a runtime tool registry with default settings
pub async fn init_runtime_registry() -> anyhow::Result<RuntimeToolRegistry> {
    RuntimeToolRegistry::new(DEFAULT_TOOLS_DIR).await
}

/// Initialize a runtime tool registry with custom tools directory
pub async fn init_runtime_registry_with_dir(tools_dir: impl AsRef<std::path::Path>) -> anyhow::Result<RuntimeToolRegistry> {
    RuntimeToolRegistry::new(tools_dir).await
}

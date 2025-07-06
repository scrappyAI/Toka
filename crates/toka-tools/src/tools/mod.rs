//! Core tool implementations for the Toka ecosystem
//!
//! This module provides essential tools that are built into the system,
//! including file operations, text processing, and validation tools.

use std::sync::Arc;
use anyhow::Result;

use crate::core::{Tool, ToolRegistry};

// Tool modules
pub mod file_tools;
pub mod text_tools;
pub mod validation;

// Re-export tools for easy access
pub use file_tools::{FileReader, FileWriter, FileLister};
pub use text_tools::{TextProcessor, RegexTool};
pub use validation::{DateValidator, BuildValidator};

/// Register essential tools with the registry
/// 
/// This function registers all the core tools that should be available
/// by default in any Toka installation.
/// 
/// # Arguments
/// 
/// * `registry` - The tool registry to register tools with
/// 
/// # Examples
/// 
/// ```rust
/// use toka_tools::{ToolRegistry, tools};
/// 
/// # tokio_test::block_on(async {
/// let registry = ToolRegistry::new().await?;
/// tools::register_essential_tools(&registry).await?;
/// 
/// // Now you can use the registered tools
/// assert!(registry.has_tool("file-reader").await);
/// assert!(registry.has_tool("date-validator").await);
/// # Ok::<(), anyhow::Error>(())
/// # });
/// ```
pub async fn register_essential_tools(registry: &ToolRegistry) -> Result<()> {
    // File operations
    registry.register_tool(Arc::new(FileReader::new())).await?;
    registry.register_tool(Arc::new(FileWriter::new())).await?;
    registry.register_tool(Arc::new(FileLister::new())).await?;
    
    // Text processing
    registry.register_tool(Arc::new(TextProcessor::new())).await?;
    registry.register_tool(Arc::new(RegexTool::new())).await?;
    
    // Validation tools
    registry.register_tool(Arc::new(DateValidator::new()?)).await?;
    registry.register_tool(Arc::new(BuildValidator::new())).await?;
    
    Ok(())
}

/// Get a list of all essential tool names
/// 
/// This is useful for testing or introspection purposes.
pub fn essential_tool_names() -> Vec<&'static str> {
    vec![
        "file-reader",
        "file-writer", 
        "file-lister",
        "text-processor",
        "regex-tool",
        "date-validator",
        "build-validator",
    ]
}

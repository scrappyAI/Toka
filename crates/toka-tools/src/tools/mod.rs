//! Core tool implementations for the Toka ecosystem
//!
//! This module provides essential tools that are built into the system,
//! including file operations, text processing, and validation tools.

use std::sync::Arc;
use anyhow::Result;

use crate::core::ToolRegistry;

// Tool modules
pub mod file_tools;
pub mod validation;

// Re-export tools for easy access
pub use file_tools::{FileReader, FileWriter, FileLister};
pub use validation::{DateValidator, BuildValidator};

/// Register essential tools for development and testing
/// 
/// This function registers a set of core tools that are commonly
/// needed for development and testing scenarios.
/// 
/// # Arguments
/// 
/// * `registry` - The tool registry to register tools in
/// 
/// # Errors
/// 
/// Returns an error if tool registration fails.
/// 
/// # Examples
/// 
/// TODO: Update doctest to match current API
/// ```ignore
/// use toka_tools::{ToolRegistry, tools::register_essential_tools};
/// 
/// # tokio_test::block_on(async {
/// let registry = ToolRegistry::new().await?;
/// register_essential_tools(&registry).await?;
/// 
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
        "date-validator",
        "build-validator",
    ]
}

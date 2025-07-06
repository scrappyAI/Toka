use anyhow::Result;
use std::sync::Arc;

use toka_tools::{ToolRegistry, tools::{ReadFileTool, register_essential_tools}};

#[tokio::test]
async fn test_registry_basic() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Register essential tools
    register_essential_tools(&registry).await?;
    
    let tools = registry.list_tools().await;
    assert!(tools.len() >= 4); // Should have at least the 4 essential tools
    assert!(tools.contains(&"read_file".to_string()));
    assert!(tools.contains(&"write_file".to_string()));
    assert!(tools.contains(&"run_command".to_string()));
    assert!(tools.contains(&"http_request".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_individual_tool_registration() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Register a single tool
    registry.register_tool(Arc::new(ReadFileTool::new())).await?;
    
    let tools = registry.list_tools().await;
    assert_eq!(tools.len(), 1);
    assert!(tools.contains(&"read_file".to_string()));
    
    Ok(())
}
use anyhow::Result;
use std::sync::Arc;

use toka_tools::{ToolRegistry, tools::{FileReader, register_essential_tools}};

#[tokio::test]
async fn test_registry_basic() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Register essential tools
    register_essential_tools(&registry).await?;
    
    let tools = registry.list_tools().await;
    assert!(tools.len() >= 5); // Should have at least the 5 essential tools
    assert!(tools.contains(&"file-reader".to_string()));
    assert!(tools.contains(&"file-writer".to_string()));
    assert!(tools.contains(&"file-lister".to_string()));
    assert!(tools.contains(&"date-validator".to_string()));
    assert!(tools.contains(&"build-validator".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_individual_tool_registration() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    
    // Register a single tool
    registry.register_tool(Arc::new(FileReader::new())).await?;
    
    let tools = registry.list_tools().await;
    assert_eq!(tools.len(), 1);
    assert!(tools.contains(&"file-reader".to_string()));
    
    Ok(())
}
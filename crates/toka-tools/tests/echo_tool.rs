use anyhow::Result;
use std::sync::Arc;
use std::collections::HashMap;

use toka_tools::{ToolRegistry, tools::ReadFileTool, ToolParams};

#[tokio::test]
async fn test_read_file_tool() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(ReadFileTool::new())).await?;
    
    // Test that the tool is registered
    let tools = registry.list_tools().await;
    assert!(tools.contains(&"read_file".to_string()));
    
    Ok(())
}
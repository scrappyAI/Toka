use anyhow::Result;
use std::sync::Arc;

use toka_tools::{ToolRegistry, tools::ReadFileTool};

#[tokio::test]
async fn test_read_file_tool() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(ReadFileTool::new())).await?;
    
    // Test that the tool is registered
    let tools = registry.list_tools().await;
    assert!(tools.contains(&"read_file".to_string()));
    
    Ok(())
}
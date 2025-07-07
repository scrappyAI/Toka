use anyhow::Result;
use std::sync::Arc;

use toka_tools::{ToolRegistry, tools::FileReader};

#[tokio::test]
async fn test_read_file_tool() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(FileReader::new())).await?;
    
    // Test that the tool is registered
    let tools = registry.list_tools().await;
    assert!(tools.contains(&"file-reader".to_string()));
    
    Ok(())
}
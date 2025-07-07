use anyhow::Result;
use std::sync::Arc;

use toka_tools::{ToolRegistry, tools::FileReader};

#[tokio::test]
async fn duplicate_registration_fails() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(FileReader::new())).await?;
    let err = registry.register_tool(Arc::new(FileReader::new())).await.unwrap_err();
    assert!(err.to_string().contains("already registered"));
    Ok(())
}

#[tokio::test]
async fn list_tools_returns_sorted_names() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(FileReader::new())).await?;
    let mut names = registry.list_tools().await;
    names.sort();
    assert_eq!(names, vec!["file-reader".to_string()]);
    Ok(())
}
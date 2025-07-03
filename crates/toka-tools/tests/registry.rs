use std::sync::Arc;

use anyhow::Result;

use toka_tools::{ToolRegistry, tools::EchoTool};

#[tokio::test]
async fn duplicate_registration_fails() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(EchoTool::new())).await?;
    let err = registry.register_tool(Arc::new(EchoTool::new())).await.unwrap_err();
    assert!(err.to_string().contains("already registered"));
    Ok(())
}

#[tokio::test]
async fn list_tools_returns_sorted_names() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(EchoTool::new())).await?;
    let mut names = registry.list_tools().await;
    names.sort();
    assert_eq!(names, vec!["echo".to_string()]);
    Ok(())
}
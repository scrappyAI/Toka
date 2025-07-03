use std::sync::Arc;

use anyhow::Result;

use toka_tools::{ToolRegistry, tools::EchoTool, ToolParams};

#[tokio::test]
async fn execute_tool_not_found_returns_error() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    let params = ToolParams { name: "missing".into(), args: Default::default() };
    let err = registry.execute_tool("missing", &params).await.unwrap_err();
    assert!(err.to_string().contains("Tool not found"));
    Ok(())
}

#[tokio::test]
async fn validate_params_enforced() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(EchoTool::new())).await?;

    // Missing required "message" arg
    let params = ToolParams { name: "echo".into(), args: Default::default() };
    let err = registry.execute_tool("echo", &params).await.unwrap_err();
    assert!(err.to_string().contains("Missing required parameter"));
    Ok(())
}
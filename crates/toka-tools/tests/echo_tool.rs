use std::sync::Arc;

use anyhow::Result;
use proptest::prelude::*;
use tokio_test::block_on;

use toka_tools::{ToolRegistry, tools::EchoTool, ToolParams};

#[tokio::test]
async fn echo_tool_happy_path() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(EchoTool::new())).await?;

    let mut args = std::collections::HashMap::new();
    args.insert("message".into(), "hello".into());
    let params = ToolParams { name: "echo".into(), args };

    let result = registry.execute_tool("echo", &params).await?;
    assert_eq!(result.output, "hello");
    Ok(())
}

#[tokio::test]
async fn echo_tool_missing_param_fails() -> Result<()> {
    let registry = ToolRegistry::new().await?;
    registry.register_tool(Arc::new(EchoTool::new())).await?;

    let params = ToolParams { name: "echo".into(), args: std::collections::HashMap::new() };
    let err = registry.execute_tool("echo", &params).await.unwrap_err();
    assert!(err.to_string().contains("Missing required parameter"));
    Ok(())
}

proptest! {
    #[test]
    fn echo_tool_property(msg in "[a-zA-Z0-9]{0,32}") {
        let output = block_on(async {
            let registry = ToolRegistry::new().await.unwrap();
            registry.register_tool(Arc::new(EchoTool::new())).await.unwrap();
            let mut args = std::collections::HashMap::new();
            args.insert("message".into(), msg.clone());
            let params = ToolParams { name: "echo".into(), args };
            let res = registry.execute_tool("echo", &params).await.unwrap();
            res.output
        });
        prop_assert_eq!(output, msg);
    }
}
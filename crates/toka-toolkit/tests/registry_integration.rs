use anyhow::Result;
use toka_toolkit::tools::{ToolParams, ToolRegistry};

#[tokio::test]
async fn echo_tool_via_registry() -> Result<()> {
    let registry = toka_toolkit::ToolRegistry::new().await?;
    let params = ToolParams {
        name: "echo".into(),
        args: [("message".to_string(), "hello".to_string())]
            .iter()
            .cloned()
            .collect(),
    };
    let result = registry.execute_tool("echo", &params).await?;
    assert!(result.success);
    assert_eq!(result.output, "hello");
    Ok(())
}

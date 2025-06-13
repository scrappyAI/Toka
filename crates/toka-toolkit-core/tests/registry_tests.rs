use toka_toolkit_core::{ToolRegistry, Tool, ToolMetadata, ToolParams, ToolResult};
use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str { "echo" }
    fn description(&self) -> &str { "Echoes the provided input." }
    fn version(&self) -> &str { "0.1.0" }

    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let text = params.args.get("text").cloned().unwrap_or_default();
        // Simulate some work
        sleep(Duration::from_millis(10)).await;
        Ok(ToolResult {
            success: true,
            output: text.clone(),
            metadata: ToolMetadata { execution_time_ms: 0, tool_version: self.version().into(), timestamp: 0 },
        })
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("text") {
            anyhow::bail!("missing 'text' arg");
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_registry_lifecycle() -> Result<()> {
    let registry = ToolRegistry::new();
    assert!(registry.list_tools().await.is_empty());

    // Register tool
    registry.register_tool(Arc::new(EchoTool)).await?;

    // Listing should now include the tool
    let tools = registry.list_tools().await;
    assert_eq!(tools, vec!["echo".to_string()]);

    // Execute tool with valid params
    let params = ToolParams { name: "echo".into(), args: std::iter::once(("text".into(), "hello".into())).collect() };
    let result = registry.execute_tool("echo", &params).await?;
    assert!(result.success);
    assert_eq!(result.output, "hello");

    // Execute with missing param — expect error
    let bad_params = ToolParams { name: "echo".into(), args: Default::default() };
    assert!(registry.execute_tool("echo", &bad_params).await.is_err());

    Ok(())
} 
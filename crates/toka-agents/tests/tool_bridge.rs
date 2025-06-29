#![cfg(feature = "toolkit")]

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use toka_agents::{EventBus, BaseAgent};
use toka_toolkit_core::{Tool, ToolMetadata, ToolParams, ToolRegistry, ToolResult};


struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "Echo back input"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }

    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let payload = params.args.get("msg").cloned().unwrap_or_default();
        Ok(ToolResult {
            success: true,
            output: payload,
            metadata: ToolMetadata {
                execution_time_ms: 0,
                tool_version: self.version().to_string(),
                timestamp: 0,
            },
        })
    }

    fn validate_params(&self, _params: &ToolParams) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn agent_invoke_tool_emits_events() -> Result<()> {
    // Setup
    let mut agent = BaseAgent::new("tester");
    let bus = EventBus::new_default();
    agent.set_event_bus(bus.clone());

    let registry = ToolRegistry::new();
    registry.register_tool(Arc::new(EchoTool)).await?;

    // Listen for events
    let mut rx = bus.get_receiver();

    // Invoke tool
    let mut args = std::collections::HashMap::new();
    args.insert("msg".to_string(), "hello".to_string());
    let params = ToolParams {
        name: "echo".to_string(),
        args,
    };
    let result = agent.invoke_tool(&registry, params.clone()).await?;
    assert!(result.success);
    assert_eq!(result.output, "hello");

    // Ensure at least one ToolEvent came through
    let mut got_invoked = false;
    while let Ok(head) = rx.try_recv() {
        if head.kind.starts_with("tool.") {
            got_invoked = true;
            break;
        }
    }
    assert!(got_invoked, "No ToolEvent header received");
    Ok(())
}

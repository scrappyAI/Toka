#![cfg(feature = "toolkit")]

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use tempfile::tempdir;
use toka_agents::BaseAgent;
use toka_runtime::runtime::{Runtime, RuntimeConfig};
use toka_toolkit_core::{Tool, ToolMetadata, ToolParams, ToolResult};

struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }
    fn description(&self) -> &str {
        "Echo back"
    }
    fn version(&self) -> &str {
        "0.1.0"
    }
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let out = params.args.get("msg").cloned().unwrap_or_default();
        Ok(ToolResult {
            success: true,
            output: out,
            metadata: ToolMetadata {
                execution_time_ms: 0,
                tool_version: self.version().into(),
                timestamp: 0,
            },
        })
    }
    fn validate_params(&self, _p: &ToolParams) -> Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn runtime_agent_tool_storage_roundtrip() -> Result<()> {
    // Runtime with isolated vault/storage
    let tmp = tempdir()?;
    let cfg = RuntimeConfig {
        vault_path: tmp.path().join("vault").to_string_lossy().into_owned(),
        max_agents: 5,
        event_buffer_size: 32,
        storage_root: tmp.path().join("storage").to_string_lossy().into_owned(),
        ..RuntimeConfig::default()
    };
    let runtime = Runtime::new(cfg).await?;

    // Register tool
    runtime
        .tool_registry()
        .register_tool(Arc::new(EchoTool))
        .await?;

    // Create agent
    let mut agent = BaseAgent::new("a1");
    agent.set_event_bus(toka_bus::MemoryBus::default());

    // Use storage adapter via runtime
    let local = runtime.storage("local").await.expect("local adapter");
    local.put("local://file.txt", b"hi").await?;
    let bytes = local.get("local://file.txt").await?;
    assert_eq!(bytes.as_ref().map(|v| v.as_slice()), Some(b"hi".as_slice()));

    // Agent invokes tool
    let mut args = std::collections::HashMap::new();
    args.insert("msg".to_string(), "hi".to_string());
    let params = ToolParams {
        name: "echo".into(),
        args,
    };
    let res = agent.invoke_tool(runtime.tool_registry(), params).await?;
    assert_eq!(res.output, "hi");
    Ok(())
}

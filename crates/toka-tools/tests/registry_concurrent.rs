use std::sync::Arc;

use anyhow::Result;
use futures::future::join_all;
use tokio::task;

use toka_tools::{ToolRegistry, tools::EchoTool, ToolParams};

#[tokio::test]
async fn concurrent_registration_and_execution() -> Result<()> {
    let registry = Arc::new(ToolRegistry::new().await?);

    // Spawn 10 concurrent registrations (only first should succeed).
    let mut tasks = vec![];
    for _ in 0..10 {
        let reg = Arc::clone(&registry);
        tasks.push(task::spawn(async move {
            let _ = reg.register_tool(Arc::new(EchoTool::new())).await;
        }));
    }
    join_all(tasks).await;

    // Execute echo concurrently 20 times.
    let mut execs = vec![];
    for i in 0..20u32 {
        let reg = Arc::clone(&registry);
        execs.push(task::spawn(async move {
            let mut args = std::collections::HashMap::new();
            args.insert("message".into(), format!("run-{i}"));
            let params = ToolParams { name: "echo".into(), args };
            let res = reg.execute_tool("echo", &params).await.unwrap();
            res.output
        }));
    }

    let outputs = join_all(execs).await;
    // Verify every output matches its input.
    for (idx, out) in outputs.into_iter().enumerate() {
        assert_eq!(out.unwrap(), format!("run-{idx}"));
    }

    Ok(())
}
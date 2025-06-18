#![cfg(all(feature = "toolkit", feature = "vault"))]

use anyhow::Result;
use tempfile::tempdir;
use toka_runtime::{runtime::Runtime, runtime::RuntimeConfig};

#[tokio::test]
async fn runtime_start_stop_cycle() -> Result<()> {
    let dir = tempdir()?;
    let cfg = RuntimeConfig {
        vault_path: dir.path().to_str().unwrap().to_string(),
        max_agents: 5,
        event_buffer_size: 32,
    };
    let runtime = Runtime::new(cfg).await?;
    runtime.start().await?;
    assert!(runtime.is_running().await);
    runtime.stop().await?;
    assert!(!runtime.is_running().await);
    Ok(())
}

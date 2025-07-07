use anyhow::Result;
use std::sync::Arc;
use tokio::task;

use toka_tools::{ToolRegistry, tools::FileReader};

#[tokio::test]
async fn concurrent_execution_is_safe() -> Result<()> {
    let registry = Arc::new(ToolRegistry::new().await?);
    registry.register_tool(Arc::new(FileReader::new())).await?;

    let mut handles = vec![];
    
    for i in 0..10 {
        let registry_clone = Arc::clone(&registry);
        let handle = task::spawn(async move {
            let result = registry_clone.list_tools().await;
            println!("Task {} completed", i);
            result
        });
        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await?;
        assert_eq!(result.len(), 1);
        assert!(result.contains(&"file-reader".to_string()));
    }

    Ok(())
}
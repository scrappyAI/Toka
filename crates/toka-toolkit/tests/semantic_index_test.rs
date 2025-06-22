use anyhow::Result;
use std::sync::Arc;
use toka_toolkit::tools::{ItemMetadata, SemanticIndexTool, TaggedItem, Tool, ToolParams};
use tokio::task;

#[tokio::test]
async fn test_concurrent_indexing() -> Result<()> {
    let tool = Arc::new(SemanticIndexTool::new());

    // Create test items
    let items = vec![
        create_test_item("1", "test1", vec!["tag1", "tag2"]),
        create_test_item("2", "test2", vec!["tag2", "tag3"]),
        create_test_item("3", "test3", vec!["tag1", "tag3"]),
    ];

    // Spawn multiple tasks to index items concurrently
    let mut handles = vec![];
    for item in items {
        let tool_clone = tool.clone();
        let handle = task::spawn(async move {
            let params = ToolParams {
                name: "semantic_index".to_string(),
                args: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("command".to_string(), "index".to_string());
                    map.insert("data".to_string(), serde_json::to_string(&item).unwrap());
                    map
                },
            };
            tool_clone.execute(&params).await
        });
        handles.push(handle);
    }

    // Wait for all indexing tasks to complete
    for handle in handles {
        let result = handle.await??;
        assert!(result.success);
    }

    // Verify all items were indexed correctly
    let search_params = ToolParams {
        name: "semantic_index".to_string(),
        args: {
            let mut map = std::collections::HashMap::new();
            map.insert("command".to_string(), "search".to_string());
            map.insert("tag".to_string(), "tag1".to_string());
            map
        },
    };

    let result = tool.execute(&search_params).await?;
    assert!(result.success);
    assert!(result.output.contains("test1"));
    assert!(result.output.contains("test3"));

    Ok(())
}

#[tokio::test]
async fn test_concurrent_search() -> Result<()> {
    let tool = Arc::new(SemanticIndexTool::new());

    // Index some test items first
    let items = vec![
        create_test_item("1", "test1", vec!["tag1", "tag2"]),
        create_test_item("2", "test2", vec!["tag2", "tag3"]),
        create_test_item("3", "test3", vec!["tag1", "tag3"]),
    ];

    for item in items {
        let params = ToolParams {
            name: "semantic_index".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), "index".to_string());
                map.insert("data".to_string(), serde_json::to_string(&item).unwrap());
                map
            },
        };
        tool.execute(&params).await?;
    }

    // Spawn multiple search tasks concurrently
    let search_tags = vec!["tag1", "tag2", "tag3"];
    let mut handles = vec![];

    for tag in search_tags {
        let tool_clone = tool.clone();
        let handle = task::spawn(async move {
            let params = ToolParams {
                name: "semantic_index".to_string(),
                args: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("command".to_string(), "search".to_string());
                    map.insert("tag".to_string(), tag.to_string());
                    map
                },
            };
            tool_clone.execute(&params).await
        });
        handles.push(handle);
    }

    // Verify all searches complete successfully
    for handle in handles {
        let result = handle.await??;
        assert!(result.success);
        assert!(!result.output.contains("error"));
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_index_and_search() -> Result<()> {
    let tool = Arc::new(SemanticIndexTool::new());

    // Spawn tasks that both index and search concurrently
    let mut handles = vec![];

    for i in 0..5 {
        let tool_clone = tool.clone();
        let handle = task::spawn(async move {
            // Index a new item
            let item = create_test_item(
                &format!("{}", i),
                &format!("test{}", i),
                vec!["tag1", &format!("tag{}", i)],
            );

            let index_params = ToolParams {
                name: "semantic_index".to_string(),
                args: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("command".to_string(), "index".to_string());
                    map.insert("data".to_string(), serde_json::to_string(&item).unwrap());
                    map
                },
            };

            let index_result = tool_clone.execute(&index_params).await?;
            assert!(index_result.success);

            // Search for items
            let search_params = ToolParams {
                name: "semantic_index".to_string(),
                args: {
                    let mut map = std::collections::HashMap::new();
                    map.insert("command".to_string(), "search".to_string());
                    map.insert("tag".to_string(), "tag1".to_string());
                    map
                },
            };

            let search_result = tool_clone.execute(&search_params).await?;
            assert!(search_result.success);

            Ok::<_, anyhow::Error>(())
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await??;
    }

    Ok(())
}

fn create_test_item(id: &str, content: &str, tags: Vec<&str>) -> TaggedItem {
    TaggedItem {
        id: id.to_string(),
        content: content.to_string(),
        tags: tags.into_iter().map(String::from).collect(),
        metadata: ItemMetadata {
            created_at: "2024-01-01".to_string(),
            last_modified: "2024-01-01".to_string(),
            item_type: "test".to_string(),
        },
    }
}

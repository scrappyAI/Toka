use super::{Tool, ToolMetadata, ToolParams, ToolResult};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedItem {
    pub id: String,
    pub content: String,
    pub tags: HashSet<String>,
    pub metadata: ItemMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemMetadata {
    pub created_at: String,
    pub last_modified: String,
    pub item_type: String,
}

/// Tool for semantic indexing and querying of financial data
#[derive(Clone)]
pub struct SemanticIndexTool {
    name: String,
    description: String,
    version: String,
    index: Arc<RwLock<HashMap<String, TaggedItem>>>,
    tag_index: Arc<RwLock<HashMap<String, HashSet<String>>>>,
}

impl SemanticIndexTool {
    pub fn new() -> Self {
        Self {
            name: "semantic_index".to_string(),
            description: "Index and query financial data using semantic tags".to_string(),
            version: "1.0.0".to_string(),
            index: Arc::new(RwLock::new(HashMap::new())),
            tag_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn parse_item(&self, data: &str) -> Result<TaggedItem> {
        serde_json::from_str(data).with_context(|| "Failed to parse tagged item data")
    }

    async fn index_item(&self, item: TaggedItem) -> Result<()> {
        // Add to main index
        {
            let mut index = self.index.write().await;
            index.insert(item.id.clone(), item.clone());
        }

        // Update tag index
        {
            let mut tag_index = self.tag_index.write().await;
            for tag in &item.tags {
                tag_index
                    .entry(tag.clone())
                    .or_insert_with(HashSet::new)
                    .insert(item.id.clone());
            }
        }

        Ok(())
    }

    async fn search_by_tag(&self, tag: &str) -> Result<Vec<TaggedItem>> {
        let tag_index = self.tag_index.read().await;
        let index = self.index.read().await;

        let results = tag_index
            .get(tag)
            .map(|ids| ids.iter().filter_map(|id| index.get(id).cloned()).collect())
            .unwrap_or_default();

        Ok(results)
    }

    async fn delete_item(&self, id: &str) -> Result<()> {
        // Get the item first to know its tags
        let item = {
            let index = self.index.read().await;
            index.get(id).cloned()
        };

        if let Some(item) = item {
            // Remove from tag index
            {
                let mut tag_index = self.tag_index.write().await;
                for tag in &item.tags {
                    if let Some(ids) = tag_index.get_mut(tag) {
                        ids.remove(id);
                        if ids.is_empty() {
                            tag_index.remove(tag);
                        }
                    }
                }
            }

            // Remove from main index
            {
                let mut index = self.index.write().await;
                index.remove(id);
            }
        }

        Ok(())
    }

    async fn list_items(&self) -> Result<Vec<TaggedItem>> {
        let index = self.index.read().await;
        Ok(index.values().cloned().collect())
    }
}

#[async_trait::async_trait]
impl Tool for SemanticIndexTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        &self.version
    }

    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let command = params
            .args
            .get("command")
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;

        match command.as_str() {
            "index" => {
                let data = params
                    .args
                    .get("data")
                    .ok_or_else(|| anyhow::anyhow!("Missing 'data' parameter"))?;
                let item = self.parse_item(data).await?;
                self.index_item(item.clone()).await?;
                Ok(ToolResult {
                    success: true,
                    output: format!("Successfully indexed item: {}", item.id),
                    metadata: ToolMetadata {
                        execution_time_ms: 0,
                        tool_version: self.version.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                })
            }
            "search" => {
                let tag = params
                    .args
                    .get("tag")
                    .ok_or_else(|| anyhow::anyhow!("Missing 'tag' parameter"))?;
                let results = self.search_by_tag(tag).await?;

                let output = if results.is_empty() {
                    format!("No items found with tag: {}", tag)
                } else {
                    let mut formatted =
                        format!("Found {} items with tag '{}':\n", results.len(), tag);
                    for item in results {
                        formatted.push_str(&format!(
                            "\nID: {}\nContent: {}\nTags: {}\n",
                            item.id,
                            item.content,
                            item.tags
                                .iter()
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    formatted
                };

                Ok(ToolResult {
                    success: true,
                    output,
                    metadata: ToolMetadata {
                        execution_time_ms: 0,
                        tool_version: self.version.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                })
            }
            "delete" => {
                let id = params
                    .args
                    .get("id")
                    .ok_or_else(|| anyhow::anyhow!("Missing 'id' parameter"))?;
                self.delete_item(id).await?;
                Ok(ToolResult {
                    success: true,
                    output: format!("Successfully deleted item: {}", id),
                    metadata: ToolMetadata {
                        execution_time_ms: 0,
                        tool_version: self.version.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                })
            }
            "list" => {
                let items = self.list_items().await?;
                let output = if items.is_empty() {
                    "No items in index".to_string()
                } else {
                    let mut formatted = format!("Found {} items in index:\n", items.len());
                    for item in items {
                        formatted.push_str(&format!(
                            "\nID: {}\nContent: {}\nTags: {}\n",
                            item.id,
                            item.content,
                            item.tags
                                .iter()
                                .map(|s| s.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        ));
                    }
                    formatted
                };

                Ok(ToolResult {
                    success: true,
                    output,
                    metadata: ToolMetadata {
                        execution_time_ms: 0,
                        tool_version: self.version.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                })
            }
            _ => Err(anyhow::anyhow!("Unknown command: {}", command)),
        }
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("command") {
            return Err(anyhow::anyhow!("Missing required parameter: command"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_index_tool() -> Result<()> {
        let tool = SemanticIndexTool::new();

        // Create test item
        let test_item = TaggedItem {
            id: "1".to_string(),
            content: "Test transaction".to_string(),
            tags: {
                let mut tags = HashSet::new();
                tags.insert("test".to_string());
                tags.insert("transaction".to_string());
                tags
            },
            metadata: ItemMetadata {
                created_at: "2024-01-01".to_string(),
                last_modified: "2024-01-01".to_string(),
                item_type: "transaction".to_string(),
            },
        };

        // Test indexing
        let params = ToolParams {
            name: "semantic_index".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), "index".to_string());
                map.insert(
                    "data".to_string(),
                    serde_json::to_string(&test_item).unwrap(),
                );
                map
            },
        };

        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("Successfully indexed"));

        // Test searching
        let search_params = ToolParams {
            name: "semantic_index".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), "search".to_string());
                map.insert("tag".to_string(), "test".to_string());
                map
            },
        };

        let result = tool.execute(&search_params).await?;
        assert!(result.success);
        assert!(result.output.contains("Found 1 items"));
        assert!(result.output.contains("Test transaction"));

        // Test listing
        let list_params = ToolParams {
            name: "semantic_index".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), "list".to_string());
                map
            },
        };

        let result = tool.execute(&list_params).await?;
        assert!(result.success);
        assert!(result.output.contains("Found 1 items"));

        // Test deletion
        let delete_params = ToolParams {
            name: "semantic_index".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), "delete".to_string());
                map.insert("id".to_string(), "1".to_string());
                map
            },
        };

        let result = tool.execute(&delete_params).await?;
        assert!(result.success);
        assert!(result.output.contains("Successfully deleted"));

        // Verify deletion
        let list_params = ToolParams {
            name: "semantic_index".to_string(),
            args: {
                let mut map = std::collections::HashMap::new();
                map.insert("command".to_string(), "list".to_string());
                map
            },
        };

        let result = tool.execute(&list_params).await?;
        assert!(result.success);
        assert!(result.output.contains("No items in index"));

        Ok(())
    }
}

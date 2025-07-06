//! Standard **toolkit** crate with essential tools for agent functionality.
//!
//! This crate provides:
//! 1. `ToolRegistry` – thin wrapper around `toka_toolkit_core::ToolRegistry`
//! 2. Essential tools for file operations, process management, network requests, and text processing
//! 3. Authoring guidelines (`TOOL_GUIDELINES`) embedded as a constant string

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::Instant;

// Re-export canonical types from `toka_toolkit_core` so internal modules can
// simply `use crate::tools::{Tool, ToolParams, …}` without caring about the
// crate boundary.
pub use crate::core::{Tool, ToolMetadata, ToolParams, ToolResult};

/// Thin wrapper that delegates every call to an inner `CoreRegistry`.
/// Essential tools are registered by default for common agent operations.
pub type ToolRegistry = crate::core::ToolRegistry;

/// Canonical guidelines for building agent tools – markdown formatted.
pub const TOOL_GUIDELINES: &str = include_str!("../../TOOL_DEVELOPMENT.md");

/// File system tool for reading file contents
pub struct ReadFileTool {
    name: String,
    description: String,
    version: String,
}

impl ReadFileTool {
    pub fn new() -> Self {
        Self {
            name: "read_file".to_string(),
            description: "Read contents of a file".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[async_trait]
impl Tool for ReadFileTool {
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
        let start_time = Instant::now();
        
        let file_path = params
            .args
            .get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;

        let content = match tokio::fs::read_to_string(file_path).await {
            Ok(content) => content,
            Err(e) => return Ok(ToolResult {
                success: false,
                output: format!("Error reading file: {}", e),
                metadata: ToolMetadata {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_version: self.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            }),
        };

        Ok(ToolResult {
            success: true,
            output: content,
            metadata: ToolMetadata {
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                tool_version: self.version.clone(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("path") {
            return Err(anyhow::anyhow!("Missing required parameter: path"));
        }
        Ok(())
    }
}

/// File system tool for writing file contents
pub struct WriteFileTool {
    name: String,
    description: String,
    version: String,
}

impl WriteFileTool {
    pub fn new() -> Self {
        Self {
            name: "write_file".to_string(),
            description: "Write contents to a file".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[async_trait]
impl Tool for WriteFileTool {
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
        let start_time = Instant::now();
        
        let file_path = params
            .args
            .get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        
        let content = params
            .args
            .get("content")
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;

        let result = match tokio::fs::write(file_path, content).await {
            Ok(_) => ToolResult {
                success: true,
                output: format!("Successfully wrote {} bytes to {}", content.len(), file_path),
                metadata: ToolMetadata {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_version: self.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            },
            Err(e) => ToolResult {
                success: false,
                output: format!("Error writing file: {}", e),
                metadata: ToolMetadata {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_version: self.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            },
        };

        Ok(result)
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("path") {
            return Err(anyhow::anyhow!("Missing required parameter: path"));
        }
        if !params.args.contains_key("content") {
            return Err(anyhow::anyhow!("Missing required parameter: content"));
        }
        Ok(())
    }
}

/// Process execution tool for running commands
pub struct RunCommandTool {
    name: String,
    description: String,
    version: String,
}

impl RunCommandTool {
    pub fn new() -> Self {
        Self {
            name: "run_command".to_string(),
            description: "Execute a system command".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[async_trait]
impl Tool for RunCommandTool {
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
        let start_time = Instant::now();
        
        let command = params
            .args
            .get("command")
            .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;
        
        let working_dir = params.args.get("working_dir").cloned();
        
        let mut cmd = tokio::process::Command::new("sh");
        cmd.arg("-c").arg(command);
        
        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }
        
        let result = match cmd.output().await {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                ToolResult {
                    success: output.status.success(),
                    output: if output.status.success() {
                        stdout.to_string()
                    } else {
                        format!("Command failed with exit code: {}\nstderr: {}", 
                                output.status.code().unwrap_or(-1), stderr)
                    },
                    metadata: ToolMetadata {
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        tool_version: self.version.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                }
            },
            Err(e) => ToolResult {
                success: false,
                output: format!("Error executing command: {}", e),
                metadata: ToolMetadata {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_version: self.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            },
        };

        Ok(result)
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("command") {
            return Err(anyhow::anyhow!("Missing required parameter: command"));
        }
        Ok(())
    }
}

/// HTTP request tool for making web requests
pub struct HttpRequestTool {
    name: String,
    description: String,
    version: String,
}

impl HttpRequestTool {
    pub fn new() -> Self {
        Self {
            name: "http_request".to_string(),
            description: "Make HTTP requests".to_string(),
            version: "1.0.0".to_string(),
        }
    }
}

#[async_trait]
impl Tool for HttpRequestTool {
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
        let start_time = Instant::now();
        
        let url = params
            .args
            .get("url")
            .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
        
        let method = params.args.get("method").cloned().unwrap_or_else(|| "GET".to_string());
        let body = params.args.get("body").cloned();
        
        let client = reqwest::Client::new();
        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => return Ok(ToolResult {
                success: false,
                output: format!("Unsupported HTTP method: {}", method),
                metadata: ToolMetadata {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_version: self.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            }),
        };
        
        if let Some(body_content) = body {
            request = request.body(body_content);
        }
        
        let result = match request.send().await {
            Ok(response) => {
                let status = response.status();
                let text = response.text().await.unwrap_or_else(|e| format!("Error reading response: {}", e));
                
                ToolResult {
                    success: status.is_success(),
                    output: format!("Status: {}\nBody: {}", status, text),
                    metadata: ToolMetadata {
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        tool_version: self.version.clone(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    },
                }
            },
            Err(e) => ToolResult {
                success: false,
                output: format!("HTTP request failed: {}", e),
                metadata: ToolMetadata {
                    execution_time_ms: start_time.elapsed().as_millis() as u64,
                    tool_version: self.version.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                },
            },
        };

        Ok(result)
    }

    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        if !params.args.contains_key("url") {
            return Err(anyhow::anyhow!("Missing required parameter: url"));
        }
        Ok(())
    }
}

/// Helper function to register all essential tools
pub async fn register_essential_tools(registry: &ToolRegistry) -> Result<()> {
    registry.register_tool(Arc::new(ReadFileTool::new())).await?;
    registry.register_tool(Arc::new(WriteFileTool::new())).await?;
    registry.register_tool(Arc::new(RunCommandTool::new())).await?;
    registry.register_tool(Arc::new(HttpRequestTool::new())).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_read_file_tool() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Hello, World!")?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        let tool = ReadFileTool::new();
        let mut params = ToolParams {
            name: "read_file".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("path".to_string(), temp_path.to_string());
        
        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("Hello, World!"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_write_file_tool() -> Result<()> {
        let temp_file = NamedTempFile::new()?;
        let temp_path = temp_file.path().to_str().unwrap();
        
        let tool = WriteFileTool::new();
        let mut params = ToolParams {
            name: "write_file".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("path".to_string(), temp_path.to_string());
        params.args.insert("content".to_string(), "Test content".to_string());
        
        let result = tool.execute(&params).await?;
        assert!(result.success);
        
        // Verify file was written
        let content = tokio::fs::read_to_string(temp_path).await?;
        assert_eq!(content, "Test content");
        
        Ok(())
    }

    #[tokio::test]
    async fn test_run_command_tool() -> Result<()> {
        let tool = RunCommandTool::new();
        let mut params = ToolParams {
            name: "run_command".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("command".to_string(), "echo 'Hello from command'".to_string());
        
        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("Hello from command"));
        
        Ok(())
    }

    #[tokio::test]
    async fn test_tool_registry_with_essential_tools() -> Result<()> {
        let registry = ToolRegistry::new().await?;
        register_essential_tools(&registry).await?;

        let tools = registry.list_tools().await;
        assert!(tools.contains(&"read_file".to_string()));
        assert!(tools.contains(&"write_file".to_string()));
        assert!(tools.contains(&"run_command".to_string()));
        assert!(tools.contains(&"http_request".to_string()));

        Ok(())
    }
}

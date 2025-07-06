//! File Reader Tool - Secure file reading with kernel enforcement
//!
//! This tool demonstrates how to implement secure file operations using
//! the toka-kernel enforcement layer. All file access is validated
//! through the security context before execution.

use std::sync::Arc;
use anyhow::Result;
use serde_json::{json, Value as JsonValue};
use tokio::fs;

use crate::{
    Tool, ToolDefinition, ToolMetadata, ExecutionContext, ToolKernel,
    Capability, CapabilitySet, FileAccess,
};

/// File reader tool implementation
pub struct FileReaderTool;

#[async_trait::async_trait]
impl Tool for FileReaderTool {
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata {
            id: "file_reader".to_string(),
            name: "File Reader".to_string(),
            description: "Securely read file contents with kernel enforcement".to_string(),
            version: "1.0.0".to_string(),
            author: "Toka Core".to_string(),
            required_capabilities: CapabilitySet::with_capabilities(vec![
                Capability::FileRead(FileAccess::Global),
            ]),
            parameter_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file to read"
                    },
                    "encoding": {
                        "type": "string",
                        "enum": ["utf-8", "ascii", "binary"],
                        "default": "utf-8",
                        "description": "File encoding format"
                    },
                    "max_size_mb": {
                        "type": "number",
                        "default": 10,
                        "description": "Maximum file size in MB"
                    }
                },
                "required": ["path"]
            }),
            output_schema: json!({
                "type": "object",
                "properties": {
                    "content": {
                        "type": "string",
                        "description": "File content as string"
                    },
                    "size_bytes": {
                        "type": "number",
                        "description": "File size in bytes"
                    },
                    "encoding": {
                        "type": "string",
                        "description": "Encoding used to read the file"
                    },
                    "path": {
                        "type": "string",
                        "description": "Path that was read"
                    }
                }
            }),
        }
    }
    
    async fn validate_parameters(&self, parameters: &JsonValue) -> Result<()> {
        // Validate required path parameter
        let path = parameters.get("path")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required parameter: path"))?;
        
        // Basic path validation
        if path.is_empty() {
            return Err(anyhow::anyhow!("Path cannot be empty"));
        }
        
        // Check for suspicious path patterns
        if path.contains("..") {
            return Err(anyhow::anyhow!("Path traversal not allowed"));
        }
        
        // Validate encoding if provided
        if let Some(encoding) = parameters.get("encoding") {
            if let Some(enc_str) = encoding.as_str() {
                match enc_str {
                    "utf-8" | "ascii" | "binary" => {},
                    _ => return Err(anyhow::anyhow!("Invalid encoding: {}", enc_str)),
                }
            }
        }
        
        // Validate max_size_mb if provided
        if let Some(max_size) = parameters.get("max_size_mb") {
            if let Some(size) = max_size.as_f64() {
                if size <= 0.0 || size > 100.0 {
                    return Err(anyhow::anyhow!("max_size_mb must be between 0 and 100"));
                }
            }
        }
        
        Ok(())
    }
    
    async fn execute(
        &self,
        context: &ExecutionContext,
        parameters: &JsonValue,
        kernel: &ToolKernel,
    ) -> Result<JsonValue> {
        // Extract parameters
        let path = parameters["path"].as_str().unwrap();
        let encoding = parameters.get("encoding")
            .and_then(|e| e.as_str())
            .unwrap_or("utf-8");
        let max_size_mb = parameters.get("max_size_mb")
            .and_then(|s| s.as_f64())
            .unwrap_or(10.0);
        
        // Validate file access through kernel security context
        kernel.security_context.validate_file_access(context, "read", path).await
            .map_err(|e| anyhow::anyhow!("File access denied: {}", e))?;
        
        // Check if file exists and get metadata
        let metadata = fs::metadata(path).await
            .map_err(|e| anyhow::anyhow!("Failed to access file {}: {}", path, e))?;
        
        // Check file size against limit
        let file_size_bytes = metadata.len();
        let file_size_mb = file_size_bytes as f64 / (1024.0 * 1024.0);
        
        if file_size_mb > max_size_mb {
            return Err(anyhow::anyhow!(
                "File size ({:.2} MB) exceeds limit ({:.2} MB)",
                file_size_mb,
                max_size_mb
            ));
        }
        
        // Read file content based on encoding
        let content = match encoding {
            "binary" => {
                let bytes = fs::read(path).await
                    .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;
                
                // Convert binary to base64 for JSON serialization
                base64::encode(&bytes)
            },
            "utf-8" | "ascii" => {
                fs::read_to_string(path).await
                    .map_err(|e| anyhow::anyhow!("Failed to read file as string: {}", e))?
            },
            _ => return Err(anyhow::anyhow!("Unsupported encoding: {}", encoding)),
        };
        
        // Log successful file access for auditing
        tracing::info!(
            tool_id = context.tool_id,
            session_id = context.session_id,
            file_path = path,
            file_size = file_size_bytes,
            encoding = encoding,
            "File read successful"
        );
        
        // Return structured result
        Ok(json!({
            "content": content,
            "size_bytes": file_size_bytes,
            "encoding": encoding,
            "path": path
        }))
    }
    
    fn help(&self) -> String {
        r#"File Reader Tool

Securely reads file contents with kernel enforcement and validation.

Parameters:
- path (required): File path to read
- encoding (optional): Encoding format - "utf-8" (default), "ascii", or "binary"
- max_size_mb (optional): Maximum file size in MB (default: 10)

Security Features:
- Path traversal protection
- File access validation through kernel security context
- File size limits
- Encoding validation
- Full audit logging

Examples:
{"path": "./README.md"}
{"path": "./data.txt", "encoding": "utf-8", "max_size_mb": 5}
{"path": "./image.png", "encoding": "binary", "max_size_mb": 20}
"#.to_string()
    }
}

/// Create file reader tool definition
pub fn create_tool_definition() -> ToolDefinition {
    ToolDefinition {
        id: "file_reader".to_string(),
        name: "File Reader".to_string(),
        description: "Securely read file contents with kernel enforcement".to_string(),
        version: "1.0.0".to_string(),
        required_capabilities: CapabilitySet::with_capabilities(vec![
            Capability::FileRead(FileAccess::Global),
        ]),
        implementation: Arc::new(FileReaderTool),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SecurityLevel, CapabilitySet};
    use toka_kernel::presets;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_file_reader_validation() {
        let tool = FileReaderTool;
        
        // Valid parameters
        let valid_params = json!({"path": "./test.txt"});
        assert!(tool.validate_parameters(&valid_params).await.is_ok());
        
        // Missing path
        let invalid_params = json!({});
        assert!(tool.validate_parameters(&invalid_params).await.is_err());
        
        // Path traversal attempt
        let traversal_params = json!({"path": "../../../etc/passwd"});
        assert!(tool.validate_parameters(&traversal_params).await.is_err());
        
        // Invalid encoding
        let invalid_encoding = json!({"path": "./test.txt", "encoding": "invalid"});
        assert!(tool.validate_parameters(&invalid_encoding).await.is_err());
    }
    
    #[tokio::test]
    async fn test_file_reader_execution() {
        // Create a temporary file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Hello, Toka!").unwrap();
        let temp_path = temp_file.path().to_str().unwrap();
        
        // Set up test environment
        let kernel = presets::testing_kernel().await.unwrap();
        let tool = FileReaderTool;
        
        // Grant file read capabilities
        let capabilities = CapabilitySet::with_capabilities(vec![
            Capability::FileRead(FileAccess::Global),
        ]);
        kernel.grant_capabilities("test_session", capabilities).await.unwrap();
        
        // Create execution context
        let context = kernel.create_execution_context(
            "file_reader",
            "test_session",
            &tool.metadata().required_capabilities,
            SecurityLevel::Restricted,
        ).await.unwrap();
        
        // Execute tool
        let parameters = json!({
            "path": temp_path,
            "encoding": "utf-8"
        });
        
        let result = tool.execute(&context, &parameters, &kernel).await.unwrap();
        
        // Verify result
        assert_eq!(result["path"].as_str().unwrap(), temp_path);
        assert_eq!(result["encoding"].as_str().unwrap(), "utf-8");
        assert!(result["content"].as_str().unwrap().contains("Hello, Toka!"));
        assert!(result["size_bytes"].as_u64().unwrap() > 0);
    }
    
    #[tokio::test]
    async fn test_file_reader_binary_mode() {
        // Create a temporary binary file
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(&[0x00, 0x01, 0x02, 0x03, 0xFF]).unwrap();
        let temp_path = temp_file.path().to_str().unwrap();
        
        // Set up test environment
        let kernel = presets::testing_kernel().await.unwrap();
        let tool = FileReaderTool;
        
        // Grant capabilities and create context
        let capabilities = CapabilitySet::with_capabilities(vec![
            Capability::FileRead(FileAccess::Global),
        ]);
        kernel.grant_capabilities("test_session", capabilities).await.unwrap();
        
        let context = kernel.create_execution_context(
            "file_reader",
            "test_session",
            &tool.metadata().required_capabilities,
            SecurityLevel::Restricted,
        ).await.unwrap();
        
        // Execute in binary mode
        let parameters = json!({
            "path": temp_path,
            "encoding": "binary"
        });
        
        let result = tool.execute(&context, &parameters, &kernel).await.unwrap();
        
        // Verify binary content is base64 encoded
        assert_eq!(result["encoding"].as_str().unwrap(), "binary");
        let content = result["content"].as_str().unwrap();
        
        // Decode and verify binary content
        let decoded = base64::decode(content).unwrap();
        assert_eq!(decoded, vec![0x00, 0x01, 0x02, 0x03, 0xFF]);
    }
    
    #[test]
    fn test_tool_metadata() {
        let tool = FileReaderTool;
        let metadata = tool.metadata();
        
        assert_eq!(metadata.id, "file_reader");
        assert_eq!(metadata.name, "File Reader");
        assert!(!metadata.description.is_empty());
        assert!(metadata.required_capabilities.contains(&Capability::FileRead(FileAccess::Global)));
    }
} 
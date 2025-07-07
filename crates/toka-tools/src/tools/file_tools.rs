//! File operation tools for reading, writing, and listing files

use std::path::{Path, PathBuf};

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::info;

use crate::core::{Tool, ToolResult, ToolParams};

/// File reader tool for reading file contents
#[derive(Debug, Clone)]
pub struct FileReader;

impl FileReader {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileReader {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Tool for FileReader {
    fn name(&self) -> &str {
        "file-reader"
    }
    
    fn description(&self) -> &str {
        "Read contents of a file"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let path = params.args.get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        
        let content = fs::read_to_string(path).await?;
        
        Ok(ToolResult {
            success: true,
            output: content,
            metadata: crate::core::ToolMetadata {
                execution_time_ms: 0, // Will be set by registry
                tool_version: self.version().to_string(),
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
        
        if let Some(path) = params.args.get("path") {
            if path.trim().is_empty() {
                return Err(anyhow::anyhow!("Path parameter cannot be empty"));
            }
        }
        
        Ok(())
    }
}

/// File writer tool for writing content to files
#[derive(Debug, Clone)]
pub struct FileWriter;

impl FileWriter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileWriter {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Tool for FileWriter {
    fn name(&self) -> &str {
        "file-writer"
    }
    
    fn description(&self) -> &str {
        "Write content to a file"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let path = params.args.get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        
        let content = params.args.get("content")
            .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
        
        fs::write(path, content).await?;
        
        info!("Successfully wrote {} bytes to {}", content.len(), path);
        
        Ok(ToolResult {
            success: true,
            output: format!("Successfully wrote {} bytes to {}", content.len(), path),
            metadata: crate::core::ToolMetadata {
                execution_time_ms: 0, // Will be set by registry
                tool_version: self.version().to_string(),
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
        
        if !params.args.contains_key("content") {
            return Err(anyhow::anyhow!("Missing required parameter: content"));
        }
        
        if let Some(path) = params.args.get("path") {
            if path.trim().is_empty() {
                return Err(anyhow::anyhow!("Path parameter cannot be empty"));
            }
        }
        
        Ok(())
    }
}

/// File lister tool for listing directory contents
#[derive(Debug, Clone)]
pub struct FileLister;

impl FileLister {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FileLister {
    fn default() -> Self { Self::new() }
}

#[async_trait]
impl Tool for FileLister {
    fn name(&self) -> &str {
        "file-lister"
    }
    
    fn description(&self) -> &str {
        "List contents of a directory"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let path = params.args.get("path")
            .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
        
        let recursive = params.args.get("recursive")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);
        
        let show_hidden = params.args.get("show_hidden")
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);
        
        let entries = if recursive {
            self.list_recursive(Path::new(path), show_hidden).await?
        } else {
            self.list_directory(Path::new(path), show_hidden).await?
        };
        
        let result = DirectoryListing {
            path: path.clone(),
            entries,
            recursive,
        };
        
        Ok(ToolResult {
            success: true,
            output: serde_json::to_string(&result)?,
            metadata: crate::core::ToolMetadata {
                execution_time_ms: 0, // Will be set by registry
                tool_version: self.version().to_string(),
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
        
        if let Some(path) = params.args.get("path") {
            if path.trim().is_empty() {
                return Err(anyhow::anyhow!("Path parameter cannot be empty"));
            }
        }
        
        Ok(())
    }
}

impl FileLister {
    async fn list_directory(&self, dir: &Path, show_hidden: bool) -> Result<Vec<FileEntry>> {
        let mut entries = Vec::new();
        let mut dir_entries = fs::read_dir(dir).await?;
        
        while let Some(entry) = dir_entries.next_entry().await? {
            let path = entry.path();
            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("").to_string();
            
            // Skip hidden files unless requested
            if !show_hidden && file_name.starts_with('.') {
                continue;
            }
            
            let metadata = entry.metadata().await?;
            
            entries.push(FileEntry {
                name: file_name,
                path: path.to_string_lossy().to_string(),
                is_directory: metadata.is_dir(),
                size: if metadata.is_file() { Some(metadata.len()) } else { None },
                modified: metadata.modified().ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
            });
        }
        
        // Sort entries: directories first, then by name
        entries.sort_by(|a, b| {
            match (a.is_directory, b.is_directory) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        });
        
        Ok(entries)
    }
    
    async fn list_recursive(&self, dir: &Path, show_hidden: bool) -> Result<Vec<FileEntry>> {
        let mut all_entries = Vec::new();
        let mut dirs_to_process = vec![dir.to_path_buf()];
        
        while let Some(current_dir) = dirs_to_process.pop() {
            let entries = self.list_directory(&current_dir, show_hidden).await?;
            
            for entry in entries {
                if entry.is_directory {
                    dirs_to_process.push(PathBuf::from(&entry.path));
                }
                all_entries.push(entry);
            }
        }
        
        Ok(all_entries)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryListing {
    pub path: String,
    pub entries: Vec<FileEntry>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub size: Option<u64>,
    pub modified: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::collections::HashMap;
    
    #[tokio::test]
    async fn test_file_reader() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        fs::write(&test_file, "Hello, World!").await.unwrap();
        
        let reader = FileReader::new();
        let mut params = ToolParams {
            name: "file-reader".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("path".to_string(), test_file.to_string_lossy().to_string());
        
        let result = reader.execute(&params).await.unwrap();
        
        assert!(result.success);
        assert_eq!(result.output, "Hello, World!");
    }
    
    #[tokio::test]
    async fn test_file_writer() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("test.txt");
        
        let writer = FileWriter::new();
        let mut params = ToolParams {
            name: "file-writer".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("path".to_string(), test_file.to_string_lossy().to_string());
        params.args.insert("content".to_string(), "Test content".to_string());
        
        let result = writer.execute(&params).await.unwrap();
        
        assert!(result.success);
        
        // Verify file was written
        let content = fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "Test content");
    }
    
    #[tokio::test]
    async fn test_file_lister() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test files and directories
        fs::write(temp_dir.path().join("file1.txt"), "content1").await.unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").await.unwrap();
        fs::create_dir(temp_dir.path().join("subdir")).await.unwrap();
        fs::write(temp_dir.path().join("subdir").join("file3.txt"), "content3").await.unwrap();
        
        let lister = FileLister::new();
        let mut params = ToolParams {
            name: "file-lister".to_string(),
            args: HashMap::new(),
        };
        params.args.insert("path".to_string(), temp_dir.path().to_string_lossy().to_string());
        
        let result = lister.execute(&params).await.unwrap();
        
        assert!(result.success);
        
        let listing: DirectoryListing = serde_json::from_str(&result.output).unwrap();
        assert!(!listing.entries.is_empty());
        assert!(listing.entries.iter().any(|e| e.name == "file1.txt"));
        assert!(listing.entries.iter().any(|e| e.name == "subdir" && e.is_directory));
    }
} 
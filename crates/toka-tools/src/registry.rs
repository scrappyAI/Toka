//! Tool registry auto-discovery and integration utilities.
//!
//! This module provides utilities for automatically discovering and registering
//! external tools (Python scripts, shell scripts) in the workspace.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

#[cfg(feature = "external-tools")]
use crate::wrappers::{ExternalTool, PythonTool, ShellTool};
use crate::core::{ToolRegistry, Tool};

/// Tool discovery configuration
#[derive(Debug, Clone)]
pub struct ToolDiscoveryConfig {
    /// Directories to search for tools
    pub search_directories: Vec<PathBuf>,
    /// File patterns to include (e.g., "*.py", "*.sh")
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Whether to follow symbolic links
    pub follow_symlinks: bool,
    /// Maximum recursion depth
    pub max_depth: usize,
}

impl Default for ToolDiscoveryConfig {
    fn default() -> Self {
        Self {
            search_directories: vec![
                PathBuf::from("scripts"),
                PathBuf::from("tools"),
                PathBuf::from("bin"),
            ],
            include_patterns: vec![
                "*.py".to_string(),
                "*.sh".to_string(),
                "*.bash".to_string(),
            ],
            exclude_patterns: vec![
                "*test*".to_string(),
                "*example*".to_string(),
                "*tmp*".to_string(),
                "__pycache__".to_string(),
            ],
            follow_symlinks: false,
            max_depth: 3,
        }
    }
}

/// Auto-discovered tool information
#[derive(Debug, Clone)]
pub struct DiscoveredTool {
    /// Tool file path
    pub path: PathBuf,
    /// Inferred tool name
    pub name: String,
    /// Inferred description
    pub description: String,
    /// Inferred capabilities
    pub capabilities: Vec<String>,
    /// Tool type (python, shell, etc.)
    pub tool_type: ToolType,
}

/// Type of discovered tool
#[derive(Debug, Clone, PartialEq)]
pub enum ToolType {
    /// Python script
    Python,
    /// Shell script
    Shell,
    /// Other executable
    Other,
}

/// Tool discovery and registration utility
pub struct ToolDiscovery {
    config: ToolDiscoveryConfig,
}

impl ToolDiscovery {
    /// Create a new tool discovery instance
    pub fn new() -> Self {
        Self {
            config: ToolDiscoveryConfig::default(),
        }
    }
    
    /// Create with custom configuration
    pub fn with_config(config: ToolDiscoveryConfig) -> Self {
        Self { config }
    }
    
    /// Discover tools in the configured directories
    pub async fn discover_tools(&self) -> Result<Vec<DiscoveredTool>> {
        let mut discovered_tools = Vec::new();
        
        for search_dir in &self.config.search_directories {
            if !search_dir.exists() {
                debug!("Search directory does not exist: {}", search_dir.display());
                continue;
            }
            
            info!("Searching for tools in: {}", search_dir.display());
            
            let tools = self.scan_directory(search_dir).await?;
            discovered_tools.extend(tools);
        }
        
        info!("Discovered {} tools", discovered_tools.len());
        Ok(discovered_tools)
    }
    
    /// Scan a directory for tools
    async fn scan_directory(&self, dir: &Path) -> Result<Vec<DiscoveredTool>> {
        let mut tools = Vec::new();
        
        let mut entries = tokio::fs::read_dir(dir).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() && self.config.max_depth > 0 {
                // Recursively scan subdirectories
                let mut sub_config = self.config.clone();
                sub_config.max_depth -= 1;
                let sub_discovery = ToolDiscovery::with_config(sub_config);
                let sub_tools = sub_discovery.scan_directory(&path).await?;
                tools.extend(sub_tools);
            } else if path.is_file() {
                if let Some(tool) = self.analyze_file(&path).await? {
                    tools.push(tool);
                }
            }
        }
        
        Ok(tools)
    }
    
    /// Analyze a file to determine if it's a tool
    async fn analyze_file(&self, path: &Path) -> Result<Option<DiscoveredTool>> {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        
        // Check if file matches include patterns
        if !self.matches_patterns(file_name, &self.config.include_patterns) {
            return Ok(None);
        }
        
        // Check if file matches exclude patterns
        if self.matches_patterns(file_name, &self.config.exclude_patterns) {
            return Ok(None);
        }
        
        // Determine tool type
        let tool_type = self.determine_tool_type(path)?;
        
        // Extract tool information
        let name = self.extract_tool_name(path);
        let description = self.extract_tool_description(path).await?;
        let capabilities = self.extract_tool_capabilities(path).await?;
        
        Ok(Some(DiscoveredTool {
            path: path.to_path_buf(),
            name,
            description,
            capabilities,
            tool_type,
        }))
    }
    
    /// Check if a filename matches any of the given patterns
    fn matches_patterns(&self, filename: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| {
            // Simple glob matching (just * wildcard)
            if pattern.contains('*') {
                let regex_pattern = pattern.replace('*', ".*");
                regex::Regex::new(&regex_pattern)
                    .map(|re| re.is_match(filename))
                    .unwrap_or(false)
            } else {
                filename == pattern
            }
        })
    }
    
    /// Determine the type of tool based on file extension and content
    fn determine_tool_type(&self, path: &Path) -> Result<ToolType> {
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension {
                "py" => Ok(ToolType::Python),
                "sh" | "bash" | "zsh" => Ok(ToolType::Shell),
                _ => Ok(ToolType::Other),
            }
        } else {
            Ok(ToolType::Other)
        }
    }
    
    /// Extract tool name from file path
    fn extract_tool_name(&self, path: &Path) -> String {
        path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown-tool")
            .replace('_', "-")
            .to_lowercase()
    }
    
    /// Extract tool description from file content
    async fn extract_tool_description(&self, path: &Path) -> Result<String> {
        let content = tokio::fs::read_to_string(path).await?;
        
        // Look for description in comments
        for line in content.lines().take(20) {
            let line = line.trim();
            
            // Python docstring or comment
            if line.starts_with("\"\"\"") || line.starts_with("'''") {
                let desc = line.trim_start_matches("\"\"\"")
                             .trim_start_matches("'''")
                             .trim();
                if !desc.is_empty() {
                    return Ok(desc.to_string());
                }
            }
            
            // Shell or Python comment
            if line.starts_with("# ") {
                let desc = line.trim_start_matches("# ").trim();
                if !desc.is_empty() && !desc.starts_with("!") {
                    return Ok(desc.to_string());
                }
            }
        }
        
        // Fallback description
        Ok(format!("Tool: {}", self.extract_tool_name(path)))
    }
    
    /// Extract tool capabilities from file content and name
    async fn extract_tool_capabilities(&self, path: &Path) -> Result<Vec<String>> {
        let mut capabilities = Vec::new();
        let name = self.extract_tool_name(path);
        
        // Infer capabilities from tool name
        if name.contains("test") {
            capabilities.push("testing".to_string());
        }
        if name.contains("build") {
            capabilities.push("build-system".to_string());
        }
        if name.contains("date") || name.contains("time") {
            capabilities.push("date-validation".to_string());
        }
        if name.contains("monitor") || name.contains("watch") {
            capabilities.push("monitoring".to_string());
        }
        if name.contains("validate") || name.contains("check") {
            capabilities.push("validation".to_string());
        }
        if name.contains("deploy") || name.contains("release") {
            capabilities.push("deployment".to_string());
        }
        if name.contains("raft") {
            capabilities.push("raft-monitoring".to_string());
        }
        
        // Default capability if none inferred
        if capabilities.is_empty() {
            capabilities.push("general".to_string());
        }
        
        Ok(capabilities)
    }
}

impl Default for ToolDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry extension for auto-discovery
pub trait ToolRegistryExt {
    /// Auto-discover and register tools from the workspace
    async fn auto_register_tools(&self) -> Result<usize>;
    
    /// Auto-discover and register tools with custom configuration
    async fn auto_register_tools_with_config(&self, config: ToolDiscoveryConfig) -> Result<usize>;
}

#[cfg(feature = "external-tools")]
impl ToolRegistryExt for ToolRegistry {
    async fn auto_register_tools(&self) -> Result<usize> {
        let discovery = ToolDiscovery::new();
        let discovered_tools = discovery.discover_tools().await?;
        
        let mut registered_count = 0;
        
        for discovered in discovered_tools {
            match self.register_discovered_tool(discovered).await {
                Ok(()) => registered_count += 1,
                Err(e) => warn!("Failed to register tool: {}", e),
            }
        }
        
        info!("Auto-registered {} tools", registered_count);
        Ok(registered_count)
    }
    
    async fn auto_register_tools_with_config(&self, config: ToolDiscoveryConfig) -> Result<usize> {
        let discovery = ToolDiscovery::with_config(config);
        let discovered_tools = discovery.discover_tools().await?;
        
        let mut registered_count = 0;
        
        for discovered in discovered_tools {
            match self.register_discovered_tool(discovered).await {
                Ok(()) => registered_count += 1,
                Err(e) => warn!("Failed to register tool: {}", e),
            }
        }
        
        info!("Auto-registered {} tools with custom config", registered_count);
        Ok(registered_count)
    }
}

#[cfg(feature = "external-tools")]
impl ToolRegistry {
    /// Register a discovered tool
    async fn register_discovered_tool(&self, discovered: DiscoveredTool) -> Result<()> {
        let tool: Arc<dyn Tool + Send + Sync> = match discovered.tool_type {
            ToolType::Python => {
                let python_tool = PythonTool::new(
                    discovered.path,
                    &discovered.name,
                    &discovered.description,
                    discovered.capabilities,
                )?;
                Arc::new(python_tool)
            }
            ToolType::Shell => {
                let shell_tool = ShellTool::new(
                    discovered.path,
                    &discovered.name,
                    &discovered.description,
                    discovered.capabilities,
                )?;
                Arc::new(shell_tool)
            }
            ToolType::Other => {
                let external_tool = ExternalTool::wrap_shell_script(
                    &discovered.path,
                    &discovered.name,
                    &discovered.description,
                    discovered.capabilities,
                )?;
                Arc::new(external_tool)
            }
        };
        
        self.register_tool(tool).await?;
        debug!("Registered tool: {}", discovered.name);
        Ok(())
    }
}

#[cfg(not(feature = "external-tools"))]
impl ToolRegistryExt for ToolRegistry {
    async fn auto_register_tools(&self) -> Result<usize> {
        warn!("Auto-registration requires 'external-tools' feature");
        Ok(0)
    }
    
    async fn auto_register_tools_with_config(&self, _config: ToolDiscoveryConfig) -> Result<usize> {
        warn!("Auto-registration requires 'external-tools' feature");
        Ok(0)
    }
}

// Simple regex implementation for pattern matching
mod regex {
    pub struct Regex {
        pattern: String,
    }
    
    impl Regex {
        pub fn new(pattern: &str) -> Result<Self, &'static str> {
            Ok(Self {
                pattern: pattern.to_string(),
            })
        }
        
        pub fn is_match(&self, text: &str) -> bool {
            // Very simple regex implementation - just handle .* wildcard
            if self.pattern == ".*" {
                true
            } else if self.pattern.starts_with(".*") && self.pattern.len() > 2 {
                let suffix = &self.pattern[2..];
                text.ends_with(suffix)
            } else if self.pattern.ends_with(".*") && self.pattern.len() > 2 {
                let prefix = &self.pattern[..self.pattern.len() - 2];
                text.starts_with(prefix)
            } else {
                text == self.pattern
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_tool_discovery() -> Result<()> {
        // Create temporary directory structure
        let temp_dir = TempDir::new()?;
        let scripts_dir = temp_dir.path().join("scripts");
        tokio::fs::create_dir(&scripts_dir).await?;
        
        // Create test Python script
        let python_script = scripts_dir.join("test_tool.py");
        tokio::fs::write(&python_script, r#"#!/usr/bin/env python3
"""
Test Python tool for validation
"""
import sys
print("Hello from Python tool!")
"#).await?;
        
        // Create test shell script
        let shell_script = scripts_dir.join("build_checker.sh");
        tokio::fs::write(&shell_script, r#"#!/bin/bash
# Build system validation tool
echo "Checking build system..."
"#).await?;
        
        // Configure discovery
        let config = ToolDiscoveryConfig {
            search_directories: vec![scripts_dir],
            ..Default::default()
        };
        
        let discovery = ToolDiscovery::with_config(config);
        let tools = discovery.discover_tools().await?;
        
        assert_eq!(tools.len(), 2);
        
        // Check Python tool
        let python_tool = tools.iter().find(|t| t.tool_type == ToolType::Python).unwrap();
        assert_eq!(python_tool.name, "test-tool");
        assert!(python_tool.capabilities.contains(&"testing".to_string()));
        
        // Check shell tool
        let shell_tool = tools.iter().find(|t| t.tool_type == ToolType::Shell).unwrap();
        assert_eq!(shell_tool.name, "build-checker");
        assert!(shell_tool.capabilities.contains(&"build-system".to_string()));
        
        Ok(())
    }
    
    #[cfg(feature = "external-tools")]
    #[tokio::test]
    async fn test_auto_registration() -> Result<()> {
        // Create temporary directory structure
        let temp_dir = TempDir::new()?;
        let scripts_dir = temp_dir.path().join("scripts");
        tokio::fs::create_dir(&scripts_dir).await?;
        
        // Create test script
        let test_script = scripts_dir.join("echo_tool.py");
        tokio::fs::write(&test_script, r#"#!/usr/bin/env python3
print("Echo tool!")
"#).await?;
        
        // Create registry and auto-register
        let registry = ToolRegistry::new_empty();
        
        let config = ToolDiscoveryConfig {
            search_directories: vec![scripts_dir],
            ..Default::default()
        };
        
        let count = registry.auto_register_tools_with_config(config).await?;
        assert_eq!(count, 1);
        
        // Verify tool was registered
        let tools = registry.list_tools().await;
        assert!(tools.contains(&"echo-tool".to_string()));
        
        Ok(())
    }
}
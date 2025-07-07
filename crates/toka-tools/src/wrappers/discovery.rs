//! Tool discovery system for automatically finding and registering external tools
//!
//! This module provides automatic discovery of Python scripts, shell scripts, and other
//! executables in the workspace, with intelligent capability inference and security
//! classification.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, info};
use super::{DiscoveredTool, ToolType};

/// Configuration for tool discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Directories to search for tools
    pub search_directories: Vec<PathBuf>,
    /// File patterns to include
    pub include_patterns: Vec<String>,
    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,
    /// Follow symbolic links
    pub follow_symlinks: bool,
    /// Maximum depth to search
    pub max_depth: usize,
    /// Capability inference rules
    pub capability_rules: CapabilityInferenceRules,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            search_directories: vec![
                PathBuf::from("scripts"),
                PathBuf::from("."),
            ],
            include_patterns: vec![
                "*.py".to_string(),
                "*.sh".to_string(),
                "*.bash".to_string(),
            ],
            exclude_patterns: vec![
                "*test*".to_string(),
                "*__pycache__*".to_string(),
                "*.pyc".to_string(),
                ".git/*".to_string(),
                "target/*".to_string(),
            ],
            follow_symlinks: false,
            max_depth: 3,
            capability_rules: CapabilityInferenceRules::default(),
        }
    }
}

/// Tool discovery system
pub struct ToolDiscovery {
    config: DiscoveryConfig,
    capability_inferrer: CapabilityInferrer,
}

impl ToolDiscovery {
    /// Create a new tool discovery system
    pub fn new(config: DiscoveryConfig) -> Self {
        let capability_inferrer = CapabilityInferrer::new(config.capability_rules.clone());
        
        Self {
            config,
            capability_inferrer,
        }
    }
    
    /// Create tool discovery with security-focused defaults
    pub fn new_with_security_defaults() -> Self {
        let mut config = DiscoveryConfig::default();
        
        // More restrictive patterns for security
        config.exclude_patterns.extend(vec![
            "*secret*".to_string(),
            "*password*".to_string(),
            "*key*".to_string(),
            "*token*".to_string(),
        ]);
        
        Self::new(config)
    }
    
    /// Discover all tools in the configured directories
    pub async fn discover_all_tools(&self) -> Result<Vec<DiscoveredTool>> {
        let mut discovered_tools = Vec::new();
        
        for search_dir in &self.config.search_directories {
            if !search_dir.exists() {
                debug!("Search directory does not exist: {}", search_dir.display());
                continue;
            }
            
            info!("Searching for tools in: {}", search_dir.display());
            let tools = self.discover_tools_in_directory(search_dir).await?;
            discovered_tools.extend(tools);
        }
        
        info!("Discovered {} tools total", discovered_tools.len());
        Ok(discovered_tools)
    }
    
    /// Discover tools in a specific directory
    pub async fn discover_tools_in_directory(&self, directory: &Path) -> Result<Vec<DiscoveredTool>> {
        let mut tools = Vec::new();
        
        let mut stack = vec![(directory.to_path_buf(), 0)];
        
        while let Some((current_dir, depth)) = stack.pop() {
            if depth >= self.config.max_depth {
                continue;
            }
            
            let mut entries = fs::read_dir(&current_dir).await
                .with_context(|| format!("Failed to read directory: {}", current_dir.display()))?;
            
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                
                if path.is_dir() {
                    if self.config.follow_symlinks || !self.is_symlink(&path).await? {
                        stack.push((path, depth + 1));
                    }
                    continue;
                }
                
                // Check if file matches include patterns
                if !self.matches_include_patterns(&path) {
                    continue;
                }
                
                // Check if file matches exclude patterns
                if self.matches_exclude_patterns(&path) {
                    continue;
                }
                
                // Discover tool
                if let Some(tool) = self.discover_tool(&path).await? {
                    tools.push(tool);
                }
            }
        }
        
        Ok(tools)
    }
    
    /// Discover a specific tool from a file path
    pub async fn discover_tool(&self, path: &Path) -> Result<Option<DiscoveredTool>> {
        if !path.is_file() {
            return Ok(None);
        }
        
        // Determine tool type
        let tool_type = self.determine_tool_type(path)?;
        
        // Extract metadata
        let (name, description) = self.extract_tool_metadata(path).await?;
        
        // Infer capabilities
        let capabilities = self.capability_inferrer.infer_capabilities(path, &tool_type).await?;
        
        Ok(Some(DiscoveredTool {
            name,
            path: path.to_path_buf(),
            description,
            capabilities,
            tool_type,
        }))
    }
    
    /// Determine tool type from file extension and content
    fn determine_tool_type(&self, path: &Path) -> Result<ToolType> {
        if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("py") => return Ok(ToolType::Python),
                Some("sh") | Some("bash") | Some("zsh") | Some("fish") => return Ok(ToolType::Shell),
                _ => {}
            }
        }
        
        // Default to external tool
        Ok(ToolType::External)
    }
    
    /// Extract tool metadata (name and description) from the file
    async fn extract_tool_metadata(&self, path: &Path) -> Result<(String, String)> {
        let name = self.generate_tool_name(path);
        let description = self.extract_description(path).await?;
        
        Ok((name, description))
    }
    
    /// Generate a tool name from the file path
    fn generate_tool_name(&self, path: &Path) -> String {
        if let Some(stem) = path.file_stem() {
            stem.to_string_lossy()
                .replace(['_', ' '], "-")
                .to_lowercase()
        } else {
            "unknown-tool".to_string()
        }
    }
    
    /// Extract description from file comments or docstrings
    async fn extract_description(&self, path: &Path) -> Result<String> {
        let content = fs::read_to_string(path).await
            .with_context(|| format!("Failed to read file: {}", path.display()))?;
        
        // Extract description based on file type
        if path.extension().and_then(|s| s.to_str()) == Some("py") {
            self.extract_python_description(&content)
        } else {
            self.extract_shell_description(&content)
        }
    }
    
    /// Extract description from Python docstring or comments
    fn extract_python_description(&self, content: &str) -> Result<String> {
        // Look for module docstring - simplified approach for the test case
        if let Some(start) = content.find("\"\"\"") {
            if let Some(end) = content[start+3..].find("\"\"\"") {
                let docstring = &content[start+3..start+3+end];
                // Return first non-empty line of docstring
                for line in docstring.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        return Ok(line.to_string());
                    }
                }
            }
        }
        
        // Try single quotes
        if let Some(start) = content.find("'''") {
            if let Some(end) = content[start+3..].find("'''") {
                let docstring = &content[start+3..start+3+end];
                // Return first non-empty line of docstring
                for line in docstring.lines() {
                    let line = line.trim();
                    if !line.is_empty() {
                        return Ok(line.to_string());
                    }
                }
            }
        }
        
        // Look for comment-based description
        for line in content.lines().take(10) {
            let line = line.trim();
            if line.starts_with('#') && !line.starts_with("#!") {
                let description = line.trim_start_matches('#').trim();
                if !description.is_empty() {
                    return Ok(description.to_string());
                }
            }
        }
        
        Ok("Python tool".to_string())
    }
    
    /// Extract description from shell script comments
    fn extract_shell_description(&self, content: &str) -> Result<String> {
        for line in content.lines().take(10) {
            let line = line.trim();
            if line.starts_with('#') && !line.starts_with("#!") {
                let description = line.trim_start_matches('#').trim();
                if !description.is_empty() {
                    return Ok(description.to_string());
                }
            }
        }
        
        Ok("Shell tool".to_string())
    }
    
    /// Check if path matches include patterns
    fn matches_include_patterns(&self, path: &Path) -> bool {
        if self.config.include_patterns.is_empty() {
            return true;
        }
        
        let path_str = path.to_string_lossy();
        self.config.include_patterns.iter().any(|pattern| {
            self.matches_glob_pattern(&path_str, pattern)
        })
    }
    
    /// Check if path matches exclude patterns
    fn matches_exclude_patterns(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        self.config.exclude_patterns.iter().any(|pattern| {
            self.matches_glob_pattern(&path_str, pattern)
        })
    }
    
    /// Simple glob pattern matching
    fn matches_glob_pattern(&self, text: &str, pattern: &str) -> bool {
        // Convert glob pattern to regex
        let regex_pattern = pattern
            .replace("*", ".*")
            .replace("?", ".");
        
        if let Ok(regex) = Regex::new(&format!("^{}$", regex_pattern)) {
            regex.is_match(text)
        } else {
            false
        }
    }
    
    /// Check if path is a symbolic link
    async fn is_symlink(&self, path: &Path) -> Result<bool> {
        let metadata = fs::symlink_metadata(path).await?;
        Ok(metadata.file_type().is_symlink())
    }
}

/// Builder for ToolDiscovery
pub struct ToolDiscoveryBuilder {
    config: DiscoveryConfig,
}

impl ToolDiscoveryBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: DiscoveryConfig::default(),
        }
    }
    
    /// Add search directory
    pub fn add_search_directory<P: Into<PathBuf>>(mut self, dir: P) -> Self {
        self.config.search_directories.push(dir.into());
        self
    }
    
    /// Add include pattern
    pub fn add_include_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.config.include_patterns.push(pattern.into());
        self
    }
    
    /// Add exclude pattern
    pub fn add_exclude_pattern<S: Into<String>>(mut self, pattern: S) -> Self {
        self.config.exclude_patterns.push(pattern.into());
        self
    }
    
    /// Set maximum depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = depth;
        self
    }
    
    /// Build the discovery system
    pub fn build(self) -> ToolDiscovery {
        ToolDiscovery::new(self.config)
    }
}

impl Default for ToolDiscoveryBuilder {
    fn default() -> Self { Self::new() }
}

/// Capability inference rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInferenceRules {
    /// Name-based rules
    pub name_rules: HashMap<String, Vec<String>>,
    /// Path-based rules
    pub path_rules: HashMap<String, Vec<String>>,
    /// Content-based rules
    pub content_rules: HashMap<String, Vec<String>>,
}

impl Default for CapabilityInferenceRules {
    fn default() -> Self {
        let mut name_rules = HashMap::new();
        name_rules.insert("*analysis*".to_string(), vec!["code-analysis".to_string()]);
        name_rules.insert("*monitor*".to_string(), vec!["system-monitoring".to_string()]);
        name_rules.insert("*test*".to_string(), vec!["testing".to_string()]);
        name_rules.insert("*build*".to_string(), vec!["build-system".to_string()]);
        name_rules.insert("*validate*".to_string(), vec!["validation".to_string()]);
        name_rules.insert("*visualize*".to_string(), vec!["visualization".to_string()]);
        
        let mut path_rules = HashMap::new();
        path_rules.insert("scripts/*".to_string(), vec!["utility".to_string()]);
        path_rules.insert("tools/*".to_string(), vec!["development".to_string()]);
        path_rules.insert("bin/*".to_string(), vec!["executable".to_string()]);
        
        let mut content_rules = HashMap::new();
        content_rules.insert("import ast".to_string(), vec!["code-analysis".to_string()]);
        content_rules.insert("import subprocess".to_string(), vec!["process-spawn".to_string()]);
        content_rules.insert("import requests".to_string(), vec!["network-access".to_string()]);
        content_rules.insert("import os".to_string(), vec!["filesystem-access".to_string()]);
        
        Self {
            name_rules,
            path_rules,
            content_rules,
        }
    }
}

/// Capability inference system
pub struct CapabilityInferrer {
    rules: CapabilityInferenceRules,
}

impl CapabilityInferrer {
    /// Create a new capability inferrer
    pub fn new(rules: CapabilityInferenceRules) -> Self {
        Self { rules }
    }
    
    /// Infer capabilities for a tool
    pub async fn infer_capabilities(&self, path: &Path, tool_type: &ToolType) -> Result<Vec<String>> {
        let mut capabilities = Vec::new();
        
        // Base capabilities by tool type
        match tool_type {
            ToolType::Python => {
                capabilities.push("filesystem-read".to_string());
            }
            ToolType::Shell => {
                capabilities.push("filesystem-read".to_string());
                capabilities.push("process-spawn".to_string());
            }
            ToolType::External => {
                capabilities.push("filesystem-read".to_string());
            }
        }
        
        // Name-based inference
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        for (pattern, caps) in &self.rules.name_rules {
            if self.matches_pattern(&name, pattern) {
                capabilities.extend(caps.clone());
            }
        }
        
        // Path-based inference
        let path_str = path.to_string_lossy().to_lowercase();
        for (pattern, caps) in &self.rules.path_rules {
            if self.matches_pattern(&path_str, pattern) {
                capabilities.extend(caps.clone());
            }
        }
        
        // Content-based inference
        if let Ok(content) = fs::read_to_string(path).await {
            let content_lower = content.to_lowercase();
            for (pattern, caps) in &self.rules.content_rules {
                if content_lower.contains(&pattern.to_lowercase()) {
                    capabilities.extend(caps.clone());
                }
            }
        }
        
        // Remove duplicates and sort
        capabilities.sort();
        capabilities.dedup();
        
        Ok(capabilities)
    }
    
    /// Simple pattern matching with wildcards
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            let regex_pattern = pattern.replace('*', ".*");
            if let Ok(regex) = Regex::new(&regex_pattern) {
                return regex.is_match(text);
            }
        }
        
        text.contains(pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use tokio::fs;
    
    #[tokio::test]
    async fn test_tool_discovery() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let scripts_dir = temp_dir.path().join("scripts");
        fs::create_dir(&scripts_dir).await?;
        
        // Create test Python script
        let python_script = scripts_dir.join("test_analyzer.py");
        fs::write(&python_script, r#"#!/usr/bin/env python3
"""Control flow analysis tool"""
import ast
import sys

def analyze_code():
    print("Analyzing code...")

if __name__ == "__main__":
    analyze_code()
"#).await?;
        
        // Create test shell script
        let shell_script = scripts_dir.join("test_monitor.sh");
        fs::write(&shell_script, r#"#!/bin/bash
# System monitoring script
ps aux | grep python
"#).await?;
        
        // Configure discovery
        let config = DiscoveryConfig {
            search_directories: vec![scripts_dir],
            include_patterns: vec!["*.py".to_string(), "*.sh".to_string()],
            exclude_patterns: vec![],
            follow_symlinks: false,
            max_depth: 2,
            capability_rules: CapabilityInferenceRules::default(),
        };
        
        let discovery = ToolDiscovery::new(config);
        let tools = discovery.discover_all_tools().await?;
        
        assert_eq!(tools.len(), 2);
        
        // Check Python tool
        let python_tool = tools.iter().find(|t| t.name == "test-analyzer").unwrap();
        assert_eq!(python_tool.tool_type, ToolType::Python);
        assert!(python_tool.capabilities.contains(&"code-analysis".to_string()));
        assert_eq!(python_tool.description, "Control flow analysis tool");
        
        // Check shell tool
        let shell_tool = tools.iter().find(|t| t.name == "test-monitor").unwrap();
        assert_eq!(shell_tool.tool_type, ToolType::Shell);
        assert!(shell_tool.capabilities.contains(&"system-monitoring".to_string()));
        assert_eq!(shell_tool.description, "System monitoring script");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_capability_inference() -> Result<()> {
        let rules = CapabilityInferenceRules::default();
        let inferrer = CapabilityInferrer::new(rules);
        
        let temp_dir = TempDir::new()?;
        let script_path = temp_dir.path().join("network_tool.py");
        fs::write(&script_path, "import requests\nimport subprocess").await?;
        
        let capabilities = inferrer.infer_capabilities(&script_path, &ToolType::Python).await?;
        
        assert!(capabilities.contains(&"filesystem-read".to_string()));
        assert!(capabilities.contains(&"network-access".to_string()));
        assert!(capabilities.contains(&"process-spawn".to_string()));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_discovery_builder() -> Result<()> {
        let discovery = ToolDiscoveryBuilder::new()
            .add_search_directory("scripts")
            .add_include_pattern("*.py")
            .add_exclude_pattern("*test*")
            .max_depth(2)
            .build();
        
        assert_eq!(discovery.config.max_depth, 2);
        assert!(discovery.config.include_patterns.contains(&"*.py".to_string()));
        assert!(discovery.config.exclude_patterns.contains(&"*test*".to_string()));
        
        Ok(())
    }
}
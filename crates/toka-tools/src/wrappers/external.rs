//! External tool wrapper for integrating system executables with the toka-tools registry.
//!
//! This module provides the `ExternalTool` struct that wraps external executables
//! (Python scripts, shell scripts, binaries) and makes them available through
//! the standard `Tool` trait interface.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::{timeout, Duration};
use tracing::{debug, error, info, warn};

use crate::core::{Tool, ToolMetadata, ToolParams, ToolResult};
use crate::manifest::{ToolManifest, SideEffect};

/// Security configuration for external tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Maximum execution time in seconds
    pub max_execution_time: u64,
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Allowed file system access paths
    pub allowed_paths: Vec<PathBuf>,
    /// Whether network access is allowed
    pub allow_network: bool,
    /// Environment variables to pass through
    pub env_whitelist: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            max_execution_time: 300, // 5 minutes
            max_memory_mb: 512,       // 512MB
            allowed_paths: vec![PathBuf::from(".")], // Current directory only
            allow_network: false,
            env_whitelist: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
                "LANG".to_string(),
            ],
        }
    }
}

/// External tool wrapper that integrates system executables with the tool registry
#[derive(Debug, Clone)]
pub struct ExternalTool {
    /// Tool manifest describing capabilities and interface
    manifest: ToolManifest,
    /// Path to the executable
    executable: PathBuf,
    /// Security configuration
    security_config: SecurityConfig,
    /// Working directory for execution
    working_directory: Option<PathBuf>,
}

impl ExternalTool {
    /// Create a new external tool from a manifest file
    pub fn from_manifest(manifest_path: &std::path::Path) -> Result<Self> {
        let manifest_content = std::fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
        
        let manifest: ToolManifest = serde_json::from_str(&manifest_content)
            .with_context(|| format!("Failed to parse manifest: {}", manifest_path.display()))?;
        
        // Validate manifest
        manifest.validate()?;
        
        // Extract executable path from transport
        let executable = Self::extract_executable_path(&manifest)?;
        
        Ok(Self {
            manifest,
            executable,
            security_config: SecurityConfig::default(),
            working_directory: None,
        })
    }
    
    /// Create a new external tool with custom configuration
    pub fn new(
        manifest: ToolManifest,
        executable: PathBuf,
        security_config: SecurityConfig,
    ) -> Result<Self> {
        manifest.validate()?;
        
        Ok(Self {
            manifest,
            executable,
            security_config,
            working_directory: None,
        })
    }
    
    /// Set the working directory for execution
    pub fn with_working_directory(mut self, working_dir: PathBuf) -> Self {
        self.working_directory = Some(working_dir);
        self
    }
    
    /// Wrap a Python script as an external tool
    pub fn wrap_python_script(
        script_path: &std::path::Path,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<Self> {
        let manifest = Self::create_python_manifest(script_path, name, description, capabilities)?;
        
        Ok(Self {
            manifest,
            executable: PathBuf::from("python3"),
            security_config: SecurityConfig::default(),
            working_directory: None,
        })
    }
    
    /// Wrap a shell script as an external tool
    pub fn wrap_shell_script(
        script_path: &std::path::Path,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<Self> {
        let manifest = Self::create_shell_manifest(script_path, name, description, capabilities)?;
        
        Ok(Self {
            manifest,
            executable: script_path.to_path_buf(),
            security_config: SecurityConfig::default(),
            working_directory: None,
        })
    }
    
    /// Extract executable path from manifest transports
    fn extract_executable_path(manifest: &ToolManifest) -> Result<PathBuf> {
        use crate::manifest::Transport;
        
        for transport in &manifest.transports {
            match transport {
                Transport::JsonRpcStdio { exec } => {
                    return Ok(PathBuf::from(exec));
                }
                Transport::InProcess => {
                    return Err(anyhow::anyhow!("InProcess transport not supported for external tools"));
                }
                Transport::JsonRpcHttp { .. } => {
                    return Err(anyhow::anyhow!("HTTP transport not supported for external tools"));
                }
                Transport::Wasm { path } => {
                    return Ok(PathBuf::from(path));
                }
            }
        }
        
        Err(anyhow::anyhow!("No supported transport found in manifest"))
    }
    
    /// Create a manifest for a Python script
    fn create_python_manifest(
        script_path: &std::path::Path,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<ToolManifest> {
        use crate::manifest::{Transport, Schema};
        
        let manifest = ToolManifest {
            id: name.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: description.to_string(),
            capability: capabilities.first().cloned().unwrap_or_else(|| "general".to_string()),
            side_effect: SideEffect::External,
            input_schema: None,
            output_schema: None,
            transports: vec![Transport::JsonRpcStdio {
                exec: format!("python3 {}", script_path.display()),
            }],
            action_id: None,
            manifest_version: "1.1".to_string(),
            protocols: vec![],
            metadata: std::collections::BTreeMap::new(),
        };
        
        Ok(manifest)
    }
    
    /// Create a manifest for a shell script
    fn create_shell_manifest(
        script_path: &std::path::Path,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<ToolManifest> {
        use crate::manifest::{Transport, Schema};
        
        let manifest = ToolManifest {
            id: name.to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            description: description.to_string(),
            capability: capabilities.first().cloned().unwrap_or_else(|| "general".to_string()),
            side_effect: SideEffect::External,
            input_schema: None,
            output_schema: None,
            transports: vec![Transport::JsonRpcStdio {
                exec: script_path.display().to_string(),
            }],
            action_id: None,
            manifest_version: "1.1".to_string(),
            protocols: vec![],
            metadata: std::collections::BTreeMap::new(),
        };
        
        Ok(manifest)
    }
    
    /// Execute the external tool with security constraints
    async fn execute_external(&self, params: &ToolParams) -> Result<ToolResult> {
        let start_time = std::time::Instant::now();
        
        // Prepare command
        let mut cmd = self.prepare_command(params)?;
        
        // Execute with timeout
        let timeout_duration = Duration::from_secs(self.security_config.max_execution_time);
        let execution_result = timeout(timeout_duration, cmd.output()).await;
        
        let output = match execution_result {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                error!("External tool execution failed: {}", e);
                return Ok(ToolResult {
                    success: false,
                    output: format!("Execution failed: {}", e),
                    metadata: self.create_metadata(start_time),
                });
            }
            Err(_) => {
                error!("External tool execution timed out after {} seconds", self.security_config.max_execution_time);
                return Ok(ToolResult {
                    success: false,
                    output: "Execution timed out".to_string(),
                    metadata: self.create_metadata(start_time),
                });
            }
        };
        
        // Process output
        let success = output.status.success();
        let output_text = if success {
            String::from_utf8_lossy(&output.stdout).to_string()
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            format!("Exit code: {}\nStderr: {}\nStdout: {}", 
                    output.status.code().unwrap_or(-1), stderr, stdout)
        };
        
        debug!("External tool {} executed in {:?}", self.name(), start_time.elapsed());
        
        Ok(ToolResult {
            success,
            output: output_text,
            metadata: self.create_metadata(start_time),
        })
    }
    
    /// Prepare the command for execution
    fn prepare_command(&self, params: &ToolParams) -> Result<Command> {
        let mut cmd = Command::new(&self.executable);
        
        // Set working directory
        if let Some(working_dir) = &self.working_directory {
            cmd.current_dir(working_dir);
        }
        
        // Add arguments from parameters
        for (key, value) in &params.args {
            // Convert key-value pairs to command line arguments
            // This is a simple implementation - more sophisticated parsing could be added
            if key.starts_with("--") {
                cmd.arg(key);
                if !value.is_empty() {
                    cmd.arg(value);
                }
            } else {
                cmd.arg(format!("--{}", key));
                if !value.is_empty() {
                    cmd.arg(value);
                }
            }
        }
        
        // Configure environment
        cmd.env_clear();
        for env_var in &self.security_config.env_whitelist {
            if let Ok(value) = std::env::var(env_var) {
                cmd.env(env_var, value);
            }
        }
        
        // Set stdio
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());
        
        Ok(cmd)
    }
    
    /// Create tool metadata
    fn create_metadata(&self, start_time: std::time::Instant) -> ToolMetadata {
        ToolMetadata {
            execution_time_ms: start_time.elapsed().as_millis() as u64,
            tool_version: self.manifest.version.clone(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
    
    /// Get the security configuration
    pub fn security_config(&self) -> &SecurityConfig {
        &self.security_config
    }
    
    /// Get the tool manifest
    pub fn manifest(&self) -> &ToolManifest {
        &self.manifest
    }
}

#[async_trait]
impl Tool for ExternalTool {
    fn name(&self) -> &str {
        &self.manifest.name
    }
    
    fn description(&self) -> &str {
        &self.manifest.description
    }
    
    fn version(&self) -> &str {
        &self.manifest.version
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        info!("Executing external tool: {} with params: {:?}", self.name(), params.args);
        
        // Validate parameters against manifest schema if available
        if let Some(input_schema) = &self.manifest.input_schema {
            debug!("Input schema validation not yet implemented for external tools");
            // TODO: Implement JSON schema validation
        }
        
        // Execute the external tool
        let result = self.execute_external(params).await?;
        
        info!("External tool {} completed: success={}, output_len={}", 
              self.name(), result.success, result.output.len());
        
        Ok(result)
    }
    
    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        // Basic validation - ensure executable exists
        if !self.executable.exists() {
            return Err(anyhow::anyhow!("Executable not found: {}", self.executable.display()));
        }
        
        // Validate required parameters based on manifest
        // This is a simplified implementation - could be enhanced with JSON schema validation
        debug!("Parameter validation for external tool: {}", self.name());
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_external_tool_python_script() -> Result<()> {
        // Create a temporary Python script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "import sys")?;
        writeln!(script_file, "print('Hello from Python!')")?;
        writeln!(script_file, "sys.exit(0)")?;
        
        let script_path = script_file.path();
        
        // Create external tool wrapper
        let tool = ExternalTool::wrap_python_script(
            script_path,
            "test-python-tool",
            "Test Python tool",
            vec!["testing".to_string()],
        )?;
        
        // Execute the tool
        let params = ToolParams {
            name: "test-python-tool".to_string(),
            args: HashMap::new(),
        };
        
        let result = tool.execute(&params).await?;
        
        assert!(result.success);
        assert!(result.output.contains("Hello from Python!"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_external_tool_shell_script() -> Result<()> {
        // Create a temporary shell script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/bin/bash")?;
        writeln!(script_file, "echo 'Hello from Shell!'")?;
        writeln!(script_file, "exit 0")?;
        
        let script_path = script_file.path();
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(script_path, perms)?;
        }
        
        // Create external tool wrapper
        let tool = ExternalTool::wrap_shell_script(
            script_path,
            "test-shell-tool",
            "Test Shell tool",
            vec!["testing".to_string()],
        )?;
        
        // Execute the tool
        let params = ToolParams {
            name: "test-shell-tool".to_string(),
            args: HashMap::new(),
        };
        
        let result = tool.execute(&params).await?;
        
        assert!(result.success);
        assert!(result.output.contains("Hello from Shell!"));
        
        Ok(())
    }
}
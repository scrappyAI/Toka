//! Python script wrapper with security controls
//!
//! This module provides a secure wrapper for Python scripts that can be executed
//! with controlled resource limits and security constraints.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::core::{Tool, ToolParams, ToolResult, ToolMetadata};
use super::security::{SecurityLevel, ResourceLimits, SandboxConfig};

/// Python tool wrapper with security controls
#[derive(Debug, Clone)]
pub struct PythonTool {
    /// Tool name
    name: String,
    /// Python script path
    script_path: PathBuf,
    /// Tool description
    description: String,
    /// Tool capabilities
    capabilities: Vec<String>,
    /// Python configuration
    python_config: PythonConfig,
    /// Security configuration
    security_config: PythonSecurityConfig,
    /// Resource limits
    resource_limits: ResourceLimits,
}

/// Python tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Python interpreter path
    pub interpreter_path: PathBuf,
    /// Python version requirement
    pub min_version: Option<String>,
    /// Required Python packages
    pub required_packages: Vec<String>,
    /// Python path additions
    pub python_path_additions: Vec<PathBuf>,
    /// Virtual environment path
    pub venv_path: Option<PathBuf>,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            interpreter_path: PathBuf::from("python3"),
            min_version: None,
            required_packages: vec![],
            python_path_additions: vec![],
            venv_path: None,
        }
    }
}

/// Security configuration for Python tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonSecurityConfig {
    /// Security level
    pub security_level: SecurityLevel,
    /// Allowed import modules
    pub allowed_imports: Vec<String>,
    /// Blocked import modules
    pub blocked_imports: Vec<String>,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Working directory restrictions
    pub allowed_working_dirs: Vec<PathBuf>,
    /// File system access restrictions
    pub allowed_file_paths: Vec<PathBuf>,
    /// Network access restrictions
    pub allow_network_access: bool,
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
}

impl Default for PythonSecurityConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Basic,
            allowed_imports: vec![
                "os".to_string(),
                "sys".to_string(),
                "json".to_string(),
                "re".to_string(),
                "datetime".to_string(),
                "math".to_string(),
                "random".to_string(),
            ],
            blocked_imports: vec![
                "subprocess".to_string(),
                "socket".to_string(),
                "urllib".to_string(),
                "requests".to_string(),
                "ctypes".to_string(),
            ],
            max_execution_time: Duration::from_secs(30),
            allowed_working_dirs: vec![],
            allowed_file_paths: vec![],
            allow_network_access: false,
            sandbox_config: SandboxConfig::default(),
        }
    }
}

impl PythonTool {
    /// Create a new Python tool
    pub fn new(
        script_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            script_path,
            description: description.to_string(),
            capabilities,
            python_config: PythonConfig::default(),
            security_config: PythonSecurityConfig::default(),
            resource_limits: ResourceLimits::default(),
        })
    }

    /// Create a new Python tool with security configuration
    pub async fn new_with_security(
        script_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
        security_level: SecurityLevel,
    ) -> Result<Self> {
        let security_config = PythonSecurityConfig {
            security_level,
            sandbox_config: security_level.default_sandbox_config(),
            ..Default::default()
        };

        Ok(Self {
            name: name.to_string(),
            script_path,
            description: description.to_string(),
            capabilities,
            python_config: PythonConfig::default(),
            security_config: security_config.clone(),
            resource_limits: security_level.default_resource_limits(),
        })
    }

    /// Validate Python script against security policies
    async fn validate_script(&self) -> Result<()> {
        // Read script content
        let script_content = tokio::fs::read_to_string(&self.script_path)
            .await
            .context("Failed to read Python script")?;

        // Check for blocked imports
        for blocked_import in &self.security_config.blocked_imports {
            if script_content.contains(&format!("import {}", blocked_import)) ||
               script_content.contains(&format!("from {}", blocked_import)) {
                return Err(anyhow::anyhow!(
                    "Script contains blocked import: {}", 
                    blocked_import
                ));
            }
        }

        // Check for dangerous operations
        let dangerous_operations = vec![
            "exec(",
            "eval(",
            "compile(",
            "__import__(",
            "open(",
            "file(",
        ];

        for dangerous_op in dangerous_operations {
            if script_content.contains(dangerous_op) {
                warn!("Script contains potentially dangerous operation: {}", dangerous_op);
                if self.security_config.security_level == SecurityLevel::High {
                    return Err(anyhow::anyhow!(
                        "Script contains dangerous operation: {}", 
                        dangerous_op
                    ));
                }
            }
        }

        Ok(())
    }

    /// Execute the Python script with security controls
    async fn execute_with_security(&self, args: Vec<String>) -> Result<ToolResult> {
        debug!("Executing Python tool: {} with args: {:?}", self.name, args);

        // Validate script
        self.validate_script().await?;

        // Create command
        let mut command = Command::new(&self.python_config.interpreter_path);
        command.arg(&self.script_path);
        command.args(&args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Set environment variables
        if let Some(venv_path) = &self.python_config.venv_path {
            command.env("VIRTUAL_ENV", venv_path);
            command.env("PATH", format!("{}/bin:{}", 
                venv_path.display(), 
                std::env::var("PATH").unwrap_or_default()
            ));
        }

        // Add Python path additions
        if !self.python_config.python_path_additions.is_empty() {
            let python_path = self.python_config.python_path_additions
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(":");
            command.env("PYTHONPATH", python_path);
        }

        // Apply resource limits
        // Note: Resource limits would be applied here in a full implementation
        // using cgroups, rlimits, etc. For now, we skip this to avoid unsafe code.

        // Execute with timeout
        let execution_future = command.output();
        let output = match timeout(
            self.security_config.max_execution_time,
            execution_future
        ).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                error!("Failed to execute Python tool {}: {}", self.name, e);
                return Err(e.into());
            }
            Err(_) => {
                error!("Python tool {} timed out", self.name);
                return Err(anyhow::anyhow!("Tool execution timed out"));
            }
        };

        // Process results
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        
        if success {
            info!("Python tool {} executed successfully", self.name);
        } else {
            warn!("Python tool {} failed with exit code: {:?}", 
                  self.name, output.status.code());
        }

        let output = if stderr.is_empty() {
            stdout
        } else {
            format!("{}\nSTDERR: {}", stdout, stderr)
        };

        Ok(ToolResult {
            success,
            output,
            metadata: ToolMetadata {
                execution_time_ms: 0, // Will be set by the registry
                tool_version: "1.0.0".to_string(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            },
        })
    }
}

#[async_trait]
impl Tool for PythonTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn validate_params(&self, _params: &ToolParams) -> Result<()> {
        // Basic validation - can be extended as needed
        Ok(())
    }

    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        let args = params.args.get("args")
            .map(|s| s.split_whitespace().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        self.execute_with_security(args).await
    }
}

/// Builder for Python tools
pub struct PythonToolBuilder {
    name: Option<String>,
    script_path: Option<PathBuf>,
    description: Option<String>,
    capabilities: Vec<String>,
    python_config: PythonConfig,
    security_config: PythonSecurityConfig,
}

impl PythonToolBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name: None,
            script_path: None,
            description: None,
            capabilities: Vec::new(),
            python_config: PythonConfig::default(),
            security_config: PythonSecurityConfig::default(),
        }
    }

    /// Set the tool name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the script path
    pub fn script_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.script_path = Some(path.into());
        self
    }

    /// Set the description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a capability
    pub fn capability(mut self, capability: impl Into<String>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Set Python interpreter path
    pub fn interpreter_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.python_config.interpreter_path = path.into();
        self
    }

    /// Set virtual environment path
    pub fn venv_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.python_config.venv_path = Some(path.into());
        self
    }

    /// Set security level
    pub fn security_level(mut self, level: SecurityLevel) -> Self {
        self.security_config.security_level = level;
        self
    }

    /// Add allowed import
    pub fn allow_import(mut self, import: impl Into<String>) -> Self {
        self.security_config.allowed_imports.push(import.into());
        self
    }

    /// Add blocked import
    pub fn block_import(mut self, import: impl Into<String>) -> Self {
        self.security_config.blocked_imports.push(import.into());
        self
    }

    /// Build the Python tool
    pub fn build(self) -> Result<PythonTool> {
        let name = self.name.context("Tool name is required")?;
        let script_path = self.script_path.context("Script path is required")?;
        let description = self.description.unwrap_or_else(|| format!("Python tool: {}", name));

        let mut tool = PythonTool::new(script_path, &name, &description, self.capabilities)?;
        tool.python_config = self.python_config;
        tool.security_config = self.security_config;
        
        Ok(tool)
    }
}

impl Default for PythonToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[tokio::test]
    async fn test_python_tool_creation() -> Result<()> {
        let tool = PythonTool::new(
            PathBuf::from("test_script.py"),
            "test-script",
            "Test Python script",
            vec!["data-processing".to_string()],
        )?;

        assert_eq!(tool.name(), "test-script");
        assert_eq!(tool.description(), "Test Python script");
        assert_eq!(tool.capabilities(), &["data-processing"]);

        Ok(())
    }

    #[tokio::test]
    async fn test_python_tool_builder() -> Result<()> {
        let tool = PythonToolBuilder::new()
            .name("test-tool")
            .script_path("test_script.py")
            .description("Test tool")
            .capability("testing")
            .interpreter_path("python3")
            .security_level(SecurityLevel::High)
            .allow_import("numpy")
            .block_import("subprocess")
            .build()?;

        assert_eq!(tool.name(), "test-tool");
        assert_eq!(tool.description(), "Test tool");
        assert_eq!(tool.capabilities(), &["testing"]);

        Ok(())
    }

    #[tokio::test]
    async fn test_script_validation() -> Result<()> {
        // Create a test script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "print('Hello, World!')")?;
        script_file.flush()?;

        let tool = PythonTool::new(
            script_file.path().to_path_buf(),
            "test-script",
            "Test script",
            vec!["testing".to_string()],
        )?;

        // Should pass validation
        assert!(tool.validate_script().await.is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_blocked_import_validation() -> Result<()> {
        // Create a script with blocked import
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "import subprocess")?;
        writeln!(script_file, "print('Hello, World!')")?;
        script_file.flush()?;

        let tool = PythonTool::new(
            script_file.path().to_path_buf(),
            "test-script",
            "Test script",
            vec!["testing".to_string()],
        )?;

        // Should fail validation
        assert!(tool.validate_script().await.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn test_python_tool_execution() -> Result<()> {
        // Create a simple Python script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "import sys")?;
        writeln!(script_file, "print('Hello from Python!')")?;
        writeln!(script_file, "if len(sys.argv) > 1:")?;
        writeln!(script_file, "    print('Args:', ' '.join(sys.argv[1:]))")?;
        script_file.flush()?;

        let tool = PythonTool::new(
            script_file.path().to_path_buf(),
            "test-script",
            "Test script",
            vec!["testing".to_string()],
        )?;

        let params = json!({
            "args": ["arg1", "arg2"]
        });

        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("Hello from Python!"));
        assert!(result.output.contains("Args: arg1 arg2"));

        Ok(())
    }
}
//! Shell script wrapper with security controls
//!
//! This module provides a secure wrapper for shell scripts that can be executed
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

/// Shell tool wrapper with security controls
#[derive(Debug, Clone)]
pub struct ShellTool {
    /// Tool name
    name: String,
    /// Shell script path
    script_path: PathBuf,
    /// Tool description
    description: String,
    /// Tool capabilities
    capabilities: Vec<String>,
    /// Shell configuration
    shell_config: ShellConfig,
    /// Security configuration
    security_config: ShellSecurityConfig,
    /// Resource limits
    resource_limits: ResourceLimits,
}

/// Shell tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Shell interpreter path
    pub interpreter_path: PathBuf,
    /// Shell options
    pub shell_options: Vec<String>,
    /// Environment variables
    pub environment_vars: HashMap<String, String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            interpreter_path: PathBuf::from("/bin/bash"),
            shell_options: vec!["-e".to_string()], // Exit on error
            environment_vars: HashMap::new(),
            working_directory: None,
        }
    }
}

/// Security configuration for shell tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellSecurityConfig {
    /// Security level
    pub security_level: SecurityLevel,
    /// Allowed commands
    pub allowed_commands: Vec<String>,
    /// Blocked commands
    pub blocked_commands: Vec<String>,
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

impl Default for ShellSecurityConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Basic,
            allowed_commands: vec![
                "echo".to_string(),
                "cat".to_string(),
                "grep".to_string(),
                "sed".to_string(),
                "awk".to_string(),
                "sort".to_string(),
                "uniq".to_string(),
                "wc".to_string(),
                "head".to_string(),
                "tail".to_string(),
            ],
            blocked_commands: vec![
                "rm".to_string(),
                "sudo".to_string(),
                "su".to_string(),
                "chmod".to_string(),
                "chown".to_string(),
                "dd".to_string(),
                "mkfs".to_string(),
                "fdisk".to_string(),
                "mount".to_string(),
                "umount".to_string(),
                "kill".to_string(),
                "killall".to_string(),
                "reboot".to_string(),
                "shutdown".to_string(),
            ],
            max_execution_time: Duration::from_secs(30),
            allowed_working_dirs: vec![],
            allowed_file_paths: vec![],
            allow_network_access: false,
            sandbox_config: SandboxConfig::default(),
        }
    }
}

impl ShellTool {
    /// Create a new shell tool
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
            shell_config: ShellConfig::default(),
            security_config: ShellSecurityConfig::default(),
            resource_limits: ResourceLimits::default(),
        })
    }

    /// Create a new shell tool with security configuration
    pub async fn new_with_security(
        script_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
        security_level: SecurityLevel,
    ) -> Result<Self> {
        let security_config = ShellSecurityConfig {
            security_level,
            sandbox_config: security_level.default_sandbox_config(),
            ..Default::default()
        };

        Ok(Self {
            name: name.to_string(),
            script_path,
            description: description.to_string(),
            capabilities,
            shell_config: ShellConfig::default(),
            security_config: security_config.clone(),
            resource_limits: security_level.default_resource_limits(),
        })
    }

    /// Validate shell script against security policies
    async fn validate_script(&self) -> Result<()> {
        // Read script content
        let script_content = tokio::fs::read_to_string(&self.script_path)
            .await
            .context("Failed to read shell script")?;

        // Check for blocked commands
        for blocked_command in &self.security_config.blocked_commands {
            if script_content.contains(blocked_command) {
                return Err(anyhow::anyhow!(
                    "Script contains blocked command: {}", 
                    blocked_command
                ));
            }
        }

        // Check for dangerous patterns
        let dangerous_patterns = vec![
            "rm -rf",
            ":(){ :|:& };:",  // Fork bomb
            "chmod +x",
            "curl | sh",
            "wget | sh",
            "eval",
            "exec",
            "> /dev/",
            "< /dev/",
        ];

        for dangerous_pattern in dangerous_patterns {
            if script_content.contains(dangerous_pattern) {
                warn!("Script contains potentially dangerous pattern: {}", dangerous_pattern);
                if self.security_config.security_level == SecurityLevel::High {
                    return Err(anyhow::anyhow!(
                        "Script contains dangerous pattern: {}", 
                        dangerous_pattern
                    ));
                }
            }
        }

        // Check for allowed commands if specified
        if !self.security_config.allowed_commands.is_empty() {
            // Extract commands from script (basic analysis)
            let lines: Vec<&str> = script_content.lines().collect();
            for line in lines {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                
                // Extract first word as command
                if let Some(command) = trimmed.split_whitespace().next() {
                    if !self.security_config.allowed_commands.contains(&command.to_string()) {
                        warn!("Script contains potentially disallowed command: {}", command);
                        if self.security_config.security_level == SecurityLevel::High {
                            return Err(anyhow::anyhow!(
                                "Script contains disallowed command: {}", 
                                command
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute the shell script with security controls
    async fn execute_with_security(&self, args: Vec<String>) -> Result<ToolResult> {
        debug!("Executing shell tool: {} with args: {:?}", self.name, args);

        // Validate script
        self.validate_script().await?;

        // Create command
        let mut command = Command::new(&self.shell_config.interpreter_path);
        
        // Add shell options
        for option in &self.shell_config.shell_options {
            command.arg(option);
        }
        
        command.arg(&self.script_path);
        command.args(&args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        // Set working directory
        if let Some(working_dir) = &self.shell_config.working_directory {
            command.current_dir(working_dir);
        }

        // Set environment variables
        for (key, value) in &self.shell_config.environment_vars {
            command.env(key, value);
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
                error!("Failed to execute shell tool {}: {}", self.name, e);
                return Err(e.into());
            }
            Err(_) => {
                error!("Shell tool {} timed out", self.name);
                return Err(anyhow::anyhow!("Tool execution timed out"));
            }
        };

        // Process results
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        
        if success {
            info!("Shell tool {} executed successfully", self.name);
        } else {
            warn!("Shell tool {} failed with exit code: {:?}", 
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
impl Tool for ShellTool {
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

/// Builder for shell tools
pub struct ShellToolBuilder {
    name: Option<String>,
    script_path: Option<PathBuf>,
    description: Option<String>,
    capabilities: Vec<String>,
    shell_config: ShellConfig,
    security_config: ShellSecurityConfig,
}

impl ShellToolBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name: None,
            script_path: None,
            description: None,
            capabilities: Vec::new(),
            shell_config: ShellConfig::default(),
            security_config: ShellSecurityConfig::default(),
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

    /// Set shell interpreter path
    pub fn interpreter_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.shell_config.interpreter_path = path.into();
        self
    }

    /// Add shell option
    pub fn shell_option(mut self, option: impl Into<String>) -> Self {
        self.shell_config.shell_options.push(option.into());
        self
    }

    /// Set working directory
    pub fn working_directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.shell_config.working_directory = Some(path.into());
        self
    }

    /// Add environment variable
    pub fn env_var(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.shell_config.environment_vars.insert(key.into(), value.into());
        self
    }

    /// Set security level
    pub fn security_level(mut self, level: SecurityLevel) -> Self {
        self.security_config.security_level = level;
        self
    }

    /// Add allowed command
    pub fn allow_command(mut self, command: impl Into<String>) -> Self {
        self.security_config.allowed_commands.push(command.into());
        self
    }

    /// Add blocked command
    pub fn block_command(mut self, command: impl Into<String>) -> Self {
        self.security_config.blocked_commands.push(command.into());
        self
    }

    /// Build the shell tool
    pub fn build(self) -> Result<ShellTool> {
        let name = self.name.context("Tool name is required")?;
        let script_path = self.script_path.context("Script path is required")?;
        let description = self.description.unwrap_or_else(|| format!("Shell tool: {}", name));

        let mut tool = ShellTool::new(script_path, &name, &description, self.capabilities)?;
        tool.shell_config = self.shell_config;
        tool.security_config = self.security_config;
        
        Ok(tool)
    }
}

impl Default for ShellToolBuilder {
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
    async fn test_shell_tool_creation() -> Result<()> {
        let tool = ShellTool::new(
            PathBuf::from("test_script.sh"),
            "test-script",
            "Test shell script",
            vec!["system-admin".to_string()],
        )?;

        assert_eq!(tool.name(), "test-script");
        assert_eq!(tool.description(), "Test shell script");
        assert_eq!(tool.capabilities(), &["system-admin"]);

        Ok(())
    }

    #[tokio::test]
    async fn test_shell_tool_builder() -> Result<()> {
        let tool = ShellToolBuilder::new()
            .name("test-tool")
            .script_path("test_script.sh")
            .description("Test tool")
            .capability("testing")
            .interpreter_path("/bin/bash")
            .shell_option("-x")
            .working_directory("/tmp")
            .env_var("TEST_VAR", "test_value")
            .security_level(SecurityLevel::High)
            .allow_command("echo")
            .block_command("rm")
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
        writeln!(script_file, "#!/bin/bash")?;
        writeln!(script_file, "echo 'Hello, World!'")?;
        script_file.flush()?;

        let tool = ShellTool::new(
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
    async fn test_blocked_command_validation() -> Result<()> {
        // Create a script with blocked command
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/bin/bash")?;
        writeln!(script_file, "rm -rf /tmp/test")?;
        script_file.flush()?;

        let tool = ShellTool::new(
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
    async fn test_shell_tool_execution() -> Result<()> {
        // Create a simple shell script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/bin/bash")?;
        writeln!(script_file, "echo 'Hello from Shell!'")?;
        writeln!(script_file, "echo 'Args:' \"$@\"")?;
        script_file.flush()?;

        let tool = ShellTool::new(
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
        assert!(result.output.contains("Hello from Shell!"));
        assert!(result.output.contains("Args: arg1 arg2"));

        Ok(())
    }
}
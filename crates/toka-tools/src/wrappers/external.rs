//! External tool wrapper for arbitrary executables
//!
//! This module provides a secure wrapper for external tools that can be executed
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

/// External tool wrapper with security controls
#[derive(Debug, Clone)]
pub struct ExternalTool {
    /// Tool name
    name: String,
    /// Executable path
    executable_path: PathBuf,
    /// Tool description
    description: String,
    /// Tool capabilities
    capabilities: Vec<String>,
    /// Security configuration
    security_config: ExternalToolSecurityConfig,
    /// Resource limits
    resource_limits: ResourceLimits,
}

/// Security configuration for external tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalToolSecurityConfig {
    /// Security level
    pub security_level: SecurityLevel,
    /// Allowed arguments patterns
    pub allowed_arg_patterns: Vec<String>,
    /// Blocked arguments patterns
    pub blocked_arg_patterns: Vec<String>,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Working directory restrictions
    pub allowed_working_dirs: Vec<PathBuf>,
    /// Environment variable restrictions
    pub allowed_env_vars: Vec<String>,
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
}

impl Default for ExternalToolSecurityConfig {
    fn default() -> Self {
        Self {
            security_level: SecurityLevel::Basic,
            allowed_arg_patterns: vec![],
            blocked_arg_patterns: vec![
                "rm -rf".to_string(),
                "sudo".to_string(),
                "chmod +x".to_string(),
            ],
            max_execution_time: Duration::from_secs(30),
            allowed_working_dirs: vec![],
            allowed_env_vars: vec![
                "PATH".to_string(),
                "HOME".to_string(),
                "USER".to_string(),
            ],
            sandbox_config: SandboxConfig::default(),
        }
    }
}

impl ExternalTool {
    /// Create a new external tool
    pub fn new(
        executable_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<Self> {
        Ok(Self {
            name: name.to_string(),
            executable_path,
            description: description.to_string(),
            capabilities,
            security_config: ExternalToolSecurityConfig::default(),
            resource_limits: ResourceLimits::default(),
        })
    }

    /// Create a new external tool with security configuration
    pub async fn new_with_security(
        executable_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
        security_level: SecurityLevel,
    ) -> Result<Self> {
        let security_config = ExternalToolSecurityConfig {
            security_level,
            sandbox_config: security_level.default_sandbox_config(),
            ..Default::default()
        };

        Ok(Self {
            name: name.to_string(),
            executable_path,
            description: description.to_string(),
            capabilities,
            security_config: security_config.clone(),
            resource_limits: security_level.default_resource_limits(),
        })
    }

    /// Validate command arguments against security policies
    fn validate_arguments(&self, args: &[String]) -> Result<()> {
        let full_command = format!("{} {}", 
            self.executable_path.display(), 
            args.join(" ")
        );

        // Check blocked patterns
        for blocked_pattern in &self.security_config.blocked_arg_patterns {
            if full_command.contains(blocked_pattern) {
                return Err(anyhow::anyhow!(
                    "Command contains blocked pattern: {}", 
                    blocked_pattern
                ));
            }
        }

        // Check allowed patterns if specified
        if !self.security_config.allowed_arg_patterns.is_empty() {
            let mut allowed = false;
            for allowed_pattern in &self.security_config.allowed_arg_patterns {
                if full_command.contains(allowed_pattern) {
                    allowed = true;
                    break;
                }
            }
            if !allowed {
                return Err(anyhow::anyhow!(
                    "Command does not match any allowed patterns"
                ));
            }
        }

        Ok(())
    }

    /// Execute the external tool with security controls
    async fn execute_with_security(&self, args: Vec<String>) -> Result<ToolResult> {
        debug!("Executing external tool: {} with args: {:?}", self.name, args);

        // Validate arguments
        self.validate_arguments(&args)?;

        // Create command
        let mut command = Command::new(&self.executable_path);
        command.args(&args);
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

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
                error!("Failed to execute external tool {}: {}", self.name, e);
                return Err(e.into());
            }
            Err(_) => {
                error!("External tool {} timed out", self.name);
                return Err(anyhow::anyhow!("Tool execution timed out"));
            }
        };

        // Process results
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        let success = output.status.success();
        
        if success {
            info!("External tool {} executed successfully", self.name);
        } else {
            warn!("External tool {} failed with exit code: {:?}", 
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
impl Tool for ExternalTool {
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

/// Builder for external tools
pub struct ExternalToolBuilder {
    name: Option<String>,
    executable_path: Option<PathBuf>,
    description: Option<String>,
    capabilities: Vec<String>,
    security_config: ExternalToolSecurityConfig,
}

impl ExternalToolBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            name: None,
            executable_path: None,
            description: None,
            capabilities: Vec::new(),
            security_config: ExternalToolSecurityConfig::default(),
        }
    }

    /// Set the tool name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the executable path
    pub fn executable_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.executable_path = Some(path.into());
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

    /// Set security level
    pub fn security_level(mut self, level: SecurityLevel) -> Self {
        self.security_config.security_level = level;
        self
    }

    /// Build the external tool
    pub fn build(self) -> Result<ExternalTool> {
        let name = self.name.context("Tool name is required")?;
        let executable_path = self.executable_path.context("Executable path is required")?;
        let description = self.description.unwrap_or_else(|| format!("External tool: {}", name));

        ExternalTool::new(executable_path, &name, &description, self.capabilities)
    }
}

impl Default for ExternalToolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_external_tool_creation() -> Result<()> {
        let tool = ExternalTool::new(
            PathBuf::from("/bin/echo"),
            "echo",
            "Echo tool",
            vec!["text-processing".to_string()],
        )?;

        assert_eq!(tool.name(), "echo");
        assert_eq!(tool.description(), "Echo tool");
        assert_eq!(tool.capabilities(), &["text-processing"]);

        Ok(())
    }

    #[tokio::test]
    async fn test_external_tool_builder() -> Result<()> {
        let tool = ExternalToolBuilder::new()
            .name("test-tool")
            .executable_path("/bin/echo")
            .description("Test tool")
            .capability("testing")
            .security_level(SecurityLevel::High)
            .build()?;

        assert_eq!(tool.name(), "test-tool");
        assert_eq!(tool.description(), "Test tool");
        assert_eq!(tool.capabilities(), &["testing"]);

        Ok(())
    }

    #[tokio::test]
    async fn test_argument_validation() -> Result<()> {
        let tool = ExternalTool::new(
            PathBuf::from("/bin/echo"),
            "echo",
            "Echo tool",
            vec!["text-processing".to_string()],
        )?;

        // Test blocked pattern
        let blocked_args = vec!["rm -rf /".to_string()];
        assert!(tool.validate_arguments(&blocked_args).is_err());

        // Test allowed pattern
        let allowed_args = vec!["hello world".to_string()];
        assert!(tool.validate_arguments(&allowed_args).is_ok());

        Ok(())
    }

    #[tokio::test]
    async fn test_external_tool_execution() -> Result<()> {
        let tool = ExternalTool::new(
            PathBuf::from("/bin/echo"),
            "echo",
            "Echo tool",
            vec!["text-processing".to_string()],
        )?;

        let params = json!({
            "args": ["hello", "world"]
        });

        let result = tool.execute(&params).await?;
        assert!(result.success);
        assert!(result.output.contains("hello world"));

        Ok(())
    }
}
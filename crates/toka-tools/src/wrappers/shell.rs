//! Shell script tool wrapper with enhanced shell script integration.
//!
//! This module provides `ShellTool` which extends `ExternalTool` with shell-specific
//! features like shell selection, environment variable handling, and script validation.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::core::{Tool, ToolParams, ToolResult};
use crate::manifest::ToolManifest;
use super::external::{ExternalTool, SecurityConfig};

/// Shell type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShellType {
    /// Bash shell
    Bash,
    /// Zsh shell
    Zsh,
    /// Fish shell
    Fish,
    /// POSIX shell
    Sh,
    /// Custom shell with path
    Custom(String),
}

impl Default for ShellType {
    fn default() -> Self {
        ShellType::Bash
    }
}

impl ShellType {
    /// Get the executable name for this shell type
    pub fn executable(&self) -> &str {
        match self {
            ShellType::Bash => "bash",
            ShellType::Zsh => "zsh",
            ShellType::Fish => "fish",
            ShellType::Sh => "sh",
            ShellType::Custom(exec) => exec,
        }
    }
    
    /// Get shell-specific arguments for script execution
    pub fn execution_args(&self) -> Vec<&str> {
        match self {
            ShellType::Bash => vec!["-c"],
            ShellType::Zsh => vec!["-c"],
            ShellType::Fish => vec!["-c"],
            ShellType::Sh => vec!["-c"],
            ShellType::Custom(_) => vec!["-c"], // Assume POSIX-like
        }
    }
}

/// Shell-specific configuration for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellConfig {
    /// Shell type to use
    pub shell_type: ShellType,
    /// Environment variables to set
    pub env_vars: HashMap<String, String>,
    /// Whether to use strict mode (set -euo pipefail for bash)
    pub strict_mode: bool,
    /// Working directory for script execution
    pub working_directory: Option<PathBuf>,
    /// Additional shell options
    pub shell_options: Vec<String>,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            shell_type: ShellType::Bash,
            env_vars: HashMap::new(),
            strict_mode: true,
            working_directory: None,
            shell_options: Vec::new(),
        }
    }
}

/// Shell tool wrapper with enhanced shell-specific features
#[derive(Debug, Clone)]
pub struct ShellTool {
    /// Underlying external tool
    external_tool: ExternalTool,
    /// Shell-specific configuration
    shell_config: ShellConfig,
    /// Shell script path
    script_path: PathBuf,
}

impl ShellTool {
    /// Create a new shell tool wrapper
    pub fn new(
        script_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<Self> {
        let external_tool = ExternalTool::wrap_shell_script(
            &script_path,
            name,
            description,
            capabilities,
        )?;
        
        Ok(Self {
            external_tool,
            shell_config: ShellConfig::default(),
            script_path,
        })
    }
    
    /// Create a shell tool with custom configuration
    pub fn with_config(
        script_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
        shell_config: ShellConfig,
        security_config: SecurityConfig,
    ) -> Result<Self> {
        let manifest = Self::create_shell_manifest(&script_path, name, description, capabilities)?;
        let shell_executable = PathBuf::from(shell_config.shell_type.executable());
        
        let external_tool = ExternalTool::new(
            manifest,
            shell_executable,
            security_config,
        )?;
        
        Ok(Self {
            external_tool,
            shell_config,
            script_path,
        })
    }
    
    /// Set shell type
    pub fn with_shell_type(mut self, shell_type: ShellType) -> Self {
        self.shell_config.shell_type = shell_type;
        self
    }
    
    /// Add environment variables
    pub fn with_env_vars(mut self, env_vars: HashMap<String, String>) -> Self {
        self.shell_config.env_vars = env_vars;
        self
    }
    
    /// Enable or disable strict mode
    pub fn with_strict_mode(mut self, strict_mode: bool) -> Self {
        self.shell_config.strict_mode = strict_mode;
        self
    }
    
    /// Set working directory
    pub fn with_working_directory(mut self, working_dir: PathBuf) -> Self {
        self.shell_config.working_directory = Some(working_dir);
        self
    }
    
    /// Validate shell environment before execution
    pub async fn validate_shell_environment(&self) -> Result<()> {
        // Check if shell executable exists
        let shell_exec = self.shell_config.shell_type.executable();
        
        let output = tokio::process::Command::new(shell_exec)
            .arg("--version")
            .output()
            .await
            .with_context(|| format!("Failed to execute {}", shell_exec))?;
        
        if !output.status.success() {
            // Some shells don't support --version, try a simple command instead
            let output = tokio::process::Command::new(shell_exec)
                .arg("-c")
                .arg("echo 'Shell test'")
                .output()
                .await
                .with_context(|| format!("Failed to test {}", shell_exec))?;
            
            if !output.status.success() {
                return Err(anyhow::anyhow!("Shell executable not working: {}", shell_exec));
            }
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        if !version.is_empty() {
            info!("Shell version: {}", version.trim());
        } else {
            info!("Shell executable validated: {}", shell_exec);
        }
        
        // Check script exists and is readable
        if !self.script_path.exists() {
            return Err(anyhow::anyhow!("Shell script not found: {}", self.script_path.display()));
        }
        
        // Check if script is executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = std::fs::metadata(&self.script_path)?;
            let permissions = metadata.permissions();
            if permissions.mode() & 0o111 == 0 {
                warn!("Script {} is not executable, may cause issues", self.script_path.display());
            }
        }
        
        Ok(())
    }
    
    /// Analyze shell script for potential issues
    pub async fn analyze_script(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        
        // Read script content
        let content = tokio::fs::read_to_string(&self.script_path).await?;
        
        // Check for common shell script issues
        if !content.starts_with("#!") {
            issues.push("Script missing shebang line".to_string());
        }
        
        if content.contains("rm -rf") {
            issues.push("Script contains potentially dangerous 'rm -rf' command".to_string());
        }
        
        if content.contains("sudo") {
            issues.push("Script contains 'sudo' command - may require elevated privileges".to_string());
        }
        
        if !content.contains("set -e") && self.shell_config.strict_mode {
            debug!("Script may benefit from 'set -e' for error handling");
        }
        
        if content.lines().any(|line| line.len() > 200) {
            issues.push("Script contains very long lines (>200 chars)".to_string());
        }
        
        Ok(issues)
    }
    
    /// Execute shell script with enhanced parameter handling
    pub async fn execute_shell(&self, params: &ToolParams) -> Result<ToolResult> {
        // Validate environment first
        self.validate_shell_environment().await?;
        
        // Analyze script for potential issues
        let issues = self.analyze_script().await?;
        if !issues.is_empty() {
            warn!("Script analysis found issues: {:?}", issues);
        }
        
        // Prepare enhanced parameters for shell execution
        let mut enhanced_params = params.clone();
        
        // Add script path
        enhanced_params.args.insert("script".to_string(), self.script_path.display().to_string());
        
        // Add shell-specific environment variables
        for (key, value) in &self.shell_config.env_vars {
            enhanced_params.args.insert(format!("env:{}", key), value.clone());
        }
        
        // Add shell options
        for (i, option) in self.shell_config.shell_options.iter().enumerate() {
            enhanced_params.args.insert(format!("shell_opt_{}", i), option.clone());
        }
        
        // Execute through the external tool
        self.external_tool.execute(&enhanced_params).await
    }
    
    /// Get shell configuration
    pub fn shell_config(&self) -> &ShellConfig {
        &self.shell_config
    }
    
    /// Get script path
    pub fn script_path(&self) -> &PathBuf {
        &self.script_path
    }
    
    /// Create a manifest for a shell script
    fn create_shell_manifest(
        script_path: &std::path::Path,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<ToolManifest> {
        use crate::manifest::{Transport, SideEffect};
        
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
    
    /// Create a shell command tool (for inline commands)
    pub fn create_command_tool(
        command: &str,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<Self> {
        // Create a temporary script file
        let temp_dir = std::env::temp_dir();
        let script_path = temp_dir.join(format!("{}.sh", name));
        
        // Write command to script file
        std::fs::write(&script_path, format!("#!/bin/bash\n{}\n", command))?;
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }
        
        Self::new(script_path, name, description, capabilities)
    }
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &str {
        self.external_tool.name()
    }
    
    fn description(&self) -> &str {
        self.external_tool.description()
    }
    
    fn version(&self) -> &str {
        self.external_tool.version()
    }
    
    async fn execute(&self, params: &ToolParams) -> Result<ToolResult> {
        info!("Executing shell tool: {} with script: {} using {}", 
              self.name(), self.script_path.display(), self.shell_config.shell_type.executable());
        
        // Use enhanced shell execution
        let result = self.execute_shell(params).await?;
        
        info!("Shell tool {} completed: success={}", self.name(), result.success);
        
        Ok(result)
    }
    
    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        // Validate through external tool first
        self.external_tool.validate_params(params)?;
        
        // Additional shell-specific validation
        if !self.script_path.exists() {
            return Err(anyhow::anyhow!("Shell script not found: {}", self.script_path.display()));
        }
        
        // Check if script has shell extension
        if let Some(extension) = self.script_path.extension() {
            let valid_extensions = ["sh", "bash", "zsh"];
            if !valid_extensions.contains(&extension.to_str().unwrap_or("")) {
                debug!("Warning: Script {} does not have a recognized shell extension", 
                       self.script_path.display());
            }
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_shell_tool_creation() -> Result<()> {
        // Create a temporary shell script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/bin/bash")?;
        writeln!(script_file, "echo 'Hello from shell tool!'")?;
        
        let script_path = script_file.path().to_path_buf();
        
        // Create shell tool
        let tool = ShellTool::new(
            script_path,
            "test-shell-tool",
            "Test shell tool",
            vec!["testing".to_string()],
        )?;
        
        assert_eq!(tool.name(), "test-shell-tool");
        assert_eq!(tool.description(), "Test shell tool");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_shell_tool_execution() -> Result<()> {
        // Create a temporary shell script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/bin/bash")?;
        writeln!(script_file, "echo 'Shell tool executed successfully!'")?;
        writeln!(script_file, "exit 0")?;
        
        let script_path = script_file.path().to_path_buf();
        
        // Make script executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&script_path)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&script_path, perms)?;
        }
        
        // Create shell tool
        let tool = ShellTool::new(
            script_path,
            "test-shell-execution",
            "Test shell execution",
            vec!["testing".to_string()],
        )?;
        
        // Execute the tool
        let params = ToolParams {
            name: "test-shell-execution".to_string(),
            args: HashMap::new(),
        };
        
        let result = tool.execute(&params).await?;
        
        assert!(result.success);
        assert!(result.output.contains("Shell tool executed successfully!"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_shell_environment_validation() -> Result<()> {
        // Create a temporary shell script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/bin/bash")?;
        writeln!(script_file, "echo 'Environment validation test'")?;
        
        let script_path = script_file.path().to_path_buf();
        
        // Create shell tool
        let tool = ShellTool::new(
            script_path,
            "test-env-validation",
            "Test environment validation",
            vec!["testing".to_string()],
        )?;
        
        // Validate environment
        let result = tool.validate_shell_environment().await;
        
        // This should succeed if bash is available
        assert!(result.is_ok());
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_command_tool_creation() -> Result<()> {
        // Create a command tool
        let tool = ShellTool::create_command_tool(
            "echo 'Hello from command tool!'",
            "test-command-tool",
            "Test command tool",
            vec!["testing".to_string()],
        )?;
        
        assert_eq!(tool.name(), "test-command-tool");
        
        // Execute the command tool
        let params = ToolParams {
            name: "test-command-tool".to_string(),
            args: HashMap::new(),
        };
        
        let result = tool.execute(&params).await?;
        
        assert!(result.success);
        assert!(result.output.contains("Hello from command tool!"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_script_analysis() -> Result<()> {
        // Create a script with some issues
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "echo 'Script without shebang'")?;
        writeln!(script_file, "rm -rf /tmp/test")?;
        writeln!(script_file, "sudo echo 'requires sudo'")?;
        
        let script_path = script_file.path().to_path_buf();
        
        // Create shell tool
        let tool = ShellTool::new(
            script_path,
            "test-analysis",
            "Test script analysis",
            vec!["testing".to_string()],
        )?;
        
        // Analyze script
        let issues = tool.analyze_script().await?;
        
        // Should find the issues we introduced
        assert!(!issues.is_empty());
        assert!(issues.iter().any(|issue| issue.contains("shebang")));
        assert!(issues.iter().any(|issue| issue.contains("rm -rf")));
        assert!(issues.iter().any(|issue| issue.contains("sudo")));
        
        Ok(())
    }
}
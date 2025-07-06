//! Python-specific tool wrapper with enhanced Python script integration.
//!
//! This module provides `PythonTool` which extends `ExternalTool` with Python-specific
//! features like virtual environment support, requirement management, and Python path handling.

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::core::{Tool, ToolParams, ToolResult};
use crate::manifest::ToolManifest;
use super::external::{ExternalTool, SecurityConfig};

/// Python-specific configuration for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonConfig {
    /// Python executable to use (default: python3)
    pub python_executable: String,
    /// Virtual environment path (optional)
    pub venv_path: Option<PathBuf>,
    /// Python requirements (optional)
    pub requirements: Vec<String>,
    /// Additional Python path entries
    pub python_path: Vec<PathBuf>,
    /// Python environment variables
    pub python_env: HashMap<String, String>,
}

impl Default for PythonConfig {
    fn default() -> Self {
        Self {
            python_executable: "python3".to_string(),
            venv_path: None,
            requirements: Vec::new(),
            python_path: Vec::new(),
            python_env: HashMap::new(),
        }
    }
}

/// Python tool wrapper with enhanced Python-specific features
#[derive(Debug, Clone)]
pub struct PythonTool {
    /// Underlying external tool
    external_tool: ExternalTool,
    /// Python-specific configuration
    python_config: PythonConfig,
    /// Python script path
    script_path: PathBuf,
}

impl PythonTool {
    /// Create a new Python tool wrapper
    pub fn new(
        script_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
    ) -> Result<Self> {
        let external_tool = ExternalTool::wrap_python_script(
            &script_path,
            name,
            description,
            capabilities,
        )?;
        
        Ok(Self {
            external_tool,
            python_config: PythonConfig::default(),
            script_path,
        })
    }
    
    /// Create a Python tool with custom configuration
    pub fn with_config(
        script_path: PathBuf,
        name: &str,
        description: &str,
        capabilities: Vec<String>,
        python_config: PythonConfig,
        security_config: SecurityConfig,
    ) -> Result<Self> {
        let manifest = Self::create_python_manifest(&script_path, name, description, capabilities)?;
        let python_executable = PathBuf::from(&python_config.python_executable);
        
        let external_tool = ExternalTool::new(
            manifest,
            python_executable,
            security_config,
        )?;
        
        Ok(Self {
            external_tool,
            python_config,
            script_path,
        })
    }
    
    /// Set virtual environment path
    pub fn with_venv(mut self, venv_path: PathBuf) -> Self {
        self.python_config.venv_path = Some(venv_path);
        self
    }
    
    /// Add Python requirements
    pub fn with_requirements(mut self, requirements: Vec<String>) -> Self {
        self.python_config.requirements = requirements;
        self
    }
    
    /// Add Python path entries
    pub fn with_python_path(mut self, python_path: Vec<PathBuf>) -> Self {
        self.python_config.python_path = python_path;
        self
    }
    
    /// Validate Python environment before execution
    pub async fn validate_python_environment(&self) -> Result<()> {
        // Check if Python executable exists
        let python_exec = &self.python_config.python_executable;
        
        let output = tokio::process::Command::new(python_exec)
            .arg("--version")
            .output()
            .await
            .with_context(|| format!("Failed to execute {}", python_exec))?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("Python executable not working: {}", python_exec));
        }
        
        let version = String::from_utf8_lossy(&output.stdout);
        info!("Python version: {}", version.trim());
        
        // Check virtual environment if specified
        if let Some(venv_path) = &self.python_config.venv_path {
            if !venv_path.exists() {
                return Err(anyhow::anyhow!("Virtual environment not found: {}", venv_path.display()));
            }
            
            debug!("Using virtual environment: {}", venv_path.display());
        }
        
        // Check script exists
        if !self.script_path.exists() {
            return Err(anyhow::anyhow!("Python script not found: {}", self.script_path.display()));
        }
        
        Ok(())
    }
    
    /// Check if required Python packages are installed
    pub async fn check_requirements(&self) -> Result<Vec<String>> {
        let mut missing_packages = Vec::new();
        
        for requirement in &self.python_config.requirements {
            let package_name = requirement.split("==").next().unwrap_or(requirement);
            
            let output = tokio::process::Command::new(&self.python_config.python_executable)
                .arg("-c")
                .arg(&format!("import {}", package_name.replace("-", "_")))
                .output()
                .await?;
            
            if !output.status.success() {
                missing_packages.push(requirement.clone());
            }
        }
        
        Ok(missing_packages)
    }
    
    /// Install missing Python requirements
    pub async fn install_requirements(&self) -> Result<()> {
        if self.python_config.requirements.is_empty() {
            return Ok(());
        }
        
        info!("Installing Python requirements: {:?}", self.python_config.requirements);
        
        for requirement in &self.python_config.requirements {
            let output = tokio::process::Command::new(&self.python_config.python_executable)
                .arg("-m")
                .arg("pip")
                .arg("install")
                .arg(requirement)
                .output()
                .await?;
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to install {}: {}", requirement, stderr));
            }
        }
        
        Ok(())
    }
    
    /// Execute Python script with enhanced parameter handling
    pub async fn execute_python(&self, params: &ToolParams) -> Result<ToolResult> {
        // Validate environment first
        self.validate_python_environment().await?;
        
        // Prepare enhanced parameters for Python execution
        let mut enhanced_params = params.clone();
        
        // Add script path as first argument
        enhanced_params.args.insert("script".to_string(), self.script_path.display().to_string());
        
        // Add Python-specific environment variables
        for (key, value) in &self.python_config.python_env {
            enhanced_params.args.insert(format!("env:{}", key), value.clone());
        }
        
        // Execute through the external tool
        self.external_tool.execute(&enhanced_params).await
    }
    
    /// Get Python configuration
    pub fn python_config(&self) -> &PythonConfig {
        &self.python_config
    }
    
    /// Get script path
    pub fn script_path(&self) -> &PathBuf {
        &self.script_path
    }
    
    /// Create a manifest for a Python script
    fn create_python_manifest(
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
                exec: format!("python3 {}", script_path.display()),
            }],
            action_id: None,
            manifest_version: "1.1".to_string(),
            protocols: vec![],
            metadata: std::collections::BTreeMap::new(),
        };
        
        Ok(manifest)
    }
}

#[async_trait]
impl Tool for PythonTool {
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
        info!("Executing Python tool: {} with script: {}", 
              self.name(), self.script_path.display());
        
        // Use enhanced Python execution
        let result = self.execute_python(params).await?;
        
        info!("Python tool {} completed: success={}", self.name(), result.success);
        
        Ok(result)
    }
    
    fn validate_params(&self, params: &ToolParams) -> Result<()> {
        // Validate through external tool first
        self.external_tool.validate_params(params)?;
        
        // Additional Python-specific validation
        if !self.script_path.exists() {
            return Err(anyhow::anyhow!("Python script not found: {}", self.script_path.display()));
        }
        
        // Check if script is actually a Python file
        if let Some(extension) = self.script_path.extension() {
            if extension != "py" {
                debug!("Warning: Script {} does not have .py extension", self.script_path.display());
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
    async fn test_python_tool_creation() -> Result<()> {
        // Create a temporary Python script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "import sys")?;
        writeln!(script_file, "print('Hello from Python tool!')")?;
        
        let script_path = script_file.path().to_path_buf();
        
        // Create Python tool
        let tool = PythonTool::new(
            script_path,
            "test-python-tool",
            "Test Python tool",
            vec!["testing".to_string()],
        )?;
        
        assert_eq!(tool.name(), "test-python-tool");
        assert_eq!(tool.description(), "Test Python tool");
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_python_tool_execution() -> Result<()> {
        // Create a temporary Python script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "import sys")?;
        writeln!(script_file, "print('Python tool executed successfully!')")?;
        writeln!(script_file, "sys.exit(0)")?;
        
        let script_path = script_file.path().to_path_buf();
        
        // Create Python tool
        let tool = PythonTool::new(
            script_path,
            "test-python-execution",
            "Test Python execution",
            vec!["testing".to_string()],
        )?;
        
        // Execute the tool
        let params = ToolParams {
            name: "test-python-execution".to_string(),
            args: HashMap::new(),
        };
        
        let result = tool.execute(&params).await?;
        
        assert!(result.success);
        assert!(result.output.contains("Python tool executed successfully!"));
        
        Ok(())
    }
    
    #[tokio::test]
    async fn test_python_environment_validation() -> Result<()> {
        // Create a temporary Python script
        let mut script_file = NamedTempFile::new()?;
        writeln!(script_file, "#!/usr/bin/env python3")?;
        writeln!(script_file, "print('Environment validation test')")?;
        
        let script_path = script_file.path().to_path_buf();
        
        // Create Python tool
        let tool = PythonTool::new(
            script_path,
            "test-env-validation",
            "Test environment validation",
            vec!["testing".to_string()],
        )?;
        
        // Validate environment
        let result = tool.validate_python_environment().await;
        
        // This should succeed if Python is available
        assert!(result.is_ok());
        
        Ok(())
    }
}
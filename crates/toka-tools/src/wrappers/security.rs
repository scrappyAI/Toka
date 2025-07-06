//! Unified security model for external tool execution
//!
//! This module provides a comprehensive security framework for external tools,
//! combining the sandboxing capabilities from the analysis tools approach with
//! the flexible security classification from the generic tool approach.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// Security level classification for tools
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// High security for analysis and sensitive tools
    High,
    /// Medium security for system and build tools
    Medium,
    /// Basic security for utility tools
    Basic,
}

impl SecurityLevel {
    /// Get default resource limits for this security level
    pub fn default_resource_limits(&self) -> ResourceLimits {
        match self {
            SecurityLevel::High => ResourceLimits {
                max_memory_mb: 512,
                max_cpu_percent: 50.0,
                max_execution_time: Duration::from_secs(600), // 10 minutes
                max_output_size: 10 * 1024 * 1024, // 10MB
                max_output_files: 50,
                max_disk_mb: 256,
            },
            SecurityLevel::Medium => ResourceLimits {
                max_memory_mb: 256,
                max_cpu_percent: 25.0,
                max_execution_time: Duration::from_secs(300), // 5 minutes
                max_output_size: 5 * 1024 * 1024, // 5MB
                max_output_files: 25,
                max_disk_mb: 128,
            },
            SecurityLevel::Basic => ResourceLimits {
                max_memory_mb: 128,
                max_cpu_percent: 10.0,
                max_execution_time: Duration::from_secs(60), // 1 minute
                max_output_size: 1024 * 1024, // 1MB
                max_output_files: 10,
                max_disk_mb: 64,
            },
        }
    }
    
    /// Get default sandbox configuration for this security level
    pub fn default_sandbox_config(&self) -> SandboxConfig {
        match self {
            SecurityLevel::High => SandboxConfig {
                use_namespaces: true,
                allow_network: false,
                readonly_paths: vec![PathBuf::from(".")],
                writable_paths: vec![PathBuf::from("target/analysis")],
                forbidden_paths: vec![PathBuf::from("/etc"), PathBuf::from("/sys")],
                allowed_syscalls: vec!["read", "write", "open", "close", "stat", "exit"],
                env_whitelist: vec!["PATH".to_string(), "HOME".to_string()],
                disable_ptrace: true,
                disable_core_dumps: true,
            },
            SecurityLevel::Medium => SandboxConfig {
                use_namespaces: false,
                allow_network: true,
                readonly_paths: vec![PathBuf::from(".")],
                writable_paths: vec![PathBuf::from("target"), PathBuf::from("tmp")],
                forbidden_paths: vec![PathBuf::from("/etc/passwd")],
                allowed_syscalls: vec![], // Default allowed
                env_whitelist: vec!["PATH".to_string(), "HOME".to_string(), "USER".to_string()],
                disable_ptrace: false,
                disable_core_dumps: false,
            },
            SecurityLevel::Basic => SandboxConfig {
                use_namespaces: false,
                allow_network: false,
                readonly_paths: vec![PathBuf::from(".")],
                writable_paths: vec![PathBuf::from("tmp")],
                forbidden_paths: vec![],
                allowed_syscalls: vec![], // Default allowed
                env_whitelist: vec!["PATH".to_string()],
                disable_ptrace: false,
                disable_core_dumps: false,
            },
        }
    }
}

/// Resource limits for tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory in MB
    pub max_memory_mb: u64,
    /// Maximum CPU percentage (0.0 - 100.0)
    pub max_cpu_percent: f64,
    /// Maximum execution time
    pub max_execution_time: Duration,
    /// Maximum output size in bytes
    pub max_output_size: usize,
    /// Maximum number of output files
    pub max_output_files: usize,
    /// Maximum disk usage in MB
    pub max_disk_mb: u64,
}

/// Sandbox configuration for external tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Use Linux namespaces for isolation
    pub use_namespaces: bool,
    /// Allow network access
    pub allow_network: bool,
    /// Read-only filesystem paths
    pub readonly_paths: Vec<PathBuf>,
    /// Writable filesystem paths
    pub writable_paths: Vec<PathBuf>,
    /// Forbidden filesystem paths
    pub forbidden_paths: Vec<PathBuf>,
    /// Allowed system calls (empty means all allowed)
    pub allowed_syscalls: Vec<&'static str>,
    /// Environment variables whitelist
    pub env_whitelist: Vec<String>,
    /// Disable ptrace
    pub disable_ptrace: bool,
    /// Disable core dumps
    pub disable_core_dumps: bool,
}

/// General security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Security level
    pub security_level: SecurityLevel,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Sandbox configuration
    pub sandbox_config: SandboxConfig,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
    /// Audit logging enabled
    pub audit_logging: bool,
}

impl SecurityConfig {
    /// Create security config for a specific level
    pub fn for_level(level: SecurityLevel) -> Self {
        Self {
            security_level: level,
            resource_limits: level.default_resource_limits(),
            sandbox_config: level.default_sandbox_config(),
            required_capabilities: vec![],
            audit_logging: true,
        }
    }
    
    /// Create security config with custom capabilities
    pub fn with_capabilities(level: SecurityLevel, capabilities: Vec<String>) -> Self {
        Self {
            security_level: level,
            resource_limits: level.default_resource_limits(),
            sandbox_config: level.default_sandbox_config(),
            required_capabilities: capabilities,
            audit_logging: true,
        }
    }
}

/// Capability validator for tool execution
pub struct CapabilityValidator {
    /// Registered capabilities
    capabilities: HashMap<String, CapabilityDefinition>,
}

impl CapabilityValidator {
    /// Create a new capability validator
    pub fn new() -> Self {
        let mut capabilities = HashMap::new();
        
        // Register standard capabilities
        capabilities.insert("filesystem-read".to_string(), CapabilityDefinition {
            name: "filesystem-read".to_string(),
            description: "Read access to filesystem".to_string(),
            risk_level: RiskLevel::Low,
            required_permissions: vec!["read".to_string()],
        });
        
        capabilities.insert("filesystem-write".to_string(), CapabilityDefinition {
            name: "filesystem-write".to_string(),
            description: "Write access to filesystem".to_string(),
            risk_level: RiskLevel::Medium,
            required_permissions: vec!["write".to_string()],
        });
        
        capabilities.insert("process-spawn".to_string(), CapabilityDefinition {
            name: "process-spawn".to_string(),
            description: "Spawn new processes".to_string(),
            risk_level: RiskLevel::High,
            required_permissions: vec!["execute".to_string()],
        });
        
        capabilities.insert("network-access".to_string(), CapabilityDefinition {
            name: "network-access".to_string(),
            description: "Access network resources".to_string(),
            risk_level: RiskLevel::High,
            required_permissions: vec!["network".to_string()],
        });
        
        capabilities.insert("code-analysis".to_string(), CapabilityDefinition {
            name: "code-analysis".to_string(),
            description: "Analyze code structure and dependencies".to_string(),
            risk_level: RiskLevel::Medium,
            required_permissions: vec!["read".to_string()],
        });
        
        capabilities.insert("system-monitoring".to_string(), CapabilityDefinition {
            name: "system-monitoring".to_string(),
            description: "Monitor system resources and processes".to_string(),
            risk_level: RiskLevel::Medium,
            required_permissions: vec!["read".to_string(), "execute".to_string()],
        });
        
        Self { capabilities }
    }
    
    /// Validate that an agent has the required capabilities to execute a tool
    pub fn validate_tool_execution(
        &self,
        tool_capabilities: &[String],
        agent_capabilities: &[String],
    ) -> Result<()> {
        for tool_cap in tool_capabilities {
            if !agent_capabilities.contains(tool_cap) {
                return Err(anyhow::anyhow!(
                    "Agent lacks required capability: {}",
                    tool_cap
                ));
            }
            
            // Check if capability is registered
            if let Some(cap_def) = self.capabilities.get(tool_cap) {
                match cap_def.risk_level {
                    RiskLevel::High => {
                        warn!("High-risk capability being used: {}", tool_cap);
                    }
                    RiskLevel::Medium => {
                        info!("Medium-risk capability being used: {}", tool_cap);
                    }
                    RiskLevel::Low => {
                        debug!("Low-risk capability being used: {}", tool_cap);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Register a new capability
    pub fn register_capability(&mut self, capability: CapabilityDefinition) {
        self.capabilities.insert(capability.name.clone(), capability);
    }
    
    /// Get capability definition
    pub fn get_capability(&self, name: &str) -> Option<&CapabilityDefinition> {
        self.capabilities.get(name)
    }
    
    /// List all registered capabilities
    pub fn list_capabilities(&self) -> Vec<&CapabilityDefinition> {
        self.capabilities.values().collect()
    }
}

/// Capability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDefinition {
    /// Capability name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Required system permissions
    pub required_permissions: Vec<String>,
}

/// Risk level for capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Sandbox executor for running external tools securely
pub struct SandboxExecutor {
    config: SandboxConfig,
    resource_limits: ResourceLimits,
}

impl SandboxExecutor {
    /// Create a new sandbox executor
    pub fn new(config: SandboxConfig, resource_limits: ResourceLimits) -> Self {
        Self {
            config,
            resource_limits,
        }
    }
    
    /// Execute a command in the sandbox
    pub async fn execute_command(
        &self,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<SandboxResult> {
        let start_time = std::time::Instant::now();
        
        // Prepare command
        let mut cmd = tokio::process::Command::new(command);
        cmd.args(args);
        
        // Set environment variables
        let filtered_env = self.filter_environment(env);
        cmd.envs(filtered_env);
        
        // Set resource limits
        self.apply_resource_limits(&mut cmd)?;
        
        // Execute with timeout
        let output = tokio::time::timeout(
            self.resource_limits.max_execution_time,
            cmd.output(),
        )
        .await
        .context("Command execution timed out")?
        .context("Failed to execute command")?;
        
        let execution_time = start_time.elapsed();
        
        // Validate output size
        let stdout_size = output.stdout.len();
        let stderr_size = output.stderr.len();
        let total_output_size = stdout_size + stderr_size;
        
        if total_output_size > self.resource_limits.max_output_size {
            return Err(anyhow::anyhow!(
                "Output size {} exceeds limit {}",
                total_output_size,
                self.resource_limits.max_output_size
            ));
        }
        
        Ok(SandboxResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            execution_time,
            output_size: total_output_size,
        })
    }
    
    /// Filter environment variables based on whitelist
    fn filter_environment(&self, env: &HashMap<String, String>) -> HashMap<String, String> {
        let mut filtered = HashMap::new();
        
        for (key, value) in env {
            if self.config.env_whitelist.contains(key) {
                filtered.insert(key.clone(), value.clone());
            }
        }
        
        // Always include PATH if not explicitly provided
        if !filtered.contains_key("PATH") {
            if let Ok(path) = std::env::var("PATH") {
                filtered.insert("PATH".to_string(), path);
            }
        }
        
        filtered
    }
    
    /// Apply resource limits to the command
    fn apply_resource_limits(&self, cmd: &mut tokio::process::Command) -> Result<()> {
        // Note: This is a simplified implementation
        // In a full implementation, you would use cgroups, rlimits, etc.
        
        // Set working directory to a safe location
        cmd.current_dir(".");
        
        // Set process group for easier cleanup
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            cmd.process_group(0);
        }
        
        Ok(())
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxResult {
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time
    pub execution_time: Duration,
    /// Total output size
    pub output_size: usize,
}

impl SandboxResult {
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        self.exit_code == 0
    }
    
    /// Get combined output
    pub fn combined_output(&self) -> String {
        if self.stderr.is_empty() {
            self.stdout.clone()
        } else if self.stdout.is_empty() {
            self.stderr.clone()
        } else {
            format!("{}\n{}", self.stdout, self.stderr)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_levels() {
        let high = SecurityLevel::High;
        let medium = SecurityLevel::Medium;
        let basic = SecurityLevel::Basic;
        
        // Test resource limits
        assert!(high.default_resource_limits().max_memory_mb > medium.default_resource_limits().max_memory_mb);
        assert!(medium.default_resource_limits().max_memory_mb > basic.default_resource_limits().max_memory_mb);
        
        // Test sandbox configs
        assert!(high.default_sandbox_config().use_namespaces);
        assert!(!medium.default_sandbox_config().use_namespaces);
        assert!(!basic.default_sandbox_config().use_namespaces);
    }
    
    #[test]
    fn test_capability_validator() {
        let validator = CapabilityValidator::new();
        
        // Test capability validation
        let tool_caps = vec!["filesystem-read".to_string()];
        let agent_caps = vec!["filesystem-read".to_string(), "filesystem-write".to_string()];
        
        assert!(validator.validate_tool_execution(&tool_caps, &agent_caps).is_ok());
        
        // Test missing capability
        let insufficient_caps = vec!["filesystem-read".to_string()];
        let required_caps = vec!["filesystem-read".to_string(), "network-access".to_string()];
        
        assert!(validator.validate_tool_execution(&required_caps, &insufficient_caps).is_err());
    }
    
    #[test]
    fn test_security_config() {
        let config = SecurityConfig::for_level(SecurityLevel::High);
        assert_eq!(config.security_level, SecurityLevel::High);
        assert!(config.audit_logging);
        
        let config_with_caps = SecurityConfig::with_capabilities(
            SecurityLevel::Medium,
            vec!["filesystem-read".to_string()],
        );
        assert_eq!(config_with_caps.required_capabilities.len(), 1);
    }
    
    #[tokio::test]
    async fn test_sandbox_executor() -> Result<()> {
        let sandbox_config = SecurityLevel::Basic.default_sandbox_config();
        let resource_limits = SecurityLevel::Basic.default_resource_limits();
        
        let executor = SandboxExecutor::new(sandbox_config, resource_limits);
        
        // Test simple command execution
        let result = executor.execute_command(
            "echo",
            &["Hello, World!".to_string()],
            &HashMap::new(),
        ).await?;
        
        assert!(result.is_success());
        assert!(result.stdout.contains("Hello, World!"));
        
        Ok(())
    }
}
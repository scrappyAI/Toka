//! Security validation and resource management for analysis tools
//!
//! This module provides security validation, resource limit enforcement, and
//! capability checking for Python analysis tools.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use toka_agent_runtime::CapabilityValidator;
use toka_types::SecurityConfig;
use toka_tools::ToolParams;

use crate::AnalysisError;

/// Resource limits for analysis tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum CPU usage percentage
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

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_percent: 50.0,
            max_execution_time: Duration::from_secs(600),
            max_output_size: 10 * 1024 * 1024, // 10MB
            max_output_files: 100,
            max_disk_mb: 1024,
        }
    }
}

/// Security validator for analysis tools
pub struct SecurityValidator {
    capability_validator: CapabilityValidator,
    resource_limits: ResourceLimits,
    allowed_output_paths: Vec<PathBuf>,
    blocked_commands: Vec<String>,
    blocked_imports: Vec<String>,
}

impl SecurityValidator {
    /// Create a new security validator
    pub fn new(
        security_config: SecurityConfig,
        resource_limits: ResourceLimits,
    ) -> Self {
        let capability_validator = CapabilityValidator::new(
            security_config.capabilities_required.clone(),
            security_config.clone(),
        );
        
        Self {
            capability_validator,
            resource_limits,
            allowed_output_paths: vec![
                PathBuf::from("target/analysis"),
                PathBuf::from("/tmp"),
            ],
            blocked_commands: vec![
                "rm".to_string(),
                "rmdir".to_string(),
                "dd".to_string(),
                "sudo".to_string(),
                "su".to_string(),
                "chmod".to_string(),
                "chown".to_string(),
                "mount".to_string(),
                "umount".to_string(),
                "kill".to_string(),
                "killall".to_string(),
                "reboot".to_string(),
                "shutdown".to_string(),
                "systemctl".to_string(),
                "service".to_string(),
                "crontab".to_string(),
                "at".to_string(),
                "batch".to_string(),
                "nc".to_string(),
                "netcat".to_string(),
                "wget".to_string(),
                "curl".to_string(),
                "ssh".to_string(),
                "scp".to_string(),
                "rsync".to_string(),
                "ftp".to_string(),
                "telnet".to_string(),
            ],
            blocked_imports: vec![
                "subprocess".to_string(),
                "os".to_string(),
                "sys".to_string(),
                "socket".to_string(),
                "urllib".to_string(),
                "requests".to_string(),
                "http".to_string(),
                "ftplib".to_string(),
                "smtplib".to_string(),
                "poplib".to_string(),
                "imaplib".to_string(),
                "telnetlib".to_string(),
                "pty".to_string(),
                "ctypes".to_string(),
                "multiprocessing".to_string(),
                "threading".to_string(),
                "concurrent".to_string(),
                "asyncio".to_string(),
                "importlib".to_string(),
                "exec".to_string(),
                "eval".to_string(),
                "compile".to_string(),
                "__import__".to_string(),
            ],
        }
    }
    
    /// Validate tool parameters for security
    pub fn validate_tool_params(&self, tool_name: &str, params: &ToolParams) -> Result<()> {
        debug!("Validating tool parameters for: {}", tool_name);
        
        // Check required capabilities
        let required_capabilities = self.get_required_capabilities(tool_name);
        for capability in &required_capabilities {
            if !self.capability_validator.can_perform(capability)? {
                return Err(AnalysisError::SecurityViolation(
                    format!("Missing required capability: {}", capability)
                ).into());
            }
        }
        
        // Validate input parameters
        self.validate_input_parameters(params)?;
        
        // Check for suspicious content
        self.check_for_suspicious_content(params)?;
        
        Ok(())
    }
    
    /// Get required capabilities for a tool
    fn get_required_capabilities(&self, tool_name: &str) -> Vec<String> {
        match tool_name {
            "control-flow-analysis" => vec![
                "filesystem-read".to_string(),
                "filesystem-write".to_string(),
                "process-spawn".to_string(),
            ],
            "dependency-analysis" => vec![
                "filesystem-read".to_string(),
                "filesystem-write".to_string(),
                "process-spawn".to_string(),
            ],
            "combined-analysis" => vec![
                "filesystem-read".to_string(),
                "filesystem-write".to_string(),
                "process-spawn".to_string(),
            ],
            _ => vec!["filesystem-read".to_string()],
        }
    }
    
    /// Validate input parameters
    fn validate_input_parameters(&self, params: &ToolParams) -> Result<()> {
        for (key, value) in &params.args {
            // Check parameter size
            if value.len() > 10000 {
                return Err(AnalysisError::InvalidInput(
                    format!("Parameter {} too large: {} bytes", key, value.len())
                ).into());
            }
            
            // Check for path traversal attempts
            if value.contains("..") || value.contains("~") {
                return Err(AnalysisError::SecurityViolation(
                    format!("Path traversal attempt detected in parameter {}: {}", key, value)
                ).into());
            }
            
            // Check for command injection attempts
            if value.contains(";") || value.contains("&&") || value.contains("||") || value.contains("|") {
                return Err(AnalysisError::SecurityViolation(
                    format!("Command injection attempt detected in parameter {}: {}", key, value)
                ).into());
            }
            
            // Check for shell metacharacters
            if value.contains("$(") || value.contains("${") || value.contains("`") {
                return Err(AnalysisError::SecurityViolation(
                    format!("Shell metacharacter detected in parameter {}: {}", key, value)
                ).into());
            }
        }
        
        Ok(())
    }
    
    /// Check for suspicious content in parameters
    fn check_for_suspicious_content(&self, params: &ToolParams) -> Result<()> {
        for (key, value) in &params.args {
            let value_lower = value.to_lowercase();
            
            // Check for blocked commands
            for blocked_cmd in &self.blocked_commands {
                if value_lower.contains(blocked_cmd) {
                    return Err(AnalysisError::SecurityViolation(
                        format!("Blocked command {} detected in parameter {}", blocked_cmd, key)
                    ).into());
                }
            }
            
            // Check for blocked imports
            for blocked_import in &self.blocked_imports {
                if value_lower.contains(&format!("import {}", blocked_import)) ||
                   value_lower.contains(&format!("from {}", blocked_import)) {
                    return Err(AnalysisError::SecurityViolation(
                        format!("Blocked import {} detected in parameter {}", blocked_import, key)
                    ).into());
                }
            }
            
            // Check for dangerous patterns
            let dangerous_patterns = [
                "eval(",
                "exec(",
                "compile(",
                "__import__(",
                "getattr(",
                "setattr(",
                "delattr(",
                "hasattr(",
                "globals(",
                "locals(",
                "vars(",
                "dir(",
                "open(",
                "file(",
                "input(",
                "raw_input(",
            ];
            
            for pattern in &dangerous_patterns {
                if value_lower.contains(pattern) {
                    return Err(AnalysisError::SecurityViolation(
                        format!("Dangerous pattern {} detected in parameter {}", pattern, key)
                    ).into());
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate output path
    pub fn validate_output_path(&self, path: &PathBuf) -> Result<()> {
        let path = path.canonicalize()
            .map_err(|e| AnalysisError::SecurityViolation(
                format!("Failed to canonicalize output path: {}", e)
            ))?;
        
        for allowed_path in &self.allowed_output_paths {
            if let Ok(allowed_path) = allowed_path.canonicalize() {
                if path.starts_with(&allowed_path) {
                    return Ok(());
                }
            }
        }
        
        Err(AnalysisError::SecurityViolation(
            format!("Output path not allowed: {}", path.display())
        ).into())
    }
    
    /// Validate resource usage
    pub fn validate_resource_usage(&self, resource_usage: &ResourceUsage) -> Result<()> {
        if resource_usage.memory_mb > self.resource_limits.max_memory_mb {
            return Err(AnalysisError::ResourceLimitExceeded(
                format!("Memory limit exceeded: {} MB > {} MB", 
                       resource_usage.memory_mb, self.resource_limits.max_memory_mb)
            ).into());
        }
        
        if resource_usage.cpu_percent > self.resource_limits.max_cpu_percent {
            return Err(AnalysisError::ResourceLimitExceeded(
                format!("CPU limit exceeded: {}% > {}%", 
                       resource_usage.cpu_percent, self.resource_limits.max_cpu_percent)
            ).into());
        }
        
        if resource_usage.execution_time > self.resource_limits.max_execution_time {
            return Err(AnalysisError::ResourceLimitExceeded(
                format!("Execution time limit exceeded: {:?} > {:?}", 
                       resource_usage.execution_time, self.resource_limits.max_execution_time)
            ).into());
        }
        
        if resource_usage.output_size > self.resource_limits.max_output_size {
            return Err(AnalysisError::ResourceLimitExceeded(
                format!("Output size limit exceeded: {} bytes > {} bytes", 
                       resource_usage.output_size, self.resource_limits.max_output_size)
            ).into());
        }
        
        if resource_usage.output_files > self.resource_limits.max_output_files {
            return Err(AnalysisError::ResourceLimitExceeded(
                format!("Output file limit exceeded: {} files > {} files", 
                       resource_usage.output_files, self.resource_limits.max_output_files)
            ).into());
        }
        
        Ok(())
    }
    
    /// Sanitize string input
    pub fn sanitize_string(&self, input: &str) -> String {
        input
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;")
            .replace("\"", "&quot;")
            .replace("'", "&#x27;")
            .replace("/", "&#x2F;")
    }
    
    /// Check if path is safe to read
    pub fn is_safe_read_path(&self, path: &PathBuf) -> bool {
        // Check for path traversal
        if path.to_string_lossy().contains("..") {
            return false;
        }
        
        // Check for absolute paths outside workspace
        if path.is_absolute() {
            let path_str = path.to_string_lossy();
            if path_str.starts_with("/etc") || 
               path_str.starts_with("/proc") ||
               path_str.starts_with("/dev") ||
               path_str.starts_with("/sys") {
                return false;
            }
        }
        
        true
    }
    
    /// Check if file content is safe
    pub fn is_safe_file_content(&self, content: &str) -> bool {
        // Check for suspicious patterns
        let suspicious_patterns = [
            "#!/bin/sh",
            "#!/bin/bash",
            "exec(",
            "eval(",
            "__import__",
            "subprocess",
            "os.system",
            "socket",
            "urllib",
        ];
        
        let content_lower = content.to_lowercase();
        for pattern in &suspicious_patterns {
            if content_lower.contains(pattern) {
                warn!("Suspicious pattern detected in file content: {}", pattern);
                return false;
            }
        }
        
        true
    }
    
    /// Get resource limits
    pub fn get_resource_limits(&self) -> &ResourceLimits {
        &self.resource_limits
    }
    
    /// Get capability validator
    pub fn get_capability_validator(&self) -> &CapabilityValidator {
        &self.capability_validator
    }
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in MB
    pub memory_mb: u64,
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Execution time
    pub execution_time: Duration,
    /// Output size in bytes
    pub output_size: usize,
    /// Number of output files
    pub output_files: usize,
    /// Disk usage in MB
    pub disk_mb: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0,
            cpu_percent: 0.0,
            execution_time: Duration::from_secs(0),
            output_size: 0,
            output_files: 0,
            disk_mb: 0,
        }
    }
}

/// Security context for analysis execution
#[derive(Debug, Clone)]
pub struct SecurityContext {
    /// Security validator
    pub validator: SecurityValidator,
    /// Execution ID for audit trail
    pub execution_id: String,
    /// Tool name
    pub tool_name: String,
    /// Start time
    pub start_time: std::time::Instant,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(
        validator: SecurityValidator,
        execution_id: String,
        tool_name: String,
    ) -> Self {
        Self {
            validator,
            execution_id,
            tool_name,
            start_time: std::time::Instant::now(),
        }
    }
    
    /// Get elapsed time
    pub fn elapsed_time(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Check if execution should be terminated due to resource limits
    pub fn should_terminate(&self) -> bool {
        self.elapsed_time() > self.validator.resource_limits.max_execution_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    fn create_test_security_config() -> SecurityConfig {
        SecurityConfig {
            sandbox: true,
            capabilities_required: vec![
                "filesystem-read".to_string(),
                "filesystem-write".to_string(),
            ],
            resource_limits: toka_types::ResourceLimits {
                max_memory: "512MB".to_string(),
                max_cpu: "50%".to_string(),
                timeout: "10m".to_string(),
            },
        }
    }
    
    #[test]
    fn test_security_validator_creation() {
        let security_config = create_test_security_config();
        let resource_limits = ResourceLimits::default();
        
        let validator = SecurityValidator::new(security_config, resource_limits);
        assert!(!validator.blocked_commands.is_empty());
        assert!(!validator.blocked_imports.is_empty());
    }
    
    #[test]
    fn test_input_parameter_validation() {
        let security_config = create_test_security_config();
        let resource_limits = ResourceLimits::default();
        let validator = SecurityValidator::new(security_config, resource_limits);
        
        // Test valid parameters
        let mut valid_params = HashMap::new();
        valid_params.insert("target_function".to_string(), "main".to_string());
        valid_params.insert("output_format".to_string(), "mermaid".to_string());
        
        let params = ToolParams {
            name: "test".to_string(),
            args: valid_params,
        };
        
        assert!(validator.validate_input_parameters(&params).is_ok());
        
        // Test invalid parameters (path traversal)
        let mut invalid_params = HashMap::new();
        invalid_params.insert("file_path".to_string(), "../etc/passwd".to_string());
        
        let params = ToolParams {
            name: "test".to_string(),
            args: invalid_params,
        };
        
        assert!(validator.validate_input_parameters(&params).is_err());
    }
    
    #[test]
    fn test_suspicious_content_detection() {
        let security_config = create_test_security_config();
        let resource_limits = ResourceLimits::default();
        let validator = SecurityValidator::new(security_config, resource_limits);
        
        // Test blocked command detection
        let mut params = HashMap::new();
        params.insert("command".to_string(), "rm -rf /".to_string());
        
        let params = ToolParams {
            name: "test".to_string(),
            args: params,
        };
        
        assert!(validator.check_for_suspicious_content(&params).is_err());
        
        // Test blocked import detection
        let mut params = HashMap::new();
        params.insert("code".to_string(), "import subprocess".to_string());
        
        let params = ToolParams {
            name: "test".to_string(),
            args: params,
        };
        
        assert!(validator.check_for_suspicious_content(&params).is_err());
    }
    
    #[test]
    fn test_resource_usage_validation() {
        let security_config = create_test_security_config();
        let resource_limits = ResourceLimits::default();
        let validator = SecurityValidator::new(security_config, resource_limits);
        
        // Test valid resource usage
        let usage = ResourceUsage {
            memory_mb: 100,
            cpu_percent: 25.0,
            execution_time: Duration::from_secs(60),
            output_size: 1024,
            output_files: 5,
            disk_mb: 10,
        };
        
        assert!(validator.validate_resource_usage(&usage).is_ok());
        
        // Test exceeded memory limit
        let usage = ResourceUsage {
            memory_mb: 1000,
            cpu_percent: 25.0,
            execution_time: Duration::from_secs(60),
            output_size: 1024,
            output_files: 5,
            disk_mb: 10,
        };
        
        assert!(validator.validate_resource_usage(&usage).is_err());
    }
    
    #[test]
    fn test_safe_path_checking() {
        let security_config = create_test_security_config();
        let resource_limits = ResourceLimits::default();
        let validator = SecurityValidator::new(security_config, resource_limits);
        
        // Test safe path
        let safe_path = PathBuf::from("src/main.rs");
        assert!(validator.is_safe_read_path(&safe_path));
        
        // Test unsafe path (path traversal)
        let unsafe_path = PathBuf::from("../etc/passwd");
        assert!(!validator.is_safe_read_path(&unsafe_path));
        
        // Test unsafe absolute path
        let unsafe_path = PathBuf::from("/etc/passwd");
        assert!(!validator.is_safe_read_path(&unsafe_path));
    }
    
    #[test]
    fn test_string_sanitization() {
        let security_config = create_test_security_config();
        let resource_limits = ResourceLimits::default();
        let validator = SecurityValidator::new(security_config, resource_limits);
        
        let input = "<script>alert('xss')</script>";
        let sanitized = validator.sanitize_string(input);
        assert!(!sanitized.contains("<script>"));
        assert!(sanitized.contains("&lt;script&gt;"));
    }
}
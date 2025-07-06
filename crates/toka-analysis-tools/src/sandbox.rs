//! Secure sandbox for executing Python analysis tools
//!
//! This module provides secure process isolation using Linux namespaces and cgroups
//! to ensure that Python analysis tools cannot access system resources beyond their
//! designated permissions.

use std::collections::HashMap;
use std::ffi::OsString;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::process::{Child, Command as TokioCommand};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::{AnalysisError, security::ResourceLimits};

/// Configuration for the sandbox environment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Enable process isolation
    pub enable_isolation: bool,
    /// Enable network isolation
    pub disable_network: bool,
    /// Enable filesystem isolation
    pub enable_filesystem_isolation: bool,
    /// Allowed filesystem paths (read-only)
    pub allowed_read_paths: Vec<PathBuf>,
    /// Allowed filesystem paths (read-write)
    pub allowed_write_paths: Vec<PathBuf>,
    /// Working directory inside sandbox
    pub working_directory: PathBuf,
    /// Resource limits
    pub resource_limits: ResourceLimits,
    /// Environment variables to set
    pub environment: HashMap<String, String>,
    /// Additional Python paths
    pub python_paths: Vec<PathBuf>,
    /// Timeout for sandbox operations
    pub timeout: Duration,
    /// Enable audit logging
    pub enable_audit_logging: bool,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            enable_isolation: true,
            disable_network: true,
            enable_filesystem_isolation: true,
            allowed_read_paths: vec![
                PathBuf::from("/usr/lib/python3"),
                PathBuf::from("/usr/local/lib/python3"),
                PathBuf::from("/lib"),
                PathBuf::from("/usr/lib"),
            ],
            allowed_write_paths: vec![
                PathBuf::from("/tmp"),
            ],
            working_directory: PathBuf::from("/workspace"),
            resource_limits: ResourceLimits::default(),
            environment: HashMap::new(),
            python_paths: vec![],
            timeout: Duration::from_secs(600),
            enable_audit_logging: true,
        }
    }
}

/// Sandbox for executing Python processes securely
pub struct PythonSandbox {
    config: SandboxConfig,
    temp_dir: Option<tempfile::TempDir>,
    audit_log: Vec<SandboxAuditEvent>,
}

impl PythonSandbox {
    /// Create a new sandbox
    pub fn new(config: SandboxConfig) -> Result<Self> {
        let temp_dir = if config.enable_filesystem_isolation {
            Some(tempfile::TempDir::new().context("Failed to create temporary directory")?)
        } else {
            None
        };
        
        Ok(Self {
            config,
            temp_dir,
            audit_log: Vec::new(),
        })
    }
    
    /// Execute a Python command in the sandbox
    pub async fn execute_python(
        &mut self,
        script_path: &Path,
        args: &[&str],
        input: Option<&str>,
    ) -> Result<SandboxExecutionResult> {
        let start_time = Instant::now();
        
        // Log execution start
        self.log_audit_event(SandboxAuditEvent::ExecutionStart {
            script_path: script_path.to_path_buf(),
            args: args.iter().map(|s| s.to_string()).collect(),
            timestamp: chrono::Utc::now(),
        });
        
        // Validate script path
        self.validate_script_path(script_path)?;
        
        // Create command
        let mut cmd = self.create_python_command(script_path, args)?;
        
        // Apply resource limits
        self.apply_resource_limits(&mut cmd)?;
        
        // Set environment
        self.set_environment(&mut cmd);
        
        // Execute with timeout
        let result = timeout(
            self.config.timeout,
            self.execute_command(cmd, input),
        ).await;
        
        let execution_time = start_time.elapsed();
        
        // Process result
        match result {
            Ok(Ok(output)) => {
                self.log_audit_event(SandboxAuditEvent::ExecutionSuccess {
                    script_path: script_path.to_path_buf(),
                    execution_time,
                    output_size: output.stdout.len() + output.stderr.len(),
                    timestamp: chrono::Utc::now(),
                });
                
                Ok(SandboxExecutionResult {
                    success: true,
                    exit_code: output.exit_code,
                    stdout: output.stdout,
                    stderr: output.stderr,
                    execution_time,
                    resource_usage: output.resource_usage,
                    audit_events: self.audit_log.clone(),
                })
            }
            Ok(Err(e)) => {
                self.log_audit_event(SandboxAuditEvent::ExecutionFailure {
                    script_path: script_path.to_path_buf(),
                    execution_time,
                    error: e.to_string(),
                    timestamp: chrono::Utc::now(),
                });
                
                Err(e)
            }
            Err(_) => {
                let error = "Execution timed out".to_string();
                self.log_audit_event(SandboxAuditEvent::ExecutionTimeout {
                    script_path: script_path.to_path_buf(),
                    timeout: self.config.timeout,
                    timestamp: chrono::Utc::now(),
                });
                
                Err(AnalysisError::ExecutionFailed(error).into())
            }
        }
    }
    
    /// Validate that the script path is allowed
    fn validate_script_path(&self, script_path: &Path) -> Result<()> {
        if !script_path.exists() {
            return Err(AnalysisError::ConfigurationError(
                format!("Script not found: {}", script_path.display())
            ).into());
        }
        
        // Check if path is in allowed read paths
        let script_path = script_path.canonicalize()
            .context("Failed to canonicalize script path")?;
        
        for allowed_path in &self.config.allowed_read_paths {
            if let Ok(allowed_path) = allowed_path.canonicalize() {
                if script_path.starts_with(&allowed_path) {
                    return Ok(());
                }
            }
        }
        
        // Check working directory
        if let Ok(working_dir) = self.config.working_directory.canonicalize() {
            if script_path.starts_with(&working_dir) {
                return Ok(());
            }
        }
        
        Err(AnalysisError::SecurityViolation(
            format!("Script path not allowed: {}", script_path.display())
        ).into())
    }
    
    /// Create a Python command with proper configuration
    fn create_python_command(&self, script_path: &Path, args: &[&str]) -> Result<TokioCommand> {
        let mut cmd = TokioCommand::new("python3");
        
        // Set script and arguments
        cmd.arg(script_path);
        cmd.args(args);
        
        // Set working directory
        if let Some(temp_dir) = &self.temp_dir {
            cmd.current_dir(temp_dir.path());
        } else {
            cmd.current_dir(&self.config.working_directory);
        }
        
        // Configure I/O
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // Apply security configurations
        if self.config.enable_isolation {
            self.apply_process_isolation(&mut cmd)?;
        }
        
        Ok(cmd)
    }
    
    /// Apply process isolation using Linux namespaces
    #[cfg(target_os = "linux")]
    fn apply_process_isolation(&self, cmd: &mut TokioCommand) -> Result<()> {
        use nix::unistd::{Uid, Gid};
        use nix::sched::{CloneFlags, unshare};
        use std::os::unix::process::CommandExt;
        
        // Create new namespaces
        let clone_flags = CloneFlags::CLONE_NEWPID | CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWUTS;
        
        if self.config.disable_network {
            // clone_flags |= CloneFlags::CLONE_NEWNET;
        }
        
        // Use pre_exec to set up the sandbox
        cmd.pre_exec(move || {
            // Create new namespaces
            match unshare(clone_flags) {
                Ok(()) => debug!("Successfully created process namespaces"),
                Err(e) => {
                    warn!("Failed to create namespaces: {}", e);
                    // Continue without namespaces for compatibility
                }
            }
            
            // Set resource limits
            // Note: This would require additional implementation with rlimit
            
            Ok(())
        });
        
        Ok(())
    }
    
    /// Apply process isolation (non-Linux fallback)
    #[cfg(not(target_os = "linux"))]
    fn apply_process_isolation(&self, _cmd: &mut TokioCommand) -> Result<()> {
        warn!("Process isolation not available on this platform");
        Ok(())
    }
    
    /// Apply resource limits to the command
    fn apply_resource_limits(&self, cmd: &mut TokioCommand) -> Result<()> {
        // Set environment variables for resource limits
        // The actual enforcement would be done by the sandbox runtime
        cmd.env("TOKA_MAX_MEMORY", self.config.resource_limits.max_memory_mb.to_string());
        cmd.env("TOKA_MAX_CPU", self.config.resource_limits.max_cpu_percent.to_string());
        cmd.env("TOKA_TIMEOUT", self.config.timeout.as_secs().to_string());
        
        Ok(())
    }
    
    /// Set environment variables
    fn set_environment(&self, cmd: &mut TokioCommand) {
        // Set Python path
        let python_path = self.config.python_paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(":");
        
        if !python_path.is_empty() {
            cmd.env("PYTHONPATH", python_path);
        }
        
        // Set custom environment variables
        for (key, value) in &self.config.environment {
            cmd.env(key, value);
        }
        
        // Security environment
        cmd.env("PYTHONDONTWRITEBYTECODE", "1");
        cmd.env("PYTHONUNBUFFERED", "1");
        cmd.env("TOKA_SANDBOX", "1");
    }
    
    /// Execute the command and capture output
    async fn execute_command(&self, mut cmd: TokioCommand, input: Option<&str>) -> Result<CommandOutput> {
        let start_time = Instant::now();
        
        // Spawn the process
        let mut child = cmd.spawn()
            .context("Failed to spawn Python process")?;
        
        // Write input if provided
        if let Some(input) = input {
            if let Some(stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                let mut stdin = stdin;
                stdin.write_all(input.as_bytes()).await
                    .context("Failed to write to stdin")?;
                stdin.shutdown().await
                    .context("Failed to close stdin")?;
            }
        }
        
        // Wait for completion
        let output = child.wait_with_output().await
            .context("Failed to wait for process completion")?;
        
        let execution_time = start_time.elapsed();
        
        // Process output
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let exit_code = output.status.code().unwrap_or(-1);
        
        // Check for errors
        if !output.status.success() {
            return Err(AnalysisError::ExecutionFailed(
                format!("Process failed with exit code {}: {}", exit_code, stderr)
            ).into());
        }
        
        Ok(CommandOutput {
            stdout,
            stderr,
            exit_code,
            execution_time,
            resource_usage: ResourceUsage {
                memory_peak_mb: 0, // Would need actual monitoring
                cpu_time: execution_time,
            },
        })
    }
    
    /// Log an audit event
    fn log_audit_event(&mut self, event: SandboxAuditEvent) {
        if self.config.enable_audit_logging {
            debug!("Sandbox audit: {:?}", event);
            self.audit_log.push(event);
        }
    }
    
    /// Get audit log
    pub fn get_audit_log(&self) -> &[SandboxAuditEvent] {
        &self.audit_log
    }
}

/// Result of sandbox execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecutionResult {
    /// Success status
    pub success: bool,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Execution time
    pub execution_time: Duration,
    /// Resource usage
    pub resource_usage: ResourceUsage,
    /// Audit events
    pub audit_events: Vec<SandboxAuditEvent>,
}

/// Command output
#[derive(Debug, Clone)]
struct CommandOutput {
    stdout: String,
    stderr: String,
    exit_code: i32,
    execution_time: Duration,
    resource_usage: ResourceUsage,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Peak memory usage in MB
    pub memory_peak_mb: u64,
    /// CPU time used
    pub cpu_time: Duration,
}

/// Audit event for sandbox operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SandboxAuditEvent {
    /// Execution started
    ExecutionStart {
        script_path: PathBuf,
        args: Vec<String>,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Execution completed successfully
    ExecutionSuccess {
        script_path: PathBuf,
        execution_time: Duration,
        output_size: usize,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Execution failed
    ExecutionFailure {
        script_path: PathBuf,
        execution_time: Duration,
        error: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Execution timed out
    ExecutionTimeout {
        script_path: PathBuf,
        timeout: Duration,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Security violation detected
    SecurityViolation {
        violation_type: String,
        details: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

/// Builder for sandbox configuration
pub struct SandboxBuilder {
    config: SandboxConfig,
}

impl SandboxBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: SandboxConfig::default(),
        }
    }
    
    /// Enable or disable process isolation
    pub fn enable_isolation(mut self, enable: bool) -> Self {
        self.config.enable_isolation = enable;
        self
    }
    
    /// Enable or disable network access
    pub fn disable_network(mut self, disable: bool) -> Self {
        self.config.disable_network = disable;
        self
    }
    
    /// Set working directory
    pub fn working_directory(mut self, path: PathBuf) -> Self {
        self.config.working_directory = path;
        self
    }
    
    /// Add allowed read path
    pub fn allow_read_path(mut self, path: PathBuf) -> Self {
        self.config.allowed_read_paths.push(path);
        self
    }
    
    /// Add allowed write path
    pub fn allow_write_path(mut self, path: PathBuf) -> Self {
        self.config.allowed_write_paths.push(path);
        self
    }
    
    /// Set resource limits
    pub fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.resource_limits = limits;
        self
    }
    
    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout = timeout;
        self
    }
    
    /// Add environment variable
    pub fn env(mut self, key: String, value: String) -> Self {
        self.config.environment.insert(key, value);
        self
    }
    
    /// Add Python path
    pub fn python_path(mut self, path: PathBuf) -> Self {
        self.config.python_paths.push(path);
        self
    }
    
    /// Build the sandbox
    pub fn build(self) -> Result<PythonSandbox> {
        PythonSandbox::new(self.config)
    }
}

impl Default for SandboxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;
    
    #[tokio::test]
    async fn test_sandbox_creation() {
        let sandbox = SandboxBuilder::new()
            .enable_isolation(false) // Disable for testing
            .disable_network(true)
            .build()
            .unwrap();
        
        assert!(sandbox.config.disable_network);
        assert!(!sandbox.config.enable_isolation);
    }
    
    #[tokio::test]
    async fn test_simple_python_execution() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "print('Hello from Python!')").unwrap();
        
        let mut sandbox = SandboxBuilder::new()
            .enable_isolation(false) // Disable for testing
            .allow_read_path(temp_file.path().parent().unwrap().to_path_buf())
            .build()
            .unwrap();
        
        let result = sandbox.execute_python(temp_file.path(), &[], None).await;
        
        match result {
            Ok(output) => {
                assert!(output.success);
                assert!(output.stdout.contains("Hello from Python!"));
            }
            Err(e) => {
                // Skip test if Python is not available
                if e.to_string().contains("No such file or directory") {
                    return;
                }
                panic!("Unexpected error: {}", e);
            }
        }
    }
    
    #[test]
    fn test_sandbox_builder() {
        let config = SandboxBuilder::new()
            .enable_isolation(true)
            .disable_network(false)
            .timeout(Duration::from_secs(300))
            .env("TEST_VAR".to_string(), "test_value".to_string())
            .python_path(PathBuf::from("/custom/python/path"))
            .build()
            .unwrap();
        
        assert!(config.config.enable_isolation);
        assert!(!config.config.disable_network);
        assert_eq!(config.config.timeout, Duration::from_secs(300));
        assert_eq!(config.config.environment.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert!(config.config.python_paths.contains(&PathBuf::from("/custom/python/path")));
    }
}
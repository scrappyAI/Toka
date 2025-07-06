//! Security context for tool execution validation
//!
//! Provides security validation and boundary enforcement for all tool operations.
//! Ensures that tools cannot escape their designated security perimeter.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Context, Result};

use super::{ExecutionContext, SecurityLevel, KernelError};

/// Security context for validating tool operations
pub struct SecurityContext {
    /// Security policies indexed by security level
    policies: RwLock<HashMap<SecurityLevel, SecurityPolicy>>,
    /// Active security violations for monitoring
    violations: RwLock<Vec<SecurityViolation>>,
}

/// Security policy defining allowed operations for a security level
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    /// Allowed file system paths
    pub allowed_paths: Vec<String>,
    /// Blocked file system paths
    pub blocked_paths: Vec<String>,
    /// Allowed network hosts
    pub allowed_hosts: Vec<String>,
    /// Allowed network ports
    pub allowed_ports: Vec<u16>,
    /// Maximum file size for operations (bytes)
    pub max_file_size: u64,
    /// Whether dynamic code execution is allowed
    pub allow_code_execution: bool,
    /// Whether process spawning is allowed
    pub allow_process_spawn: bool,
}

/// Security violation record for auditing
#[derive(Debug, Clone)]
pub struct SecurityViolation {
    pub tool_id: String,
    pub session_id: String,
    pub violation_type: ViolationType,
    pub description: String,
    pub timestamp: std::time::SystemTime,
}

/// Types of security violations
#[derive(Debug, Clone)]
pub enum ViolationType {
    UnauthorizedFileAccess,
    UnauthorizedNetworkAccess,
    UnauthorizedProcessSpawn,
    ResourceLimitExceeded,
    CodeExecutionViolation,
    CapabilityViolation,
}

impl SecurityContext {
    /// Create new security context with default policies
    pub async fn new() -> Result<Self> {
        let mut policies = HashMap::new();
        
        // Sandboxed security policy - most restrictive
        policies.insert(SecurityLevel::Sandboxed, SecurityPolicy {
            allowed_paths: vec![
                "./temp".to_string(),
                "./sandbox".to_string(),
            ],
            blocked_paths: vec![
                "/etc".to_string(),
                "/proc".to_string(),
                "/sys".to_string(),
                "/dev".to_string(),
                "/root".to_string(),
                "/home".to_string(),
            ],
            allowed_hosts: vec![], // No network access
            allowed_ports: vec![],
            max_file_size: 1024 * 1024, // 1MB
            allow_code_execution: false,
            allow_process_spawn: false,
        });
        
        // Restricted security policy - moderate restrictions
        policies.insert(SecurityLevel::Restricted, SecurityPolicy {
            allowed_paths: vec![
                "./".to_string(),
                "/tmp".to_string(),
            ],
            blocked_paths: vec![
                "/etc/passwd".to_string(),
                "/etc/shadow".to_string(),
                "/proc".to_string(),
                "/sys".to_string(),
                "/dev".to_string(),
                "/root".to_string(),
            ],
            allowed_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "api.github.com".to_string(),
            ],
            allowed_ports: vec![80, 443, 8080],
            max_file_size: 10 * 1024 * 1024, // 10MB
            allow_code_execution: true,
            allow_process_spawn: false,
        });
        
        // Privileged security policy - minimal restrictions
        policies.insert(SecurityLevel::Privileged, SecurityPolicy {
            allowed_paths: vec!["*".to_string()], // Global access
            blocked_paths: vec![],
            allowed_hosts: vec!["*".to_string()], // Global access
            allowed_ports: vec![], // All ports allowed
            max_file_size: 100 * 1024 * 1024, // 100MB
            allow_code_execution: true,
            allow_process_spawn: true,
        });
        
        Ok(Self {
            policies: RwLock::new(policies),
            violations: RwLock::new(Vec::new()),
        })
    }
    
    /// Validate that an operation is allowed for the given context
    pub async fn validate_operation(&self, context: &ExecutionContext) -> Result<(), KernelError> {
        let policies = self.policies.read().await;
        let policy = policies.get(&context.security_level)
            .ok_or_else(|| KernelError::SecurityViolation {
                reason: format!("No policy found for security level: {:?}", context.security_level)
            })?;
        
        // Basic validation - more specific validations are done per-operation
        if context.started_at.elapsed() > context.resource_limits.max_execution_time {
            self.record_violation(context, ViolationType::ResourceLimitExceeded,
                "Execution time limit exceeded".to_string()).await;
            return Err(KernelError::ExecutionTimeout {
                duration: context.started_at.elapsed()
            });
        }
        
        Ok(())
    }
    
    /// Validate file access operation
    pub async fn validate_file_access(
        &self,
        context: &ExecutionContext,
        operation: &str,
        path: &str,
    ) -> Result<(), KernelError> {
        let policies = self.policies.read().await;
        let policy = policies.get(&context.security_level)
            .ok_or_else(|| KernelError::SecurityViolation {
                reason: "No security policy found".to_string()
            })?;
        
        // Check if path is explicitly blocked
        for blocked in &policy.blocked_paths {
            if path.starts_with(blocked) {
                self.record_violation(context, ViolationType::UnauthorizedFileAccess,
                    format!("Access to blocked path: {}", path)).await;
                return Err(KernelError::SecurityViolation {
                    reason: format!("Access denied to blocked path: {}", path)
                });
            }
        }
        
        // Check if path is allowed
        let mut allowed = false;
        for allowed_path in &policy.allowed_paths {
            if allowed_path == "*" || path.starts_with(allowed_path) {
                allowed = true;
                break;
            }
        }
        
        if !allowed {
            self.record_violation(context, ViolationType::UnauthorizedFileAccess,
                format!("Access to unauthorized path: {}", path)).await;
            return Err(KernelError::SecurityViolation {
                reason: format!("Access denied to path: {}", path)
            });
        }
        
        // Additional validation for file size on write operations
        if operation == "write" || operation == "create" {
            if let Ok(metadata) = std::fs::metadata(path) {
                if metadata.len() > policy.max_file_size {
                    self.record_violation(context, ViolationType::ResourceLimitExceeded,
                        format!("File size exceeds limit: {} bytes", metadata.len())).await;
                    return Err(KernelError::SecurityViolation {
                        reason: format!("File size exceeds limit: {} bytes", metadata.len())
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Validate network access operation
    pub async fn validate_network_access(
        &self,
        context: &ExecutionContext,
        operation: &str,
        host: &str,
        port: u16,
    ) -> Result<(), KernelError> {
        let policies = self.policies.read().await;
        let policy = policies.get(&context.security_level)
            .ok_or_else(|| KernelError::SecurityViolation {
                reason: "No security policy found".to_string()
            })?;
        
        // Check if host is allowed
        let mut host_allowed = false;
        for allowed_host in &policy.allowed_hosts {
            if allowed_host == "*" || allowed_host == host {
                host_allowed = true;
                break;
            }
        }
        
        if !host_allowed {
            self.record_violation(context, ViolationType::UnauthorizedNetworkAccess,
                format!("Access to unauthorized host: {}", host)).await;
            return Err(KernelError::SecurityViolation {
                reason: format!("Network access denied to host: {}", host)
            });
        }
        
        // Check if port is allowed (empty list means all ports allowed)
        if !policy.allowed_ports.is_empty() && !policy.allowed_ports.contains(&port) {
            self.record_violation(context, ViolationType::UnauthorizedNetworkAccess,
                format!("Access to unauthorized port: {}", port)).await;
            return Err(KernelError::SecurityViolation {
                reason: format!("Network access denied to port: {}", port)
            });
        }
        
        Ok(())
    }
    
    /// Validate process spawning operation
    pub async fn validate_process_spawn(
        &self,
        context: &ExecutionContext,
        command: &str,
    ) -> Result<(), KernelError> {
        let policies = self.policies.read().await;
        let policy = policies.get(&context.security_level)
            .ok_or_else(|| KernelError::SecurityViolation {
                reason: "No security policy found".to_string()
            })?;
        
        if !policy.allow_process_spawn {
            self.record_violation(context, ViolationType::UnauthorizedProcessSpawn,
                format!("Process spawn not allowed: {}", command)).await;
            return Err(KernelError::SecurityViolation {
                reason: "Process spawning not allowed for this security level".to_string()
            });
        }
        
        // Additional validation for dangerous commands
        let dangerous_commands = ["rm", "sudo", "su", "chmod", "chown", "dd", "mkfs"];
        for dangerous in &dangerous_commands {
            if command.contains(dangerous) {
                self.record_violation(context, ViolationType::UnauthorizedProcessSpawn,
                    format!("Dangerous command detected: {}", command)).await;
                return Err(KernelError::SecurityViolation {
                    reason: format!("Dangerous command not allowed: {}", dangerous)
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate code execution operation
    pub async fn validate_code_execution(
        &self,
        context: &ExecutionContext,
        code_type: &str,
    ) -> Result<(), KernelError> {
        let policies = self.policies.read().await;
        let policy = policies.get(&context.security_level)
            .ok_or_else(|| KernelError::SecurityViolation {
                reason: "No security policy found".to_string()
            })?;
        
        if !policy.allow_code_execution {
            self.record_violation(context, ViolationType::CodeExecutionViolation,
                format!("Code execution not allowed: {}", code_type)).await;
            return Err(KernelError::SecurityViolation {
                reason: "Code execution not allowed for this security level".to_string()
            });
        }
        
        Ok(())
    }
    
    /// Record a security violation for auditing
    async fn record_violation(
        &self,
        context: &ExecutionContext,
        violation_type: ViolationType,
        description: String,
    ) {
        let violation = SecurityViolation {
            tool_id: context.tool_id.clone(),
            session_id: context.session_id.clone(),
            violation_type,
            description,
            timestamp: std::time::SystemTime::now(),
        };
        
        let mut violations = self.violations.write().await;
        violations.push(violation);
        
        // Keep only recent violations (last 1000)
        if violations.len() > 1000 {
            violations.drain(0..100);
        }
    }
    
    /// Get recent security violations for monitoring
    pub async fn get_recent_violations(&self) -> Vec<SecurityViolation> {
        let violations = self.violations.read().await;
        violations.clone()
    }
    
    /// Update security policy for a level
    pub async fn update_policy(
        &self,
        level: SecurityLevel,
        policy: SecurityPolicy,
    ) -> Result<()> {
        let mut policies = self.policies.write().await;
        policies.insert(level, policy);
        Ok(())
    }
    
    /// Get current security policy for a level
    pub async fn get_policy(&self, level: &SecurityLevel) -> Option<SecurityPolicy> {
        let policies = self.policies.read().await;
        policies.get(level).cloned()
    }
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        Self {
            allowed_paths: vec!["./".to_string()],
            blocked_paths: vec![],
            allowed_hosts: vec!["localhost".to_string()],
            allowed_ports: vec![80, 443],
            max_file_size: 1024 * 1024, // 1MB
            allow_code_execution: false,
            allow_process_spawn: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_security_context_creation() {
        let context = SecurityContext::new().await.unwrap();
        
        // Should have policies for all security levels
        assert!(context.get_policy(&SecurityLevel::Sandboxed).await.is_some());
        assert!(context.get_policy(&SecurityLevel::Restricted).await.is_some());
        assert!(context.get_policy(&SecurityLevel::Privileged).await.is_some());
    }
    
    #[tokio::test]
    async fn test_file_access_validation() {
        let context = SecurityContext::new().await.unwrap();
        let exec_context = ExecutionContext {
            tool_id: "test".to_string(),
            session_id: "test_session".to_string(),
            capabilities: crate::tools::capabilities::CapabilitySet::new(),
            resource_limits: crate::tools::ResourceLimits::default(),
            security_level: SecurityLevel::Sandboxed,
            started_at: std::time::Instant::now(),
        };
        
        // Should allow access to sandbox directory
        assert!(context.validate_file_access(&exec_context, "read", "./sandbox/test.txt").await.is_ok());
        
        // Should deny access to system directories
        assert!(context.validate_file_access(&exec_context, "read", "/etc/passwd").await.is_err());
    }
    
    #[tokio::test]
    async fn test_network_access_validation() {
        let context = SecurityContext::new().await.unwrap();
        let exec_context = ExecutionContext {
            tool_id: "test".to_string(),
            session_id: "test_session".to_string(),
            capabilities: crate::tools::capabilities::CapabilitySet::new(),
            resource_limits: crate::tools::ResourceLimits::default(),
            security_level: SecurityLevel::Restricted,
            started_at: std::time::Instant::now(),
        };
        
        // Should allow access to allowed hosts
        assert!(context.validate_network_access(&exec_context, "connect", "localhost", 80).await.is_ok());
        
        // Should deny access to unauthorized hosts
        assert!(context.validate_network_access(&exec_context, "connect", "evil.com", 80).await.is_err());
    }
} 
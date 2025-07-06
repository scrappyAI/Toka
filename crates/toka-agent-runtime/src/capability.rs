//! Capability validation and permission checking for agent runtime.
//!
//! This module provides security validation by checking agent operations against
//! declared capabilities and enforcing the principle of least privilege.

use std::collections::HashSet;

use anyhow::Result;
use tracing::{debug, warn};

use toka_orchestration::SecurityConfig;
use crate::AgentRuntimeError;

/// Validates agent actions against declared capabilities
pub struct CapabilityValidator {
    /// Capabilities declared by the agent
    declared_capabilities: HashSet<String>,
    /// Security configuration
    security_config: SecurityConfig,
    /// Available tools based on capabilities
    available_tools: Vec<String>,
}

impl CapabilityValidator {
    /// Create a new capability validator
    pub fn new(
        capabilities: Vec<String>,
        security_config: SecurityConfig,
    ) -> Self {
        let declared_capabilities: HashSet<String> = capabilities.into_iter().collect();
        let available_tools = Self::map_capabilities_to_tools(&declared_capabilities);

        debug!("Created capability validator with {} capabilities", 
               declared_capabilities.len());

        Self {
            declared_capabilities,
            security_config,
            available_tools,
        }
    }

    /// Check if agent can perform a specific action
    pub fn can_perform(&self, capability: &str) -> Result<bool> {
        let has_capability = self.declared_capabilities.contains(capability);

        if !has_capability {
            warn!("Capability denied: {} not in declared capabilities {:?}", 
                  capability, self.declared_capabilities);
        } else {
            debug!("Capability approved: {}", capability);
        }

        Ok(has_capability)
    }

    /// Validate capability requirements for an operation
    pub fn validate_operation(&self, operation: &str, required_capabilities: &[String]) -> Result<()> {
        for capability in required_capabilities {
            if !self.can_perform(capability)? {
                return Err(AgentRuntimeError::CapabilityDenied {
                    capability: capability.clone(),
                    operation: operation.to_string(),
                }.into());
            }
        }
        Ok(())
    }

    /// Get list of available tools based on capabilities
    pub fn get_available_tools(&self) -> Vec<String> {
        self.available_tools.clone()
    }

    /// Check if agent is running in sandbox mode
    pub fn is_sandboxed(&self) -> bool {
        self.security_config.sandbox
    }

    /// Get all declared capabilities
    pub fn get_declared_capabilities(&self) -> Vec<String> {
        self.declared_capabilities.iter().cloned().collect()
    }

    /// Map capabilities to available tools
    fn map_capabilities_to_tools(capabilities: &HashSet<String>) -> Vec<String> {
        let mut tools = Vec::new();

        for capability in capabilities {
            match capability.as_str() {
                "filesystem-read" => {
                    tools.extend_from_slice(&[
                        "cat".to_string(),
                        "ls".to_string(),
                        "find".to_string(),
                        "grep".to_string(),
                        "head".to_string(),
                        "tail".to_string(),
                    ]);
                }
                "filesystem-write" => {
                    tools.extend_from_slice(&[
                        "touch".to_string(),
                        "mkdir".to_string(),
                        "mv".to_string(),
                        "cp".to_string(),
                        "rm".to_string(),
                        "echo".to_string(),
                    ]);
                }
                "cargo-execution" => {
                    tools.extend_from_slice(&[
                        "cargo".to_string(),
                        "rustc".to_string(),
                        "rustfmt".to_string(),
                        "clippy".to_string(),
                    ]);
                }
                "git-access" => {
                    tools.extend_from_slice(&[
                        "git".to_string(),
                    ]);
                }
                "network-access" => {
                    tools.extend_from_slice(&[
                        "curl".to_string(),
                        "wget".to_string(),
                    ]);
                }
                "database-access" => {
                    tools.extend_from_slice(&[
                        "sqlite3".to_string(),
                        "psql".to_string(),
                    ]);
                }
                "ci-integration" => {
                    tools.extend_from_slice(&[
                        "ci-tools".to_string(),
                    ]);
                }
                "test-execution" => {
                    tools.extend_from_slice(&[
                        "cargo test".to_string(),
                        "cargo bench".to_string(),
                    ]);
                }
                "build-tools" => {
                    tools.extend_from_slice(&[
                        "make".to_string(),
                        "cmake".to_string(),
                        "ninja".to_string(),
                    ]);
                }
                "security-tools" => {
                    tools.extend_from_slice(&[
                        "openssl".to_string(),
                        "gpg".to_string(),
                    ]);
                }
                _ => {
                    // Generic tool based on capability name
                    tools.push(capability.clone());
                }
            }
        }

        // Remove duplicates and sort
        tools.sort();
        tools.dedup();
        tools
    }

    /// Validate file system operation
    pub fn validate_filesystem_operation(&self, path: &str, operation: FileSystemOperation) -> Result<()> {
        match operation {
            FileSystemOperation::Read => {
                self.validate_operation(
                    &format!("read file: {}", path),
                    &["filesystem-read".to_string()],
                )
            }
            FileSystemOperation::Write => {
                self.validate_operation(
                    &format!("write file: {}", path),
                    &["filesystem-write".to_string()],
                )
            }
            FileSystemOperation::Execute => {
                self.validate_operation(
                    &format!("execute file: {}", path),
                    &["filesystem-write".to_string()], // Write permission implies execute
                )
            }
        }
    }

    /// Validate network operation
    pub fn validate_network_operation(&self, url: &str) -> Result<()> {
        // Check for restricted domains or patterns
        if self.is_restricted_url(url) {
            return Err(AgentRuntimeError::CapabilityDenied {
                capability: "network-access".to_string(),
                operation: format!("access restricted URL: {}", url),
            }.into());
        }

        self.validate_operation(
            &format!("network access: {}", url),
            &["network-access".to_string()],
        )
    }

    /// Check if URL is restricted
    fn is_restricted_url(&self, url: &str) -> bool {
        let restricted_patterns = [
            "localhost",
            "127.0.0.1",
            "internal",
            "admin",
        ];

        let url_lower = url.to_lowercase();
        for pattern in &restricted_patterns {
            if url_lower.contains(pattern) {
                return true;
            }
        }

        false
    }

    /// Validate command execution
    pub fn validate_command_execution(&self, command: &str, args: &[String]) -> Result<()> {
        let required_capabilities = self.infer_command_capabilities(command);
        
        self.validate_operation(
            &format!("execute command: {} {}", command, args.join(" ")),
            &required_capabilities,
        )
    }

    /// Infer required capabilities from command
    fn infer_command_capabilities(&self, command: &str) -> Vec<String> {
        match command {
            "cargo" => vec!["cargo-execution".to_string()],
            "git" => vec!["git-access".to_string()],
            "curl" | "wget" => vec!["network-access".to_string()],
            "cat" | "ls" | "find" | "grep" | "head" | "tail" => {
                vec!["filesystem-read".to_string()]
            }
            "touch" | "mkdir" | "mv" | "cp" | "rm" | "echo" => {
                vec!["filesystem-write".to_string()]
            }
            "sqlite3" | "psql" => vec!["database-access".to_string()],
            "make" | "cmake" | "ninja" => vec!["build-tools".to_string()],
            "openssl" | "gpg" => vec!["security-tools".to_string()],
            _ => {
                // For unknown commands, require explicit permission
                vec![format!("command-{}", command)]
            }
        }
    }
}

/// File system operations that can be validated
#[derive(Debug, Clone, Copy)]
pub enum FileSystemOperation {
    /// Read operation
    Read,
    /// Write operation
    Write,
    /// Execute operation
    Execute,
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_orchestration::ResourceLimits;

    fn create_test_security_config() -> SecurityConfig {
        SecurityConfig {
            sandbox: true,
            capabilities_required: vec![
                "filesystem-read".to_string(),
                "filesystem-write".to_string(),
                "cargo-execution".to_string(),
            ],
            resource_limits: ResourceLimits {
                max_memory: "100MB".to_string(),
                max_cpu: "50%".to_string(),
                timeout: "5m".to_string(),
            },
        }
    }

    #[test]
    fn test_capability_validator_creation() {
        let capabilities = vec![
            "filesystem-read".to_string(),
            "cargo-execution".to_string(),
        ];
        let security_config = create_test_security_config();

        let validator = CapabilityValidator::new(capabilities.clone(), security_config);

        assert_eq!(validator.get_declared_capabilities().len(), 2);
        assert!(validator.can_perform("filesystem-read").unwrap());
        assert!(validator.can_perform("cargo-execution").unwrap());
        assert!(!validator.can_perform("network-access").unwrap());
    }

    #[test]
    fn test_operation_validation() {
        let capabilities = vec![
            "filesystem-read".to_string(),
            "cargo-execution".to_string(),
        ];
        let security_config = create_test_security_config();
        let validator = CapabilityValidator::new(capabilities, security_config);

        // Valid operation
        assert!(validator.validate_operation(
            "read file",
            &["filesystem-read".to_string()]
        ).is_ok());

        // Invalid operation
        assert!(validator.validate_operation(
            "network request",
            &["network-access".to_string()]
        ).is_err());
    }

    #[test]
    fn test_filesystem_operation_validation() {
        let capabilities = vec![
            "filesystem-read".to_string(),
            "filesystem-write".to_string(),
        ];
        let security_config = create_test_security_config();
        let validator = CapabilityValidator::new(capabilities, security_config);

        assert!(validator.validate_filesystem_operation(
            "/tmp/test.txt",
            FileSystemOperation::Read
        ).is_ok());

        assert!(validator.validate_filesystem_operation(
            "/tmp/test.txt",
            FileSystemOperation::Write
        ).is_ok());
    }

    #[test]
    fn test_network_operation_validation() {
        let capabilities = vec!["network-access".to_string()];
        let security_config = create_test_security_config();
        let validator = CapabilityValidator::new(capabilities, security_config);

        // Valid URL
        assert!(validator.validate_network_operation("https://api.github.com").is_ok());

        // Restricted URL
        assert!(validator.validate_network_operation("http://localhost:8080").is_err());
    }

    #[test]
    fn test_command_capability_inference() {
        let capabilities = vec![
            "cargo-execution".to_string(),
            "filesystem-read".to_string(),
        ];
        let security_config = create_test_security_config();
        let validator = CapabilityValidator::new(capabilities, security_config);

        // Valid command
        assert!(validator.validate_command_execution("cargo", &["build".to_string()]).is_ok());
        assert!(validator.validate_command_execution("cat", &["file.txt".to_string()]).is_ok());

        // Invalid command
        assert!(validator.validate_command_execution("rm", &["file.txt".to_string()]).is_err());
    }

    #[test]
    fn test_tools_mapping() {
        let capabilities = vec![
            "filesystem-read".to_string(),
            "cargo-execution".to_string(),
        ];
        let security_config = create_test_security_config();
        let validator = CapabilityValidator::new(capabilities, security_config);

        let tools = validator.get_available_tools();
        assert!(tools.contains(&"cat".to_string()));
        assert!(tools.contains(&"cargo".to_string()));
        assert!(!tools.contains(&"curl".to_string()));
    }

    #[test]
    fn test_sandbox_mode() {
        let capabilities = vec!["filesystem-read".to_string()];
        let mut security_config = create_test_security_config();
        
        security_config.sandbox = true;
        let validator = CapabilityValidator::new(capabilities.clone(), security_config);
        assert!(validator.is_sandboxed());

        let mut security_config = create_test_security_config();
        security_config.sandbox = false;
        let validator = CapabilityValidator::new(capabilities, security_config);
        assert!(!validator.is_sandboxed());
    }
}
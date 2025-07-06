//! Kernel-level tool execution enforcement
//!
//! This module provides the foundational security and resource management layer
//! for all tool operations in the Toka system. It ensures that tools execute
//! within proper security boundaries while maintaining performance and dynamic
//! capabilities.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use anyhow::{Context, Result};
use thiserror::Error;

pub mod security;
pub mod resources;
pub mod execution;
pub mod capabilities;

pub use security::SecurityContext;
pub use resources::ResourceManager;
pub use execution::ExecutionMonitor;
pub use capabilities::{CapabilitySet, Capability};

/// Kernel-level tool execution enforcement
///
/// Provides secure, performant foundation for all tool operations
/// while maintaining dynamic code generation capabilities
#[derive(Clone)]
pub struct ToolKernel {
    security_context: Arc<SecurityContext>,
    resource_manager: Arc<ResourceManager>,
    execution_monitor: Arc<ExecutionMonitor>,
    capability_enforcer: Arc<CapabilityEnforcer>,
}

/// Capability-based access control enforcer
pub struct CapabilityEnforcer {
    granted_capabilities: RwLock<HashMap<String, CapabilitySet>>,
}

/// Execution context for tool operations
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub tool_id: String,
    pub session_id: String,
    pub capabilities: CapabilitySet,
    pub resource_limits: ResourceLimits,
    pub security_level: SecurityLevel,
    pub started_at: Instant,
}

/// Resource limits for tool execution
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: u8,
    pub max_execution_time: Duration,
    pub max_file_handles: u32,
    pub max_network_connections: u32,
}

/// Security levels for tool execution
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SecurityLevel {
    /// Untrusted code, maximum restrictions
    Sandboxed,
    /// Trusted code with some restrictions
    Restricted,
    /// Core system tools, minimal restrictions
    Privileged,
}

/// Kernel-level errors for tool operations
#[derive(Error, Debug)]
pub enum KernelError {
    #[error("Security validation failed: {reason}")]
    SecurityViolation { reason: String },
    
    #[error("Resource limit exceeded: {resource}")]
    ResourceExhausted { resource: String },
    
    #[error("Capability insufficient: required {required:?}")]
    InsufficientCapabilities { required: Capability },
    
    #[error("Execution timeout after {duration:?}")]
    ExecutionTimeout { duration: Duration },
    
    #[error("Tool kernel initialization failed: {context}")]
    InitializationFailure { context: String },
}

impl ToolKernel {
    /// Create a new tool kernel instance
    ///
    /// # Errors
    /// Returns `KernelError::InitializationFailure` if kernel components
    /// cannot be properly initialized
    pub async fn new() -> Result<Self, KernelError> {
        let security_context = Arc::new(SecurityContext::new().await
            .map_err(|e| KernelError::InitializationFailure { 
                context: format!("Security context: {}", e) 
            })?);
        
        let resource_manager = Arc::new(ResourceManager::new().await
            .map_err(|e| KernelError::InitializationFailure { 
                context: format!("Resource manager: {}", e) 
            })?);
        
        let execution_monitor = Arc::new(ExecutionMonitor::new().await
            .map_err(|e| KernelError::InitializationFailure { 
                context: format!("Execution monitor: {}", e) 
            })?);
        
        let capability_enforcer = Arc::new(CapabilityEnforcer::new());
        
        Ok(Self {
            security_context,
            resource_manager,
            execution_monitor,
            capability_enforcer,
        })
    }
    
    /// Create execution context for a tool
    ///
    /// Validates security requirements and resource availability
    /// before allowing tool execution to proceed
    pub async fn create_execution_context(
        &self,
        tool_id: &str,
        session_id: &str,
        required_capabilities: &CapabilitySet,
        security_level: SecurityLevel,
    ) -> Result<ExecutionContext, KernelError> {
        // Validate capabilities
        self.capability_enforcer
            .validate_capabilities(session_id, required_capabilities)
            .await?;
        
        // Check resource availability
        let resource_limits = self.resource_manager
            .allocate_resources(&security_level)
            .await
            .map_err(|e| KernelError::ResourceExhausted { 
                resource: e.to_string() 
            })?;
        
        // Create secure execution context
        let context = ExecutionContext {
            tool_id: tool_id.to_string(),
            session_id: session_id.to_string(),
            capabilities: required_capabilities.clone(),
            resource_limits,
            security_level,
            started_at: Instant::now(),
        };
        
        // Register execution with monitor
        self.execution_monitor.register_execution(&context).await;
        
        Ok(context)
    }
    
    /// Execute operation with kernel enforcement
    ///
    /// All tool operations must go through this method to ensure
    /// proper security validation and resource management
    pub async fn enforce_execution<F, T>(
        &self,
        context: &ExecutionContext,
        operation: F,
    ) -> Result<T, KernelError>
    where
        F: std::future::Future<Output = Result<T>> + Send,
    {
        // Pre-execution validation
        self.security_context.validate_operation(&context).await?;
        
        // Execute with monitoring
        let result = self.execution_monitor
            .monitor_execution(context, operation)
            .await;
        
        // Post-execution cleanup
        self.resource_manager.release_resources(&context).await;
        
        result.map_err(|e| KernelError::SecurityViolation { 
            reason: e.to_string() 
        })
    }
    
    /// Grant capabilities to a session
    ///
    /// This is typically called during agent authentication
    /// to establish what tools the agent is allowed to use
    pub async fn grant_capabilities(
        &self,
        session_id: &str,
        capabilities: CapabilitySet,
    ) -> Result<(), KernelError> {
        self.capability_enforcer
            .grant_capabilities(session_id, capabilities)
            .await;
        Ok(())
    }
    
    /// Revoke capabilities from a session
    ///
    /// Used for security updates or session termination
    pub async fn revoke_capabilities(
        &self,
        session_id: &str,
    ) -> Result<(), KernelError> {
        self.capability_enforcer
            .revoke_capabilities(session_id)
            .await;
        Ok(())
    }
    
    /// Get current system resource usage
    ///
    /// Provides visibility into system resource consumption
    /// for monitoring and optimization
    pub async fn get_resource_usage(&self) -> ResourceUsage {
        self.resource_manager.get_current_usage().await
    }
    
    /// Get execution statistics
    ///
    /// Provides insights into tool execution patterns
    /// for performance optimization and security auditing
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        self.execution_monitor.get_statistics().await
    }
}

/// Current system resource usage
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub memory_used_mb: u64,
    pub cpu_usage_percent: f32,
    pub active_file_handles: u32,
    pub active_network_connections: u32,
    pub active_executions: u32,
}

/// Execution statistics for monitoring
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_execution_time: Duration,
    pub resource_violations: u32,
    pub security_violations: u32,
}

impl CapabilityEnforcer {
    fn new() -> Self {
        Self {
            granted_capabilities: RwLock::new(HashMap::new()),
        }
    }
    
    async fn validate_capabilities(
        &self,
        session_id: &str,
        required: &CapabilitySet,
    ) -> Result<(), KernelError> {
        let granted = self.granted_capabilities.read().await;
        
        if let Some(session_caps) = granted.get(session_id) {
            for capability in required.iter() {
                if !session_caps.contains(capability) {
                    return Err(KernelError::InsufficientCapabilities { 
                        required: capability.clone() 
                    });
                }
            }
            Ok(())
        } else {
            Err(KernelError::InsufficientCapabilities { 
                required: required.iter().next().unwrap().clone() 
            })
        }
    }
    
    async fn grant_capabilities(
        &self,
        session_id: &str,
        capabilities: CapabilitySet,
    ) {
        let mut granted = self.granted_capabilities.write().await;
        granted.insert(session_id.to_string(), capabilities);
    }
    
    async fn revoke_capabilities(&self, session_id: &str) {
        let mut granted = self.granted_capabilities.write().await;
        granted.remove(session_id);
    }
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_mb: 512,
            max_cpu_percent: 50,
            max_execution_time: Duration::from_secs(30),
            max_file_handles: 100,
            max_network_connections: 10,
        }
    }
}

impl SecurityLevel {
    /// Get default resource limits for this security level
    pub fn default_resource_limits(&self) -> ResourceLimits {
        match self {
            SecurityLevel::Sandboxed => ResourceLimits {
                max_memory_mb: 128,
                max_cpu_percent: 25,
                max_execution_time: Duration::from_secs(10),
                max_file_handles: 10,
                max_network_connections: 2,
            },
            SecurityLevel::Restricted => ResourceLimits::default(),
            SecurityLevel::Privileged => ResourceLimits {
                max_memory_mb: 2048,
                max_cpu_percent: 80,
                max_execution_time: Duration::from_secs(300),
                max_file_handles: 1000,
                max_network_connections: 100,
            },
        }
    }
} 
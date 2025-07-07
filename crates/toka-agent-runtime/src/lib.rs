#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-agent-runtime** – Agent execution runtime for Toka OS.
//!
//! This crate provides the execution runtime that interprets and executes agent configurations
//! loaded from YAML files. It bridges the gap between agent orchestration and actual task
//! execution by providing:
//!
//! - **AgentExecutor**: Core agent execution loop that interprets agent configurations
//! - **TaskExecutor**: LLM-integrated task execution with security validation
//! - **AgentProcessManager**: Process lifecycle management for spawned agents
//! - **Progress Reporting**: Real-time progress updates to orchestration system
//! - **Resource Management**: CPU, memory, and timeout enforcement
//! - **Capability Validation**: Runtime permission checking against declared capabilities
//!
//! ## Architecture
//!
//! The agent runtime sits between the orchestration engine and the actual task execution:
//!
//! ```text
//! Orchestration Engine → Agent Runtime → Task Execution → LLM Integration
//!                                    ↓
//!                               Progress Reporting
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use toka_agent_runtime::{AgentExecutor, AgentProcessManager};
//! use toka_types::AgentConfig;
//! use toka_types::EntityId;
//! use std::sync::Arc;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! # // Mock runtime and LLM gateway for example
//! # let runtime: Arc<toka_runtime::Runtime> = unimplemented!();
//! # let llm_gateway: Arc<toka_llm_gateway::LlmGateway> = unimplemented!();
//! 
//! // Load agent configuration (example)
//! # let config: AgentConfig = unimplemented!();
//! 
//! // Create agent executor
//! let executor = AgentExecutor::new(
//!     config,
//!     EntityId(42),
//!     runtime,
//!     llm_gateway,
//! ).await?;
//!
//! // Run agent execution loop
//! executor.run().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Security
//!
//! The agent runtime enforces security at multiple levels:
//!
//! - **Capability Validation**: All operations validated against declared capabilities
//! - **Resource Limits**: CPU, memory, and timeout enforcement
//! - **Sandboxing**: Process isolation and restricted system access
//! - **Audit Logging**: All agent actions logged for security monitoring
//! - **LLM Safety**: Request sanitization and response validation

use std::collections::HashMap;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::error;

use toka_types::{AgentConfig};
use toka_types::{EntityId};

pub mod executor;
pub mod process;
pub mod task;
pub mod capability;
pub mod resource;
pub mod progress;

pub use executor::AgentExecutor;
pub use process::AgentProcessManager;
pub use task::TaskExecutor;
pub use capability::CapabilityValidator;
pub use resource::ResourceManager;
pub use progress::{ProgressReporter, AgentProgress, TaskResult};

/// Maximum time to wait for agent startup
pub const AGENT_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum time for task execution before timeout
pub const DEFAULT_TASK_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

/// Current execution state of an agent
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentExecutionState {
    /// Agent is initializing
    Initializing,
    /// Agent is ready to execute tasks
    Ready,
    /// Agent is actively executing a task
    ExecutingTask { 
        /// ID of the currently executing task
        task_id: String 
    },
    /// Agent is waiting for a resource or dependency
    Waiting { 
        /// Reason for waiting
        reason: String 
    },
    /// Agent is paused by user or system
    Paused,
    /// Agent execution completed successfully
    Completed,
    /// Agent execution failed
    Failed { 
        /// Error message describing the failure
        error: String 
    },
    /// Agent was terminated by user or system
    Terminated { 
        /// Reason for termination
        reason: String 
    },
}

/// Context information for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentContext {
    /// Agent entity ID
    pub agent_id: EntityId,
    /// Agent configuration
    pub config: AgentConfig,
    /// Current execution state
    pub state: AgentExecutionState,
    /// Execution start time
    pub started_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Accumulated execution metrics
    pub metrics: AgentMetrics,
    /// Environment variables and context
    pub environment: HashMap<String, String>,
}

/// Metrics collected during agent execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentMetrics {
    /// Total tasks attempted
    pub tasks_attempted: u64,
    /// Tasks completed successfully
    pub tasks_completed: u64,
    /// Tasks that failed
    pub tasks_failed: u64,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Average task execution time
    pub avg_task_time: Duration,
    /// Memory usage (bytes)
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// LLM requests made
    pub llm_requests: u64,
    /// LLM tokens consumed
    pub llm_tokens_consumed: u64,
}

/// Configuration for agent execution behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Maximum number of concurrent tasks
    pub max_concurrent_tasks: usize,
    /// Default task timeout
    pub default_task_timeout: Duration,
    /// Enable detailed execution logging
    pub verbose_logging: bool,
    /// Retry configuration
    pub retry_config: RetryConfig,
    /// Resource monitoring interval
    pub resource_check_interval: Duration,
}

/// Configuration for task retry behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries per task
    pub max_retries: u32,
    /// Base delay between retries
    pub base_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_concurrent_tasks: 3,
            default_task_timeout: DEFAULT_TASK_TIMEOUT,
            verbose_logging: false,
            retry_config: RetryConfig::default(),
            resource_check_interval: Duration::from_secs(30),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            backoff_multiplier: 2.0,
        }
    }
}

/// Trait for agent task execution
#[async_trait]
pub trait AgentTask: Send + Sync + std::fmt::Debug {
    /// Execute the task with the given context
    async fn execute(&self, context: &AgentContext) -> Result<TaskResult>;
    
    /// Get task identifier
    fn task_id(&self) -> &str;
    
    /// Get task description
    fn description(&self) -> &str;
    
    /// Get estimated execution time
    fn estimated_duration(&self) -> Option<Duration> {
        None
    }
    
    /// Check if task can be retried on failure
    fn is_retryable(&self) -> bool {
        true
    }
}

/// Error types for agent runtime operations
#[derive(Debug, thiserror::Error)]
pub enum AgentRuntimeError {
    /// Agent configuration is invalid
    #[error("invalid agent configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Agent execution failed
    #[error("agent execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Task execution timeout
    #[error("task execution timeout: {task_id} exceeded {timeout:?}")]
    TaskTimeout { 
        /// ID of the task that timed out
        task_id: String, 
        /// Timeout duration that was exceeded
        timeout: Duration 
    },
    
    /// Resource limit exceeded
    #[error("resource limit exceeded: {resource} usage {current} > limit {limit}")]
    ResourceLimitExceeded {
        /// Name of the resource that exceeded its limit
        resource: String,
        /// Current usage value
        current: String,
        /// Maximum allowed limit
        limit: String,
    },
    
    /// Capability not authorized
    #[error("capability not authorized: {capability} required for {operation}")]
    CapabilityDenied {
        /// Name of the required capability
        capability: String,
        /// Operation that was denied
        operation: String,
    },
    
    /// LLM integration error
    #[error("LLM integration error: {0}")]
    LlmError(String),
    
    /// Internal runtime error
    #[error("internal runtime error: {0}")]
    Internal(String),
}

/// Result type for agent runtime operations
pub type AgentRuntimeResult<T> = std::result::Result<T, AgentRuntimeError>;

/// Agent runtime statistics for monitoring and debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeStats {
    /// Number of active agents
    pub active_agents: u64,
    /// Total agents started
    pub total_agents_started: u64,
    /// Total agents completed
    pub total_agents_completed: u64,
    /// Total agents failed
    pub total_agents_failed: u64,
    /// Total tasks executed
    pub total_tasks_executed: u64,
    /// Average agent execution time
    pub avg_agent_execution_time: Duration,
    /// Total LLM requests across all agents
    pub total_llm_requests: u64,
    /// Total LLM tokens consumed
    pub total_llm_tokens: u64,
    /// Runtime uptime
    pub uptime: Duration,
}

impl Default for RuntimeStats {
    fn default() -> Self {
        Self {
            active_agents: 0,
            total_agents_started: 0,
            total_agents_completed: 0,
            total_agents_failed: 0,
            total_tasks_executed: 0,
            avg_agent_execution_time: Duration::ZERO,
            total_llm_requests: 0,
            total_llm_tokens: 0,
            uptime: Duration::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_execution_state_serialization() {
        let state = AgentExecutionState::ExecutingTask {
            task_id: "test-task-123".to_string(),
        };
        
        let serialized = serde_json::to_string(&state).unwrap();
        let deserialized: AgentExecutionState = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(state, deserialized);
    }

    #[test]
    fn test_agent_metrics_default() {
        let metrics = AgentMetrics::default();
        assert_eq!(metrics.tasks_attempted, 0);
        assert_eq!(metrics.tasks_completed, 0);
        assert_eq!(metrics.total_execution_time, Duration::ZERO);
    }

    #[test]
    fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert_eq!(config.max_concurrent_tasks, 3);
        assert_eq!(config.default_task_timeout, DEFAULT_TASK_TIMEOUT);
        assert!(!config.verbose_logging);
    }
}
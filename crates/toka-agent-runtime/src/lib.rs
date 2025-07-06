//! Toka Agent Runtime
//!
//! This crate provides the runtime environment for executing agents within Toka OS.
//! It includes agent lifecycle management, task execution, and integration with the
//! kernel and other system components.
//!
//! ## Features
//!
//! - **Agent Execution**: Core agent lifecycle and execution management
//! - **Task Coordination**: LLM-integrated task execution with retry logic
//! - **Progress Reporting**: Real-time progress updates to orchestration
//! - **Capability Validation**: Security validation against declared capabilities
//! - **Resource Management**: CPU, memory, and timeout enforcement
//! - **Tool Integration**: Seamless integration with toka-tools registry
//!
//! ## Security Model
//!
//! The agent runtime enforces a strict security model:
//! - **Capability Enforcement**: All operations validated against declared capabilities
//! - **Resource Limits**: CPU, memory, and timeout enforcement
//! - **Sandboxing**: Process isolation and restricted system access
//! - **Audit Logging**: All agent actions logged for security monitoring
//! - **LLM Safety**: Request sanitization and response validation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn, instrument};
use uuid::Uuid;

use toka_bus_core::KernelEvent;
use toka_llm_gateway::{LlmGateway, LlmRequest, LlmResponse};
use toka_types::{AgentConfig, TaskConfig, SecurityConfig, ResourceLimits};
use toka_runtime::Runtime;
use toka_types::{EntityId, Message, Operation, TaskSpec};

pub mod executor;
pub mod process;
pub mod task;
pub mod capability;
pub mod resource;
pub mod progress;
pub mod integration;

pub use executor::AgentExecutor;
pub use process::AgentProcessManager;
pub use task::TaskExecutor;
pub use capability::CapabilityValidator;
pub use resource::ResourceManager;
pub use progress::{ProgressReporter, AgentProgress, TaskResult};
pub use integration::{
    ToolRegistryTaskExecutor, AgentExecutorExt, ToolRegistryFactory, 
    AgentRuntimeToolIntegration
};

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
    /// Agent is currently executing a task
    ExecutingTask { task_id: String },
    /// Agent is paused/suspended
    Paused,
    /// Agent has completed all tasks
    Completed,
    /// Agent has failed and terminated
    Failed { error: String },
    /// Agent has been terminated
    Terminated { reason: String },
}

/// Agent execution context
#[derive(Debug, Clone)]
pub struct AgentContext {
    /// Agent configuration
    pub config: AgentConfig,
    /// Agent entity ID
    pub agent_id: EntityId,
    /// Current execution state
    pub state: AgentExecutionState,
    /// Environment variables for agent execution
    pub environment: HashMap<String, String>,
    /// Agent start time
    pub start_time: Instant,
}

/// Agent execution configuration
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Maximum task execution time
    pub max_task_time: Duration,
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// LLM request timeout
    pub llm_timeout: Duration,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            max_task_time: DEFAULT_TASK_TIMEOUT,
            max_retries: 3,
            llm_timeout: Duration::from_secs(30),
            resource_limits: ResourceLimits {
                max_memory_mb: 512,
                max_cpu_percent: 50.0,
                max_execution_time_minutes: 30,
            },
        }
    }
}

/// Agent execution metrics
#[derive(Debug, Clone, Default)]
pub struct AgentMetrics {
    /// Total tasks executed
    pub tasks_executed: u64,
    /// Tasks completed successfully
    pub tasks_successful: u64,
    /// Tasks that failed
    pub tasks_failed: u64,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Average task execution time
    pub avg_task_time: Duration,
    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
}

/// Runtime statistics for all agents
#[derive(Debug, Clone, Default)]
pub struct RuntimeStats {
    /// Number of active agents
    pub active_agents: u64,
    /// Total agents started
    pub total_agents_started: u64,
    /// Total agents completed
    pub total_agents_completed: u64,
    /// Total agents failed
    pub total_agents_failed: u64,
    /// Runtime start time
    pub runtime_start: Option<DateTime<Utc>>,
}

/// Agent runtime error types
#[derive(Debug, thiserror::Error)]
pub enum AgentRuntimeError {
    /// Agent execution failed
    #[error("Agent execution failed: {0}")]
    ExecutionFailed(String),
    
    /// Agent capability denied
    #[error("Capability denied: {capability} for operation: {operation}")]
    CapabilityDenied {
        capability: String,
        operation: String,
    },
    
    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} - {details}")]
    ResourceLimitExceeded {
        resource: String,
        details: String,
    },
    
    /// Task timeout
    #[error("Task execution timed out after {timeout:?}")]
    TaskTimeout {
        timeout: Duration,
    },
    
    /// LLM communication error
    #[error("LLM communication error: {0}")]
    LlmError(String),
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    /// Internal runtime error
    #[error("Internal runtime error: {0}")]
    InternalError(String),
}

/// Result type for agent runtime operations
pub type AgentRuntimeResult<T> = Result<T, AgentRuntimeError>;

/// Task execution trait for different execution strategies
#[async_trait]
pub trait AgentTask: Send + Sync {
    /// Unique task identifier
    fn task_id(&self) -> String;
    
    /// Task description
    fn description(&self) -> &str;
    
    /// Required capabilities for this task
    fn required_capabilities(&self) -> Vec<String>;
    
    /// Execute the task
    async fn execute(&self, context: &AgentContext) -> AgentRuntimeResult<TaskResult>;
    
    /// Validate task parameters
    fn validate(&self) -> AgentRuntimeResult<()>;
}

impl AgentContext {
    /// Create a new agent context
    pub fn new(config: AgentConfig, agent_id: EntityId) -> Self {
        Self {
            config,
            agent_id,
            state: AgentExecutionState::Initializing,
            environment: HashMap::new(),
            start_time: Instant::now(),
        }
    }
    
    /// Update execution state
    pub fn set_state(&mut self, state: AgentExecutionState) {
        self.state = state;
    }
    
    /// Get uptime
    pub fn uptime(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    /// Add environment variable
    pub fn add_env_var(&mut self, key: String, value: String) {
        self.environment.insert(key, value);
    }
    
    /// Get environment variable
    pub fn get_env_var(&self, key: &str) -> Option<&String> {
        self.environment.get(key)
    }
}

/// Enhanced agent runtime that integrates with the tool registry
pub struct TokaAgentRuntime {
    /// Agent process manager
    process_manager: Arc<AgentProcessManager>,
    /// Tool registry integration
    tool_integration: Option<Arc<AgentRuntimeToolIntegration>>,
    /// Runtime statistics
    stats: Arc<RwLock<RuntimeStats>>,
}

impl TokaAgentRuntime {
    /// Create a new Toka agent runtime with tool integration
    pub async fn new_with_tools(
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<Self> {
        let process_manager = Arc::new(AgentProcessManager::new(
            runtime.clone(),
            llm_gateway.clone(),
        ));
        
        // Create tool integration for development by default
        let tool_integration = Some(Arc::new(
            AgentRuntimeToolIntegration::new_development(runtime, llm_gateway).await?
        ));
        
        let stats = Arc::new(RwLock::new(RuntimeStats {
            runtime_start: Some(Utc::now()),
            ..Default::default()
        }));
        
        Ok(Self {
            process_manager,
            tool_integration,
            stats,
        })
    }
    
    /// Create a production runtime with restricted tool access
    pub async fn new_production(
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<Self> {
        let process_manager = Arc::new(AgentProcessManager::new(
            runtime.clone(),
            llm_gateway.clone(),
        ));
        
        let tool_integration = Some(Arc::new(
            AgentRuntimeToolIntegration::new_production(runtime, llm_gateway).await?
        ));
        
        let stats = Arc::new(RwLock::new(RuntimeStats {
            runtime_start: Some(Utc::now()),
            ..Default::default()
        }));
        
        Ok(Self {
            process_manager,
            tool_integration,
            stats,
        })
    }
    
    /// Get the tool integration
    pub fn tool_integration(&self) -> Option<&Arc<AgentRuntimeToolIntegration>> {
        self.tool_integration.as_ref()
    }
    
    /// Get runtime statistics
    pub async fn get_stats(&self) -> RuntimeStats {
        self.stats.read().await.clone()
    }
}
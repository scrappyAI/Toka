#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-agents** – Canonical agent system for Toka OS.
//!
//! This crate provides the unified agent system that consolidates all agent functionality
//! into a single, canonical architecture. It eliminates duplication between `toka-agent-runtime`
//! and `toka-orchestration` by providing a unified agent management system.
//!
//! # Architecture
//!
//! The canonical agent system consists of:
//!
//! - **Agent**: Core agent abstraction with unified lifecycle management
//! - **AgentManager**: Central coordinator for all agent operations
//! - **AgentExecutor**: Unified execution engine for agent tasks
//! - **AgentOrchestrator**: Multi-agent coordination and orchestration
//! - **CoordinationEngine**: Inter-agent communication and coordination
//! - **ResourceManager**: Resource allocation and enforcement
//! - **CapabilityManager**: Security and capability validation
//! - **ProgressTracker**: Real-time progress monitoring and reporting
//!
//! # Key Features
//!
//! ## Unified Agent API
//! Single interface for all agent operations:
//! - Agent creation and configuration
//! - Task execution and coordination
//! - Progress monitoring and reporting
//! - Resource management and limits
//! - Capability validation and security
//!
//! ## Orchestration Integration
//! Built-in orchestration capabilities:
//! - Multi-agent coordination
//! - Dependency resolution
//! - Workstream management
//! - Phase-based execution
//!
//! ## Runtime Integration
//! Direct integration with `toka-runtime`:
//! - Unified execution model
//! - Shared security context
//! - Resource tracking
//! - Performance monitoring
//!
//! # Usage
//!
//! ## Basic Agent Management
//!
//! ```rust,no_run
//! use toka_agents::{AgentManager, Agent, AgentConfig};
//! use toka_runtime::RuntimeManager;
//! use toka_types::EntityId;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! // Initialize agent manager
//! let runtime = RuntimeManager::new(/* ... */).await?;
//! let agent_manager = AgentManager::new(runtime).await?;
//!
//! // Create and spawn an agent
//! let agent_config = AgentConfig::from_file("agent.yaml")?;
//! let agent_id = agent_manager.spawn_agent(agent_config).await?;
//!
//! // Monitor agent progress
//! let progress = agent_manager.get_progress(&agent_id).await?;
//! println!("Agent progress: {:.1}%", progress.completion_percentage);
//!
//! // Wait for completion
//! agent_manager.wait_for_completion(&agent_id).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Multi-Agent Orchestration
//!
//! ```rust,no_run
//! use toka_agents::{AgentManager, OrchestrationConfig};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let agent_manager = AgentManager::new(/* ... */).await?;
//!
//! // Load orchestration configuration
//! let config = OrchestrationConfig::from_directory("agents/config")?;
//!
//! // Start orchestration
//! let session = agent_manager.orchestrate_agents(config).await?;
//!
//! // Monitor orchestration progress
//! while !session.is_complete().await? {
//!     let progress = session.get_progress().await?;
//!     println!("Orchestration: {:.1}% complete", progress.overall_progress);
//!     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//! }
//! # Ok(())
//! # }
//! ```
//!
//! # Security
//!
//! The canonical agent system follows security-first principles:
//!
//! - **Capability-based Access Control**: Agents must declare required capabilities
//! - **Resource Limits**: CPU, memory, and timeout enforcement
//! - **Sandbox Isolation**: Agents run in isolated environments
//! - **Audit Logging**: All agent actions are logged for security monitoring
//! - **LLM Safety**: Request sanitization and response validation
//!
//! # Performance
//!
//! The unified architecture provides several performance benefits:
//!
//! - **Reduced Overhead**: Eliminates inter-crate communication
//! - **Shared Resources**: Efficient resource pooling and management
//! - **Optimized Coordination**: Direct agent-to-agent communication
//! - **Unified Monitoring**: Single system for all performance metrics

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn, instrument};

// Re-export core types
pub use toka_types::{
    AgentConfig, TaskConfig, EntityId, TaskSpec, SecurityConfig, ResourceLimits,
    AgentMetadata, AgentSpecConfig, AgentPriority, AgentCapabilities,
    AgentObjective, AgentTasks, AgentDependencies, ReportingConfig,
};
pub use toka_bus_core::KernelEvent;
pub use toka_runtime::{RuntimeManager, ExecutionResult, ExecutionModel};
pub use toka_llm_gateway::{LlmGateway, LlmRequest, LlmResponse};

// Core modules
pub mod agent;
pub mod executor;
pub mod orchestration;
pub mod coordination;
pub mod capabilities;
pub mod resources;
pub mod progress;
pub mod lifecycle;
pub mod integration;

// Re-export key types
pub use agent::{Agent, AgentState, AgentStatus};
pub use executor::{AgentExecutor, TaskExecutor, ExecutionContext};
pub use orchestration::{AgentOrchestrator, OrchestrationConfig, OrchestrationSession};
pub use coordination::{CoordinationEngine, DependencyResolver, WorkstreamCoordinator};
pub use capabilities::{CapabilityManager, CapabilityValidator, CapabilitySet};
pub use resources::{ResourceManager, ResourceTracker, ResourceUsage};
pub use progress::{ProgressTracker, ProgressReporter, AgentProgress, TaskResult};
pub use lifecycle::{LifecycleManager, LifecycleEvent, LifecycleState};
pub use integration::{LlmIntegration, RuntimeIntegration};

/// Maximum number of agents that can be managed simultaneously
pub const MAX_CONCURRENT_AGENTS: usize = 100;

/// Default timeout for agent operations
pub const DEFAULT_AGENT_TIMEOUT: Duration = Duration::from_secs(300);

/// Default timeout for orchestration operations
pub const DEFAULT_ORCHESTRATION_TIMEOUT: Duration = Duration::from_secs(3600);

/// **Canonical Agent Manager**
///
/// The central coordinator for all agent operations in Toka OS. This manager
/// consolidates functionality from both `toka-agent-runtime` and `toka-orchestration`
/// into a single, unified system.
///
/// # Features
///
/// - **Unified Agent Management**: Single interface for all agent operations
/// - **Built-in Orchestration**: Multi-agent coordination and orchestration
/// - **Resource Management**: Centralized resource allocation and enforcement
/// - **Progress Monitoring**: Real-time progress tracking and reporting
/// - **Security Integration**: Capability validation and security enforcement
/// - **LLM Integration**: Intelligent agent coordination and task execution
///
/// # Example
///
/// ```rust,no_run
/// use toka_agents::{AgentManager, AgentConfig};
/// use toka_runtime::RuntimeManager;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let runtime = RuntimeManager::new(/* ... */).await?;
/// let agent_manager = AgentManager::new(runtime).await?;
///
/// // Spawn an agent
/// let config = AgentConfig::from_file("agent.yaml")?;
/// let agent_id = agent_manager.spawn_agent(config).await?;
///
/// // Monitor progress
/// let progress = agent_manager.get_progress(&agent_id).await?;
/// println!("Progress: {:.1}%", progress.completion_percentage);
/// # Ok(())
/// # }
/// ```
pub struct AgentManager {
    /// Active agents indexed by ID
    agents: Arc<DashMap<EntityId, Agent>>,
    /// Agent orchestrator for multi-agent coordination
    orchestrator: Arc<AgentOrchestrator>,
    /// Coordination engine for inter-agent communication
    coordination: Arc<CoordinationEngine>,
    /// Resource manager for allocation and enforcement
    resource_manager: Arc<ResourceManager>,
    /// Capability manager for security validation
    capability_manager: Arc<CapabilityManager>,
    /// Progress tracker for monitoring
    progress_tracker: Arc<ProgressTracker>,
    /// Lifecycle manager for agent state transitions
    lifecycle_manager: Arc<LifecycleManager>,
    /// Runtime integration
    runtime: Arc<RuntimeManager>,
    /// LLM integration for intelligent coordination
    llm_integration: Option<Arc<LlmIntegration>>,
    /// Manager configuration
    config: AgentManagerConfig,
}

/// Configuration for the agent manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManagerConfig {
    /// Maximum number of concurrent agents
    pub max_concurrent_agents: usize,
    /// Default timeout for agent operations
    pub default_timeout: Duration,
    /// Enable orchestration features
    pub enable_orchestration: bool,
    /// Enable LLM integration
    pub enable_llm_integration: bool,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Resource limits for agents
    pub default_resource_limits: ResourceLimits,
    /// Security configuration
    pub security_config: SecurityConfig,
}

impl Default for AgentManagerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_agents: MAX_CONCURRENT_AGENTS,
            default_timeout: DEFAULT_AGENT_TIMEOUT,
            enable_orchestration: true,
            enable_llm_integration: true,
            enable_performance_monitoring: true,
            default_resource_limits: ResourceLimits {
                max_memory: "1024MB".to_string(),
                max_cpu: "50%".to_string(),
                timeout: "300s".to_string(),
            },
            security_config: SecurityConfig {
                sandbox: true,
                capabilities_required: vec![],
                resource_limits: ResourceLimits {
                    max_memory: "1024MB".to_string(),
                    max_cpu: "50%".to_string(),
                    timeout: "300s".to_string(),
                },
            },
        }
    }
}

/// Error types for the canonical agent system
#[derive(Debug, thiserror::Error)]
pub enum AgentError {
    /// Agent not found
    #[error("Agent not found: {agent_id:?}")]
    AgentNotFound { 
        /// The agent ID that was not found
        agent_id: EntityId 
    },
    
    /// Agent already exists
    #[error("Agent already exists: {agent_id:?}")]
    AgentAlreadyExists { 
        /// The agent ID that already exists
        agent_id: EntityId 
    },
    
    /// Maximum agents reached
    #[error("Maximum number of agents reached: {max_agents}")]
    MaxAgentsReached { 
        /// The maximum number of agents allowed
        max_agents: usize 
    },
    
    /// Agent execution failed
    #[error("Agent execution failed: {agent_id:?}: {error}")]
    ExecutionFailed { 
        /// The agent ID that failed
        agent_id: EntityId, 
        /// The error message
        error: String 
    },
    
    /// Orchestration failed
    #[error("Orchestration failed: {error}")]
    OrchestrationFailed { 
        /// The error message
        error: String 
    },
    
    /// Resource allocation failed
    #[error("Resource allocation failed: {error}")]
    ResourceAllocationFailed { 
        /// The error message
        error: String 
    },
    
    /// Capability validation failed
    #[error("Capability validation failed: {error}")]
    CapabilityValidationFailed { 
        /// The error message
        error: String 
    },
    
    /// Runtime error
    #[error("Runtime error: {error}")]
    RuntimeError { 
        /// The error message
        error: String 
    },
    
    /// Configuration error
    #[error("Configuration error: {error}")]
    ConfigurationError { 
        /// The error message
        error: String 
    },
}

/// Result type for agent operations
pub type AgentResult<T> = Result<T, AgentError>;

impl From<anyhow::Error> for AgentError {
    fn from(error: anyhow::Error) -> Self {
        AgentError::RuntimeError { error: error.to_string() }
    }
}

impl AgentManager {
    /// Create a new agent manager with the given runtime
    pub async fn new(runtime: Arc<RuntimeManager>) -> Result<Self> {
        Self::with_config(runtime, AgentManagerConfig::default()).await
    }
    
    /// Create a new agent manager with custom configuration
    pub async fn with_config(
        runtime: Arc<RuntimeManager>, 
        config: AgentManagerConfig
    ) -> Result<Self> {
        info!("Initializing canonical agent manager");
        
        // Initialize core components
        let orchestrator = Arc::new(AgentOrchestrator::new().await?);
        let coordination = Arc::new(CoordinationEngine::new().await?);
        let resource_manager = Arc::new(ResourceManager::new(config.default_resource_limits.clone()).await?);
        let capability_manager = Arc::new(CapabilityManager::new(config.security_config.clone()).await?);
        let progress_tracker = Arc::new(ProgressTracker::new().await?);
        let lifecycle_manager = Arc::new(LifecycleManager::new().await?);
        
        // Initialize LLM integration if enabled
        let llm_integration = if config.enable_llm_integration {
            match LlmIntegration::new().await {
                Ok(integration) => Some(Arc::new(integration)),
                Err(e) => {
                    warn!("Failed to initialize LLM integration: {}. Continuing without LLM support.", e);
                    None
                }
            }
        } else {
            None
        };
        
        let manager = Self {
            agents: Arc::new(DashMap::new()),
            orchestrator,
            coordination,
            resource_manager,
            capability_manager,
            progress_tracker,
            lifecycle_manager,
            runtime,
            llm_integration,
            config,
        };
        
        info!("Canonical agent manager initialized successfully");
        Ok(manager)
    }
    
    /// Spawn a new agent with the given configuration
    #[instrument(skip(self, config))]
    pub async fn spawn_agent(&self, config: AgentConfig) -> AgentResult<EntityId> {
        let agent_id = EntityId(uuid::Uuid::new_v4().as_u128());
        
        // Check if we're at capacity
        if self.agents.len() >= self.config.max_concurrent_agents {
            return Err(AgentError::MaxAgentsReached { 
                max_agents: self.config.max_concurrent_agents 
            });
        }
        
        // Validate capabilities
        self.capability_manager.validate_agent_capabilities(&config)
            .await
            .map_err(|e| AgentError::CapabilityValidationFailed { error: e.to_string() })?;
        
        // Allocate resources
        let resource_allocation = self.resource_manager.allocate_resources(&config.security.resource_limits)
            .await
            .map_err(|e| AgentError::ResourceAllocationFailed { error: e.to_string() })?;
        
        // Create agent
        let agent = Agent::new(
            agent_id,
            config,
            self.runtime.clone(),
            self.llm_integration.clone(),
            resource_allocation,
        ).await?;
        
        // Register with lifecycle manager
        self.lifecycle_manager.register_agent(agent_id, &agent).await?;
        
        // Start agent execution
        let agent_clone = agent.clone();
        let progress_tracker = self.progress_tracker.clone();
        let lifecycle_manager = self.lifecycle_manager.clone();
        
        tokio::spawn(async move {
            let result = agent_clone.run().await;
            
            // Update progress and lifecycle
            match result {
                Ok(final_result) => {
                    let _ = progress_tracker.mark_completed(agent_id, final_result).await;
                    let _ = lifecycle_manager.transition_to_completed(agent_id).await;
                }
                Err(error) => {
                    let _ = progress_tracker.mark_failed(agent_id, error.to_string()).await;
                    let _ = lifecycle_manager.transition_to_failed(agent_id, error.to_string()).await;
                }
            }
        });
        
        // Store agent
        self.agents.insert(agent_id, agent);
        
        info!("Agent spawned successfully: {:?}", agent_id);
        Ok(agent_id)
    }
    
    /// Get agent progress
    pub async fn get_progress(&self, agent_id: &EntityId) -> AgentResult<AgentProgress> {
        self.progress_tracker.get_progress(agent_id).await
            .map_err(|e| AgentError::AgentNotFound { agent_id: *agent_id })
    }
    
    /// Wait for agent completion
    pub async fn wait_for_completion(&self, agent_id: &EntityId) -> AgentResult<TaskResult> {
        self.lifecycle_manager.wait_for_completion(agent_id).await
            .map_err(|e| AgentError::AgentNotFound { agent_id: *agent_id })
    }
    
    /// Start multi-agent orchestration
    pub async fn orchestrate_agents(&self, config: OrchestrationConfig) -> AgentResult<OrchestrationSession> {
        if !self.config.enable_orchestration {
            return Err(AgentError::ConfigurationError { 
                error: "Orchestration is disabled".to_string() 
            });
        }
        
        self.orchestrator.start_orchestration(config, self.clone()).await
            .map_err(|e| AgentError::OrchestrationFailed { error: e.to_string() })
    }
    
    /// Get all active agents
    pub async fn list_agents(&self) -> Vec<EntityId> {
        self.agents.iter().map(|entry| *entry.key()).collect()
    }
    
    /// Stop an agent
    pub async fn stop_agent(&self, agent_id: &EntityId) -> AgentResult<()> {
        if let Some(agent) = self.agents.get(agent_id) {
            agent.stop().await?;
            self.lifecycle_manager.transition_to_stopped(*agent_id).await?;
            Ok(())
        } else {
            Err(AgentError::AgentNotFound { agent_id: *agent_id })
        }
    }
    
    /// Get agent manager statistics
    pub async fn get_statistics(&self) -> AgentManagerStatistics {
        AgentManagerStatistics {
            total_agents: self.agents.len(),
            max_agents: self.config.max_concurrent_agents,
            active_orchestrations: self.orchestrator.active_sessions().await,
            total_resources_allocated: self.resource_manager.total_allocated().await,
            uptime: self.progress_tracker.uptime().await,
        }
    }
}

/// Statistics for the agent manager
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentManagerStatistics {
    /// Total number of active agents
    pub total_agents: usize,
    /// Maximum number of agents allowed
    pub max_agents: usize,
    /// Number of active orchestration sessions
    pub active_orchestrations: usize,
    /// Total resources allocated
    pub total_resources_allocated: u64,
    /// Manager uptime
    pub uptime: Duration,
}

// Implement Clone for AgentManager (needed for orchestration)
impl Clone for AgentManager {
    fn clone(&self) -> Self {
        Self {
            agents: self.agents.clone(),
            orchestrator: self.orchestrator.clone(),
            coordination: self.coordination.clone(),
            resource_manager: self.resource_manager.clone(),
            capability_manager: self.capability_manager.clone(),
            progress_tracker: self.progress_tracker.clone(),
            lifecycle_manager: self.lifecycle_manager.clone(),
            runtime: self.runtime.clone(),
            llm_integration: self.llm_integration.clone(),
            config: self.config.clone(),
        }
    }
}
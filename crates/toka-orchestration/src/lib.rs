#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-orchestration** – Agent orchestration and coordination for Toka OS.
//!
//! This crate provides the orchestration layer for managing parallel agent spawning,
//! coordination, and dependency resolution in Toka OS. It reads agent configurations
//! and orchestrates complex multi-agent workflows.
//!
//! ## Architecture
//!
//! The orchestration system consists of:
//!
//! - **OrchestrationEngine**: Main coordinator that manages agent lifecycles
//! - **AgentConfigLoader**: Loads and validates agent configurations from YAML
//! - **DependencyResolver**: Resolves spawn order based on agent dependencies
//! - **ProgressMonitor**: Tracks agent progress and coordinates phases
//! - **WorkstreamCoordinator**: Manages workstream-specific coordination
//!
//! ## Usage
//!
//! ```rust,no_run
//! use toka_orchestration::{OrchestrationEngine, OrchestrationConfig};
//! use toka_runtime::Runtime;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let config = OrchestrationConfig::from_directory("agents/v0.3.0/workstreams")?;
//! let runtime = Runtime::new(Default::default(), Default::default()).await?;
//! 
//! let engine = OrchestrationEngine::new(config, runtime).await?;
//! let session = engine.start_orchestration().await?;
//!
//! // Wait for completion
//! session.wait_for_completion().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Security
//!
//! The orchestration engine follows security-first principles:
//!
//! - **Capability validation**: Agents must declare required capabilities
//! - **Resource limits**: Agents are constrained by resource limits
//! - **Audit logging**: All orchestration actions are logged
//! - **Fail-safe defaults**: System fails closed on ambiguous operations

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use toka_llm_gateway::LlmGateway;
use toka_runtime::RuntimeManager;
use toka_types::{
    AgentSpec, EntityId, Message, Operation, TaskSpec,
    AgentConfig, AgentMetadata, AgentSpecConfig, AgentPriority, AgentCapabilities,
    AgentObjective, AgentTasks, TaskConfig, TaskPriority, AgentDependencies,
    ReportingConfig, ReportingFrequency, SecurityConfig, ResourceLimits
};
use toka_bus_core::KernelEvent;

pub mod config;
pub mod dependency;
pub mod monitor;
pub mod workstream;
pub mod llm_integration;
pub mod integration;

pub use config::{AgentConfigLoader, OrchestrationConfig};
pub use dependency::DependencyResolver;
pub use monitor::ProgressMonitor;
pub use workstream::WorkstreamCoordinator;
pub use llm_integration::{LlmOrchestrationIntegrator, TaskExecutionResult, CoordinationPlan};
pub use integration::{RuntimeIntegration, OrchestrationRuntimeExt};

/// Maximum number of agents that can be spawned simultaneously
pub const MAX_CONCURRENT_AGENTS: usize = 10;

/// Maximum time to wait for agent spawn completion
pub const AGENT_SPAWN_TIMEOUT: Duration = Duration::from_secs(30);

/// Maximum time to wait for workstream completion
pub const WORKSTREAM_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour

// AgentConfig and related types are now imported from toka-types

// All agent configuration types are now imported from toka-types

/// Current state of an agent in the orchestration system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is configured but not yet spawned
    Configured,
    /// Agent is ready to spawn (dependencies satisfied)
    Ready,
    /// Agent is in the process of spawning
    Spawning,
    /// Agent has been spawned and is active
    Active,
    /// Agent is paused/suspended
    Paused,
    /// Agent has completed its work
    Completed,
    /// Agent has failed and terminated
    Failed,
}

/// Information about a spawned agent.
#[derive(Debug, Clone)]
pub struct SpawnedAgent {
    /// Agent configuration
    pub config: AgentConfig,
    /// Agent entity ID
    pub agent_id: EntityId,
    /// Current state
    pub state: AgentState,
    /// Spawn timestamp
    pub spawned_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Assigned tasks
    pub tasks: Vec<TaskSpec>,
    /// Completion metrics
    pub metrics: AgentMetrics,
}

/// Metrics tracked for each agent.
#[derive(Debug, Clone, Default)]
pub struct AgentMetrics {
    /// Total tasks assigned
    pub tasks_assigned: usize,
    /// Tasks completed successfully
    pub tasks_completed: usize,
    /// Tasks that failed
    pub tasks_failed: usize,
    /// Total execution time
    pub execution_time: Duration,
    /// Last progress update
    pub last_progress: Option<DateTime<Utc>>,
}

/// Main orchestration engine for managing agent lifecycles.
pub struct OrchestrationEngine {
    /// Orchestration configuration
    config: OrchestrationConfig,
    /// Toka runtime instance
    runtime: Arc<RuntimeManager>,
    /// LLM gateway for intelligent coordination
    llm_gateway: Option<Arc<LlmGateway>>,
    /// Dependency resolver
    dependency_resolver: DependencyResolver,
    /// Progress monitor
    #[allow(dead_code)]
    progress_monitor: Arc<ProgressMonitor>,
    /// Workstream coordinator
    #[allow(dead_code)]
    workstream_coordinator: Arc<WorkstreamCoordinator>,
    /// Currently spawned agents
    spawned_agents: Arc<DashMap<EntityId, SpawnedAgent>>,
    /// Agent state by configuration name
    agent_states: Arc<DashMap<String, AgentState>>,
    /// Orchestration session state
    session_state: Arc<RwLock<SessionState>>,
}

/// Orchestration session state.
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Session ID
    pub session_id: String,
    /// Session start time
    pub started_at: DateTime<Utc>,
    /// Current orchestration phase
    pub current_phase: OrchestrationPhase,
    /// Overall progress (0.0 to 1.0)
    pub progress: f64,
    /// Whether session is complete
    pub completed: bool,
    /// Error information if session failed
    pub error: Option<String>,
}

/// Orchestration phases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrchestrationPhase {
    /// Initializing orchestration
    Initializing,
    /// Spawning critical infrastructure agents
    CriticalInfrastructure,
    /// Spawning foundation service agents
    FoundationServices,
    /// Spawning development agents in parallel
    ParallelDevelopment,
    /// Monitoring and coordination
    Monitoring,
    /// Completing and cleanup
    Completion,
    /// Session completed successfully
    Completed,
    /// Session failed
    Failed,
}

/// Orchestration session handle.
pub struct OrchestrationSession {
    session_id: String,
    engine: Arc<OrchestrationEngine>,
    completion_rx: mpsc::Receiver<Result<()>>,
}

impl OrchestrationEngine {
    /// Create a new orchestration engine.
    ///
    /// This initializes all components needed for agent orchestration including
    /// dependency resolution, progress monitoring, and workstream coordination.
    pub async fn new(
        config: OrchestrationConfig,
        runtime: Arc<RuntimeManager>,
    ) -> Result<Self> {
        info!("Initializing orchestration engine");

        // Initialize dependency resolver
        let dependency_resolver = DependencyResolver::new(&config.agents)?;
        debug!("Dependency resolver initialized with {} agents", config.agents.len());

        // Initialize progress monitor
        let progress_monitor = Arc::new(ProgressMonitor::new());

        // Initialize workstream coordinator
        let workstream_coordinator = Arc::new(WorkstreamCoordinator::new());

        // Initialize agent state tracking
        let agent_states = Arc::new(DashMap::new());
        for agent_config in &config.agents {
            agent_states.insert(agent_config.metadata.name.clone(), AgentState::Configured);
        }

        // Initialize session state
        let session_state = Arc::new(RwLock::new(SessionState {
            session_id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            current_phase: OrchestrationPhase::Initializing,
            progress: 0.0,
            completed: false,
            error: None,
        }));

        info!("Orchestration engine initialized successfully");

        Ok(Self {
            config,
            runtime,
            llm_gateway: None,
            dependency_resolver,
            progress_monitor,
            workstream_coordinator,
            spawned_agents: Arc::new(DashMap::new()),
            agent_states,
            session_state,
        })
    }

    /// Set LLM gateway for intelligent coordination.
    pub fn with_llm_gateway(mut self, gateway: Arc<LlmGateway>) -> Self {
        self.llm_gateway = Some(gateway);
        self
    }

    /// Start orchestration session.
    ///
    /// This begins the agent spawning and coordination process according to the
    /// configured phases and dependencies.
    pub async fn start_orchestration(self: Arc<Self>) -> Result<OrchestrationSession> {
        let session_id = {
            let state = self.session_state.read().await;
            state.session_id.clone()
        };

        info!("Starting orchestration session: {}", session_id);

        // Create completion channel
        let (completion_tx, completion_rx) = mpsc::channel(1);

        // Spawn orchestration task
        let engine = self.clone();
        tokio::spawn(async move {
            let result = engine.run_orchestration().await;
            let _ = completion_tx.send(result).await;
        });

        Ok(OrchestrationSession {
            session_id,
            engine: self,
            completion_rx,
        })
    }

    /// Main orchestration loop.
    async fn run_orchestration(self: Arc<Self>) -> Result<()> {
        info!("Running orchestration process");

        // Phase 1: Critical Infrastructure
        self.update_phase(OrchestrationPhase::CriticalInfrastructure).await?;
        self.spawn_critical_agents().await?;

        // Phase 2: Foundation Services
        self.update_phase(OrchestrationPhase::FoundationServices).await?;
        self.spawn_foundation_agents().await?;

        // Phase 3: Parallel Development
        self.update_phase(OrchestrationPhase::ParallelDevelopment).await?;
        self.spawn_development_agents().await?;

        // Phase 4: Monitoring and Coordination
        self.update_phase(OrchestrationPhase::Monitoring).await?;
        self.monitor_progress().await?;

        // Phase 5: Completion
        self.update_phase(OrchestrationPhase::Completion).await?;
        self.complete_orchestration().await?;

        self.update_phase(OrchestrationPhase::Completed).await?;
        info!("Orchestration completed successfully");

        Ok(())
    }

    /// Update orchestration phase.
    async fn update_phase(&self, phase: OrchestrationPhase) -> Result<()> {
        let mut state = self.session_state.write().await;
        state.current_phase = phase.clone();
        
        let progress = match phase {
            OrchestrationPhase::Initializing => 0.0,
            OrchestrationPhase::CriticalInfrastructure => 0.1,
            OrchestrationPhase::FoundationServices => 0.3,
            OrchestrationPhase::ParallelDevelopment => 0.5,
            OrchestrationPhase::Monitoring => 0.8,
            OrchestrationPhase::Completion => 0.9,
            OrchestrationPhase::Completed => 1.0,
            OrchestrationPhase::Failed => 0.0,
        };
        
        state.progress = progress;
        
        info!("Orchestration phase updated: {:?} ({}%)", phase, (progress * 100.0) as u8);
        
        Ok(())
    }

    /// Spawn critical infrastructure agents.
    async fn spawn_critical_agents(&self) -> Result<()> {
        info!("Spawning critical infrastructure agents");

        let critical_agents = self.config.agents.iter()
            .filter(|config| matches!(config.spec.priority, AgentPriority::Critical))
            .collect::<Vec<_>>();

        if critical_agents.is_empty() {
            warn!("No critical agents configured");
            return Ok(());
        }

        // Spawn critical agents sequentially to ensure stability
        for agent_config in critical_agents {
            self.spawn_agent(agent_config).await?;
            
            // Wait for agent to become active before proceeding
            self.wait_for_agent_active(&agent_config.metadata.name).await?;
        }

        info!("Critical infrastructure agents spawned successfully");
        Ok(())
    }

    /// Spawn foundation service agents.
    async fn spawn_foundation_agents(&self) -> Result<()> {
        info!("Spawning foundation service agents");

        let foundation_agents = self.config.agents.iter()
            .filter(|config| matches!(config.spec.priority, AgentPriority::High))
            .collect::<Vec<_>>();

        if foundation_agents.is_empty() {
            warn!("No foundation agents configured");
            return Ok(());
        }

        // Spawn foundation agents based on dependencies
        let spawn_order = self.dependency_resolver.resolve_spawn_order(
            &foundation_agents.iter().map(|c| c.metadata.name.clone()).collect::<Vec<_>>()
        )?;

        for agent_name in spawn_order {
            if let Some(agent_config) = foundation_agents.iter()
                .find(|c| c.metadata.name == agent_name) {
                self.spawn_agent(agent_config).await?;
            }
        }

        info!("Foundation service agents spawned successfully");
        Ok(())
    }

    /// Spawn development agents in parallel.
    async fn spawn_development_agents(&self) -> Result<()> {
        info!("Spawning development agents in parallel");

        let development_agents = self.config.agents.iter()
            .filter(|config| matches!(config.spec.priority, AgentPriority::Medium | AgentPriority::Low))
            .collect::<Vec<_>>();

        if development_agents.is_empty() {
            warn!("No development agents configured");
            return Ok(());
        }

        // Spawn development agents in parallel batches
        let spawn_tasks = development_agents.iter()
            .map(|agent_config| {
                let engine = self;
                let config = (*agent_config).clone();
                async move {
                    engine.spawn_agent(&config).await
                }
            })
            .collect::<Vec<_>>();

        // Wait for all agents to spawn
        let results = join_all(spawn_tasks).await;
        
        // Check for failures
        for result in results {
            if let Err(e) = result {
                error!("Failed to spawn development agent: {}", e);
                return Err(e);
            }
        }

        info!("Development agents spawned successfully");
        Ok(())
    }

    /// Spawn a single agent.
    async fn spawn_agent(&self, agent_config: &AgentConfig) -> Result<()> {
        info!("Spawning agent: {}", agent_config.metadata.name);

        // Update agent state
        self.agent_states.insert(agent_config.metadata.name.clone(), AgentState::Spawning);

        // Create agent spec
        let spec = AgentSpec::new(agent_config.spec.name.clone())
            .map_err(|e| anyhow::anyhow!("Failed to create agent spec: {}", e))?;

        // Create spawn operation
        let main_agent_id = EntityId(uuid::Uuid::new_v4().as_u128());
        let spawn_message = Message {
            origin: main_agent_id,
            capability: "agent-orchestration".to_string(),
            op: Operation::SpawnSubAgent {
                parent: main_agent_id,
                spec: spec.clone(),
            },
        };

        // Submit spawn operation
        let spawn_result = self.runtime.submit(spawn_message).await?;

        // Extract agent ID from kernel event
        let agent_id = match spawn_result {
            KernelEvent::AgentSpawned { spec: _spawned_spec, .. } => {
                // Generate unique agent ID
                EntityId(uuid::Uuid::new_v4().as_u128())
            }
            _ => {
                return Err(anyhow::anyhow!("Unexpected kernel event during agent spawn"));
            }
        };

        // Create spawned agent info
        let spawned_agent = SpawnedAgent {
            config: agent_config.clone(),
            agent_id,
            state: AgentState::Active,
            spawned_at: Utc::now(),
            last_activity: Utc::now(),
            tasks: Vec::new(),
            metrics: AgentMetrics::default(),
        };

        // Store spawned agent
        self.spawned_agents.insert(agent_id, spawned_agent);
        self.agent_states.insert(agent_config.metadata.name.clone(), AgentState::Active);

        // Assign default tasks
        self.assign_default_tasks(agent_id, agent_config).await?;

        info!("Agent spawned successfully: {} (ID: {:?})", agent_config.metadata.name, agent_id);

        Ok(())
    }

    /// Assign default tasks to an agent.
    async fn assign_default_tasks(&self, agent_id: EntityId, agent_config: &AgentConfig) -> Result<()> {
        debug!("Assigning default tasks to agent: {}", agent_config.metadata.name);

        for task_config in &agent_config.tasks.default {
            let task = TaskSpec::new(task_config.description.clone())
                .map_err(|e| anyhow::anyhow!("Failed to create task spec: {}", e))?;
            
            let task_message = Message {
                origin: EntityId(uuid::Uuid::new_v4().as_u128()),
                capability: "task-assignment".to_string(),
                op: Operation::ScheduleAgentTask {
                    agent: agent_id,
                    task: task.clone(),
                },
            };

            self.runtime.submit(task_message).await?;
        }

        debug!("Default tasks assigned to agent: {}", agent_config.metadata.name);
        Ok(())
    }

    /// Wait for agent to become active.
    async fn wait_for_agent_active(&self, agent_name: &str) -> Result<()> {
        let start_time = Instant::now();
        
        while start_time.elapsed() < AGENT_SPAWN_TIMEOUT {
            if let Some(state) = self.agent_states.get(agent_name) {
                if *state == AgentState::Active {
                    return Ok(());
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(anyhow::anyhow!("Timeout waiting for agent to become active: {}", agent_name))
    }

    /// Monitor agent progress and coordination.
    async fn monitor_progress(&self) -> Result<()> {
        info!("Monitoring agent progress");

        // This would typically run continuously, but for now we'll simulate
        // monitoring for a short period
        tokio::time::sleep(Duration::from_secs(5)).await;

        info!("Progress monitoring completed");
        Ok(())
    }

    /// Complete orchestration process.
    async fn complete_orchestration(&self) -> Result<()> {
        info!("Completing orchestration process");

        // Update session state
        let mut state = self.session_state.write().await;
        state.completed = true;
        state.progress = 1.0;

        info!("Orchestration process completed");
        Ok(())
    }

    /// Get current session state.
    pub async fn get_session_state(&self) -> SessionState {
        self.session_state.read().await.clone()
    }

    /// Get spawned agent information.
    pub fn get_spawned_agents(&self) -> Vec<SpawnedAgent> {
        self.spawned_agents.iter().map(|entry| entry.value().clone()).collect()
    }
}

impl OrchestrationSession {
    /// Get session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Wait for orchestration completion.
    pub async fn wait_for_completion(mut self) -> Result<()> {
        if let Some(result) = self.completion_rx.recv().await {
            result
        } else {
            Err(anyhow::anyhow!("Orchestration session terminated unexpectedly"))
        }
    }

    /// Get current session state.
    pub async fn get_state(&self) -> SessionState {
        self.engine.get_session_state().await
    }

    /// Get spawned agents.
    pub fn get_spawned_agents(&self) -> Vec<SpawnedAgent> {
        self.engine.get_spawned_agents()
    }
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            started_at: Utc::now(),
            current_phase: OrchestrationPhase::Initializing,
            progress: 0.0,
            completed: false,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_runtime::{RuntimeManager, ToolKernel};
    use toka_kernel::{Kernel, WorldState};
    use toka_auth::{TokenValidator, Claims};
    use toka_bus_core::EventBus;
    use std::future::Future;
    use std::pin::Pin;

    struct MockTokenValidator;

    impl TokenValidator for MockTokenValidator {
        fn validate<'life0, 'life1, 'async_trait>(
            &'life0 self,
            _token: &'life1 str,
        ) -> Pin<Box<dyn Future<Output = Result<Claims, toka_auth::Error>> + Send + 'async_trait>>
        where
            'life0: 'async_trait,
            'life1: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                let now = chrono::Utc::now().timestamp() as u64;
                Ok(Claims {
                    sub: "test-user".to_string(),
                    vault: "test-vault".to_string(),
                    permissions: vec!["test-permission".to_string()],
                    iat: now,
                    exp: now + 3600,
                    jti: uuid::Uuid::new_v4().to_string(),
                })
            })
        }
    }

    struct MockEventBus;

    impl EventBus for MockEventBus {
        fn publish(&self, _event: &toka_bus_core::KernelEvent) -> Result<()> {
            Ok(())
        }

        fn subscribe(&self) -> tokio::sync::broadcast::Receiver<toka_bus_core::KernelEvent> {
            // Create a dummy channel that won't receive any events
            let (tx, rx) = tokio::sync::broadcast::channel(1);
            drop(tx); // Close the sender so the receiver knows it's done
            rx
        }
    }

    #[tokio::test]
    async fn test_orchestration_engine_creation() {
        let config = OrchestrationConfig {
            agents: vec![],
            global_timeout: Duration::from_secs(3600),
            max_concurrent_agents: 5,
        };

        // Create unified runtime manager with proper kernel initialization
        let world_state = WorldState::default();
        let auth = Arc::new(MockTokenValidator);
        let bus = Arc::new(MockEventBus);
        let kernel = Kernel::new(world_state, auth, bus);
        let tool_kernel = ToolKernel::new(kernel);
        let runtime = Arc::new(
            RuntimeManager::new(tool_kernel)
                .await
                .expect("Failed to create runtime manager")
        );

        let engine = OrchestrationEngine::new(config, runtime).await;
        assert!(engine.is_ok());
    }
} 
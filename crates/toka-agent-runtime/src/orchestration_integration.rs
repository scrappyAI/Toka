//! Orchestration Integration for Agent Runtime
//!
//! This module provides the integration layer between toka-agent-runtime and
//! toka-orchestration, enabling agents to be spawned, managed, and coordinated
//! through the orchestration system.
//!
//! Generated: 2025-07-11 (UTC) - Phase 2 Integration

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, error, info, warn, instrument};

use toka_orchestration::{OrchestrationEngine, SpawnedAgent, AgentState, OrchestrationSession};
use toka_runtime::RuntimeManager;
use toka_llm_gateway::LlmGateway;
use toka_types::{AgentConfig, EntityId};

use crate::{
    AgentExecutor, AgentContext, AgentExecutionState, AgentMetrics, 
    ProgressReporter, TaskResult, AgentRuntimeError, AgentRuntimeResult
};

/// Integration bridge between agent runtime and orchestration system
pub struct OrchestrationIntegration {
    /// Orchestration engine reference
    orchestration_engine: Arc<OrchestrationEngine>,
    /// Runtime manager for agent execution
    runtime_manager: Arc<RuntimeManager>,
    /// LLM gateway for intelligent execution
    llm_gateway: Arc<LlmGateway>,
    /// Active agent executors
    active_agents: Arc<RwLock<HashMap<EntityId, ActiveAgentInfo>>>,
    /// Progress monitoring channels
    progress_channels: Arc<RwLock<HashMap<EntityId, mpsc::UnboundedSender<ProgressUpdate>>>>,
    /// Integration metrics
    metrics: Arc<RwLock<IntegrationMetrics>>,
}

/// Information about an active agent in the runtime
#[derive(Debug, Clone)]
pub struct ActiveAgentInfo {
    /// Agent configuration
    pub config: AgentConfig,
    /// Agent executor handle
    pub executor_handle: Option<tokio::task::JoinHandle<Result<()>>>,
    /// Progress reporter
    pub progress_tx: mpsc::UnboundedSender<ProgressUpdate>,
    /// Agent start time
    pub started_at: DateTime<Utc>,
    /// Current execution state
    pub state: AgentExecutionState,
    /// Latest metrics
    pub metrics: AgentMetrics,
}

/// Progress update message
#[derive(Debug, Clone)]
pub struct ProgressUpdate {
    /// Agent ID
    pub agent_id: EntityId,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
    /// Progress message
    pub message: Option<String>,
    /// Current agent state
    pub state: AgentExecutionState,
    /// Updated metrics
    pub metrics: Option<AgentMetrics>,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

/// Integration metrics for monitoring and debugging
#[derive(Debug, Clone, Default)]
pub struct IntegrationMetrics {
    /// Total agents spawned through integration
    pub total_agents_spawned: u64,
    /// Currently active agents
    pub active_agents: u64,
    /// Successfully completed agents
    pub completed_agents: u64,
    /// Failed agents
    pub failed_agents: u64,
    /// Total orchestration sessions handled
    pub orchestration_sessions: u64,
    /// Average agent execution time
    pub avg_execution_time: Duration,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl OrchestrationIntegration {
    /// Create a new orchestration integration
    pub async fn new(
        orchestration_engine: Arc<OrchestrationEngine>,
        runtime_manager: Arc<RuntimeManager>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<Self> {
        info!("Creating orchestration integration");

        Ok(Self {
            orchestration_engine,
            runtime_manager,
            llm_gateway,
            active_agents: Arc::new(RwLock::new(HashMap::new())),
            progress_channels: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(IntegrationMetrics::default())),
        })
    }

    /// Start orchestration session with agent runtime integration
    #[instrument(skip(self))]
    pub async fn start_orchestrated_execution(&self) -> Result<OrchestrationSession> {
        info!("Starting orchestrated execution with agent runtime integration");

        // Start orchestration session
        let session = self.orchestration_engine.clone().start_orchestration().await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.orchestration_sessions += 1;
            metrics.last_updated = Utc::now();
        }

        // Start progress monitoring for orchestration
        self.start_orchestration_monitoring().await?;

        info!("Orchestration session started with runtime integration: {}", session.session_id());
        Ok(session)
    }

    /// Spawn an agent through the integration layer
    #[instrument(skip(self, config), fields(agent_name = %config.metadata.name))]
    pub async fn spawn_agent(
        &self,
        config: AgentConfig,
        agent_id: EntityId,
    ) -> AgentRuntimeResult<()> {
        info!("Spawning agent through integration: {}", config.metadata.name);

        // Create progress channel for this agent
        let (progress_tx, progress_rx) = mpsc::unbounded_channel();

        // Store progress channel
        {
            let mut channels = self.progress_channels.write().await;
            channels.insert(agent_id, progress_tx.clone());
        }

        // Create agent executor
        let agent_executor = AgentExecutor::new(
            config.clone(),
            agent_id,
            self.runtime_manager.clone(),
            self.llm_gateway.clone(),
        ).await.map_err(|e| AgentRuntimeError::ExecutionFailed(
            format!("Failed to create agent executor: {}", e)
        ))?;

        // Start agent execution in background
        let executor_handle = tokio::spawn(async move {
            agent_executor.run().await
        });

        // Create active agent info
        let active_info = ActiveAgentInfo {
            config: config.clone(),
            executor_handle: Some(executor_handle),
            progress_tx,
            started_at: Utc::now(),
            state: AgentExecutionState::Spawning,
            metrics: AgentMetrics::default(),
        };

        // Store active agent
        {
            let mut active_agents = self.active_agents.write().await;
            active_agents.insert(agent_id, active_info);
        }

        // Start progress monitoring for this agent
        self.start_agent_progress_monitoring(agent_id, progress_rx).await;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_agents_spawned += 1;
            metrics.active_agents += 1;
            metrics.last_updated = Utc::now();
        }

        info!("Agent spawned successfully through integration: {}", config.metadata.name);
        Ok(())
    }

    /// Start progress monitoring for orchestration
    async fn start_orchestration_monitoring(&self) -> Result<()> {
        debug!("Starting orchestration progress monitoring");

        let orchestration_engine = self.orchestration_engine.clone();
        let integration_metrics = self.metrics.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Get orchestration state
                let session_state = orchestration_engine.get_session_state().await;
                let spawned_agents = orchestration_engine.get_spawned_agents();

                debug!("Orchestration monitoring: phase={:?}, progress={:.1}%, agents={}",
                       session_state.current_phase,
                       session_state.progress * 100.0,
                       spawned_agents.len());

                // Update integration metrics
                {
                    let mut metrics = integration_metrics.write().await;
                    metrics.last_updated = Utc::now();
                }

                // Check if orchestration is complete
                if session_state.completed {
                    info!("Orchestration session completed - stopping monitoring");
                    break;
                }
            }
        });

        Ok(())
    }

    /// Start progress monitoring for a specific agent
    async fn start_agent_progress_monitoring(
        &self,
        agent_id: EntityId,
        mut progress_rx: mpsc::UnboundedReceiver<ProgressUpdate>,
    ) {
        let active_agents = self.active_agents.clone();
        let metrics = self.metrics.clone();

        tokio::spawn(async move {
            while let Some(progress_update) = progress_rx.recv().await {
                debug!("Received progress update for agent {}: {:.1}%",
                       agent_id.0, progress_update.progress * 100.0);

                // Update active agent state
                {
                    let mut agents = active_agents.write().await;
                    if let Some(agent_info) = agents.get_mut(&agent_id) {
                        agent_info.state = progress_update.state.clone();
                        if let Some(agent_metrics) = progress_update.metrics {
                            agent_info.metrics = agent_metrics;
                        }
                    }
                }

                // Handle completion
                if matches!(progress_update.state, 
                           AgentExecutionState::Completed | 
                           AgentExecutionState::Failed { .. }) {
                    info!("Agent {} execution completed: state={:?}",
                          agent_id.0, progress_update.state);

                    // Update integration metrics
                    {
                        let mut integration_metrics = metrics.write().await;
                        integration_metrics.active_agents -= 1;
                        
                        match progress_update.state {
                            AgentExecutionState::Completed => {
                                integration_metrics.completed_agents += 1;
                            }
                            AgentExecutionState::Failed { .. } => {
                                integration_metrics.failed_agents += 1;
                            }
                            _ => {}
                        }
                        
                        integration_metrics.last_updated = Utc::now();
                    }

                    // Clean up completed agent
                    {
                        let mut agents = active_agents.write().await;
                        agents.remove(&agent_id);
                    }

                    break;
                }
            }
        });
    }

    /// Get current integration metrics
    pub async fn get_metrics(&self) -> IntegrationMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get active agent information
    pub async fn get_active_agents(&self) -> Vec<(EntityId, ActiveAgentInfo)> {
        let agents = self.active_agents.read().await;
        agents.iter().map(|(&id, info)| (id, info.clone())).collect()
    }

    /// Stop a specific agent
    #[instrument(skip(self), fields(agent_id = %agent_id.0))]
    pub async fn stop_agent(&self, agent_id: EntityId) -> AgentRuntimeResult<()> {
        info!("Stopping agent through integration: {}", agent_id.0);

        let mut agents = self.active_agents.write().await;
        if let Some(agent_info) = agents.remove(&agent_id) {
            // Abort the executor task
            if let Some(handle) = agent_info.executor_handle {
                handle.abort();
            }

            // Update metrics
            {
                let mut metrics = self.metrics.write().await;
                metrics.active_agents -= 1;
                metrics.last_updated = Utc::now();
            }

            info!("Agent stopped successfully: {}", agent_id.0);
            Ok(())
        } else {
            Err(AgentRuntimeError::ExecutionFailed(
                format!("Agent {} not found in active agents", agent_id.0)
            ))
        }
    }

    /// Shutdown the integration, stopping all active agents
    #[instrument(skip(self))]
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down orchestration integration");

        // Stop all active agents
        let agent_ids: Vec<EntityId> = {
            let agents = self.active_agents.read().await;
            agents.keys().cloned().collect()
        };

        for agent_id in agent_ids {
            if let Err(e) = self.stop_agent(agent_id).await {
                warn!("Failed to stop agent {} during shutdown: {}", agent_id.0, e);
            }
        }

        // Clear progress channels
        {
            let mut channels = self.progress_channels.write().await;
            channels.clear();
        }

        info!("Orchestration integration shutdown complete");
        Ok(())
    }

    /// Send progress update for an agent
    pub async fn send_progress_update(&self, update: ProgressUpdate) -> Result<()> {
        let channels = self.progress_channels.read().await;
        if let Some(tx) = channels.get(&update.agent_id) {
            tx.send(update).map_err(|e| {
                anyhow::anyhow!("Failed to send progress update: {}", e)
            })?;
        }
        Ok(())
    }
}

/// Extension trait to add orchestration capabilities to OrchestrationEngine
pub trait OrchestrationEngineExt {
    /// Create orchestration integration with agent runtime
    fn with_agent_runtime_integration(
        self: Arc<Self>,
        runtime_manager: Arc<RuntimeManager>,
        llm_gateway: Arc<LlmGateway>,
    ) -> impl std::future::Future<Output = Result<OrchestrationIntegration>>;
}

impl OrchestrationEngineExt for OrchestrationEngine {
    async fn with_agent_runtime_integration(
        self: Arc<Self>,
        runtime_manager: Arc<RuntimeManager>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<OrchestrationIntegration> {
        OrchestrationIntegration::new(self, runtime_manager, llm_gateway).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_types::{
        AgentMetadata, AgentSpecConfig, AgentPriority, AgentCapabilities,
        AgentTasks, AgentDependencies, ReportingConfig, ReportingFrequency,
        SecurityConfig, ResourceLimits, TaskConfig, TaskPriority
    };
    use std::collections::HashMap;

    fn create_test_agent_config() -> AgentConfig {
        AgentConfig {
            metadata: AgentMetadata {
                name: "test-integration-agent".to_string(),
                version: "v1.0".to_string(),
                created: "2025-07-11".to_string(),
                workstream: "integration-test".to_string(),
                branch: "main".to_string(),
            },
            spec: AgentSpecConfig {
                name: "Test Integration Agent".to_string(),
                domain: "testing".to_string(),
                priority: AgentPriority::Medium,
            },
            capabilities: AgentCapabilities {
                primary: vec!["testing".to_string()],
                secondary: vec![],
            },
            objectives: vec![],
            tasks: AgentTasks {
                default: vec![
                    TaskConfig {
                        description: "Test integration task".to_string(),
                        priority: TaskPriority::High,
                    },
                ],
            },
            dependencies: AgentDependencies {
                required: HashMap::new(),
                optional: HashMap::new(),
            },
            reporting: ReportingConfig {
                frequency: ReportingFrequency::Daily,
                channels: vec!["test".to_string()],
                metrics: HashMap::new(),
            },
            security: SecurityConfig {
                sandbox: true,
                capabilities_required: vec!["testing".to_string()],
                resource_limits: ResourceLimits {
                    max_memory: "100MB".to_string(),
                    max_cpu: "25%".to_string(),
                    timeout: "5m".to_string(),
                },
            },
        }
    }

    #[test]
    fn test_progress_update_creation() {
        let progress_update = ProgressUpdate {
            agent_id: EntityId(123),
            progress: 0.75,
            message: Some("Test progress".to_string()),
            state: AgentExecutionState::Ready,
            metrics: None,
            timestamp: Utc::now(),
        };

        assert_eq!(progress_update.agent_id.0, 123);
        assert_eq!(progress_update.progress, 0.75);
        assert!(progress_update.message.is_some());
    }

    #[test]
    fn test_integration_metrics_default() {
        let metrics = IntegrationMetrics::default();
        
        assert_eq!(metrics.total_agents_spawned, 0);
        assert_eq!(metrics.active_agents, 0);
        assert_eq!(metrics.completed_agents, 0);
        assert_eq!(metrics.failed_agents, 0);
    }

    #[test]
    fn test_active_agent_info_creation() {
        let config = create_test_agent_config();
        let (tx, _rx) = mpsc::unbounded_channel();

        let active_info = ActiveAgentInfo {
            config: config.clone(),
            executor_handle: None,
            progress_tx: tx,
            started_at: Utc::now(),
            state: AgentExecutionState::Spawning,
            metrics: AgentMetrics::default(),
        };

        assert_eq!(active_info.config.metadata.name, "test-integration-agent");
        assert_eq!(active_info.state, AgentExecutionState::Spawning);
    }
}
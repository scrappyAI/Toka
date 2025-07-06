//! Agent process management and lifecycle coordination.
//!
//! This module provides the AgentProcessManager that manages multiple agent
//! processes, handles their lifecycles, and coordinates with the orchestration
//! system.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use dashmap::DashMap;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, instrument, warn};

use toka_llm_gateway::LlmGateway;
use toka_types::AgentConfig;
use toka_runtime::Runtime;
use toka_types::EntityId;

use crate::{
    AgentExecutor, AgentExecutionState, RuntimeStats, AgentRuntimeError, AgentRuntimeResult,
    AGENT_STARTUP_TIMEOUT,
};

/// Manages agent processes and their lifecycles
pub struct AgentProcessManager {
    /// Map of running agent processes
    agents: Arc<DashMap<EntityId, AgentProcess>>,
    /// System runtime
    runtime: Arc<Runtime>,
    /// LLM gateway for agent execution
    llm_gateway: Arc<LlmGateway>,
    /// Runtime statistics
    stats: Arc<RwLock<RuntimeStats>>,
    /// Process manager start time
    start_time: Instant,
}

/// Information about a running agent process
pub struct AgentProcess {
    /// Agent configuration
    pub config: AgentConfig,
    /// Agent entity ID
    pub agent_id: EntityId,
    /// Tokio task handle for the agent execution
    pub task_handle: JoinHandle<Result<()>>,
    /// Agent executor reference (for control operations)
    pub executor: Arc<AgentExecutor>,
    /// Process start time
    pub started_at: Instant,
    /// Current state
    pub state: AgentExecutionState,
}

/// Result of agent process operation
#[derive(Debug, Clone)]
pub struct ProcessResult {
    /// Agent ID
    pub agent_id: EntityId,
    /// Operation success
    pub success: bool,
    /// Result message
    pub message: String,
    /// Operation duration
    pub duration: Duration,
}

impl AgentProcessManager {
    /// Create a new agent process manager
    pub fn new(
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Self {
        info!("Creating agent process manager");

        Self {
            agents: Arc::new(DashMap::new()),
            runtime,
            llm_gateway,
            stats: Arc::new(RwLock::new(RuntimeStats::default())),
            start_time: Instant::now(),
        }
    }

    /// Start an agent process from configuration
    #[instrument(skip(self, config), fields(agent_name = %config.metadata.name))]
    pub async fn start_agent(
        &self,
        config: AgentConfig,
        agent_id: EntityId,
    ) -> AgentRuntimeResult<ProcessResult> {
        let start_time = Instant::now();
        
        info!("Starting agent process: {} (ID: {:?})", config.metadata.name, agent_id);

        // Check if agent is already running
        if self.agents.contains_key(&agent_id) {
            return Err(AgentRuntimeError::ExecutionFailed(
                format!("Agent {} is already running", agent_id.0)
            ));
        }

        // Create agent executor
        let executor = match AgentExecutor::new(
            config.clone(),
            agent_id,
            self.runtime.clone(),
            self.llm_gateway.clone(),
        ).await {
            Ok(executor) => Arc::new(executor),
            Err(error) => {
                error!("Failed to create agent executor: {}", error);
                return Err(AgentRuntimeError::ExecutionFailed(
                    format!("Failed to create agent executor: {}", error)
                ));
            }
        };

        // We need to handle the executor ownership issue differently
        // For now, let's create a second executor just for control operations
        let control_executor = AgentExecutor::new(
            config.clone(),
            agent_id,
            self.runtime.clone(),
            self.llm_gateway.clone(),
        ).await
        .map_err(|e| AgentRuntimeError::ExecutionFailed(e.to_string()))?;
        
        // Spawn agent execution task
        let executor_for_task = match Arc::try_unwrap(executor) {
            Ok(executor) => executor,
            Err(_) => {
                // This shouldn't happen since we just created it
                return Err(AgentRuntimeError::ExecutionFailed(
                    "Failed to unwrap executor for task execution".to_string()
                ));
            }
        };
        
        let task_handle = tokio::spawn(async move {
            executor_for_task.run().await
        });

        // Create agent process
        let agent_process = AgentProcess {
            config: config.clone(),
            agent_id,
            task_handle,
            executor: Arc::new(control_executor),
            started_at: start_time,
            state: AgentExecutionState::Initializing,
        };

        // Store agent process
        self.agents.insert(agent_id, agent_process);

        // Update statistics
        {
            let mut stats = self.stats.write().await;
            stats.active_agents += 1;
            stats.total_agents_started += 1;
        }

        // Wait for agent to become ready (with timeout)
        match self.wait_for_agent_ready(agent_id).await {
            Ok(()) => {
                let duration = start_time.elapsed();
                info!("Agent started successfully: {} (duration: {:?})", 
                      config.metadata.name, duration);
                
                Ok(ProcessResult {
                    agent_id,
                    success: true,
                    message: "Agent started successfully".to_string(),
                    duration,
                })
            }
            Err(error) => {
                error!("Agent failed to start: {} (error: {})", config.metadata.name, error);
                
                // Clean up failed agent
                self.remove_agent(agent_id).await;
                
                Err(AgentRuntimeError::ExecutionFailed(
                    format!("Agent failed to start: {}", error)
                ))
            }
        }
    }

    /// Stop an agent process
    #[instrument(skip(self), fields(agent_id = ?agent_id))]
    pub async fn stop_agent(&self, agent_id: EntityId) -> AgentRuntimeResult<ProcessResult> {
        let start_time = Instant::now();
        
        info!("Stopping agent process: {:?}", agent_id);

        // First, extract the necessary data from the agent process
        let agent_name = {
            let agent_process = self.agents.get(&agent_id)
                .ok_or_else(|| AgentRuntimeError::ExecutionFailed(
                    format!("Agent {} not found", agent_id.0)
                ))?;

            let agent_name = agent_process.config.metadata.name.clone();

            // Terminate the agent
            if let Err(error) = agent_process.executor.terminate("Requested by process manager".to_string()).await {
                warn!("Failed to gracefully terminate agent {}: {}", agent_name, error);
            }

            // Cancel the task and get the handle
            agent_process.task_handle.abort();
            
            // We need to extract the name but can't clone the task handle
            agent_name
        };

        // Remove from tracking first to get ownership
        let removed_agent = self.agents.remove(&agent_id);
        
        // Wait for task completion or timeout
        let result = if let Some((_, agent_process)) = removed_agent {
            tokio::time::timeout(
                Duration::from_secs(10),
                agent_process.task_handle
            ).await
        } else {
            // Agent was already removed
            return Err(AgentRuntimeError::ExecutionFailed(
                format!("Agent {} was already removed", agent_id.0)
            ));
        };

        // Update statistics (agent already removed from map above)
        {
            let mut stats = self.stats.write().await;
            stats.active_agents = stats.active_agents.saturating_sub(1);
        }

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(Ok(()))) => {
                info!("Agent stopped successfully: {} (duration: {:?})", agent_name, duration);
                Ok(ProcessResult {
                    agent_id,
                    success: true,
                    message: "Agent stopped successfully".to_string(),
                    duration,
                })
            }
            _ => {
                warn!("Agent stop may have been forceful: {} (duration: {:?})", agent_name, duration);
                Ok(ProcessResult {
                    agent_id,
                    success: true,
                    message: "Agent stopped (may have been forceful)".to_string(),
                    duration,
                })
            }
        }
    }

    /// Pause an agent process
    pub async fn pause_agent(&self, agent_id: EntityId) -> AgentRuntimeResult<ProcessResult> {
        let start_time = Instant::now();
        
        let agent_process = self.agents.get(&agent_id)
            .ok_or_else(|| AgentRuntimeError::ExecutionFailed(
                format!("Agent {} not found", agent_id.0)
            ))?;

        agent_process.executor.pause().await
            .map_err(|e| AgentRuntimeError::ExecutionFailed(e.to_string()))?;

        let duration = start_time.elapsed();
        
        Ok(ProcessResult {
            agent_id,
            success: true,
            message: "Agent paused successfully".to_string(),
            duration,
        })
    }

    /// Resume an agent process
    pub async fn resume_agent(&self, agent_id: EntityId) -> AgentRuntimeResult<ProcessResult> {
        let start_time = Instant::now();
        
        let agent_process = self.agents.get(&agent_id)
            .ok_or_else(|| AgentRuntimeError::ExecutionFailed(
                format!("Agent {} not found", agent_id.0)
            ))?;

        agent_process.executor.resume().await
            .map_err(|e| AgentRuntimeError::ExecutionFailed(e.to_string()))?;

        let duration = start_time.elapsed();
        
        Ok(ProcessResult {
            agent_id,
            success: true,
            message: "Agent resumed successfully".to_string(),
            duration,
        })
    }

    /// Get current state of an agent
    pub async fn get_agent_state(&self, agent_id: EntityId) -> Option<AgentExecutionState> {
        if let Some(agent_process) = self.agents.get(&agent_id) {
            Some(agent_process.executor.get_state().await)
        } else {
            None
        }
    }

    /// Get list of all running agents
    pub fn get_running_agents(&self) -> Vec<EntityId> {
        self.agents.iter().map(|entry| *entry.key()).collect()
    }

    /// Get agent process information
    pub fn get_agent_info(&self, agent_id: EntityId) -> Option<AgentProcessInfo> {
        self.agents.get(&agent_id).map(|agent_process| {
            AgentProcessInfo {
                agent_id,
                config: agent_process.config.clone(),
                started_at: agent_process.started_at,
                uptime: agent_process.started_at.elapsed(),
                state: agent_process.state.clone(),
            }
        })
    }

    /// Monitor agent health and restart if needed
    pub async fn monitor_agents(&self) -> Result<()> {
        debug!("Monitoring {} agents", self.agents.len());

        let mut completed_agents: Vec<EntityId> = Vec::new();
        let mut failed_agents: Vec<EntityId> = Vec::new();

        // Check each agent's status
        for entry in self.agents.iter() {
            let agent_id = *entry.key();
            let agent_process = entry.value();

            // Check if task is still running
            if agent_process.task_handle.is_finished() {
                // Task has completed, we need to handle this by removing the agent
                // and checking the result in a separate step
                completed_agents.push(agent_id);
                continue;
            }
        }

        // Handle completed agents
        let mut successful_agents: Vec<EntityId> = Vec::new();
        let mut failed_agents: Vec<EntityId> = Vec::new();
        
        for agent_id in completed_agents {
            // Remove the agent and check its result
            if let Some((_, agent_process)) = self.agents.remove(&agent_id) {
                match agent_process.task_handle.await {
                    Ok(Ok(())) => {
                        info!("Agent completed successfully: {:?}", agent_id);
                        successful_agents.push(agent_id);
                    }
                    Ok(Err(error)) => {
                        error!("Agent failed: {:?} (error: {})", agent_id, error);
                        failed_agents.push(agent_id);
                    }
                    Err(error) => {
                        error!("Agent task panicked: {:?} (error: {})", agent_id, error);
                        failed_agents.push(agent_id);
                    }
                }
            }
        }

        // Clean up completed and failed agents
        for agent_id in successful_agents {
            self.handle_agent_completion(agent_id).await;
        }

        for agent_id in failed_agents {
            self.handle_agent_failure(agent_id).await;
        }

        Ok(())
    }

    /// Get runtime statistics
    pub async fn get_stats(&self) -> RuntimeStats {
        let mut stats = self.stats.read().await.clone();
        stats.active_agents = self.agents.len() as u64;
        stats.uptime = self.start_time.elapsed();
        stats
    }

    /// Shutdown all agents gracefully
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down agent process manager with {} agents", self.agents.len());

        let agent_ids: Vec<EntityId> = self.agents.iter().map(|entry| *entry.key()).collect();
        
        // Stop all agents
        for agent_id in agent_ids {
            if let Err(error) = self.stop_agent(agent_id).await {
                warn!("Failed to stop agent {:?} during shutdown: {}", agent_id, error);
            }
        }

        info!("Agent process manager shutdown complete");
        Ok(())
    }

    /// Wait for agent to become ready
    async fn wait_for_agent_ready(&self, agent_id: EntityId) -> Result<()> {
        let start_time = Instant::now();
        
        while start_time.elapsed() < AGENT_STARTUP_TIMEOUT {
            if let Some(state) = self.get_agent_state(agent_id).await {
                match state {
                    AgentExecutionState::Ready | AgentExecutionState::ExecutingTask { .. } => {
                        return Ok(());
                    }
                    AgentExecutionState::Failed { error } => {
                        return Err(anyhow::anyhow!("Agent failed during startup: {}", error));
                    }
                    AgentExecutionState::Terminated { reason } => {
                        return Err(anyhow::anyhow!("Agent terminated during startup: {}", reason));
                    }
                    _ => {
                        // Still initializing, continue waiting
                    }
                }
            }
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(anyhow::anyhow!("Timeout waiting for agent to become ready"))
    }

    /// Handle agent completion
    async fn handle_agent_completion(&self, agent_id: EntityId) {
        info!("Handling agent completion: {:?}", agent_id);
        
        {
            let mut stats = self.stats.write().await;
            stats.total_agents_completed += 1;
        }
        
        self.remove_agent(agent_id).await;
    }

    /// Handle agent failure
    async fn handle_agent_failure(&self, agent_id: EntityId) {
        error!("Handling agent failure: {:?}", agent_id);
        
        {
            let mut stats = self.stats.write().await;
            stats.total_agents_failed += 1;
        }
        
        self.remove_agent(agent_id).await;
    }

    /// Remove agent from tracking
    async fn remove_agent(&self, agent_id: EntityId) {
        self.agents.remove(&agent_id);
        
        let mut stats = self.stats.write().await;
        if stats.active_agents > 0 {
            stats.active_agents -= 1;
        }
    }
}

/// Information about an agent process
#[derive(Debug, Clone)]
pub struct AgentProcessInfo {
    /// Agent entity ID
    pub agent_id: EntityId,
    /// Agent configuration
    pub config: AgentConfig,
    /// Process start time
    pub started_at: Instant,
    /// Process uptime
    pub uptime: Duration,
    /// Current execution state
    pub state: AgentExecutionState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_types::{
        AgentMetadata, AgentSpecConfig, AgentPriority, AgentCapabilities,
        AgentTasks, AgentDependencies, ReportingConfig, SecurityConfig, ResourceLimits,
        ReportingFrequency
    };
    use std::collections::HashMap;

    fn create_test_agent_config() -> AgentConfig {
        AgentConfig {
            metadata: AgentMetadata {
                name: "test-agent".to_string(),
                version: "v1.0".to_string(),
                created: "2025-01-27".to_string(),
                workstream: "test".to_string(),
                branch: "main".to_string(),
            },
            spec: AgentSpecConfig {
                name: "Test Agent".to_string(),
                domain: "test".to_string(),
                priority: AgentPriority::Medium,
            },
            capabilities: AgentCapabilities {
                primary: vec!["testing".to_string()],
                secondary: vec![],
            },
            objectives: vec![],
            tasks: AgentTasks {
                default: vec![],
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
                capabilities_required: vec!["test".to_string()],
                resource_limits: ResourceLimits {
                    max_memory: "100MB".to_string(),
                    max_cpu: "50%".to_string(),
                    timeout: "5m".to_string(),
                },
            },
        }
    }

    #[test]
    fn test_process_result_creation() {
        let result = ProcessResult {
            agent_id: EntityId(42),
            success: true,
            message: "Test message".to_string(),
            duration: Duration::from_secs(1),
        };

        assert_eq!(result.agent_id, EntityId(42));
        assert!(result.success);
        assert_eq!(result.message, "Test message");
        assert_eq!(result.duration, Duration::from_secs(1));
    }

    #[test]
    fn test_agent_process_info() {
        let config = create_test_agent_config();
        let info = AgentProcessInfo {
            agent_id: EntityId(123),
            config: config.clone(),
            started_at: Instant::now(),
            uptime: Duration::from_secs(60),
            state: AgentExecutionState::Ready,
        };

        assert_eq!(info.agent_id, EntityId(123));
        assert_eq!(info.config.metadata.name, "test-agent");
        assert_eq!(info.uptime, Duration::from_secs(60));
        assert!(matches!(info.state, AgentExecutionState::Ready));
    }

    #[tokio::test]
    async fn test_runtime_stats() {
        let stats = RuntimeStats {
            active_agents: 2,
            total_agents_started: 5,
            total_agents_completed: 2,
            total_agents_failed: 1,
            total_tasks_executed: 15,
            avg_agent_execution_time: Duration::from_secs(120),
            total_llm_requests: 50,
            total_llm_tokens: 10000,
            uptime: Duration::from_secs(3600),
        };

        assert_eq!(stats.active_agents, 2);
        assert_eq!(stats.total_agents_started, 5);
        assert_eq!(stats.total_tasks_executed, 15);
        assert_eq!(stats.uptime, Duration::from_secs(3600));
    }
}
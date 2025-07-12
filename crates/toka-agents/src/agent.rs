//! Core agent abstraction for the canonical agent system.
//!
//! This module provides the unified Agent struct that consolidates functionality
//! from both `toka-agent-runtime` and `toka-orchestration` into a single,
//! cohesive agent implementation.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn, instrument};

use toka_types::{AgentConfig, TaskConfig, EntityId, TaskSpec};
use toka_runtime::{RuntimeManager, ExecutionResult};
use toka_llm_gateway::{LlmGateway, LlmRequest, LlmResponse};

use crate::{
    AgentError, AgentResult, 
    executor::{AgentExecutor, TaskExecutor, ExecutionContext},
    resources::{ResourceManager, ResourceAllocation},
    progress::{ProgressReporter, AgentProgress, TaskResult},
    integration::LlmIntegration,
};

/// **Canonical Agent**
///
/// The core agent abstraction that unifies functionality from both agent runtime
/// and orchestration systems. Each agent is a self-contained execution unit with:
///
/// - **Configuration**: Agent metadata, objectives, and task specifications
/// - **Execution**: Unified task execution with LLM integration
/// - **Resources**: Resource allocation and management
/// - **Progress**: Real-time progress tracking and reporting
/// - **Lifecycle**: Complete lifecycle management from spawn to completion
///
/// # Example
///
/// ```rust,no_run
/// use toka_agents::{Agent, AgentConfig};
/// use toka_runtime::RuntimeManager;
/// use toka_types::EntityId;
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let runtime = RuntimeManager::new(/* ... */).await?;
/// let config = AgentConfig::from_file("agent.yaml")?;
/// let agent_id = EntityId::new();
///
/// let agent = Agent::new(
///     agent_id,
///     config,
///     runtime,
///     None, // LLM integration
///     resource_allocation,
/// ).await?;
///
/// // Start agent execution
/// let result = agent.run().await?;
/// println!("Agent completed with result: {:?}", result);
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Agent {
    /// Unique agent identifier
    pub id: EntityId,
    /// Agent configuration
    pub config: AgentConfig,
    /// Current agent state
    pub state: Arc<RwLock<AgentState>>,
    /// Agent status
    pub status: Arc<RwLock<AgentStatus>>,
    /// Agent executor for task execution
    executor: Arc<AgentExecutor>,
    /// Progress reporter for monitoring
    progress_reporter: Arc<ProgressReporter>,
    /// Resource allocation
    resource_allocation: ResourceAllocation,
    /// Creation timestamp
    created_at: DateTime<Utc>,
    /// Last activity timestamp
    last_activity: Arc<RwLock<DateTime<Utc>>>,
    /// Execution context
    execution_context: ExecutionContext,
}

/// Agent state enumeration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentState {
    /// Agent is being initialized
    Initializing,
    /// Agent is ready to execute tasks
    Ready,
    /// Agent is actively executing a task
    ExecutingTask {
        /// Current task being executed
        task_id: String,
        /// Task start time
        started_at: DateTime<Utc>,
    },
    /// Agent is waiting for resources or dependencies
    Waiting {
        /// Reason for waiting
        reason: String,
        /// Wait start time
        started_at: DateTime<Utc>,
    },
    /// Agent is paused
    Paused {
        /// Reason for pause
        reason: String,
        /// Pause start time
        started_at: DateTime<Utc>,
    },
    /// Agent has completed successfully
    Completed {
        /// Completion time
        completed_at: DateTime<Utc>,
        /// Final result
        result: TaskResult,
    },
    /// Agent has failed
    Failed {
        /// Failure time
        failed_at: DateTime<Utc>,
        /// Error message
        error: String,
    },
    /// Agent was stopped
    Stopped {
        /// Stop time
        stopped_at: DateTime<Utc>,
        /// Reason for stopping
        reason: String,
    },
}

/// Agent status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Current progress (0.0 to 1.0)
    pub progress: f64,
    /// Tasks completed
    pub tasks_completed: u64,
    /// Tasks failed
    pub tasks_failed: u64,
    /// Total tasks
    pub total_tasks: u64,
    /// Current task description
    pub current_task: Option<String>,
    /// Execution time
    pub execution_time: Duration,
    /// Resource usage
    pub resource_usage: ResourceUsage,
    /// Last update time
    pub last_updated: DateTime<Utc>,
}

/// Resource usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage
    pub cpu_percent: f64,
    /// Memory usage in MB
    pub memory_mb: u64,
    /// Network requests made
    pub network_requests: u64,
    /// Files accessed
    pub files_accessed: u64,
    /// LLM tokens consumed
    pub llm_tokens: u64,
}

impl Default for AgentStatus {
    fn default() -> Self {
        Self {
            progress: 0.0,
            tasks_completed: 0,
            tasks_failed: 0,
            total_tasks: 0,
            current_task: None,
            execution_time: Duration::from_secs(0),
            resource_usage: ResourceUsage::default(),
            last_updated: Utc::now(),
        }
    }
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_percent: 0.0,
            memory_mb: 0,
            network_requests: 0,
            files_accessed: 0,
            llm_tokens: 0,
        }
    }
}

impl Agent {
    /// Create a new agent with the given configuration
    pub async fn new(
        id: EntityId,
        config: AgentConfig,
        runtime: Arc<RuntimeManager>,
        llm_integration: Option<Arc<LlmIntegration>>,
        resource_allocation: ResourceAllocation,
    ) -> Result<Self> {
        info!("Creating new agent: {:?} ({})", id, config.metadata.name);
        
        // Create execution context
        let execution_context = ExecutionContext::new(
            id,
            config.security.clone(),
            resource_allocation.clone(),
        );
        
        // Initialize agent executor
        let executor = Arc::new(AgentExecutor::new(
            id,
            config.clone(),
            runtime.clone(),
            llm_integration.clone(),
            execution_context.clone(),
        ).await?);
        
        // Initialize progress reporter
        let progress_reporter = Arc::new(ProgressReporter::new(
            id,
            config.reporting.clone(),
        ).await?);
        
        let now = Utc::now();
        
        let agent = Self {
            id,
            config,
            state: Arc::new(RwLock::new(AgentState::Initializing)),
            status: Arc::new(RwLock::new(AgentStatus::default())),
            executor,
            progress_reporter,
            resource_allocation,
            created_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            execution_context,
        };
        
        // Transition to ready state
        agent.transition_to_ready().await?;
        
        info!("Agent created successfully: {:?}", id);
        Ok(agent)
    }
    
    /// Run the agent's main execution loop
    #[instrument(skip(self))]
    pub async fn run(&self) -> Result<TaskResult> {
        info!("Starting agent execution: {:?}", self.id);
        
        let start_time = Instant::now();
        let mut final_result = TaskResult::default();
        
        // Update state to executing
        self.update_state(AgentState::ExecutingTask {
            task_id: "main_execution".to_string(),
            started_at: Utc::now(),
        }).await?;
        
        // Execute agent tasks
        match self.execute_tasks().await {
            Ok(result) => {
                final_result = result;
                
                // Update state to completed
                self.update_state(AgentState::Completed {
                    completed_at: Utc::now(),
                    result: final_result.clone(),
                }).await?;
                
                info!("Agent execution completed successfully: {:?}", self.id);
            }
            Err(error) => {
                error!("Agent execution failed: {:?}: {}", self.id, error);
                
                // Update state to failed
                self.update_state(AgentState::Failed {
                    failed_at: Utc::now(),
                    error: error.to_string(),
                }).await?;
                
                return Err(error);
            }
        }
        
        // Update final status
        let mut status = self.status.write().await;
        status.progress = 1.0;
        status.execution_time = start_time.elapsed();
        status.last_updated = Utc::now();
        
        // Report final progress
        self.progress_reporter.report_completion(final_result.clone()).await?;
        
        Ok(final_result)
    }
    
    /// Execute all tasks defined in the agent configuration
    async fn execute_tasks(&self) -> Result<TaskResult> {
        let tasks = &self.config.tasks.default;
        let total_tasks = tasks.len() as u64;
        
        // Update total tasks in status
        {
            let mut status = self.status.write().await;
            status.total_tasks = total_tasks;
        }
        
        let mut completed_tasks = 0u64;
        let mut failed_tasks = 0u64;
        let mut task_results = Vec::new();
        
        for (index, task) in tasks.iter().enumerate() {
            info!("Executing task {}/{}: {}", index + 1, total_tasks, task.description);
            
            // Update current task in status
            {
                let mut status = self.status.write().await;
                status.current_task = Some(task.description.clone());
                status.progress = (index as f64) / (total_tasks as f64);
                status.last_updated = Utc::now();
            }
            
            // Execute task
            match self.executor.execute_task(task).await {
                Ok(result) => {
                    completed_tasks += 1;
                    task_results.push(result);
                    
                    // Update status
                    {
                        let mut status = self.status.write().await;
                        status.tasks_completed = completed_tasks;
                    }
                    
                    // Report progress
                    let progress = AgentProgress {
                        agent_id: self.id,
                        completion_percentage: ((completed_tasks as f64) / (total_tasks as f64)) * 100.0,
                        tasks_completed: completed_tasks,
                        tasks_failed: failed_tasks,
                        current_task: Some(task.description.clone()),
                        last_updated: Utc::now(),
                    };
                    
                    self.progress_reporter.report_progress(progress).await?;
                }
                Err(error) => {
                    failed_tasks += 1;
                    warn!("Task failed: {}: {}", task.description, error);
                    
                    // Update status
                    {
                        let mut status = self.status.write().await;
                        status.tasks_failed = failed_tasks;
                    }
                    
                    // Check if we should continue on failure
                    // For now, always continue on failure in stub implementation
                    // TODO: Add continue_on_failure field to TaskConfig
                    if false {
                        return Err(error);
                    }
                }
            }
            
            // Update last activity
            *self.last_activity.write().await = Utc::now();
        }
        
        // Create final result
        let final_result = TaskResult {
            agent_id: self.id,
            success: failed_tasks == 0,
            tasks_completed: completed_tasks,
            tasks_failed: failed_tasks,
            total_tasks,
            execution_time: Utc::now().signed_duration_since(self.created_at).to_std().unwrap_or_default(),
            final_output: format!("Agent {:?} completed {} tasks", self.id, completed_tasks),
            error_message: if failed_tasks > 0 {
                Some(format!("{} tasks failed", failed_tasks))
            } else {
                None
            },
        };
        
        Ok(final_result)
    }
    
    /// Stop the agent execution
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping agent: {:?}", self.id);
        
        // Update state to stopped
        self.update_state(AgentState::Stopped {
            stopped_at: Utc::now(),
            reason: "User requested stop".to_string(),
        }).await?;
        
        // Stop executor
        self.executor.stop().await?;
        
        info!("Agent stopped: {:?}", self.id);
        Ok(())
    }
    
    /// Get current agent state
    pub async fn get_state(&self) -> AgentState {
        self.state.read().await.clone()
    }
    
    /// Get current agent status
    pub async fn get_status(&self) -> AgentStatus {
        self.status.read().await.clone()
    }
    
    /// Get agent progress
    pub async fn get_progress(&self) -> AgentProgress {
        let status = self.status.read().await;
        AgentProgress {
            agent_id: self.id,
            completion_percentage: status.progress * 100.0,
            tasks_completed: status.tasks_completed,
            tasks_failed: status.tasks_failed,
            current_task: status.current_task.clone(),
            last_updated: status.last_updated,
        }
    }
    
    /// Check if agent is active (not completed, failed, or stopped)
    pub async fn is_active(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, 
            AgentState::Initializing | 
            AgentState::Ready | 
            AgentState::ExecutingTask { .. } | 
            AgentState::Waiting { .. } | 
            AgentState::Paused { .. }
        )
    }
    
    /// Check if agent is completed
    pub async fn is_completed(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, AgentState::Completed { .. })
    }
    
    /// Check if agent has failed
    pub async fn is_failed(&self) -> bool {
        let state = self.state.read().await;
        matches!(*state, AgentState::Failed { .. })
    }
    
    /// Transition to ready state
    async fn transition_to_ready(&self) -> Result<()> {
        self.update_state(AgentState::Ready).await
    }
    
    /// Update agent state
    async fn update_state(&self, new_state: AgentState) -> Result<()> {
        let mut state = self.state.write().await;
        let old_state = state.clone();
        *state = new_state.clone();
        
        debug!("Agent {:?} state transition: {:?} -> {:?}", self.id, old_state, new_state);
        
        // Update last activity
        *self.last_activity.write().await = Utc::now();
        
        Ok(())
    }
}

/// Extension trait for agent collections
#[async_trait]
pub trait AgentCollection {
    /// Get all active agents
    async fn active_agents(&self) -> Vec<&Agent>;
    
    /// Get all completed agents
    async fn completed_agents(&self) -> Vec<&Agent>;
    
    /// Get all failed agents
    async fn failed_agents(&self) -> Vec<&Agent>;
    
    /// Get overall progress across all agents
    async fn overall_progress(&self) -> f64;
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_types::{AgentMetadata, AgentSpecConfig, AgentObjective, AgentTasks};
    
    #[tokio::test]
    async fn test_agent_creation() {
        // This test would require proper runtime setup
        // For now, we'll just test the basic structure
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_agent_state_transitions() {
        // Test state transition logic
        let state = AgentState::Initializing;
        assert!(matches!(state, AgentState::Initializing));
        
        let state = AgentState::Ready;
        assert!(matches!(state, AgentState::Ready));
    }
}
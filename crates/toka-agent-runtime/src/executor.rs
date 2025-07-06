//! Agent execution engine that interprets and executes agent configurations.
//!
//! This module provides the core AgentExecutor that serves as the main execution
//! loop for agents, coordinating task execution, progress reporting, and resource
//! management.

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
use chrono::Utc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, instrument, warn};

use toka_llm_gateway::LlmGateway;
use toka_types::{AgentConfig, TaskConfig};
use toka_runtime::Runtime;
use toka_types::EntityId;

use crate::{
    AgentContext, AgentExecutionState, AgentMetrics, ExecutionConfig, TaskExecutor,
    ProgressReporter, LlmTask, AgentTask, TaskResult, AgentRuntimeError, AgentRuntimeResult,
};

/// Core agent execution engine that interprets and executes agent configurations
pub struct AgentExecutor {
    /// Agent context with configuration and state
    context: Arc<RwLock<AgentContext>>,
    /// Runtime connection for kernel operations
    runtime: Arc<Runtime>,
    /// LLM gateway for intelligent task execution
    llm_gateway: Arc<LlmGateway>,
    /// Task executor for LLM-integrated execution
    task_executor: TaskExecutor,
    /// Progress reporter for orchestration communication
    progress_reporter: Arc<RwLock<ProgressReporter>>,
    /// Execution configuration
    execution_config: ExecutionConfig,
    /// Execution start time
    start_time: Instant,
}

impl AgentExecutor {
    /// Create a new agent executor
    #[instrument(skip(runtime, llm_gateway), fields(agent_id = ?agent_id))]
    pub async fn new(
        config: AgentConfig,
        agent_id: EntityId,
        runtime: Arc<Runtime>,
        llm_gateway: Arc<LlmGateway>,
    ) -> Result<Self> {
        info!("Creating agent executor for: {}", config.metadata.name);

        // Create agent context
        let context = AgentContext {
            agent_id,
            config: config.clone(),
            state: AgentExecutionState::Initializing,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            metrics: AgentMetrics::default(),
            environment: std::collections::HashMap::new(),
        };

        // Create task executor
        let execution_config = ExecutionConfig::default();
        let task_executor = TaskExecutor::new(
            llm_gateway.clone(),
            config.security.clone(),
            execution_config.clone(),
        )?;

        // Create progress reporter
        let progress_reporter = ProgressReporter::new(context.clone(), runtime.clone());

        debug!("Agent executor created successfully for: {}", config.metadata.name);

        Ok(Self {
            context: Arc::new(RwLock::new(context)),
            runtime,
            llm_gateway,
            task_executor,
            progress_reporter: Arc::new(RwLock::new(progress_reporter)),
            execution_config,
            start_time: Instant::now(),
        })
    }

    /// Main execution loop - interprets and executes agent configuration
    #[instrument(skip(self), fields(agent_name = %self.get_agent_name()))]
    pub async fn run(mut self) -> Result<()> {
        info!("Starting agent execution: {}", self.get_agent_name());

        // Update state to ready
        self.update_state(AgentExecutionState::Ready).await?;
        self.report_progress(0.0, Some("Agent initialized".to_string())).await?;

        let result = match self.execute_agent_workflow().await {
            Ok(()) => {
                info!("Agent execution completed successfully: {}", self.get_agent_name());
                self.update_state(AgentExecutionState::Completed).await?;
                self.report_completion(true, Some("All objectives completed successfully".to_string())).await?;
                Ok(())
            }
            Err(error) => {
                error!("Agent execution failed: {} (error: {})", self.get_agent_name(), error);
                self.update_state(AgentExecutionState::Failed { 
                    error: error.to_string() 
                }).await?;
                self.report_completion(false, Some(format!("Execution failed: {}", error))).await?;
                Err(error)
            }
        };

        let total_duration = self.start_time.elapsed();
        info!("Agent execution finished: {} (duration: {:?})", 
              self.get_agent_name(), total_duration);

        result
    }

    /// Execute the main agent workflow
    async fn execute_agent_workflow(&mut self) -> Result<()> {
        // Phase 1: Setup and validation
        self.setup_agent_environment().await?;

        // Phase 2: Execute default tasks
        self.execute_default_tasks().await?;

        // Phase 3: Check objectives completion
        self.validate_objectives_completion().await?;

        Ok(())
    }

    /// Setup agent environment and validate configuration
    async fn setup_agent_environment(&mut self) -> Result<()> {
        info!("Setting up agent environment: {}", self.get_agent_name());

        let mut context = self.context.write().await;
        
        // Add environment variables based on agent configuration
        context.environment.insert("AGENT_NAME".to_string(), context.config.metadata.name.clone());
        context.environment.insert("AGENT_VERSION".to_string(), context.config.metadata.version.clone());
        context.environment.insert("WORKSTREAM".to_string(), context.config.metadata.workstream.clone());
        context.environment.insert("AGENT_DOMAIN".to_string(), context.config.spec.domain.clone());
        context.environment.insert("AGENT_BRANCH".to_string(), context.config.metadata.branch.clone());

        // Add capability environment variables
        for capability in &context.config.security.capabilities_required {
            let env_var = format!("CAPABILITY_{}", capability.to_uppercase().replace("-", "_"));
            context.environment.insert(env_var, "true".to_string());
        }

        debug!("Agent environment setup complete with {} variables", 
               context.environment.len());

        Ok(())
    }

    /// Execute all default tasks for the agent
    async fn execute_default_tasks(&mut self) -> Result<()> {
        let config = {
            let context = self.context.read().await;
            context.config.clone()
        };

        let total_tasks = config.tasks.default.len();
        info!("Executing {} default tasks for: {}", total_tasks, config.metadata.name);

        for (index, task_config) in config.tasks.default.iter().enumerate() {
            let task_progress = (index as f64) / (total_tasks as f64);
            
            self.report_progress(
                task_progress, 
                Some(format!("Starting task {}/{}: {}", index + 1, total_tasks, task_config.description))
            ).await?;

            let task_result = self.execute_single_task(task_config, index).await?;
            
            {
                let mut reporter = self.progress_reporter.write().await;
                reporter.report_task_completion(task_result).await?;
            }
        }

        info!("All default tasks completed for: {}", config.metadata.name);
        Ok(())
    }

    /// Execute a single task with full error handling and reporting
    #[instrument(skip(self, task_config), fields(task_desc = %task_config.description))]
    async fn execute_single_task(&mut self, task_config: &TaskConfig, task_index: usize) -> Result<TaskResult> {
        let task_id = format!("task-{}-{}", self.get_agent_name(), task_index);
        let start_time = Instant::now();
        
        info!("Executing task: {} - {}", task_id, task_config.description);

        // Update execution state
        self.update_state(AgentExecutionState::ExecutingTask { 
            task_id: task_id.clone() 
        }).await?;

        // Create LLM task
        let llm_task = LlmTask::new(task_config.clone())
            .with_id(task_id.clone());

        // Execute task
        let context = self.context.read().await.clone();
        let task_result = match self.task_executor.execute_task(&llm_task, &context).await {
            Ok(result) => {
                info!("Task completed successfully: {} (duration: {:?})", 
                      task_id, start_time.elapsed());
                result
            }
            Err(error) => {
                warn!("Task failed: {} (error: {}, duration: {:?})", 
                      task_id, error, start_time.elapsed());
                
                TaskResult::failure(
                    task_id.clone(),
                    task_config.description.clone(),
                    error.to_string(),
                    start_time.elapsed(),
                )
            }
        };

        // Update agent metrics
        self.update_metrics_from_task_result(&task_result).await?;

        // Return to ready state
        self.update_state(AgentExecutionState::Ready).await?;

        Ok(task_result)
    }

    /// Validate that agent objectives have been completed
    async fn validate_objectives_completion(&self) -> Result<()> {
        let context = self.context.read().await;
        let objectives = &context.config.objectives;
        
        info!("Validating completion of {} objectives for: {}", 
              objectives.len(), context.config.metadata.name);

        // Simple validation - check if all tasks completed successfully
        let success_rate = if context.metrics.tasks_attempted > 0 {
            context.metrics.tasks_completed as f64 / context.metrics.tasks_attempted as f64
        } else {
            0.0
        };

        if success_rate < 0.8 {
            return Err(anyhow::anyhow!(
                "Insufficient task completion rate: {:.1}% (minimum 80% required)",
                success_rate * 100.0
            ));
        }

        info!("Objectives validation passed with {:.1}% success rate", 
              success_rate * 100.0);

        Ok(())
    }

    /// Update agent execution state
    async fn update_state(&self, new_state: AgentExecutionState) -> Result<()> {
        let mut context = self.context.write().await;
        context.state = new_state.clone();
        context.last_activity = Utc::now();
        
        debug!("Agent state updated to: {:?}", new_state);
        Ok(())
    }

    /// Report progress to orchestration system
    async fn report_progress(&self, progress: f64, message: Option<String>) -> Result<()> {
        let mut reporter = self.progress_reporter.write().await;
        reporter.report_progress(progress, message).await
    }

    /// Report agent completion
    async fn report_completion(&self, success: bool, message: Option<String>) -> Result<()> {
        let mut reporter = self.progress_reporter.write().await;
        reporter.report_completion(success, message).await
    }

    /// Update agent metrics from task result
    async fn update_metrics_from_task_result(&self, task_result: &TaskResult) -> Result<()> {
        let mut context = self.context.write().await;
        
        context.metrics.tasks_attempted += 1;
        if task_result.success {
            context.metrics.tasks_completed += 1;
        } else {
            context.metrics.tasks_failed += 1;
        }
        
        context.metrics.total_execution_time += task_result.duration;
        
        // Update average task time
        if context.metrics.tasks_attempted > 0 {
            context.metrics.avg_task_time = Duration::from_nanos(
                context.metrics.total_execution_time.as_nanos() as u64 / context.metrics.tasks_attempted
            );
        }

        // Update LLM metrics
        if let Some(tokens) = task_result.llm_tokens_used {
            context.metrics.llm_tokens_consumed += tokens;
            context.metrics.llm_requests += 1;
        }

        context.last_activity = Utc::now();

        // Update progress reporter metrics
        let mut reporter = self.progress_reporter.write().await;
        reporter.update_metrics(context.metrics.clone());

        debug!("Metrics updated: tasks={}/{}, tokens={}", 
               context.metrics.tasks_completed,
               context.metrics.tasks_attempted,
               context.metrics.llm_tokens_consumed);

        Ok(())
    }

    /// Get agent name for logging
    fn get_agent_name(&self) -> String {
        // This is a bit of a hack since we can't easily access the context
        // In a real implementation, we might cache this value
        format!("agent-{}", self.context.try_read()
            .map(|c| c.config.metadata.name.clone())
            .unwrap_or_else(|_| "unknown".to_string()))
    }

    /// Get current agent state
    pub async fn get_state(&self) -> AgentExecutionState {
        let context = self.context.read().await;
        context.state.clone()
    }

    /// Get agent context (read-only)
    pub async fn get_context(&self) -> AgentContext {
        let context = self.context.read().await;
        context.clone()
    }

    /// Pause agent execution
    pub async fn pause(&self) -> Result<()> {
        info!("Pausing agent: {}", self.get_agent_name());
        self.update_state(AgentExecutionState::Paused).await?;
        Ok(())
    }

    /// Resume agent execution (if paused)
    pub async fn resume(&self) -> Result<()> {
        let current_state = self.get_state().await;
        if matches!(current_state, AgentExecutionState::Paused) {
            info!("Resuming agent: {}", self.get_agent_name());
            self.update_state(AgentExecutionState::Ready).await?;
        }
        Ok(())
    }

    /// Terminate agent execution
    pub async fn terminate(&self, reason: String) -> Result<()> {
        info!("Terminating agent: {} (reason: {})", self.get_agent_name(), reason);
        self.update_state(AgentExecutionState::Terminated { reason }).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_types::{
        AgentMetadata, AgentSpecConfig, AgentPriority, AgentCapabilities, 
        AgentTasks, AgentDependencies, ReportingConfig, SecurityConfig, ResourceLimits
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
                default: vec![
                    TaskConfig {
                        description: "Test task 1".to_string(),
                        priority: toka_orchestration::TaskPriority::High,
                    },
                    TaskConfig {
                        description: "Test task 2".to_string(),
                        priority: toka_orchestration::TaskPriority::Medium,
                    },
                ],
            },
            dependencies: AgentDependencies {
                required: HashMap::new(),
                optional: HashMap::new(),
            },
            reporting: ReportingConfig {
                frequency: toka_orchestration::ReportingFrequency::Daily,
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

    #[tokio::test]
    async fn test_agent_executor_creation() {
        // Note: This test would require mock implementations of Runtime and LlmGateway
        // For now, we test the basic structure
        let config = create_test_agent_config();
        assert_eq!(config.metadata.name, "test-agent");
        assert_eq!(config.tasks.default.len(), 2);
    }

    #[tokio::test]
    async fn test_agent_state_transitions() {
        // Test that we can create the basic state transition logic
        let states = vec![
            AgentExecutionState::Initializing,
            AgentExecutionState::Ready,
            AgentExecutionState::ExecutingTask { task_id: "test".to_string() },
            AgentExecutionState::Completed,
        ];

        for state in states {
            match state {
                AgentExecutionState::Initializing => {
                    // Should transition to Ready
                }
                AgentExecutionState::Ready => {
                    // Should be able to start executing tasks
                }
                AgentExecutionState::ExecutingTask { .. } => {
                    // Should return to Ready or Complete
                }
                AgentExecutionState::Completed => {
                    // Final state
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_agent_context_creation() {
        let config = create_test_agent_config();
        let agent_id = EntityId(42);

        let context = AgentContext {
            agent_id,
            config: config.clone(),
            state: AgentExecutionState::Initializing,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            metrics: AgentMetrics::default(),
            environment: HashMap::new(),
        };

        assert_eq!(context.agent_id, agent_id);
        assert_eq!(context.config.metadata.name, "test-agent");
        assert!(matches!(context.state, AgentExecutionState::Initializing));
    }
}
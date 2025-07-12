//! Progress reporting for agent execution.
//!
//! This module provides the ProgressReporter that communicates agent progress
//! back to the orchestration system in real-time.

use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use toka_runtime::RuntimeManager;
use toka_types::{Message, Operation};

use crate::{AgentContext, AgentMetrics};

/// Progress reporter for communicating agent status to orchestration
pub struct ProgressReporter {
    /// Agent context for identification
    agent_context: AgentContext,
    /// Runtime connection for progress reporting
    runtime: std::sync::Arc<RuntimeManager>,
    /// Current progress percentage (0.0 to 1.0)
    current_progress: f64,
    /// Last progress report timestamp
    last_report: DateTime<Utc>,
    /// Agent metrics for reporting
    metrics: AgentMetrics,
}

/// Agent progress report sent to orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProgress {
    /// Agent entity ID
    pub agent_id: crate::EntityId,
    /// Agent name
    pub agent_name: String,
    /// Current progress (0.0 to 1.0)
    pub progress: f64,
    /// Current status message
    pub message: Option<String>,
    /// Current agent state
    pub state: crate::AgentExecutionState,
    /// Timestamp of this progress report
    pub timestamp: DateTime<Utc>,
    /// Agent metrics
    pub metrics: AgentMetrics,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Unique task identifier
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Whether task succeeded
    pub success: bool,
    /// Task output or error message
    pub output: Option<String>,
    /// Task execution duration
    pub duration: Duration,
    /// LLM tokens used (if applicable)
    pub llm_tokens_used: Option<u64>,
    /// Task completion timestamp
    pub completed_at: DateTime<Utc>,
}

impl TaskResult {
    /// Create a successful task result
    pub fn success(
        task_id: String,
        description: String,
        output: Option<String>,
        duration: Duration,
    ) -> Self {
        Self {
            task_id,
            description,
            success: true,
            output,
            duration,
            llm_tokens_used: None,
            completed_at: Utc::now(),
        }
    }

    /// Create a failed task result
    pub fn failure(
        task_id: String,
        description: String,
        error_message: String,
        duration: Duration,
    ) -> Self {
        Self {
            task_id,
            description,
            success: false,
            output: Some(error_message),
            duration,
            llm_tokens_used: None,
            completed_at: Utc::now(),
        }
    }

    /// Add LLM token usage information
    pub fn with_llm_tokens(mut self, tokens: u64) -> Self {
        self.llm_tokens_used = Some(tokens);
        self
    }
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(
        agent_context: AgentContext,
        runtime: std::sync::Arc<RuntimeManager>,
    ) -> Self {
        Self {
            agent_context,
            runtime,
            current_progress: 0.0,
            last_report: Utc::now(),
            metrics: AgentMetrics::default(),
        }
    }

    /// Report progress to orchestration system
    #[instrument(skip(self), fields(agent_id = ?self.agent_context.agent_id, progress = %progress))]
    pub async fn report_progress(
        &mut self, 
        progress: f64, 
        message: Option<String>
    ) -> Result<()> {
        // Clamp progress to valid range
        let progress = if progress < 0.0 {
            0.0
        } else if progress > 1.0 {
            1.0
        } else {
            progress
        };
        self.current_progress = progress;
        self.last_report = Utc::now();

        let progress_report = AgentProgress {
            agent_id: self.agent_context.agent_id,
            agent_name: self.agent_context.config.metadata.name.clone(),
            progress,
            message: message.clone(),
            state: self.agent_context.state.clone(),
            timestamp: self.last_report,
            metrics: self.metrics.clone(),
        };

        debug!("Reporting progress: {}% for agent: {}", 
               (progress * 100.0) as u32, 
               self.agent_context.config.metadata.name);

        // Serialize progress report as observation data
        let observation_data = serde_json::to_vec(&progress_report)?;
        
        // Create progress observation message
        let progress_message = Message::new(
            self.agent_context.agent_id,
            format!("progress-reporting-{}", self.agent_context.config.metadata.workstream),
            Operation::EmitObservation {
                agent: self.agent_context.agent_id,
                data: observation_data,
            },
        ).map_err(|e| anyhow::anyhow!("Failed to create progress message: {}", e))?;

        // Submit progress observation to runtime
        match self.runtime.submit(progress_message).await {
            Ok(_kernel_event) => {
                debug!("Progress reported successfully for agent: {}", 
                      self.agent_context.config.metadata.name);
            }
            Err(e) => {
                // Log error but don't fail the operation - progress reporting shouldn't block agent execution
                tracing::warn!(
                    "Failed to submit progress report for agent {}: {}",
                    self.agent_context.config.metadata.name,
                    e
                );
            }
        }

        // Also log progress for immediate visibility
        info!("Agent {} progress: {}%{}", 
              self.agent_context.config.metadata.name,
              (progress * 100.0) as u32,
              message.map(|m| format!(" - {}", m)).unwrap_or_default());

        Ok(())
    }

    /// Report task completion
    #[instrument(skip(self), fields(task_id = %task_result.task_id, success = %task_result.success))]
    pub async fn report_task_completion(&mut self, task_result: TaskResult) -> Result<()> {
        info!("Task completed: {} (success: {}, duration: {:?})",
              task_result.task_id,
              task_result.success,
              task_result.duration);

        // Update metrics based on task result
        if task_result.success {
            self.metrics.tasks_completed += 1;
        } else {
            self.metrics.tasks_failed += 1;
        }

        if let Some(tokens) = task_result.llm_tokens_used {
            self.metrics.llm_tokens_consumed += tokens;
            self.metrics.llm_requests += 1;
        }

        // Serialize task completion as observation data
        let observation_data = serde_json::to_vec(&task_result)?;
        
        // Create task completion observation message
        let completion_message = Message::new(
            self.agent_context.agent_id,
            format!("task-completion-{}", self.agent_context.config.metadata.workstream),
            Operation::EmitObservation {
                agent: self.agent_context.agent_id,
                data: observation_data,
            },
        ).map_err(|e| anyhow::anyhow!("Failed to create task completion message: {}", e))?;

        // Submit task completion observation to runtime
        match self.runtime.submit(completion_message).await {
            Ok(_kernel_event) => {
                debug!("Task completion reported successfully for agent: {}", 
                      self.agent_context.config.metadata.name);
            }
            Err(e) => {
                // Log error but don't fail the operation
                tracing::warn!(
                    "Failed to submit task completion report for agent {}: {}",
                    self.agent_context.config.metadata.name,
                    e
                );
            }
        }
        
        Ok(())
    }

    /// Report agent completion (success or failure)
    #[instrument(skip(self), fields(agent_id = ?self.agent_context.agent_id, success = %success))]
    pub async fn report_completion(
        &mut self, 
        success: bool, 
        message: Option<String>
    ) -> Result<()> {
        let final_progress = if success { 1.0 } else { self.current_progress };
        
        // Report final progress
        self.report_progress(final_progress, message.clone()).await?;
        
        // Create completion summary
        let completion_data = serde_json::json!({
            "type": "agent_completion",
            "agent_id": self.agent_context.agent_id.0,
            "agent_name": self.agent_context.config.metadata.name,
            "workstream": self.agent_context.config.metadata.workstream,
            "success": success,
            "final_metrics": self.metrics,
            "completed_at": Utc::now(),
            "message": message,
        });

        let completion_observation = serde_json::to_vec(&completion_data)?;
        
        // Create agent completion observation message
        let completion_message = Message::new(
            self.agent_context.agent_id,
            format!("agent-completion-{}", self.agent_context.config.metadata.workstream),
            Operation::EmitObservation {
                agent: self.agent_context.agent_id,
                data: completion_observation,
            },
        ).map_err(|e| anyhow::anyhow!("Failed to create completion message: {}", e))?;

        // Submit completion observation to runtime
        match self.runtime.submit(completion_message).await {
            Ok(_kernel_event) => {
                debug!("Agent completion reported successfully: {}", 
                      self.agent_context.config.metadata.name);
            }
            Err(e) => {
                // Log error but don't fail the operation
                tracing::warn!(
                    "Failed to submit agent completion report for {}: {}",
                    self.agent_context.config.metadata.name,
                    e
                );
            }
        }
        
        info!("Agent {} completed with {} (final progress: {}%)",
              self.agent_context.config.metadata.name,
              if success { "SUCCESS" } else { "FAILURE" },
              (final_progress * 100.0) as u32);

        Ok(())
    }

    /// Update metrics
    pub fn update_metrics(&mut self, metrics: AgentMetrics) {
        self.metrics = metrics;
    }

    /// Get current progress
    pub fn current_progress(&self) -> f64 {
        self.current_progress
    }

    /// Get current metrics
    pub fn metrics(&self) -> &AgentMetrics {
        &self.metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentExecutionState, EntityId};
    use toka_types::{
        AgentConfig, AgentMetadata, AgentSpecConfig, AgentPriority
    };
    use std::collections::HashMap;

    fn create_test_context() -> AgentContext {
        AgentContext {
            agent_id: EntityId(123),
            config: AgentConfig {
                metadata: AgentMetadata {
                    name: "test-agent".to_string(),
                    version: "v1.0".to_string(),
                    created: "2025-07-11".to_string(),
                    workstream: "test".to_string(),
                    branch: "main".to_string(),
                },
                spec: AgentSpecConfig {
                    name: "Test Agent".to_string(),
                    domain: "testing".to_string(),
                    priority: AgentPriority::Medium,
                },
                capabilities: toka_types::AgentCapabilities {
                    primary: vec!["testing".to_string()],
                    secondary: vec![],
                },
                objectives: vec![],
                tasks: toka_types::AgentTasks {
                    default: vec![],
                },
                dependencies: toka_types::AgentDependencies {
                    required: HashMap::new(),
                    optional: HashMap::new(),
                },
                reporting: toka_types::ReportingConfig {
                    frequency: toka_types::ReportingFrequency::Daily,
                    channels: vec!["test".to_string()],
                    metrics: HashMap::new(),
                },
                security: toka_types::SecurityConfig {
                    sandbox: true,
                    capabilities_required: vec!["test".to_string()],
                    resource_limits: toka_types::ResourceLimits {
                        max_memory: "100MB".to_string(),
                        max_cpu: "50%".to_string(),
                        timeout: "5m".to_string(),
                    },
                },
            },
            state: AgentExecutionState::Ready,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            metrics: AgentMetrics::default(),
            environment: HashMap::new(),
        }
    }

    #[test]
    fn test_task_result_creation() {
        let success_result = TaskResult::success(
            "task-1".to_string(),
            "Test task".to_string(),
            Some("Task completed successfully".to_string()),
            Duration::from_secs(10),
        );

        assert!(success_result.success);
        assert_eq!(success_result.task_id, "task-1");
        assert_eq!(success_result.duration, Duration::from_secs(10));

        let failure_result = TaskResult::failure(
            "task-2".to_string(),
            "Failed task".to_string(),
            "Task failed with error".to_string(),
            Duration::from_secs(5),
        );

        assert!(!failure_result.success);
        assert_eq!(failure_result.task_id, "task-2");
        assert!(failure_result.output.is_some());
    }

    #[test]
    fn test_task_result_with_tokens() {
        let result = TaskResult::success(
            "task-1".to_string(),
            "Test task".to_string(),
            None,
            Duration::from_secs(10),
        ).with_llm_tokens(150);

        assert_eq!(result.llm_tokens_used, Some(150));
    }

    #[tokio::test]
    async fn test_progress_clamping() {
        // Test that progress is properly clamped
        let test_values = vec![-0.5, 0.0, 0.5, 1.0, 1.5];
        let expected = vec![0.0, 0.0, 0.5, 1.0, 1.0];
        
        for (input, expected) in test_values.iter().zip(expected.iter()) {
            let clamped = if *input < 0.0 {
                0.0
            } else if *input > 1.0 {
                1.0
            } else {
                *input
            };
            assert_eq!(clamped, *expected);
        }
    }

    #[test]
    fn test_agent_progress_serialization() {
        let progress = AgentProgress {
            agent_id: EntityId(456),
            agent_name: "test-agent".to_string(),
            progress: 0.75,
            message: Some("Test message".to_string()),
            state: AgentExecutionState::Ready,
            timestamp: Utc::now(),
            metrics: AgentMetrics::default(),
        };

        let serialized = serde_json::to_string(&progress).unwrap();
        let deserialized: AgentProgress = serde_json::from_str(&serialized).unwrap();

        assert_eq!(progress.agent_id, deserialized.agent_id);
        assert_eq!(progress.progress, deserialized.progress);
        assert_eq!(progress.agent_name, deserialized.agent_name);
    }
}
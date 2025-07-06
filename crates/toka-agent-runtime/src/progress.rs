//! Progress reporting and monitoring for agent execution.
//!
//! This module provides functionality for agents to report their progress back to the
//! orchestration system and for monitoring agent health and task completion status.

use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument};

use toka_types::{EntityId, Message, Operation};
use toka_runtime::Runtime;

use crate::{AgentContext, AgentMetrics};

/// Progress information for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProgress {
    /// Agent entity ID
    pub agent_id: EntityId,
    /// Agent workstream name
    pub workstream: String,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
    /// Current task being executed
    pub current_task: Option<String>,
    /// Tasks completed
    pub tasks_completed: u64,
    /// Total tasks assigned
    pub total_tasks: u64,
    /// Agent metrics
    pub metrics: AgentMetrics,
    /// Progress timestamp
    pub timestamp: DateTime<Utc>,
    /// Additional status message
    pub message: Option<String>,
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Task identifier
    pub task_id: String,
    /// Task description
    pub description: String,
    /// Whether task completed successfully
    pub success: bool,
    /// Result data (if successful)
    pub result_data: Option<String>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Task execution duration
    pub duration: Duration,
    /// Timestamp when task completed
    pub completed_at: DateTime<Utc>,
    /// LLM tokens consumed during task
    pub llm_tokens_used: Option<u64>,
}

/// Progress reporter for agent execution
pub struct ProgressReporter {
    /// Agent context
    context: AgentContext,
    /// Runtime connection for reporting
    runtime: std::sync::Arc<Runtime>,
    /// Last reported progress
    last_progress: f64,
    /// Last report timestamp
    last_report_time: DateTime<Utc>,
    /// Minimum progress change to trigger report
    min_progress_delta: f64,
    /// Minimum time between reports
    min_report_interval: Duration,
}

impl ProgressReporter {
    /// Create a new progress reporter
    pub fn new(
        context: AgentContext,
        runtime: std::sync::Arc<Runtime>,
    ) -> Self {
        Self {
            context,
            runtime,
            last_progress: 0.0,
            last_report_time: Utc::now(),
            min_progress_delta: 0.05, // Report every 5% progress
            min_report_interval: Duration::from_secs(30), // Report at least every 30 seconds
        }
    }

    /// Report progress to orchestration system
    #[instrument(skip(self), fields(agent_id = ?self.context.agent_id))]
    pub async fn report_progress(&mut self, progress: f64, message: Option<String>) -> Result<()> {
        let now = Utc::now();
        
        // Check if we should report based on progress delta and time interval
        let progress_delta = (progress - self.last_progress).abs();
        let time_since_last = now - self.last_report_time;
        
        let should_report = progress_delta >= self.min_progress_delta
            || time_since_last >= self.min_report_interval
            || progress >= 1.0; // Always report completion

        if !should_report {
            debug!("Skipping progress report: delta={:.3}, time_since={:?}", 
                   progress_delta, time_since_last);
            return Ok(());
        }

        let progress_report = AgentProgress {
            agent_id: self.context.agent_id,
            workstream: self.context.config.metadata.workstream.clone(),
            progress: progress.clamp(0.0, 1.0),
            current_task: self.get_current_task(),
            tasks_completed: self.context.metrics.tasks_completed,
            total_tasks: self.context.config.tasks.default.len() as u64,
            metrics: self.context.metrics.clone(),
            timestamp: now,
            message,
        };

        self.send_progress_observation(progress_report).await?;
        
        self.last_progress = progress;
        self.last_report_time = now;

        info!("Progress reported: {:.1}% complete", progress * 100.0);
        Ok(())
    }

    /// Report task completion
    #[instrument(skip(self), fields(agent_id = ?self.context.agent_id, task_id = %task_result.task_id))]
    pub async fn report_task_completion(&mut self, task_result: TaskResult) -> Result<()> {
        debug!("Reporting task completion: {} (success: {})", 
               task_result.task_id, task_result.success);

        // Update metrics
        if task_result.success {
            self.context.metrics.tasks_completed += 1;
        } else {
            self.context.metrics.tasks_failed += 1;
        }
        
        self.context.metrics.total_execution_time += task_result.duration;
        if let Some(tokens) = task_result.llm_tokens_used {
            self.context.metrics.llm_tokens_consumed += tokens;
        }

        // Send task completion observation
        self.send_task_completion_observation(task_result).await?;

        // Update progress based on task completion
        let total_tasks = self.context.config.tasks.default.len() as f64;
        let completed_tasks = self.context.metrics.tasks_completed as f64;
        let progress = if total_tasks > 0.0 {
            completed_tasks / total_tasks
        } else {
            1.0
        };

        self.report_progress(progress, Some("Task completed".to_string())).await?;

        Ok(())
    }

    /// Report agent completion
    #[instrument(skip(self), fields(agent_id = ?self.context.agent_id))]
    pub async fn report_completion(&mut self, success: bool, message: Option<String>) -> Result<()> {
        info!("Reporting agent completion: success={}, agent_id={:?}", 
              success, self.context.agent_id);

        self.report_progress(1.0, message).await?;

        // Send final completion observation
        let completion_report = serde_json::json!({
            "type": "agent_completion",
            "agent_id": self.context.agent_id.0,
            "workstream": self.context.config.metadata.workstream,
            "success": success,
            "final_metrics": self.context.metrics,
            "completed_at": Utc::now(),
        });

        let observation_data = serde_json::to_vec(&completion_report)?;
        let message = Message {
            origin: self.context.agent_id,
            capability: "agent-completion".to_string(),
            op: Operation::EmitObservation {
                agent: self.context.agent_id,
                data: observation_data,
            },
        };

        self.runtime.submit(message).await.map_err(|e| {
            anyhow::anyhow!("Failed to submit completion observation: {}", e)
        })?;

        Ok(())
    }

    /// Update agent metrics
    pub fn update_metrics(&mut self, metrics: AgentMetrics) {
        self.context.metrics = metrics;
        self.context.last_activity = Utc::now();
    }

    /// Get current executing task
    fn get_current_task(&self) -> Option<String> {
        match &self.context.state {
            crate::AgentExecutionState::ExecutingTask { task_id } => Some(task_id.clone()),
            _ => None,
        }
    }

    /// Send progress observation to kernel
    async fn send_progress_observation(&self, progress: AgentProgress) -> Result<()> {
        let observation_data = serde_json::to_vec(&progress)?;
        let message = Message {
            origin: self.context.agent_id,
            capability: "progress-reporting".to_string(),
            op: Operation::EmitObservation {
                agent: self.context.agent_id,
                data: observation_data,
            },
        };

        self.runtime.submit(message).await.map_err(|e| {
            anyhow::anyhow!("Failed to submit progress observation: {}", e)
        })?;

        Ok(())
    }

    /// Send task completion observation to kernel
    async fn send_task_completion_observation(&self, task_result: TaskResult) -> Result<()> {
        let observation_data = serde_json::to_vec(&task_result)?;
        let message = Message {
            origin: self.context.agent_id,
            capability: "task-completion".to_string(),
            op: Operation::EmitObservation {
                agent: self.context.agent_id,
                data: observation_data,
            },
        };

        self.runtime.submit(message).await.map_err(|e| {
            anyhow::anyhow!("Failed to submit task completion observation: {}", e)
        })?;

        Ok(())
    }
}

impl TaskResult {
    /// Create a successful task result
    pub fn success(
        task_id: String,
        description: String,
        result_data: Option<String>,
        duration: Duration,
    ) -> Self {
        Self {
            task_id,
            description,
            success: true,
            result_data,
            error: None,
            duration,
            completed_at: Utc::now(),
            llm_tokens_used: None,
        }
    }

    /// Create a failed task result
    pub fn failure(
        task_id: String,
        description: String,
        error: String,
        duration: Duration,
    ) -> Self {
        Self {
            task_id,
            description,
            success: false,
            result_data: None,
            error: Some(error),
            duration,
            completed_at: Utc::now(),
            llm_tokens_used: None,
        }
    }

    /// Add LLM token usage information
    pub fn with_llm_tokens(mut self, tokens: u64) -> Self {
        self.llm_tokens_used = Some(tokens);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use toka_orchestration::{AgentConfig, AgentMetadata, AgentSpecConfig, AgentPriority};
    
    fn create_test_context() -> AgentContext {
        let config = AgentConfig {
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
            capabilities: Default::default(),
            objectives: vec![],
            tasks: Default::default(),
            dependencies: Default::default(),
            reporting: Default::default(),
            security: Default::default(),
        };

        AgentContext {
            agent_id: EntityId(123),
            config,
            state: crate::AgentExecutionState::Ready,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            metrics: AgentMetrics::default(),
            environment: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_task_result_success() {
        let result = TaskResult::success(
            "test-task".to_string(),
            "Test task description".to_string(),
            Some("Test result".to_string()),
            Duration::from_secs(10),
        ).with_llm_tokens(100);

        assert!(result.success);
        assert_eq!(result.task_id, "test-task");
        assert_eq!(result.llm_tokens_used, Some(100));
        assert!(result.error.is_none());
    }

    #[test]
    fn test_task_result_failure() {
        let result = TaskResult::failure(
            "test-task".to_string(),
            "Test task description".to_string(),
            "Test error".to_string(),
            Duration::from_secs(5),
        );

        assert!(!result.success);
        assert_eq!(result.error, Some("Test error".to_string()));
        assert!(result.result_data.is_none());
    }

    #[test]
    fn test_agent_progress_serialization() {
        let progress = AgentProgress {
            agent_id: EntityId(456),
            workstream: "test-workstream".to_string(),
            progress: 0.75,
            current_task: Some("current-task".to_string()),
            tasks_completed: 3,
            total_tasks: 4,
            metrics: AgentMetrics::default(),
            timestamp: Utc::now(),
            message: Some("Test message".to_string()),
        };

        let serialized = serde_json::to_string(&progress).unwrap();
        let deserialized: AgentProgress = serde_json::from_str(&serialized).unwrap();

        assert_eq!(progress.agent_id, deserialized.agent_id);
        assert_eq!(progress.progress, deserialized.progress);
        assert_eq!(progress.workstream, deserialized.workstream);
    }
}
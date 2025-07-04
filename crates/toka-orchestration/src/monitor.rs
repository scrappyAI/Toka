//! Progress monitoring for agent orchestration.
//!
//! This module provides functionality to monitor agent progress, track completion
//! status, and coordinate between different orchestration phases based on agent
//! state and milestone achievement.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use toka_types::TaskSpec;

use crate::{AgentConfig, AgentState, OrchestrationPhase};

/// Progress monitor for tracking agent orchestration progress.
pub struct ProgressMonitor {
    /// Agent progress tracking
    agent_progress: Arc<DashMap<String, AgentProgress>>,
    /// Phase completion tracking
    phase_progress: Arc<RwLock<PhaseProgress>>,
    /// Event listeners for real-time updates
    event_listeners: Arc<RwLock<Vec<mpsc::UnboundedSender<ProgressEvent>>>>,
    /// Metrics collection
    metrics: Arc<RwLock<OrchestrationMetrics>>,
}

/// Progress information for a single agent.
#[derive(Debug, Clone)]
pub struct AgentProgress {
    /// Agent name
    pub name: String,
    /// Current state
    pub state: AgentState,
    /// Progress percentage (0.0 to 1.0)
    pub progress: f64,
    /// Last update timestamp
    pub last_update: DateTime<Utc>,
    /// Completed tasks
    pub completed_tasks: usize,
    /// Total assigned tasks
    pub total_tasks: usize,
    /// Active duration
    pub active_duration: Duration,
    /// Last known milestone
    pub last_milestone: Option<String>,
    /// Error information if agent failed
    pub error: Option<String>,
}

/// Progress tracking for orchestration phases.
#[derive(Debug, Clone)]
pub struct PhaseProgress {
    /// Current phase
    pub current_phase: OrchestrationPhase,
    /// Phase start time
    pub phase_start: DateTime<Utc>,
    /// Phase progress (0.0 to 1.0)
    pub phase_progress: f64,
    /// Agents in current phase
    pub agents_in_phase: Vec<String>,
    /// Completed agents in current phase
    pub completed_agents: Vec<String>,
    /// Phase-specific metrics
    pub phase_metrics: HashMap<String, f64>,
}

/// Overall orchestration metrics.
#[derive(Debug, Clone, Default)]
pub struct OrchestrationMetrics {
    /// Total agents spawned
    pub total_agents: usize,
    /// Agents currently active
    pub active_agents: usize,
    /// Agents completed successfully
    pub completed_agents: usize,
    /// Agents that failed
    pub failed_agents: usize,
    /// Average agent execution time
    pub avg_execution_time: Duration,
    /// Total orchestration time
    pub total_orchestration_time: Duration,
    /// Tasks completed across all agents
    pub total_tasks_completed: usize,
    /// Tasks failed across all agents
    pub total_tasks_failed: usize,
    /// Current throughput (agents/minute)
    pub throughput: f64,
}

/// Progress events emitted by the monitor.
#[derive(Debug, Clone)]
pub enum ProgressEvent {
    /// Agent state changed
    AgentStateChanged {
        /// Agent name
        agent: String,
        /// Previous state
        old_state: AgentState,
        /// New state
        new_state: AgentState,
    },
    /// Agent progress updated
    AgentProgressUpdated {
        /// Agent name
        agent: String,
        /// Progress percentage (0.0 to 1.0)
        progress: f64,
    },
    /// Phase changed
    PhaseChanged {
        /// Previous phase
        old_phase: OrchestrationPhase,
        /// New phase
        new_phase: OrchestrationPhase,
    },
    /// Milestone reached
    MilestoneReached {
        /// Agent name
        agent: String,
        /// Milestone name
        milestone: String,
    },
    /// Agent completed
    AgentCompleted {
        /// Agent name
        agent: String,
        /// Total execution time
        execution_time: Duration,
    },
    /// Agent failed
    AgentFailed {
        /// Agent name
        agent: String,
        /// Error message
        error: String,
    },
}

/// Milestone tracking for agents.
#[derive(Debug, Clone)]
pub struct Milestone {
    /// Milestone name
    pub name: String,
    /// Milestone description
    pub description: String,
    /// Progress threshold (0.0 to 1.0)
    pub threshold: f64,
    /// Whether milestone is reached
    pub reached: bool,
    /// Timestamp when reached
    pub reached_at: Option<DateTime<Utc>>,
}

impl ProgressMonitor {
    /// Create a new progress monitor.
    pub fn new() -> Self {
        Self {
            agent_progress: Arc::new(DashMap::new()),
            phase_progress: Arc::new(RwLock::new(PhaseProgress {
                current_phase: OrchestrationPhase::Initializing,
                phase_start: Utc::now(),
                phase_progress: 0.0,
                agents_in_phase: Vec::new(),
                completed_agents: Vec::new(),
                phase_metrics: HashMap::new(),
            })),
            event_listeners: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(RwLock::new(OrchestrationMetrics::default())),
        }
    }

    /// Initialize agent progress tracking.
    pub async fn initialize_agent_tracking(&self, agents: &[AgentConfig]) -> Result<()> {
        info!("Initializing progress tracking for {} agents", agents.len());

        for agent in agents {
            let progress = AgentProgress {
                name: agent.metadata.name.clone(),
                state: AgentState::Configured,
                progress: 0.0,
                last_update: Utc::now(),
                completed_tasks: 0,
                total_tasks: agent.tasks.default.len(),
                active_duration: Duration::from_secs(0),
                last_milestone: None,
                error: None,
            };

            self.agent_progress.insert(agent.metadata.name.clone(), progress);
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_agents = agents.len();

        info!("Progress tracking initialized for {} agents", agents.len());
        Ok(())
    }

    /// Update agent state and progress.
    pub async fn update_agent_state(&self, agent_name: &str, new_state: AgentState) -> Result<()> {
        let old_state = if let Some(mut progress) = self.agent_progress.get_mut(agent_name) {
            let old_state = progress.state.clone();
            progress.state = new_state.clone();
            progress.last_update = Utc::now();
            old_state
        } else {
            return Err(anyhow::anyhow!("Agent not found: {}", agent_name));
        };

        // Update metrics based on state change
        self.update_metrics_for_state_change(&old_state, &new_state).await;

        // Emit event
        self.emit_event(ProgressEvent::AgentStateChanged {
            agent: agent_name.to_string(),
            old_state,
            new_state: new_state.clone(),
        }).await;

        debug!("Agent state updated: {} -> {:?}", agent_name, new_state);
        Ok(())
    }

    /// Update agent progress percentage.
    pub async fn update_agent_progress(&self, agent_name: &str, progress: f64) -> Result<()> {
        if let Some(mut agent_progress) = self.agent_progress.get_mut(agent_name) {
            agent_progress.progress = progress.clamp(0.0, 1.0);
            agent_progress.last_update = Utc::now();

            // Check for completion
            if progress >= 1.0 && agent_progress.state == AgentState::Active {
                agent_progress.state = AgentState::Completed;
                
                // Emit completion event
                self.emit_event(ProgressEvent::AgentCompleted {
                    agent: agent_name.to_string(),
                    execution_time: agent_progress.active_duration,
                }).await;
            }
        } else {
            return Err(anyhow::anyhow!("Agent not found: {}", agent_name));
        }

        // Emit progress event
        self.emit_event(ProgressEvent::AgentProgressUpdated {
            agent: agent_name.to_string(),
            progress,
        }).await;

        debug!("Agent progress updated: {} -> {:.2}%", agent_name, progress * 100.0);
        Ok(())
    }

    /// Update phase progress.
    pub async fn update_phase(&self, new_phase: OrchestrationPhase) -> Result<()> {
        let old_phase = {
            let mut phase_progress = self.phase_progress.write().await;
            let old_phase = phase_progress.current_phase.clone();
            phase_progress.current_phase = new_phase.clone();
            phase_progress.phase_start = Utc::now();
            phase_progress.phase_progress = 0.0;
            old_phase
        };

        // Emit event
        self.emit_event(ProgressEvent::PhaseChanged {
            old_phase,
            new_phase: new_phase.clone(),
        }).await;

        info!("Orchestration phase updated: {:?}", new_phase);
        Ok(())
    }

    /// Mark agent as completed.
    pub async fn mark_agent_completed(&self, agent_name: &str, execution_time: Duration) -> Result<()> {
        if let Some(mut progress) = self.agent_progress.get_mut(agent_name) {
            progress.state = AgentState::Completed;
            progress.progress = 1.0;
            progress.active_duration = execution_time;
            progress.last_update = Utc::now();
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.completed_agents += 1;
        metrics.active_agents = metrics.active_agents.saturating_sub(1);

        // Emit completion event
        self.emit_event(ProgressEvent::AgentCompleted {
            agent: agent_name.to_string(),
            execution_time,
        }).await;

        info!("Agent completed: {} (execution time: {:?})", agent_name, execution_time);
        Ok(())
    }

    /// Mark agent as failed.
    pub async fn mark_agent_failed(&self, agent_name: &str, error: String) -> Result<()> {
        if let Some(mut progress) = self.agent_progress.get_mut(agent_name) {
            progress.state = AgentState::Failed;
            progress.error = Some(error.clone());
            progress.last_update = Utc::now();
        }

        // Update metrics
        let mut metrics = self.metrics.write().await;
        metrics.failed_agents += 1;
        metrics.active_agents = metrics.active_agents.saturating_sub(1);

        // Emit failure event
        self.emit_event(ProgressEvent::AgentFailed {
            agent: agent_name.to_string(),
            error,
        }).await;

        warn!("Agent failed: {}", agent_name);
        Ok(())
    }

    /// Record agent task completion.
    pub async fn record_task_completion(&self, agent_name: &str, task: &TaskSpec) -> Result<()> {
        if let Some(mut progress) = self.agent_progress.get_mut(agent_name) {
            progress.completed_tasks += 1;
            progress.last_update = Utc::now();

            // Update progress based on task completion
            if progress.total_tasks > 0 {
                progress.progress = progress.completed_tasks as f64 / progress.total_tasks as f64;
            }
        }

        // Update global metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_tasks_completed += 1;

        debug!("Task completed for agent {}: {}", agent_name, task.description);
        Ok(())
    }

    /// Record agent task failure.
    pub async fn record_task_failure(&self, agent_name: &str, task: &TaskSpec, error: &str) -> Result<()> {
        // Update global metrics
        let mut metrics = self.metrics.write().await;
        metrics.total_tasks_failed += 1;

        warn!("Task failed for agent {}: {} - {}", agent_name, task.description, error);
        Ok(())
    }

    /// Get agent progress.
    pub fn get_agent_progress(&self, agent_name: &str) -> Option<AgentProgress> {
        self.agent_progress.get(agent_name).map(|p| p.clone())
    }

    /// Get all agent progress.
    pub fn get_all_agent_progress(&self) -> HashMap<String, AgentProgress> {
        self.agent_progress.iter()
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Get phase progress.
    pub async fn get_phase_progress(&self) -> PhaseProgress {
        self.phase_progress.read().await.clone()
    }

    /// Get orchestration metrics.
    pub async fn get_metrics(&self) -> OrchestrationMetrics {
        self.metrics.read().await.clone()
    }

    /// Calculate overall orchestration progress.
    pub fn calculate_overall_progress(&self) -> f64 {
        let all_progress = self.get_all_agent_progress();
        if all_progress.is_empty() {
            return 0.0;
        }

        let total_progress: f64 = all_progress.values().map(|p| p.progress).sum();
        total_progress / all_progress.len() as f64
    }

    /// Check if phase is complete.
    pub fn is_phase_complete(&self, phase: &OrchestrationPhase) -> bool {
        let all_progress = self.get_all_agent_progress();
        
        match phase {
            OrchestrationPhase::CriticalInfrastructure => {
                // Critical phase is complete when all critical agents are completed
                all_progress.values().all(|p| 
                    p.state == AgentState::Completed || 
                    p.state != AgentState::Active
                )
            }
            OrchestrationPhase::FoundationServices => {
                // Foundation phase is complete when all high-priority agents are completed
                all_progress.values().all(|p| 
                    p.state == AgentState::Completed || 
                    p.state != AgentState::Active
                )
            }
            OrchestrationPhase::ParallelDevelopment => {
                // Development phase is complete when all agents are completed
                all_progress.values().all(|p| 
                    p.state == AgentState::Completed || 
                    p.state == AgentState::Failed
                )
            }
            _ => false,
        }
    }

    /// Subscribe to progress events.
    pub async fn subscribe_events(&self) -> mpsc::UnboundedReceiver<ProgressEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut listeners = self.event_listeners.write().await;
        listeners.push(tx);
        rx
    }

    /// Emit a progress event to all listeners.
    async fn emit_event(&self, event: ProgressEvent) {
        let listeners = self.event_listeners.read().await;
        for listener in listeners.iter() {
            // Ignore send errors (listener disconnected)
            let _ = listener.send(event.clone());
        }
    }

    /// Update metrics based on state change.
    async fn update_metrics_for_state_change(&self, old_state: &AgentState, new_state: &AgentState) {
        let mut metrics = self.metrics.write().await;
        
        match (old_state, new_state) {
            (AgentState::Spawning, AgentState::Active) => {
                metrics.active_agents += 1;
            }
            (AgentState::Active, AgentState::Completed) => {
                metrics.completed_agents += 1;
                metrics.active_agents = metrics.active_agents.saturating_sub(1);
            }
            (AgentState::Active, AgentState::Failed) => {
                metrics.failed_agents += 1;
                metrics.active_agents = metrics.active_agents.saturating_sub(1);
            }
            _ => {}
        }
    }
}

impl Default for ProgressMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AgentProgress {
    fn default() -> Self {
        Self {
            name: String::new(),
            state: AgentState::Configured,
            progress: 0.0,
            last_update: Utc::now(),
            completed_tasks: 0,
            total_tasks: 0,
            active_duration: Duration::from_secs(0),
            last_milestone: None,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentConfig, AgentMetadata, AgentSpecConfig, AgentCapabilities, AgentObjective, AgentTasks, AgentDependencies, ReportingConfig, SecurityConfig, TaskConfig, TaskPriority, ReportingFrequency, ResourceLimits, AgentPriority};
    use std::collections::HashMap;

    fn create_test_agent(name: &str) -> AgentConfig {
        AgentConfig {
            metadata: AgentMetadata {
                name: name.to_string(),
                version: "v1.0".to_string(),
                created: "2024-01-01".to_string(),
                workstream: "test".to_string(),
                branch: "main".to_string(),
            },
            spec: AgentSpecConfig {
                name: name.to_string(),
                domain: "test".to_string(),
                priority: AgentPriority::Medium,
            },
            capabilities: AgentCapabilities {
                primary: vec!["test".to_string()],
                secondary: vec![],
            },
            objectives: vec![AgentObjective {
                description: "Test objective".to_string(),
                deliverable: "Test deliverable".to_string(),
                validation: "Test validation".to_string(),
            }],
            tasks: AgentTasks {
                default: vec![TaskConfig {
                    description: "Test task".to_string(),
                    priority: TaskPriority::Medium,
                }],
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
                    timeout: "1h".to_string(),
                },
            },
        }
    }

    #[tokio::test]
    async fn test_progress_monitor_initialization() {
        let monitor = ProgressMonitor::new();
        let agents = vec![create_test_agent("test-agent")];
        
        monitor.initialize_agent_tracking(&agents).await.unwrap();
        
        let progress = monitor.get_agent_progress("test-agent").unwrap();
        assert_eq!(progress.name, "test-agent");
        assert_eq!(progress.state, AgentState::Configured);
        assert_eq!(progress.total_tasks, 1);
    }

    #[tokio::test]
    async fn test_agent_state_updates() {
        let monitor = ProgressMonitor::new();
        let agents = vec![create_test_agent("test-agent")];
        
        monitor.initialize_agent_tracking(&agents).await.unwrap();
        monitor.update_agent_state("test-agent", AgentState::Active).await.unwrap();
        
        let progress = monitor.get_agent_progress("test-agent").unwrap();
        assert_eq!(progress.state, AgentState::Active);
    }

    #[tokio::test]
    async fn test_progress_calculation() {
        let monitor = ProgressMonitor::new();
        let agents = vec![
            create_test_agent("agent1"),
            create_test_agent("agent2"),
        ];
        
        monitor.initialize_agent_tracking(&agents).await.unwrap();
        monitor.update_agent_progress("agent1", 0.5).await.unwrap();
        monitor.update_agent_progress("agent2", 0.8).await.unwrap();
        
        let overall = monitor.calculate_overall_progress();
        assert_eq!(overall, 0.65); // (0.5 + 0.8) / 2
    }

    #[tokio::test]
    async fn test_task_completion_tracking() {
        let monitor = ProgressMonitor::new();
        let agents = vec![create_test_agent("test-agent")];
        
        monitor.initialize_agent_tracking(&agents).await.unwrap();
        
        let task = TaskSpec::new("test task".to_string()).unwrap();
        monitor.record_task_completion("test-agent", &task).await.unwrap();
        
        let progress = monitor.get_agent_progress("test-agent").unwrap();
        assert_eq!(progress.completed_tasks, 1);
        assert_eq!(progress.progress, 1.0); // 1/1 task completed
    }
} 
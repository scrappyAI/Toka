//! Workstream coordination for agent orchestration.
//!
//! This module provides functionality to coordinate agents within workstreams,
//! manage cross-workstream dependencies, and ensure proper communication
//! between agents working on related tasks.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use crate::{AgentConfig, AgentState};

/// Workstream coordinator for managing agent coordination within workstreams.
pub struct WorkstreamCoordinator {
    /// Workstream definitions
    workstreams: Arc<RwLock<HashMap<String, Workstream>>>,
    /// Agent to workstream mapping
    agent_workstreams: Arc<DashMap<String, String>>,
    /// Cross-workstream dependencies
    workstream_dependencies: Arc<RwLock<HashMap<String, HashSet<String>>>>,
    /// Communication channels between workstreams
    communication_channels: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<WorkstreamMessage>>>>,
    /// Coordination events
    event_listeners: Arc<RwLock<Vec<mpsc::UnboundedSender<CoordinationEvent>>>>,
}

/// Workstream definition and state.
#[derive(Debug, Clone)]
pub struct Workstream {
    /// Workstream name
    pub name: String,
    /// Workstream description
    pub description: String,
    /// Agents assigned to this workstream
    pub agents: Vec<String>,
    /// Workstream state
    pub state: WorkstreamState,
    /// Start time
    pub started_at: Option<DateTime<Utc>>,
    /// Completion time
    pub completed_at: Option<DateTime<Utc>>,
    /// Workstream progress (0.0 to 1.0)
    pub progress: f64,
    /// Workstream deliverables
    pub deliverables: Vec<WorkstreamDeliverable>,
    /// Communication preferences
    pub communication: WorkstreamCommunication,
}

/// Workstream state tracking.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkstreamState {
    /// Workstream is defined but not started
    Defined,
    /// Workstream is waiting for dependencies
    Waiting,
    /// Workstream is actively running
    Active,
    /// Workstream is paused
    Paused,
    /// Workstream is completed successfully
    Completed,
    /// Workstream failed
    Failed,
}

/// Workstream deliverable tracking.
#[derive(Debug, Clone)]
pub struct WorkstreamDeliverable {
    /// Deliverable name
    pub name: String,
    /// Deliverable description
    pub description: String,
    /// Responsible agent
    pub responsible_agent: String,
    /// Completion status
    pub completed: bool,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
    /// Deliverable artifacts
    pub artifacts: Vec<String>,
}

/// Workstream communication configuration.
#[derive(Debug, Clone)]
pub struct WorkstreamCommunication {
    /// Communication frequency
    pub frequency: CommunicationFrequency,
    /// Communication channels
    pub channels: Vec<String>,
    /// Notification preferences
    pub notifications: Vec<NotificationPreference>,
}

/// Communication frequency options.
#[derive(Debug, Clone, PartialEq)]
pub enum CommunicationFrequency {
    /// Real-time communication
    RealTime,
    /// Hourly updates
    Hourly,
    /// Daily updates
    Daily,
    /// Weekly updates
    Weekly,
    /// Milestone-based updates
    Milestone,
}

/// Notification preference configuration.
#[derive(Debug, Clone)]
pub struct NotificationPreference {
    /// Event type to notify about
    pub event_type: String,
    /// Notification method
    pub method: NotificationMethod,
    /// Notification recipients
    pub recipients: Vec<String>,
}

/// Notification delivery methods.
#[derive(Debug, Clone)]
pub enum NotificationMethod {
    /// In-system event
    Event,
    /// Log message
    Log,
    /// External webhook
    Webhook { 
        /// Webhook URL endpoint
        url: String 
    },
}

/// Messages exchanged between workstreams.
#[derive(Debug, Clone)]
pub enum WorkstreamMessage {
    /// Status update from workstream
    StatusUpdate {
        /// Name of the workstream
        workstream: String,
        /// Name of the agent
        agent: String,
        /// Current agent status
        status: AgentState,
        /// Progress percentage (0.0 to 1.0)
        progress: f64,
    },
    /// Deliverable completion notification
    DeliverableCompleted {
        /// Name of the workstream
        workstream: String,
        /// Name of the deliverable
        deliverable: String,
        /// Name of the agent
        agent: String,
        /// List of artifacts produced
        artifacts: Vec<String>,
    },
    /// Dependency satisfaction notification
    DependencySatisfied {
        /// Name of the workstream
        workstream: String,
        /// Name of the dependency
        dependency: String,
    },
    /// Request for assistance
    AssistanceRequest {
        /// Workstream requesting assistance
        from_workstream: String,
        /// Workstream being asked for assistance
        to_workstream: String,
        /// Description of the assistance request
        request: String,
        /// Priority level of the request
        priority: RequestPriority,
    },
    /// Resource sharing request
    ResourceRequest {
        /// Workstream requesting resource
        from_workstream: String,
        /// Workstream being asked to share resource
        to_workstream: String,
        /// Description of the resource needed
        resource: String,
        /// Duration for which resource is needed
        duration: Duration,
    },
}

/// Priority levels for assistance requests.
#[derive(Debug, Clone, PartialEq)]
pub enum RequestPriority {
    /// Low priority request
    Low,
    /// Medium priority request
    Medium,
    /// High priority request
    High,
    /// Critical priority request
    Critical,
}

/// Coordination events emitted by the coordinator.
#[derive(Debug, Clone)]
pub enum CoordinationEvent {
    /// Workstream state changed
    WorkstreamStateChanged {
        /// Name of the workstream
        workstream: String,
        /// Previous state of the workstream
        old_state: WorkstreamState,
        /// New state of the workstream
        new_state: WorkstreamState,
    },
    /// Cross-workstream dependency satisfied
    DependencySatisfied {
        /// Name of the workstream
        workstream: String,
        /// Name of the dependency that was satisfied
        dependency: String,
    },
    /// Workstream communication event
    CommunicationEvent {
        /// Source workstream
        from_workstream: String,
        /// Destination workstream
        to_workstream: String,
        /// Message being sent
        message: WorkstreamMessage,
    },
    /// Deliverable completed
    DeliverableCompleted {
        /// Name of the workstream
        workstream: String,
        /// Name of the deliverable
        deliverable: String,
        /// Name of the agent who completed it
        agent: String,
    },
}

impl WorkstreamCoordinator {
    /// Create a new workstream coordinator.
    pub fn new() -> Self {
        Self {
            workstreams: Arc::new(RwLock::new(HashMap::new())),
            agent_workstreams: Arc::new(DashMap::new()),
            workstream_dependencies: Arc::new(RwLock::new(HashMap::new())),
            communication_channels: Arc::new(RwLock::new(HashMap::new())),
            event_listeners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Initialize workstreams from agent configurations.
    pub async fn initialize_workstreams(&self, agents: &[AgentConfig]) -> Result<()> {
        info!("Initializing workstreams from {} agent configurations", agents.len());

        let mut workstreams = self.workstreams.write().await;
        let mut workstream_dependencies = self.workstream_dependencies.write().await;

        // Group agents by workstream
        let mut workstream_agents: HashMap<String, Vec<String>> = HashMap::new();
        for agent in agents {
            self.agent_workstreams.insert(agent.metadata.name.clone(), agent.metadata.workstream.clone());
            workstream_agents.entry(agent.metadata.workstream.clone())
                .or_default()
                .push(agent.metadata.name.clone());
        }

        // Create workstream definitions
        for (workstream_name, agent_names) in workstream_agents {
            let workstream = Workstream {
                name: workstream_name.clone(),
                description: format!("Workstream for {}", workstream_name),
                agents: agent_names,
                state: WorkstreamState::Defined,
                started_at: None,
                completed_at: None,
                progress: 0.0,
                deliverables: Vec::new(),
                communication: WorkstreamCommunication {
                    frequency: CommunicationFrequency::Daily,
                    channels: vec!["default".to_string()],
                    notifications: Vec::new(),
                },
            };

            workstreams.insert(workstream_name.clone(), workstream);
            workstream_dependencies.insert(workstream_name, HashSet::new());
        }

        // Analyze cross-workstream dependencies
        for agent in agents {
            let agent_workstream = &agent.metadata.workstream;
            
            // Check if agent has dependencies on other workstreams
            for (dep_name, _reason) in &agent.dependencies.required {
                if let Some(dep_workstream) = self.agent_workstreams.get(dep_name) {
                    if dep_workstream.value() != agent_workstream {
                        // Cross-workstream dependency
                        workstream_dependencies.entry(agent_workstream.clone())
                            .or_default()
                            .insert(dep_workstream.value().clone());
                    }
                }
            }
        }

        info!("Initialized {} workstreams", workstreams.len());
        Ok(())
    }

    /// Start a workstream.
    pub async fn start_workstream(&self, workstream_name: &str) -> Result<()> {
        // Check if dependencies are satisfied before acquiring write lock
        let dependencies_satisfied = self.check_workstream_dependencies(workstream_name).await?;
        
        let mut workstreams = self.workstreams.write().await;
        
        if let Some(workstream) = workstreams.get_mut(workstream_name) {
            if dependencies_satisfied {
                let old_state = workstream.state.clone();
                workstream.state = WorkstreamState::Active;
                workstream.started_at = Some(Utc::now());

                // Emit event
                self.emit_event(CoordinationEvent::WorkstreamStateChanged {
                    workstream: workstream_name.to_string(),
                    old_state,
                    new_state: WorkstreamState::Active,
                }).await;

                info!("Workstream started: {}", workstream_name);
            } else {
                workstream.state = WorkstreamState::Waiting;
                info!("Workstream waiting for dependencies: {}", workstream_name);
            }
        } else {
            return Err(anyhow::anyhow!("Workstream not found: {}", workstream_name));
        }

        Ok(())
    }

    /// Update workstream progress.
    pub async fn update_workstream_progress(&self, workstream_name: &str, progress: f64) -> Result<()> {
        let mut workstreams = self.workstreams.write().await;
        
        if let Some(workstream) = workstreams.get_mut(workstream_name) {
            workstream.progress = progress.clamp(0.0, 1.0);
            
            // Check for completion
            if progress >= 1.0 && workstream.state == WorkstreamState::Active {
                let old_state = workstream.state.clone();
                workstream.state = WorkstreamState::Completed;
                workstream.completed_at = Some(Utc::now());

                // Emit event
                self.emit_event(CoordinationEvent::WorkstreamStateChanged {
                    workstream: workstream_name.to_string(),
                    old_state,
                    new_state: WorkstreamState::Completed,
                }).await;

                // Notify dependent workstreams
                self.notify_dependent_workstreams(workstream_name).await?;

                info!("Workstream completed: {}", workstream_name);
            }
        } else {
            return Err(anyhow::anyhow!("Workstream not found: {}", workstream_name));
        }

        Ok(())
    }

    /// Record deliverable completion.
    pub async fn complete_deliverable(
        &self,
        workstream_name: &str,
        deliverable_name: &str,
        agent_name: &str,
        artifacts: Vec<String>,
    ) -> Result<()> {
        let mut workstreams = self.workstreams.write().await;
        
        if let Some(workstream) = workstreams.get_mut(workstream_name) {
            // Find and update deliverable
            for deliverable in &mut workstream.deliverables {
                if deliverable.name == deliverable_name {
                    deliverable.completed = true;
                    deliverable.completed_at = Some(Utc::now());
                    deliverable.artifacts = artifacts.clone();
                    break;
                }
            }

            // Emit event
            self.emit_event(CoordinationEvent::DeliverableCompleted {
                workstream: workstream_name.to_string(),
                deliverable: deliverable_name.to_string(),
                agent: agent_name.to_string(),
            }).await;

            info!("Deliverable completed: {} in workstream {}", deliverable_name, workstream_name);
        } else {
            return Err(anyhow::anyhow!("Workstream not found: {}", workstream_name));
        }

        Ok(())
    }

    /// Send message between workstreams.
    pub async fn send_workstream_message(
        &self,
        from_workstream: &str,
        to_workstream: &str,
        message: WorkstreamMessage,
    ) -> Result<()> {
        let channels = self.communication_channels.read().await;
        
        if let Some(sender) = channels.get(to_workstream) {
            sender.send(message.clone()).map_err(|e| {
                anyhow::anyhow!("Failed to send message to workstream {}: {}", to_workstream, e)
            })?;

            // Emit coordination event
            self.emit_event(CoordinationEvent::CommunicationEvent {
                from_workstream: from_workstream.to_string(),
                to_workstream: to_workstream.to_string(),
                message,
            }).await;

            debug!("Message sent from {} to {}", from_workstream, to_workstream);
        } else {
            warn!("No communication channel found for workstream: {}", to_workstream);
        }

        Ok(())
    }

    /// Get workstream information.
    pub async fn get_workstream(&self, workstream_name: &str) -> Option<Workstream> {
        let workstreams = self.workstreams.read().await;
        workstreams.get(workstream_name).cloned()
    }

    /// Get all workstreams.
    pub async fn get_all_workstreams(&self) -> HashMap<String, Workstream> {
        self.workstreams.read().await.clone()
    }

    /// Get workstream for an agent.
    pub fn get_agent_workstream(&self, agent_name: &str) -> Option<String> {
        self.agent_workstreams.get(agent_name).map(|w| w.value().clone())
    }

    /// Check if workstream dependencies are satisfied.
    async fn check_workstream_dependencies(&self, workstream_name: &str) -> Result<bool> {
        let workstream_dependencies = self.workstream_dependencies.read().await;
        let workstreams = self.workstreams.read().await;

        if let Some(deps) = workstream_dependencies.get(workstream_name) {
            for dep in deps {
                if let Some(dep_workstream) = workstreams.get(dep) {
                    if dep_workstream.state != WorkstreamState::Completed {
                        return Ok(false);
                    }
                } else {
                    return Err(anyhow::anyhow!("Dependency workstream not found: {}", dep));
                }
            }
        }

        Ok(true)
    }

    /// Notify dependent workstreams when a workstream completes.
    async fn notify_dependent_workstreams(&self, completed_workstream: &str) -> Result<()> {
        let workstream_dependencies = self.workstream_dependencies.read().await;
        
        for (workstream, deps) in workstream_dependencies.iter() {
            if deps.contains(completed_workstream) {
                // Notify this workstream that its dependency is satisfied
                self.emit_event(CoordinationEvent::DependencySatisfied {
                    workstream: workstream.clone(),
                    dependency: completed_workstream.to_string(),
                }).await;

                // Check if workstream can now start (only if it's currently waiting)
                let workstreams = self.workstreams.read().await;
                if let Some(ws) = workstreams.get(workstream) {
                    if ws.state == WorkstreamState::Waiting {
                        // Check if all dependencies are satisfied
                        if self.check_workstream_dependencies(workstream).await? {
                            // Drop the read lock before calling start_workstream to avoid deadlock
                            drop(workstreams);
                            self.start_workstream(workstream).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Subscribe to coordination events.
    pub async fn subscribe_events(&self) -> mpsc::UnboundedReceiver<CoordinationEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut listeners = self.event_listeners.write().await;
        listeners.push(tx);
        rx
    }

    /// Emit a coordination event.
    async fn emit_event(&self, event: CoordinationEvent) {
        let listeners = self.event_listeners.read().await;
        for listener in listeners.iter() {
            // Ignore send errors (listener disconnected)
            let _ = listener.send(event.clone());
        }
    }

    /// Calculate overall workstream progress.
    pub async fn calculate_overall_progress(&self) -> f64 {
        let workstreams = self.workstreams.read().await;
        if workstreams.is_empty() {
            return 0.0;
        }

        let total_progress: f64 = workstreams.values().map(|w| w.progress).sum();
        total_progress / workstreams.len() as f64
    }

    /// Get workstreams by state.
    pub async fn get_workstreams_by_state(&self, state: WorkstreamState) -> Vec<Workstream> {
        let workstreams = self.workstreams.read().await;
        workstreams.values()
            .filter(|w| w.state == state)
            .cloned()
            .collect()
    }
}

impl Default for WorkstreamCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AgentConfig, AgentMetadata, AgentSpecConfig, AgentCapabilities, AgentObjective, AgentTasks, AgentDependencies, ReportingConfig, SecurityConfig, TaskConfig, TaskPriority, ReportingFrequency, ResourceLimits, AgentPriority};
    use std::collections::HashMap;

    fn create_test_agent(name: &str, workstream: &str) -> AgentConfig {
        AgentConfig {
            metadata: AgentMetadata {
                name: name.to_string(),
                version: "v1.0".to_string(),
                created: "2024-01-01".to_string(),
                workstream: workstream.to_string(),
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
    async fn test_workstream_coordinator_initialization() {
        let coordinator = WorkstreamCoordinator::new();
        let agents = vec![
            create_test_agent("agent1", "workstream1"),
            create_test_agent("agent2", "workstream1"),
            create_test_agent("agent3", "workstream2"),
        ];

        coordinator.initialize_workstreams(&agents).await.unwrap();

        let workstreams = coordinator.get_all_workstreams().await;
        assert_eq!(workstreams.len(), 2);
        assert!(workstreams.contains_key("workstream1"));
        assert!(workstreams.contains_key("workstream2"));
    }

    #[tokio::test]
    async fn test_workstream_progress_tracking() {
        let coordinator = WorkstreamCoordinator::new();
        let agents = vec![create_test_agent("agent1", "workstream1")];

        coordinator.initialize_workstreams(&agents).await.unwrap();
        
        // Add timeout to prevent hanging
        let start_result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            coordinator.start_workstream("workstream1")
        ).await;
        start_result.unwrap().unwrap();
        
        coordinator.update_workstream_progress("workstream1", 0.5).await.unwrap();

        let workstream = coordinator.get_workstream("workstream1").await.unwrap();
        assert_eq!(workstream.progress, 0.5);
        assert_eq!(workstream.state, WorkstreamState::Active);
    }

    #[tokio::test]
    async fn test_workstream_completion() {
        let coordinator = WorkstreamCoordinator::new();
        let agents = vec![create_test_agent("agent1", "workstream1")];

        coordinator.initialize_workstreams(&agents).await.unwrap();
        
        // Add timeout to prevent hanging
        let start_result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            coordinator.start_workstream("workstream1")
        ).await;
        start_result.unwrap().unwrap();
        
        coordinator.update_workstream_progress("workstream1", 1.0).await.unwrap();

        let workstream = coordinator.get_workstream("workstream1").await.unwrap();
        assert_eq!(workstream.state, WorkstreamState::Completed);
        assert!(workstream.completed_at.is_some());
    }
} 
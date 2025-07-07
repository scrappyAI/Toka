#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-types** – Shared primitive data structures for Toka OS.
//!
//! The crate is dependency‐light and sits at the very bottom of the crate
//! graph so that *every* other crate can depend on it without causing cycles.
//! It intentionally makes no assumptions about I/O, cryptography, or storage.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//─────────────────────────────
//  Security constants
//─────────────────────────────

/// Maximum allowed size for task descriptions to prevent memory exhaustion attacks
pub const MAX_TASK_DESCRIPTION_LEN: usize = 4096;

/// Maximum allowed size for agent names to prevent memory exhaustion attacks  
pub const MAX_AGENT_NAME_LEN: usize = 256;

/// Maximum allowed size for observation data to prevent memory exhaustion attacks
pub const MAX_OBSERVATION_DATA_LEN: usize = 1_048_576; // 1MB

/// Maximum allowed size for capability tokens to prevent memory exhaustion attacks
pub const MAX_CAPABILITY_TOKEN_LEN: usize = 8192;

//─────────────────────────────
//  Core behaviour traits
//─────────────────────────────

/// Behaviour traits (`Tool`, `Agent`, `Resource`) shared across crates.
pub mod traits;
pub use traits::{Agent, Tool, Resource, Params, ToolResult, ToolMetadata};

//─────────────────────────────
//  Core identifiers
//─────────────────────────────

/// Unique, 128-bit identifier for *any* entity inside Toka.
///
/// Entities can be users, agents, assets, system modules, etc.  The kernel
/// treats them uniformly which keeps capability checks and storage schemas
/// simple.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntityId(pub u128);

//─────────────────────────────
//  Domain abstractions (stubs)
//─────────────────────────────

/// Specification of a task to be executed by an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Human-readable description (v0.1 placeholder).
    pub description: String,
}

impl TaskSpec {
    /// Create a new task specification with validation.
    /// 
    /// # Security
    /// Validates that the description doesn't exceed maximum length to prevent
    /// memory exhaustion attacks.
    pub fn new(description: String) -> Result<Self, String> {
        if description.len() > MAX_TASK_DESCRIPTION_LEN {
            return Err(format!(
                "Task description too long: {} > {}",
                description.len(),
                MAX_TASK_DESCRIPTION_LEN
            ));
        }
        if description.trim().is_empty() {
            return Err("Task description cannot be empty".to_string());
        }
        Ok(Self { description })
    }

    /// Validate an existing task specification.
    pub fn validate(&self) -> Result<(), String> {
        if self.description.len() > MAX_TASK_DESCRIPTION_LEN {
            return Err("Task description exceeds maximum length".to_string());
        }
        if self.description.trim().is_empty() {
            return Err("Task description cannot be empty".to_string());
        }
        Ok(())
    }
}

/// Blueprint for spawning a sub-agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSpec {
    /// Optional display name.
    pub name: String,
}

impl AgentSpec {
    /// Create a new agent specification with validation.
    /// 
    /// # Security
    /// Validates that the name doesn't exceed maximum length to prevent
    /// memory exhaustion attacks.
    pub fn new(name: String) -> Result<Self, String> {
        if name.len() > MAX_AGENT_NAME_LEN {
            return Err(format!(
                "Agent name too long: {} > {}",
                name.len(),
                MAX_AGENT_NAME_LEN
            ));
        }
        if name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        Ok(Self { name })
    }

    /// Validate an existing agent specification.
    pub fn validate(&self) -> Result<(), String> {
        if self.name.len() > MAX_AGENT_NAME_LEN {
            return Err("Agent name exceeds maximum length".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Agent name cannot be empty".to_string());
        }
        Ok(())
    }
}

//─────────────────────────────
//  Kernel opcode enumeration
//─────────────────────────────

/// Canonical list of **agent-centric** operations for kernel v0.2.
#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    /// Enqueue a task in the agent inbox.
    ScheduleAgentTask { agent: EntityId, task: TaskSpec },
    /// Spawn a sub-agent as a child of `parent`.
    SpawnSubAgent     { parent: EntityId, spec: AgentSpec },
    /// Emit opaque observation data.
    EmitObservation   { agent: EntityId, data: Vec<u8> },
}

//─────────────────────────────
//  Kernel message envelope
//─────────────────────────────

/// Authenticated envelope submitted to the kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Sender entity.
    pub origin: EntityId,
    /// Raw capability token string (validated by `toka-auth`).
    pub capability: String,
    /// Requested operation.
    pub op: Operation,
}

impl Message {
    /// Create a new message with validation.
    /// 
    /// # Security
    /// Validates all components to prevent various attack vectors including
    /// memory exhaustion and injection attacks.
    pub fn new(origin: EntityId, capability: String, op: Operation) -> Result<Self, String> {
        // SECURITY: Validate capability token size to prevent memory exhaustion
        if capability.len() > MAX_CAPABILITY_TOKEN_LEN {
            return Err(format!(
                "Capability token too long: {} > {}",
                capability.len(),
                MAX_CAPABILITY_TOKEN_LEN
            ));
        }
        
        if capability.trim().is_empty() {
            return Err("Capability token cannot be empty".to_string());
        }

        // SECURITY: Validate the operation
        op.validate()?;

        Ok(Self { origin, capability, op })
    }

    /// Validate an existing message.
    pub fn validate(&self) -> Result<(), String> {
        if self.capability.len() > MAX_CAPABILITY_TOKEN_LEN {
            return Err("Capability token exceeds maximum length".to_string());
        }
        if self.capability.trim().is_empty() {
            return Err("Capability token cannot be empty".to_string());
        }
        self.op.validate()
    }
}

impl Operation {
    /// Validate the operation to ensure it meets security constraints.
    /// 
    /// # Security
    /// Validates all operation parameters to prevent various attack vectors.
    pub fn validate(&self) -> Result<(), String> {
        match self {
            Operation::ScheduleAgentTask { task, .. } => {
                task.validate()
            }
            Operation::SpawnSubAgent { spec, .. } => {
                spec.validate()
            }
            Operation::EmitObservation { data, .. } => {
                // SECURITY: Prevent memory exhaustion via large observation data
                if data.len() > MAX_OBSERVATION_DATA_LEN {
                    return Err(format!(
                        "Observation data too large: {} > {}",
                        data.len(),
                        MAX_OBSERVATION_DATA_LEN
                    ));
                }
                Ok(())
            }
        }
    }
}

//─────────────────────────────
//  Agent configuration types
//─────────────────────────────

/// Agent configuration loaded from YAML files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Agent metadata
    pub metadata: AgentMetadata,
    /// Agent specification
    pub spec: AgentSpecConfig,
    /// Agent capabilities
    pub capabilities: AgentCapabilities,
    /// Agent objectives and deliverables
    pub objectives: Vec<AgentObjective>,
    /// Default tasks for the agent
    pub tasks: AgentTasks,
    /// Agent dependencies
    pub dependencies: AgentDependencies,
    /// Reporting configuration
    pub reporting: ReportingConfig,
    /// Security configuration
    pub security: SecurityConfig,
}

/// Agent metadata from configuration files.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentMetadata {
    /// Agent name (used as identifier)
    pub name: String,
    /// Configuration version
    pub version: String,
    /// Creation date
    pub created: String,
    /// Associated workstream
    pub workstream: String,
    /// Git branch for this agent's work
    pub branch: String,
}

/// Agent specification configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSpecConfig {
    /// Human-readable agent name
    pub name: String,
    /// Agent domain
    pub domain: String,
    /// Agent priority level
    pub priority: AgentPriority,
}

/// Agent priority levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AgentPriority {
    /// Critical path agents - must complete first
    Critical,
    /// High priority agents - important for progress
    High,
    /// Medium priority agents - standard priority
    Medium,
    /// Low priority agents - can be delayed
    Low,
}

/// Agent capabilities configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentCapabilities {
    /// Primary capabilities
    pub primary: Vec<String>,
    /// Secondary capabilities
    pub secondary: Vec<String>,
}

/// Agent objective definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentObjective {
    /// Objective description
    pub description: String,
    /// Expected deliverable
    pub deliverable: String,
    /// Validation criteria
    pub validation: String,
}

/// Agent tasks configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentTasks {
    /// Default tasks to assign to agent
    pub default: Vec<TaskConfig>,
}

/// Task configuration from agent specs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskConfig {
    /// Task description
    pub description: String,
    /// Task priority
    pub priority: TaskPriority,
}

/// Task priority levels.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    /// High priority task
    High,
    /// Medium priority task
    Medium,
    /// Low priority task
    Low,
}

/// Agent dependencies configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentDependencies {
    /// Required dependencies (must complete before this agent starts)
    pub required: HashMap<String, String>,
    /// Optional dependencies (nice to have, but not blocking)
    pub optional: HashMap<String, String>,
}

/// Reporting configuration for agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Reporting frequency
    pub frequency: ReportingFrequency,
    /// Reporting channels
    pub channels: Vec<String>,
    /// Metrics to track
    pub metrics: HashMap<String, String>,
}

/// Reporting frequency options.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportingFrequency {
    /// Report daily
    Daily,
    /// Report weekly
    Weekly,
    /// Report on milestone completion
    #[serde(rename = "on-milestone")]
    OnMilestone,
}

/// Security configuration for agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether agent runs in sandbox
    pub sandbox: bool,
    /// Required capabilities for agent
    pub capabilities_required: Vec<String>,
    /// Resource limits for agent
    pub resource_limits: ResourceLimits,
}

/// Resource limits for agents.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage (e.g., "100MB")
    pub max_memory: String,
    /// Maximum CPU usage (e.g., "50%")
    pub max_cpu: String,
    /// Timeout for agent operations (e.g., "1h")
    pub timeout: String,
}

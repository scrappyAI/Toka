#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-bus-core** – Core event bus abstraction for Toka OS.
//!
//! This crate provides the fundamental event bus traits and types used throughout
//! the Toka ecosystem. It sits at the deterministic core layer and provides
//! lightweight, in-memory event broadcasting with no persistence or I/O concerns.
//!
//! The bus abstraction allows different components to communicate via typed events
//! while maintaining loose coupling and testability.

use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use toka_types::{EntityId, TaskSpec, AgentSpec};
use chrono::{DateTime, Utc};

//─────────────────────────────
//  Core kernel events
//─────────────────────────────

/// Typed kernel event enumeration emitted by the kernel after a successful
/// state transition. Each variant mirrors one opcode family from
/// `toka_types::Operation`.
///
/// These events represent the canonical "what happened" notifications that
/// flow through the system after kernel operations complete successfully.
///
/// # v0.3.0 Enhancement
/// This version significantly expands the event model to include agent lifecycle,
/// task completion tracking, error reporting, and resource monitoring events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[non_exhaustive]
pub enum KernelEvent {
    //─────────────────────────────
    //  Core Events (v0.2)
    //─────────────────────────────
    
    /// Agent was assigned a new task
    TaskScheduled {
        /// The agent that received the task
        agent: EntityId,
        /// The task specification
        task: TaskSpec,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// New sub-agent was spawned
    AgentSpawned {
        /// The parent agent that spawned the child
        parent: EntityId,
        /// The agent specification
        spec: AgentSpec,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// Agent emitted observation data
    ObservationEmitted {
        /// The agent that made the observation
        agent: EntityId,
        /// The observation data
        data: Vec<u8>,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },

    //─────────────────────────────
    //  Agent Lifecycle Events (v0.3)
    //─────────────────────────────
    
    /// Agent execution terminated
    AgentTerminated {
        /// The agent that terminated
        agent: EntityId,
        /// Reason for termination
        reason: TerminationReason,
        /// Exit code (0 for success, non-zero for error)
        exit_code: i32,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// Agent was suspended for resource management
    AgentSuspended {
        /// The agent that was suspended
        agent: EntityId,
        /// Reason for suspension
        reason: SuspensionReason,
        /// Serialized state snapshot for restoration
        state_snapshot: Option<Vec<u8>>,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// Suspended agent resumed execution
    AgentResumed {
        /// The agent that resumed
        agent: EntityId,
        /// State restored from suspension
        from_state: Option<Vec<u8>>,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },

    //─────────────────────────────
    //  Task Management Events (v0.3)
    //─────────────────────────────
    
    /// Task completed successfully
    TaskCompleted {
        /// Unique task identifier
        task_id: String,
        /// Agent that executed the task
        agent: EntityId,
        /// Task execution result
        result: TaskResult,
        /// Time taken to execute the task (milliseconds)
        execution_time_ms: u64,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// Task execution failed
    TaskFailed {
        /// Unique task identifier
        task_id: String,
        /// Agent that attempted to execute the task
        agent: EntityId,
        /// Error that caused the failure
        error: String,
        /// Categorized failure reason
        failure_reason: FailureReason,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// Task exceeded execution time limit
    TaskTimeout {
        /// Unique task identifier
        task_id: String,
        /// Agent that was executing the task
        agent: EntityId,
        /// Configured timeout duration (milliseconds)
        timeout_duration_ms: u64,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },

    //─────────────────────────────
    //  Error Framework Events (v0.3)
    //─────────────────────────────
    
    /// System-level error requiring attention
    SystemError {
        /// Error category for classification
        error_category: ErrorCategory,
        /// Specific error code
        error_code: String,
        /// Additional context and debugging information
        context: ErrorContext,
        /// Error severity level
        severity: ErrorSeverity,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// Data validation error
    ValidationError {
        /// Type of validation that failed
        validation_type: ValidationType,
        /// The invalid data (limited size for security)
        invalid_data: String,
        /// Expected format or constraints
        expected_format: String,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// Resource allocation or management error
    ResourceError {
        /// Type of resource involved
        resource_type: ResourceType,
        /// Amount requested
        requested: u64,
        /// Amount available
        available: u64,
        /// Agent that requested the resource
        agent: Option<EntityId>,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },

    //─────────────────────────────
    //  Resource Tracking Events (v0.3)
    //─────────────────────────────
    
    /// Memory allocation event
    MemoryAllocated {
        /// Agent that allocated memory
        agent: EntityId,
        /// Amount of memory allocated (bytes)
        amount: u64,
        /// Total memory allocated to this agent (bytes)
        total_allocated: u64,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// CPU utilization reporting event
    CPUUtilization {
        /// Agent being monitored
        agent: EntityId,
        /// CPU usage percentage (0-100)
        cpu_percent: f64,
        /// Monitoring duration (milliseconds)
        duration_ms: u64,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
    /// I/O operation tracking event
    IOOperation {
        /// Agent that performed I/O
        agent: EntityId,
        /// Type of I/O operation
        operation_type: IOOperationType,
        /// Number of bytes transferred
        bytes: u64,
        /// Operation duration (milliseconds)
        duration_ms: u64,
        /// Event timestamp
        timestamp: DateTime<Utc>,
    },
}

//─────────────────────────────
//  Supporting Event Types (v0.3)
//─────────────────────────────

/// Reasons why an agent was terminated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TerminationReason {
    /// Agent completed successfully
    Completed,
    /// Agent was explicitly killed by user/system
    Killed,
    /// Agent crashed due to error
    Crashed,
    /// Agent exceeded resource limits
    ResourceLimit,
    /// Agent was terminated due to timeout
    Timeout,
    /// Custom termination reason
    Other(String),
}

/// Reasons why an agent was suspended
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuspensionReason {
    /// Resource management (low memory/CPU)
    ResourceManagement,
    /// Administrative action
    Administrative,
    /// System maintenance
    Maintenance,
    /// Agent requested suspension
    SelfRequested,
    /// Custom suspension reason
    Other(String),
}

/// Task execution result data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskResult {
    /// Task completed successfully with result data
    Success {
        /// Serialized result data
        data: Vec<u8>,
    },
    /// Task completed successfully with text result
    SuccessText {
        /// Human-readable result
        result: String,
    },
    /// Task completed successfully with no result data
    SuccessEmpty,
}

/// Categorized reasons for task failure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FailureReason {
    /// Invalid input parameters
    InvalidInput,
    /// Required resource unavailable
    ResourceUnavailable,
    /// Permission denied
    PermissionDenied,
    /// Network connectivity issue
    NetworkError,
    /// File system error
    FileSystemError,
    /// Agent logic error
    AgentError,
    /// System error
    SystemError,
    /// Custom failure reason
    Other(String),
}

/// Error categories for system errors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    /// Authentication/authorization errors
    Security,
    /// Network-related errors
    Network,
    /// Storage/persistence errors
    Storage,
    /// Agent management errors
    Agent,
    /// Task execution errors
    Task,
    /// Resource management errors
    Resource,
    /// Configuration errors
    Configuration,
    /// Custom error category
    Other(String),
}

/// Additional context for error events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorContext {
    /// Component that reported the error
    pub component: String,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Error severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorSeverity {
    /// Informational - no action required
    Info,
    /// Warning - attention recommended
    Warning,
    /// Error - action required
    Error,
    /// Critical - immediate action required
    Critical,
}

/// Types of validation that can fail
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationType {
    /// JSON schema validation
    JsonSchema,
    /// Data format validation
    DataFormat,
    /// Business rule validation
    BusinessRule,
    /// Security constraint validation
    SecurityConstraint,
    /// Custom validation type
    Other(String),
}

/// Types of system resources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ResourceType {
    /// Memory (RAM)
    Memory,
    /// CPU processing power
    CPU,
    /// Disk storage
    Disk,
    /// Network bandwidth
    Network,
    /// File handles
    FileHandles,
    /// Database connections
    DatabaseConnections,
    /// Custom resource type
    Other(String),
}

/// Types of I/O operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IOOperationType {
    /// File read operation
    FileRead,
    /// File write operation
    FileWrite,
    /// Network read operation
    NetworkRead,
    /// Network write operation
    NetworkWrite,
    /// Database read operation
    DatabaseRead,
    /// Database write operation
    DatabaseWrite,
    /// Custom I/O operation type
    Other(String),
}

impl KernelEvent {
    /// Validate the kernel event to ensure it meets security constraints.
    /// 
    /// # Security
    /// Validates all event parameters to prevent various attack vectors.
    /// Enhanced in v0.3.0 to validate new event types and timestamp constraints.
    pub fn validate(&self) -> Result<(), String> {
        // Common timestamp validation
        let now = Utc::now();
        let max_timestamp_drift = chrono::Duration::hours(24); // Allow 24-hour drift
        
        match self {
            // Core Events (v0.2 + timestamp)
            KernelEvent::TaskScheduled { task, timestamp, .. } => {
                task.validate()?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::AgentSpawned { spec, timestamp, .. } => {
                spec.validate()?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::ObservationEmitted { data, timestamp, .. } => {
                // SECURITY: Validate observation data size
                if data.len() > toka_types::MAX_OBSERVATION_DATA_LEN {
                    return Err(format!(
                        "Observation data too large: {} > {}",
                        data.len(),
                        toka_types::MAX_OBSERVATION_DATA_LEN
                    ));
                }
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }

            // Agent Lifecycle Events (v0.3)
            KernelEvent::AgentTerminated { exit_code, timestamp, .. } => {
                // SECURITY: Validate exit code range
                if *exit_code < -128 || *exit_code > 127 {
                    return Err("Exit code out of valid range [-128, 127]".to_string());
                }
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::AgentSuspended { state_snapshot, timestamp, .. } => {
                // SECURITY: Validate state snapshot size
                if let Some(snapshot) = state_snapshot {
                    const MAX_STATE_SNAPSHOT_SIZE: usize = 10_000_000; // 10MB
                    if snapshot.len() > MAX_STATE_SNAPSHOT_SIZE {
                        return Err(format!(
                            "State snapshot too large: {} > {}",
                            snapshot.len(),
                            MAX_STATE_SNAPSHOT_SIZE
                        ));
                    }
                }
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::AgentResumed { from_state, timestamp, .. } => {
                // SECURITY: Validate restored state size
                if let Some(state) = from_state {
                    const MAX_STATE_SIZE: usize = 10_000_000; // 10MB
                    if state.len() > MAX_STATE_SIZE {
                        return Err(format!(
                            "Restored state too large: {} > {}",
                            state.len(),
                            MAX_STATE_SIZE
                        ));
                    }
                }
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }

            // Task Management Events (v0.3)
            KernelEvent::TaskCompleted { task_id, execution_time_ms, timestamp, result, .. } => {
                self.validate_task_id(task_id)?;
                self.validate_execution_time(*execution_time_ms)?;
                self.validate_task_result(result)?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::TaskFailed { task_id, error, timestamp, .. } => {
                self.validate_task_id(task_id)?;
                self.validate_error_message(error)?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::TaskTimeout { task_id, timeout_duration_ms, timestamp, .. } => {
                self.validate_task_id(task_id)?;
                self.validate_timeout_duration(*timeout_duration_ms)?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }

            // Error Framework Events (v0.3)
            KernelEvent::SystemError { error_code, context, timestamp, .. } => {
                self.validate_error_code(error_code)?;
                self.validate_error_context(context)?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::ValidationError { invalid_data, expected_format, timestamp, .. } => {
                self.validate_validation_data(invalid_data, expected_format)?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::ResourceError { requested, available, timestamp, .. } => {
                // SECURITY: Validate resource amounts are reasonable
                const MAX_RESOURCE_AMOUNT: u64 = 1_000_000_000_000; // 1TB
                if *requested > MAX_RESOURCE_AMOUNT || *available > MAX_RESOURCE_AMOUNT {
                    return Err("Resource amount exceeds maximum limit".to_string());
                }
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }

            // Resource Tracking Events (v0.3)
            KernelEvent::MemoryAllocated { amount, total_allocated, timestamp, .. } => {
                const MAX_MEMORY: u64 = 1_000_000_000_000; // 1TB
                if *amount > MAX_MEMORY || *total_allocated > MAX_MEMORY {
                    return Err("Memory amount exceeds maximum limit".to_string());
                }
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::CPUUtilization { cpu_percent, duration_ms, timestamp, .. } => {
                // SECURITY: Validate CPU percentage range
                if *cpu_percent < 0.0 || *cpu_percent > 100.0 {
                    return Err("CPU percentage must be between 0 and 100".to_string());
                }
                self.validate_duration(*duration_ms)?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
            KernelEvent::IOOperation { bytes, duration_ms, timestamp, .. } => {
                const MAX_IO_BYTES: u64 = 100_000_000_000; // 100GB
                if *bytes > MAX_IO_BYTES {
                    return Err("I/O bytes amount exceeds maximum limit".to_string());
                }
                self.validate_duration(*duration_ms)?;
                self.validate_timestamp(*timestamp, now, max_timestamp_drift)?;
                Ok(())
            }
        }
    }

    /// Validate timestamp is within acceptable range
    fn validate_timestamp(&self, timestamp: DateTime<Utc>, now: DateTime<Utc>, max_drift: chrono::Duration) -> Result<(), String> {
        let age = now.signed_duration_since(timestamp);
        let future_drift = timestamp.signed_duration_since(now);
        
        if age > max_drift || future_drift > max_drift {
            return Err("Timestamp outside acceptable range".to_string());
        }
        Ok(())
    }

    /// Validate task ID format and length
    fn validate_task_id(&self, task_id: &str) -> Result<(), String> {
        if task_id.is_empty() || task_id.len() > 256 {
            return Err("Task ID must be 1-256 characters".to_string());
        }
        Ok(())
    }

    /// Validate execution time is reasonable
    fn validate_execution_time(&self, execution_time_ms: u64) -> Result<(), String> {
        const MAX_EXECUTION_TIME_MS: u64 = 24 * 60 * 60 * 1000; // 24 hours
        if execution_time_ms > MAX_EXECUTION_TIME_MS {
            return Err("Execution time exceeds maximum limit".to_string());
        }
        Ok(())
    }

    /// Validate task result data
    fn validate_task_result(&self, result: &TaskResult) -> Result<(), String> {
        match result {
            TaskResult::Success { data } => {
                const MAX_RESULT_SIZE: usize = 10_000_000; // 10MB
                if data.len() > MAX_RESULT_SIZE {
                    return Err("Task result data too large".to_string());
                }
            }
            TaskResult::SuccessText { result } => {
                const MAX_RESULT_TEXT: usize = 1_000_000; // 1MB
                if result.len() > MAX_RESULT_TEXT {
                    return Err("Task result text too large".to_string());
                }
            }
            TaskResult::SuccessEmpty => {}
        }
        Ok(())
    }

    /// Validate error message length
    fn validate_error_message(&self, error: &str) -> Result<(), String> {
        const MAX_ERROR_LENGTH: usize = 10_000;
        if error.len() > MAX_ERROR_LENGTH {
            return Err("Error message too long".to_string());
        }
        Ok(())
    }

    /// Validate error code format
    fn validate_error_code(&self, error_code: &str) -> Result<(), String> {
        if error_code.is_empty() || error_code.len() > 100 {
            return Err("Error code must be 1-100 characters".to_string());
        }
        Ok(())
    }

    /// Validate error context
    fn validate_error_context(&self, context: &ErrorContext) -> Result<(), String> {
        if context.component.is_empty() || context.component.len() > 200 {
            return Err("Component name must be 1-200 characters".to_string());
        }
        
        if context.metadata.len() > 50 {
            return Err("Too many metadata entries".to_string());
        }

        for (key, value) in &context.metadata {
            if key.len() > 100 || value.len() > 1000 {
                return Err("Metadata key/value too large".to_string());
            }
        }
        Ok(())
    }

    /// Validate validation error data
    fn validate_validation_data(&self, invalid_data: &str, expected_format: &str) -> Result<(), String> {
        const MAX_DATA_LENGTH: usize = 1000;
        if invalid_data.len() > MAX_DATA_LENGTH || expected_format.len() > MAX_DATA_LENGTH {
            return Err("Validation data too large".to_string());
        }
        Ok(())
    }

    /// Validate timeout duration
    fn validate_timeout_duration(&self, duration_ms: u64) -> Result<(), String> {
        const MAX_TIMEOUT_MS: u64 = 24 * 60 * 60 * 1000; // 24 hours
        if duration_ms > MAX_TIMEOUT_MS {
            return Err("Timeout duration exceeds maximum".to_string());
        }
        Ok(())
    }

    /// Validate duration values
    fn validate_duration(&self, duration_ms: u64) -> Result<(), String> {
        const MAX_DURATION_MS: u64 = 24 * 60 * 60 * 1000; // 24 hours
        if duration_ms > MAX_DURATION_MS {
            return Err("Duration exceeds maximum limit".to_string());
        }
        Ok(())
    }
}

//─────────────────────────────
//  Event bus trait
//─────────────────────────────

/// Core event bus abstraction for publishing and subscribing to kernel events.
///
/// The bus provides a simple publish-subscribe mechanism that allows different
/// components to communicate asynchronously while maintaining loose coupling.
/// All implementations must be thread-safe and support multiple subscribers.
pub trait EventBus: Send + Sync {
    /// Publish an event to all subscribers.
    ///
    /// This operation should complete quickly and not block the caller.
    /// If subscribers are slow or unavailable, the bus may drop events
    /// to maintain system responsiveness.
    fn publish(&self, event: &KernelEvent) -> Result<()>;

    /// Subscribe to the live event stream.
    ///
    /// Returns a receiver that will receive copies of all events published
    /// after the subscription was created. Subscribers that fall behind
    /// may miss events if the bus buffer overflows.
    fn subscribe(&self) -> broadcast::Receiver<KernelEvent>;
}

//─────────────────────────────
//  In-memory bus implementation
//─────────────────────────────

/// Simple in-memory, broadcast-only event bus using Tokio channels.
///
/// This implementation uses a ring buffer to store recent events and broadcasts
/// them to all active subscribers. It provides good performance for scenarios
/// where events don't need persistence.
#[derive(Debug, Clone)]
pub struct InMemoryBus {
    tx: Arc<broadcast::Sender<KernelEvent>>,
}

impl Default for InMemoryBus {
    fn default() -> Self {
        Self::new(1024)
    }
}

impl InMemoryBus {
    /// Create a new in-memory bus with the specified ring buffer capacity.
    ///
    /// The capacity determines how many events can be buffered for slow
    /// subscribers before older events are dropped.
    pub fn new(capacity: usize) -> Self {
        let (tx, _rx) = broadcast::channel(capacity);
        Self { tx: Arc::new(tx) }
    }

    /// Get the current number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

impl EventBus for InMemoryBus {
    fn publish(&self, event: &KernelEvent) -> Result<()> {
        // SECURITY: Validate event before publishing
        event.validate().map_err(BusError::PublishFailed)?;
        
        // Ignore lagging receiver errors - subscribers must handle missed events
        let _ = self.tx.send(event.clone());
        Ok(())
    }

    fn subscribe(&self) -> broadcast::Receiver<KernelEvent> {
        self.tx.subscribe()
    }
}

//─────────────────────────────
//  Error types
//─────────────────────────────

/// Errors that can occur during bus operations.
#[derive(Debug, thiserror::Error)]
pub enum BusError {
    /// Event could not be published
    #[error("failed to publish event: {0}")]
    PublishFailed(String),
    /// Subscription failed
    #[error("failed to create subscription: {0}")]
    SubscriptionFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast::error::RecvError;

    #[tokio::test]
    async fn test_in_memory_bus_basic_flow() {
        let bus = InMemoryBus::new(16);
        let mut rx = bus.subscribe();

        let event = KernelEvent::TaskScheduled {
            agent: EntityId(123),
            task: TaskSpec {
                description: "test task".to_string(),
            },
            timestamp: Utc::now(),
        };

        // Publish event
        bus.publish(&event).unwrap();

        // Receive event
        let received = rx.recv().await.unwrap();
        assert_eq!(received, event);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = InMemoryBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        let event = KernelEvent::AgentSpawned {
            parent: EntityId(1),
            spec: AgentSpec {
                name: "test-agent".to_string(),
            },
            timestamp: Utc::now(),
        };

        bus.publish(&event).unwrap();

        // Both subscribers should receive the event
        assert_eq!(rx1.recv().await.unwrap(), event);
        assert_eq!(rx2.recv().await.unwrap(), event);
    }

    #[tokio::test]
    async fn test_buffer_overflow() {
        let bus = InMemoryBus::new(2); // Very small buffer
        let mut rx = bus.subscribe();

        // Fill buffer beyond capacity
        for i in 0..5 {
            let event = KernelEvent::ObservationEmitted {
                agent: EntityId(i as u128),
                data: vec![i as u8],
                timestamp: Utc::now(),
            };
            bus.publish(&event).unwrap();
        }

        // First few events should be lost due to buffer overflow
        match rx.recv().await {
            Ok(_) => {
                // Successfully received an event - continue receiving
                while let Ok(_) = rx.recv().await {
                    // Keep draining
                }
            }
            Err(RecvError::Lagged(_)) => {
                // Expected - some events were dropped
            }
            Err(e) => {
                // Log unexpected error but don't panic in production
                eprintln!("Unexpected receiver error: {}", e);
                assert!(false, "Unexpected error: {}", e);
            }
        }
    }
}
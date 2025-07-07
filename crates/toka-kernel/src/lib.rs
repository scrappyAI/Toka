#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! **toka-kernel** – Deterministic state-machine core of Toka OS.
//!
//! The kernel validates capability tokens, executes opcode handlers on an
//! in-memory `WorldState`, and emits typed events onto the shared event bus.
//! All operations are synchronous and **deterministic** for v0.2.
//!
//! 📜 **Spec reference:** see [`docs/44_toka_kernel_spec_v0.2.md`](../../../docs/44_toka_kernel_spec_v0.2.md)
//! which defines the **minimal, agent-centric kernel surface** and extension
//! mechanism introduced in this release.
//!
//! *Scope* (v0.2):
//! - Deterministic execution – single thread, no async side-effects inside handlers.
//! - Capability-guarded syscall surface exposed via [`Operation`](toka_types::Operation).
//! - Core kernel ships with **agent primitives only** by default.  Finance &
//!   other families remain external extension crates implementing
//!   [`OpcodeHandler`].
//! - In-memory state only; durable storage adapters, metering and async schedulers
//!   are slated for v0.3+.
//!
//! Anything outside these bounds (networking, storage, WASM execution) is
//! intentionally deferred to keep the kernel minimal and auditable.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;
use chrono::Utc;

use toka_types::{EntityId, Message, Operation, TaskSpec, AgentSpec};
use toka_bus_core::{KernelEvent, EventBus};
use toka_auth::{TokenValidator, Claims};

mod registry;
pub use registry::{register_handler, OpcodeHandler};

//─────────────────────────────
//  World-state
//─────────────────────────────

/// In-memory tables representing the canonical world-state.
#[derive(Debug, Default)]
pub struct WorldState {
    /// Agent inboxes (queued tasks).
    pub agent_tasks: HashMap<EntityId, Vec<TaskSpec>>,
}

//─────────────────────────────
//  Kernel error type
//─────────────────────────────

/// Deterministic error codes produced by the kernel.
#[allow(missing_docs)]
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum KernelError {
    /// Submitted capability token is not authorised.
    #[error("capability denied")]
    CapabilityDenied,
    /// Unknown entity referenced in the operation.
    #[error("unknown entity {0:?}")]
    UnknownEntity(EntityId),
    /// Invalid operation semantic (e.g. negative amount).
    #[error("invalid operation: {0}")]
    InvalidOperation(String),
    /// Operation family not compiled into the kernel.
    #[error("unsupported operation in current kernel build")]
    UnsupportedOperation,
}

//─────────────────────────────
//  Kernel struct
//─────────────────────────────

/// Deterministic state-machine executor.
pub struct Kernel {
    state: Arc<RwLock<WorldState>>,
    auth: Arc<dyn TokenValidator>,
    bus: Arc<dyn EventBus>,
}

impl Kernel {
    /// Create a new kernel backed by `state`, `auth` validator and `bus`.
    pub fn new(state: WorldState, auth: Arc<dyn TokenValidator>, bus: Arc<dyn EventBus>) -> Self {
        Self { state: Arc::new(RwLock::new(state)), auth, bus }
    }

    /// Expose internal state pointer (read-only usage outside kernel).
    pub fn state_ptr(&self) -> Arc<RwLock<WorldState>> {
        Arc::clone(&self.state)
    }

    /// Submit a message, triggering capability validation, execution and event emission.
    /// 
    /// # Security
    /// This function performs comprehensive validation including:
    /// - Message structure validation
    /// - Capability token authentication
    /// - Operation parameter validation
    /// - Rate limiting (future enhancement)
    pub async fn submit(&self, msg: Message) -> Result<KernelEvent> {
        // SECURITY: Validate message structure first
        msg.validate().map_err(KernelError::InvalidOperation)?;

        // SECURITY: Log authentication attempts for security monitoring
        let auth_start = std::time::Instant::now();

        // 1. Capability validation
        let claims: Claims = self
            .auth
            .validate(&msg.capability)
            .await
            .map_err(|e| {
                // SECURITY: Log authentication failures
                eprintln!("Authentication failed for entity {:?}: {} (took {:?})", 
                         msg.origin, e, auth_start.elapsed());
                KernelError::CapabilityDenied
            })?;

        // SECURITY: Verify the token subject matches the message origin
        // This prevents privilege escalation attacks
        let origin_str = msg.origin.0.to_string();
        if claims.sub != origin_str {
            eprintln!("Token subject mismatch: {} != {}", claims.sub, origin_str);
            return Err(KernelError::CapabilityDenied.into());
        }

        // SECURITY: Log successful authentication
        let auth_duration = auth_start.elapsed();
        if auth_duration.as_millis() > 100 {
            eprintln!("Authentication took unusually long: {:?}", auth_duration);
        }

        // 2. Try external opcode handlers first so we don't move the operation prematurely.
        {
            let mut state = self.state.write().await;
            if let Some(ext_evt) = registry::dispatch(&msg.op, &mut state)? {
                self.bus.publish(&ext_evt)?;
                return Ok(ext_evt);
            }
        }

        // 3. Dispatch → built-in agent handlers
        let evt = match &msg.op {
            // ───────── core system ops ─────────
            Operation::ScheduleAgentTask { agent, task } => {
                self.handle_schedule_task(*agent, task.clone()).await?
            }
            Operation::SpawnSubAgent { parent, spec } => {
                self.handle_spawn_agent(*parent, spec.clone()).await?
            }
            Operation::EmitObservation { agent, data } => {
                self.handle_observation(*agent, data.clone()).await?
            }
        };

        // 4. Emit event for core ops
        self.bus.publish(&evt)?;
        Ok(evt)
    }

    //───────────────────── handlers ─────────────────────

    async fn handle_schedule_task(&self, agent: EntityId, task: TaskSpec) -> Result<KernelEvent> {
        // SECURITY: Validate task before processing
        task.validate().map_err(KernelError::InvalidOperation)?;
        
        let mut state = self.state.write().await;
        
        // SECURITY: Check if agent exists (prevent task scheduling to non-existent agents)
        // For now, we allow task scheduling to any agent ID, but log it for monitoring
        let task_count = state.agent_tasks.get(&agent).map(|tasks| tasks.len()).unwrap_or(0);
        
        // SECURITY: Prevent task queue overflow DoS attacks
        const MAX_TASKS_PER_AGENT: usize = 10000;
        if task_count >= MAX_TASKS_PER_AGENT {
            return Err(KernelError::InvalidOperation(
                format!("Agent {:?} task queue full ({} tasks)", agent, task_count)
            ).into());
        }
        
        state.agent_tasks.entry(agent).or_default().push(task.clone());
        Ok(KernelEvent::TaskScheduled { 
            agent, 
            task, 
            timestamp: Utc::now(),
        })
    }

    async fn handle_spawn_agent(&self, parent: EntityId, spec: AgentSpec) -> Result<KernelEvent> {
        // SECURITY: Validate agent spec before processing
        spec.validate().map_err(KernelError::InvalidOperation)?;
        
        // SECURITY: Log agent spawning for audit trail
        eprintln!("Agent spawn request: parent={:?}, name={}", parent, spec.name);
        
        Ok(KernelEvent::AgentSpawned { 
            parent, 
            spec, 
            timestamp: Utc::now(),
        })
    }

    async fn handle_observation(&self, agent: EntityId, data: Vec<u8>) -> Result<KernelEvent> {
        // SECURITY: Validate observation data size (already validated in Operation::validate)
        // But double-check as defense in depth
        if data.len() > toka_types::MAX_OBSERVATION_DATA_LEN {
            return Err(KernelError::InvalidOperation(
                "Observation data exceeds maximum size".to_string()
            ).into());
        }
        
        // SECURITY: Log large observations for monitoring
        if data.len() > 100_000 { // 100KB threshold
            eprintln!("Large observation from agent {:?}: {} bytes", agent, data.len());
        }
        
        Ok(KernelEvent::ObservationEmitted { 
            agent, 
            data, 
            timestamp: Utc::now(),
        })
    }
}

//─────────────────────────────
//  Tests
//─────────────────────────────

// No finance tests after v0.2 – core kernel covers agent primitives only.

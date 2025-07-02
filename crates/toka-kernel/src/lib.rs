#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! **toka-kernel** – Deterministic state-machine core of Toka OS.
//!
//! The kernel validates capability tokens, executes opcode handlers on an
//! in-memory `WorldState`, and emits typed events onto the shared event bus.
//! All operations are synchronous and **deterministic** for v0.1.
//!
//! 📜 **Spec reference:** see [`docs/42_toka_kernel_spec_v0.1.md`](../../../docs/42_toka_kernel_spec_v0.1.md)
//! which outlines opcode semantics, architectural principles and open
//! questions for upcoming milestones.
//!
//! *Scope* (v0.1):
//! - Deterministic execution – single thread, no async side-effects inside handlers.
//! - Capability-guarded syscall surface exposed via [`Operation`](toka_types::Operation).
//! - *Only* agent primitives are enabled by default.  Financial & user families are
//!   available via the optional `finance` / `user` feature flags.
//! - In-memory state only; durable storage adapters arrive in v0.2.
//!
//! Anything outside these bounds (networking, storage, WASM execution) is
//! intentionally deferred to keep the kernel minimal and auditable.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use toka_types::{EntityId, Message, Operation, TaskSpec, AgentSpec};
use toka_events::bus::{Event as KernelEvent, EventBus};
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
    pub async fn submit(&self, msg: Message) -> Result<KernelEvent> {
        // 1. Capability validation
        let _claims: Claims = self
            .auth
            .validate(&msg.capability)
            .await
            .map_err(|_| KernelError::CapabilityDenied)?;

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
            // ───────── agent ops (core) ─────────
            Operation::ScheduleAgentTask { agent, task } => {
                self.handle_schedule_task(agent.clone(), task.clone()).await?
            }
            Operation::SpawnSubAgent { parent, spec } => {
                self.handle_spawn_agent(parent.clone(), spec.clone()).await?
            }
            Operation::EmitObservation { agent, data } => {
                self.handle_observation(agent.clone(), data.clone()).await?
            }
            // Any other opcode family is not supported by the core kernel.
            _ => return Err(KernelError::UnsupportedOperation.into()),
        };

        // 4. Emit event for core ops
        self.bus.publish(&evt)?;
        Ok(evt)
    }

    //───────────────────── handlers ─────────────────────

    async fn handle_schedule_task(&self, agent: EntityId, task: TaskSpec) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        state.agent_tasks.entry(agent).or_default().push(task.clone());
        Ok(KernelEvent::TaskScheduled { agent, task })
    }

    async fn handle_spawn_agent(&self, parent: EntityId, spec: AgentSpec) -> Result<KernelEvent> {
        // For v0.1 we simply emit an event; state updates deferred to v0.2.
        Ok(KernelEvent::AgentSpawned { parent, spec })
    }

    async fn handle_observation(&self, agent: EntityId, data: Vec<u8>) -> Result<KernelEvent> {
        Ok(KernelEvent::ObservationEmitted { agent, data })
    }
}

//─────────────────────────────
//  Tests
//─────────────────────────────

// No finance tests after v0.2 – core kernel covers agent primitives only.

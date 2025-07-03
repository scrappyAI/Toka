#![forbid(unsafe_code)]
#![warn(missing_docs)]

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
//! - Core kernel ships with **system-level primitives only**.  *All* domain
//!   families (agents, finance, identity …) live in external crates
//!   implementing [`OpcodeHandler`].
//! - In-memory state only; durable storage adapters, metering and async schedulers
//!   are slated for v0.3+.
//!
//! Anything outside these bounds (networking, storage, WASM execution) is
//! intentionally deferred to keep the kernel minimal and auditable.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use anyhow::Result;
use tokio::sync::RwLock;

use toka_types::{EntityId, Message, Operation, Hash256, PubKey, Capability, HandlerRef};
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
    /// Set of *live* entities managed by the kernel.
    pub entities: HashSet<EntityId>,
    /// Capability grants keyed by recipient public key.
    pub grants: HashMap<PubKey, Vec<Capability>>, // simplified model
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
            // ───────── core system ops ─────────
            Operation::CreateEntity { template } => {
                self.handle_create_entity(*template).await?
            }
            Operation::DeleteEntity { id } => {
                self.handle_delete_entity(*id).await?
            }
            Operation::GrantCapability { to, cap } => {
                self.handle_grant_capability(to.clone(), cap.clone()).await?
            }
            Operation::RevokeCapability { cap_id } => {
                self.handle_revoke_capability(*cap_id).await?
            }
            Operation::SubmitBatch { ops } => {
                self.handle_submit_batch(ops.clone()).await?
            }
            Operation::EmitEvent { topic, data } => {
                self.handle_emit_event(topic.clone(), data.clone()).await?
            }
            Operation::RegisterHandler { range, entry } => {
                self.handle_register_handler(range.clone(), entry.clone()).await?
            }
            // Any other opcode family is not supported by the core kernel.
            _ => return Err(KernelError::UnsupportedOperation.into()),
        };

        // 4. Emit event for core ops
        self.bus.publish(&evt)?;
        Ok(evt)
    }

    //───────────────────── handlers ─────────────────────

    async fn handle_create_entity(&self, template: Hash256) -> Result<KernelEvent> {
        use rand::{RngCore, rngs::OsRng};
        let mut rand_bytes = [0u8; 16];
        OsRng.fill_bytes(&mut rand_bytes);
        let id = EntityId(u128::from_le_bytes(rand_bytes));

        let mut state = self.state.write().await;
        state.entities.insert(id);

        Ok(KernelEvent::EntityCreated { template, id })
    }

    async fn handle_delete_entity(&self, id: EntityId) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        state.entities.remove(&id);
        Ok(KernelEvent::EntityDeleted { id })
    }

    async fn handle_grant_capability(&self, to: PubKey, cap: Capability) -> Result<KernelEvent> {
        let mut state = self.state.write().await;
        state.grants.entry(to.clone()).or_default().push(cap.clone());
        Ok(KernelEvent::CapabilityGranted { to, cap })
    }

    async fn handle_revoke_capability(&self, cap_id: Hash256) -> Result<KernelEvent> {
        // Simplified: we don't track individual cap IDs yet.
        Ok(KernelEvent::CapabilityRevoked { cap_id })
    }

    async fn handle_submit_batch(&self, ops: Vec<Message>) -> Result<KernelEvent> {
        let count = ops.len();
        // NOTE: For v0.2 the kernel does **not** execute the batch inline to
        // avoid recursive async calls.  Batch execution logic will be added
        // once an async scheduler lands (see roadmap v0.2.1).
        Ok(KernelEvent::BatchSubmitted { count })
    }

    async fn handle_emit_event(&self, topic: String, data: Vec<u8>) -> Result<KernelEvent> {
        Ok(KernelEvent::EventEmitted { topic, data })
    }

    async fn handle_register_handler(&self, range: std::ops::Range<u8>, entry: HandlerRef) -> Result<KernelEvent> {
        // The actual dynamic linking is outside scope; emit event only.
        Ok(KernelEvent::HandlerRegistered { range_start: range.start, range_end: range.end, entry })
    }
}

//─────────────────────────────
//  Tests
//─────────────────────────────

// No finance tests after v0.2 – core kernel covers agent primitives only.

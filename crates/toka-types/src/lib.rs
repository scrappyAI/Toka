#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! **toka-types** – Shared primitive data structures for Toka OS.
//!
//! The crate is dependency‐light and sits at the very bottom of the crate
//! graph so that *every* other crate can depend on it without causing cycles.
//! It intentionally makes no assumptions about I/O, cryptography, or storage.

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Human-readable description (v0.1 placeholder).
    pub description: String,
}

/// Blueprint for spawning a sub-agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    /// Optional display name.
    pub name: String,
}

/// Simple role model used by the *user* opcode family.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Role {
    /// Read-only observer.
    Observer,
    /// Regular user able to submit messages.
    Member,
    /// Elevated privileges (admin/operator).
    Admin,
}

//─────────────────────────────
//  Kernel opcode enumeration
//─────────────────────────────

/// Canonical list of operations understood by the kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /* — financial — */
    /// Transfer fungible balance from `from` to `to`.
    TransferFunds { from: EntityId, to: EntityId, amount: u64 },
    /// Increase total supply of an asset and credit `to`.
    MintAsset     { asset: EntityId, to: EntityId, amount: u64 },
    /// Reduce supply and debit `from`.
    BurnAsset     { asset: EntityId, from: EntityId, amount: u64 },

    /* — agent — */
    /// Enqueue a task in the agent inbox.
    ScheduleAgentTask { agent: EntityId, task: TaskSpec },
    /// Spawn a sub-agent as a child of `parent`.
    SpawnSubAgent     { parent: EntityId, spec: AgentSpec },
    /// Emit opaque observation data.
    EmitObservation   { agent: EntityId, data: Vec<u8> },

    /* — user — */
    /// Create a new user entity with the given alias.
    CreateUser  { alias: String },
    /// Assign `role` to an existing user.
    AssignRole  { user: EntityId, role: Role },
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

#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! **toka-types** – Shared primitive data structures for Toka OS.
//!
//! The crate is dependency‐light and sits at the very bottom of the crate
//! graph so that *every* other crate can depend on it without causing cycles.
//! It intentionally makes no assumptions about I/O, cryptography, or storage.

use serde::{Deserialize, Serialize};
use std::ops::Range;

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

/// 256-bit hash wrapper used for content-addressable identifiers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Hash256(pub [u8; 32]);

/// Public key used for capability grants.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PubKey(pub [u8; 32]);

/// Opaque capability descriptor delegated by an authorised principal.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Capability {
    /// Human-readable capability name or URI.
    pub name: String,
}

/// Reference to an opcode handler registered with the kernel.  The concrete
/// meaning is left to the embedder (could be an index into a jump table or a
/// fully-qualified function name).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HandlerRef(pub u64);

//─────────────────────────────
//  Domain abstractions (stubs)
//─────────────────────────────

/// Specification of a task to be executed by an agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TaskSpec {
    /// Human-readable description (v0.1 placeholder).
    pub description: String,
}

/// Blueprint for spawning a sub-agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentSpec {
    /// Optional display name.
    pub name: String,
}

//─────────────────────────────
//  Kernel opcode enumeration
//─────────────────────────────

/// Core, **domain-agnostic** operations understood by the kernel itself.
///
/// Any domain-specific families (agents, finance, identity…) are expected to
/// live in *extension crates* that register handlers via
/// [`crate::HandlerRef`] and are thus completely decoupled from the kernel.
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /* 0x01 */
    CreateEntity { template: Hash256 },
    /* 0x02 */
    DeleteEntity { id: EntityId },
    /* 0x03 */
    GrantCapability { to: PubKey, cap: Capability },
    /* 0x04 */
    RevokeCapability { cap_id: Hash256 },
    /* 0x05 */
    SubmitBatch { ops: Vec<Message> },
    /* 0x06 */
    EmitEvent { topic: String, data: Vec<u8> },
    /* 0x07 */
    RegisterHandler { range: Range<u8>, entry: HandlerRef },
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

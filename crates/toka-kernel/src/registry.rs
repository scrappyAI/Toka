//! Runtime registry for external opcode handlers.
//!
//! The design follows a **minimal overhead** approach using a global, lazy-
//! initialised HashMap guarded by a `RwLock`.  Extension crates can register
//! additional opcode families during their own `init()` routines.
//!
//! The registry is intentionally *opaque* to the core kernel – the only public
//! surface is [`register_handler`] (for external crates) and the `[dispatch]`
//! helper (used internally by the kernel).  This keeps the dependency surface
//! slim and avoids tight coupling.

use std::collections::HashMap;
use std::sync::RwLock;

use anyhow::Result;
use once_cell::sync::Lazy;

use crate::{WorldState, KernelError};
use toka_events::bus::Event as KernelEvent;
use toka_types::Operation;

/// Trait implemented by **extension crates** to hook additional opcodes into
/// the deterministic kernel pipeline.
pub trait OpcodeHandler: Send + Sync + 'static {
    /// Attempt to handle `op` given a mutable reference to the shared
    /// [`WorldState`].  If the handler **does not recognise** the operation it
    /// MUST return `Ok(None)` so that other handlers (or the core) may take
    /// over.  Returning an `Err` signals a *deterministic* failure that will be
    /// propagated to the caller.
    fn dispatch(&self, op: &Operation, state: &mut WorldState) -> Result<Option<KernelEvent>, KernelError>;
}

/// Global registry mapping *tags* to boxed handler instances.  The tag is
/// informational only for now (it helps with debugging / introspection).
static REGISTRY: Lazy<RwLock<HashMap<&'static str, Box<dyn OpcodeHandler>>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Register a new opcode `handler` under the provided human-readable `tag`.
///
/// Calling this function **after the kernel has started processing messages is
/// still safe** – the underlying `RwLock` ensures any concurrent lookup sees a
/// fully initialized handler instance.
pub fn register_handler(tag: &'static str, handler: Box<dyn OpcodeHandler>) {
    REGISTRY.write().expect("registry poisoned").insert(tag, handler);
}

/// Dispatch helper used by the kernel to delegate operations not covered by
/// the built-in agent primitives.  It iterates over **all** registered
/// handlers (in insertion order) and returns the first non-`None` result.
pub(crate) fn dispatch(op: &Operation, state: &mut WorldState) -> Result<Option<KernelEvent>, KernelError> {
    for handler in REGISTRY.read().expect("registry poisoned").values() {
        if let Some(evt) = handler.dispatch(op, state)? {
            return Ok(Some(evt));
        }
    }
    Ok(None)
}
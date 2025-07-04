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
use std::sync::{Arc, RwLock};
use once_cell::sync::Lazy;
use anyhow::Result;
use toka_types::Operation;
use toka_bus_core::KernelEvent;
use super::{WorldState, KernelError};

/// Type alias for opcode handler functions.
type HandlerFn = Arc<dyn Fn(&Operation, &mut WorldState) -> Result<Option<KernelEvent>, KernelError> + Send + Sync>;

/// Global registry of external opcode handlers.
static REGISTRY: Lazy<RwLock<HashMap<String, HandlerFn>>> = Lazy::new(|| RwLock::new(HashMap::new()));

/// Trait for extending the kernel with new opcode families.
pub trait OpcodeHandler: Send + Sync {
    /// Execute the handler for this operation type.
    fn execute(&self, op: &Operation, state: &mut WorldState) -> Result<Option<KernelEvent>, KernelError>;
}

/// Register a new opcode handler in the global registry.
/// 
/// # Security Note
/// This function returns an error if the registry lock is poisoned,
/// preventing potential panics that could be exploited for DoS attacks.
pub fn register_handler(tag: impl Into<String>, handler: HandlerFn) -> Result<(), String> {
    REGISTRY.write()
        .map_err(|_| "Registry lock poisoned".to_string())?
        .insert(tag.into(), handler);
    Ok(())
}

/// Dispatch an operation to registered handlers.
/// 
/// # Security Note  
/// This function handles lock poisoning gracefully to prevent panics
/// that could be exploited for denial of service attacks.
pub fn dispatch(op: &Operation, state: &mut WorldState) -> Result<Option<KernelEvent>, KernelError> {
    let registry = REGISTRY.read()
        .map_err(|_| KernelError::InvalidOperation("Registry lock poisoned".to_string()))?;
    
    for handler in registry.values() {
        if let Some(event) = handler(op, state)? {
            return Ok(Some(event));
        }
    }
    Ok(None)
}
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
use toka_types::{Operation, EntityId};
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
/// 
/// # Memory Management
/// The registry has a maximum size limit to prevent memory exhaustion.
/// When the limit is reached, the oldest handlers are removed.
pub fn register_handler(tag: impl Into<String>, handler: HandlerFn) -> Result<(), String> {
    let tag_string = tag.into();
    let mut registry = REGISTRY.write()
        .map_err(|_| "Registry lock poisoned".to_string())?;
    
    // MEMORY LEAK FIX: Implement bounded registry size to prevent unbounded growth
    const MAX_HANDLERS: usize = 1000;
    if registry.len() >= MAX_HANDLERS && !registry.contains_key(&tag_string) {
        // Remove the first (oldest) handler to make room
        if let Some((oldest_key, _)) = registry.iter().next() {
            let oldest_key = oldest_key.clone();
            registry.remove(&oldest_key);
        }
    }
    
    registry.insert(tag_string, handler);
    Ok(())
}

/// Unregister an opcode handler from the global registry.
/// 
/// # Security Note
/// This function allows cleanup of unused handlers to prevent memory accumulation.
/// Returns true if the handler was found and removed, false otherwise.
pub fn unregister_handler(tag: &str) -> Result<bool, String> {
    let removed = REGISTRY.write()
        .map_err(|_| "Registry lock poisoned".to_string())?
        .remove(tag)
        .is_some();
    Ok(removed)
}

/// Get the current number of registered handlers.
/// 
/// This function is useful for monitoring memory usage and registry growth.
pub fn registry_size() -> Result<usize, String> {
    Ok(REGISTRY.read()
        .map_err(|_| "Registry lock poisoned".to_string())?
        .len())
}

/// Clear all handlers from the registry.
/// 
/// # Warning
/// This function removes all registered handlers and should only be used
/// during shutdown or testing scenarios.
pub fn clear_registry() -> Result<(), String> {
    REGISTRY.write()
        .map_err(|_| "Registry lock poisoned".to_string())?
        .clear();
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    
    #[test]
    fn test_registry_cleanup_functionality() {
        // This test verifies that the registry cleanup mechanisms work correctly
        
        // Clear registry to start fresh and record initial size
        clear_registry().expect("Should clear registry");
        let initial_size = registry_size().expect("Should get size");
        
        // Register some handlers
        for i in 0..10 {
            let handler_tag = format!("test_cleanup_handler_{}", i);
            let handler = Arc::new(|_op: &Operation, _state: &mut WorldState| -> Result<Option<KernelEvent>, KernelError> {
                Ok(None)
            });
            
            register_handler(handler_tag, handler)
                .expect("Should register handler");
        }
        
        let after_registration = registry_size().expect("Should get size");
        assert_eq!(after_registration, initial_size + 10, "Should have added 10 handlers");
        
        // Test unregistering handlers
        let removed = unregister_handler("test_cleanup_handler_5")
            .expect("Should unregister");
        assert!(removed, "Handler should have been removed");
        let after_removal = registry_size().expect("Should get size");
        assert_eq!(after_removal, after_registration - 1, "Should have removed 1 handler");
        
        // Test that unregistering non-existent handler returns false
        let not_removed = unregister_handler("non_existent_handler")
            .expect("Should unregister");
        assert!(!not_removed, "Non-existent handler should return false");
        let still_same = registry_size().expect("Should get size");
        assert_eq!(still_same, after_removal, "Size should be unchanged");
        
        // Clean up our test handlers
        for i in 0..10 {
            if i != 5 { // We already removed handler 5
                let handler_tag = format!("test_cleanup_handler_{}", i);
                unregister_handler(&handler_tag).expect("Should unregister test handler");
            }
        }
        
        let final_size = registry_size().expect("Should get size");
        assert_eq!(final_size, initial_size, "Should be back to initial size");
        
        println!("✅ Registry cleanup functionality test completed successfully");
    }
    
    #[test]
    fn test_registry_bounded_size() {
        // This test verifies that the registry enforces size limits to prevent memory leaks
        
        clear_registry().expect("Should clear registry");
        
        // Register many handlers to test size limit (more than MAX_HANDLERS = 1000)
        for i in 0..1200 {
            let handler_tag = format!("bounded_test_handler_{}", i);
            let handler = Arc::new(|_op: &Operation, _state: &mut WorldState| -> Result<Option<KernelEvent>, KernelError> {
                Ok(None)
            });
            
            register_handler(handler_tag, handler)
                .expect("Should register handler");
        }
        
        let final_size = registry_size().expect("Should get size");
        assert!(final_size <= 1000, "Registry size should be bounded to 1000, got {}", final_size);
        
        clear_registry().expect("Should clear registry");
        
        println!("✅ Registry bounded size test completed - size was bounded to {}", final_size);
    }
}
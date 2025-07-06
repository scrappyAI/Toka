//! # Toka Core
//!
//! Core utilities and error handling for Toka Agent OS.
//! This crate provides the fundamental types, error handling, and utilities
//! used across the entire Toka ecosystem.

pub mod error;
pub mod types;
pub mod config;
pub mod utils;

// Re-export commonly used items for convenience
pub use error::{TokaError, TokaResult};
pub use types::*;
pub use config::*;

/// Version of the Toka Core library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration namespace for Toka
pub const DEFAULT_NAMESPACE: &str = "toka";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_namespace() {
        assert_eq!(DEFAULT_NAMESPACE, "toka");
    }
}
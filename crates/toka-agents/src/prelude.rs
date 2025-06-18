//! Toka Agents – common re-exports
//! 
//! Import this prelude to quickly access the default agent types and traits.

pub use crate::{SymbolicAgent, Observation, Belief};
 
// Re-export the core Agent trait so consumers only need one glob import.
pub use crate::Agent; 
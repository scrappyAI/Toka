//! Toka Agents – common re-exports
//!
//! Import this prelude to quickly access the default agent types and traits.

pub use crate::{Belief, Observation, BaseAgent};

// Re-export the core Agent trait so consumers only need one glob import.
pub use crate::Agent as AgentTrait;

pub use crate::{AgentBundle, ToolSpec, SystemAgent, SystemAgentKind};

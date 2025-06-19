use anyhow::Result;
use async_trait::async_trait;

/// Result produced by a reasoning engine.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReasonOutcome {
    /// Updated belief key with probabilities.
    BeliefUpdates(Vec<(String, f64)>),
    /// Generated plans.
    Plans(Vec<String>),
    /// Declaration of desired tool calls.
    ToolCalls(Vec<ToolInvocation>),
    /// Engine decided no action needed.
    NoOp,
}

/// Declaration of a tool invocation – placeholder until ToolRegistry types exist.
#[derive(Debug)]
pub struct ToolInvocation {
    pub name: String,
    pub args: std::collections::HashMap<String, String>,
}

/// A minimal dependency-injected context passed to reasoners.
/// This will be expanded in future phases with vault, memory, etc.
pub struct AgentContext<'a> {
    pub event_bus: &'a crate::EventBus,
}

/// Core abstraction over any reasoning backend.
#[async_trait]
pub trait ReasoningEngine: Send + Sync {
    async fn reason(&self, ctx: &AgentContext<'_>) -> Result<ReasonOutcome>;
}

/// A no-op engine that always returns `ReasonOutcome::NoOp`.
#[derive(Default)]
pub struct NoOpReasoner;

#[async_trait]
impl ReasoningEngine for NoOpReasoner {
    async fn reason(&self, _ctx: &AgentContext<'_>) -> Result<ReasonOutcome> {
        Ok(ReasonOutcome::NoOp)
    }
} 
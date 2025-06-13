//! # Agent Pricing
//!
//! Defines pricing configurations specific to AI agents.

use serde::{Serialize, Deserialize};
use crate::ids::AgentID;
use crate::currency::MicroUSD;

/// Configuration for pricing related to a specific AI agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentPricingConfig {
    pub agent_id: AgentID,
    pub cost_per_invocation_micro_usd: MicroUSD,
    pub cost_per_token_micro_usd: Option<MicroUSD>,
} 
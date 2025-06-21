use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{Belief, Observation, EventBus, AgentEvent};
use super::{AgentContext, ReasonOutcome, ReasoningEngine};
use async_trait::async_trait;

/// Core Bayesian symbolic reasoner separated from agent container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicReasoner {
    pub beliefs: HashMap<String, Belief>,
    pub action_threshold: f64,
    pub planning_threshold: f64,
}

impl Default for SymbolicReasoner {
    fn default() -> Self {
        Self {
            beliefs: HashMap::new(),
            action_threshold: 0.7,
            planning_threshold: 0.6,
        }
    }
}

impl SymbolicReasoner {
    /// Update beliefs based on new observations using Bayesian inference
    pub async fn observe(&mut self, observation: Observation, bus: Option<&EventBus>, agent_id: &str) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let belief = self
            .beliefs
            .entry(observation.key.clone())
            .or_insert(Belief {
                probability: 0.5,
                last_updated: current_time,
            });

        // Bayesian update
        let likelihood_ratio = if observation.supports {
            observation.evidence_strength
        } else {
            1.0 / observation.evidence_strength
        };
        let prior_odds = belief.probability / (1.0 - belief.probability);
        let posterior_odds = prior_odds * likelihood_ratio;
        let new_prob = posterior_odds / (1.0 + posterior_odds);

        belief.probability = new_prob;
        belief.last_updated = current_time;

        if let Some(bus) = bus {
            let _ = bus
                .emit_agent_event(
                    AgentEvent::BeliefUpdated {
                        agent_id: agent_id.to_string(),
                        belief_key: observation.key.clone(),
                        probability: new_prob,
                        timestamp: current_time,
                    },
                    &format!("agent:{}", agent_id),
                )
                .await;
        }
        Ok(())
    }

    /// Returns hypotheses and probabilities
    pub fn hypotheses(&self) -> Vec<(String, f64)> {
        self.beliefs.iter().map(|(k,v)| (k.clone(), v.probability)).collect()
    }

    pub fn actions(&self) -> Vec<String> {
        self.beliefs.iter().filter(|(_,v)| v.probability > self.action_threshold)
            .map(|(k,_)| format!("Trigger action for hypothesis: {}", k)).collect()
    }

    pub fn plans(&self) -> Vec<String> {
        self.beliefs.iter().filter(|(_,v)| v.probability > self.planning_threshold)
            .map(|(k,_)| format!("Design plan to test hypothesis: {}", k)).collect()
    }

    pub fn new() -> Self { Self::default() }
}

#[async_trait]
impl ReasoningEngine for SymbolicReasoner {
    async fn reason(&self, _ctx: &AgentContext<'_>) -> Result<ReasonOutcome> {
        // For now no top-level reasoning cycle; callers use observe/act/plan.
        Ok(ReasonOutcome::NoOp)
    }
} 
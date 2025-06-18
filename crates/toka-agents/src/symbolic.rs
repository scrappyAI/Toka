use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{AgentEvent, EventBus};

/// Represents a belief state with probability and timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub probability: f64,
    pub last_updated: u64,
}

/// Represents an observation that can update beliefs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub key: String,
    pub evidence_strength: f64, // How strong is the observed evidence
    pub supports: bool,         // Does it support or refute the hypothesis
}

/// Core agent structure with belief state management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicAgent {
    pub id: String,
    pub beliefs: HashMap<String, Belief>,
    pub context: HashMap<String, String>,
    pub action_threshold: f64,
    pub planning_threshold: f64,
    #[serde(skip)]
    pub event_bus: Option<EventBus>,
}

impl SymbolicAgent {
    /// Create a new symbolic agent with a unique identifier and configurable thresholds
    pub fn new(id: &str) -> Self {
        SymbolicAgent {
            id: id.to_string(),
            beliefs: HashMap::new(),
            context: HashMap::new(),
            action_threshold: 0.7,   // Default threshold for triggering actions
            planning_threshold: 0.6, // Default threshold for planning
            event_bus: None,
        }
    }

    /// Create a new symbolic agent with custom thresholds
    pub fn new_with_thresholds(id: &str, action_threshold: f64, planning_threshold: f64) -> Self {
        SymbolicAgent {
            id: id.to_string(),
            beliefs: HashMap::new(),
            context: HashMap::new(),
            action_threshold,
            planning_threshold,
            event_bus: None,
        }
    }

    /// Set the event bus for this agent
    pub fn set_event_bus(&mut self, event_bus: EventBus) {
        self.event_bus = Some(event_bus);
    }

    /// Update beliefs based on new observations using Bayesian inference
    pub async fn observe(&mut self, observation: Observation) -> Result<()> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let belief = self
            .beliefs
            .entry(observation.key.clone())
            .or_insert(Belief {
                probability: 0.5, // neutral prior
                last_updated: current_time,
            });

        // Bayesian update with likelihood ratio
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

        // Emit belief update event
        if let Some(event_bus) = &self.event_bus {
            let _ = event_bus
                .emit_agent_event(
                    AgentEvent::BeliefUpdated {
                        agent_id: self.id.clone(),
                        belief_key: observation.key.clone(),
                        probability: new_prob,
                        timestamp: current_time,
                    },
                    &format!("agent:{}", self.id),
                )
                .await;

            let _ = event_bus
                .emit_agent_event(
                    AgentEvent::ObservationProcessed {
                        agent_id: self.id.clone(),
                        observation_key: observation.key,
                        timestamp: current_time,
                    },
                    &format!("agent:{}", self.id),
                )
                .await;
        }

        Ok(())
    }

    /// Get current hypotheses with their probabilities
    pub async fn hypothesize(&self) -> Vec<(String, f64)> {
        self.beliefs
            .iter()
            .map(|(k, v)| (k.clone(), v.probability))
            .collect()
    }

    /// Generate actions based on high-confidence beliefs
    pub async fn act(&self) -> Vec<String> {
        let actions: Vec<String> = self
            .beliefs
            .iter()
            .filter(|(_, v)| v.probability > self.action_threshold)
            .map(|(k, _)| format!("Trigger action for hypothesis: {}", k))
            .collect();

        // Emit action events
        if let Some(event_bus) = &self.event_bus {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(SystemTime::UNIX_EPOCH.duration_since(UNIX_EPOCH).unwrap())
                .as_secs();

            for action in &actions {
                let _ = event_bus
                    .emit_agent_event(
                        AgentEvent::ActionTriggered {
                            agent_id: self.id.clone(),
                            action: action.clone(),
                            timestamp: current_time,
                        },
                        &format!("agent:{}", self.id),
                    )
                    .await;
            }
        }

        actions
    }

    /// Generate plans to test hypotheses
    pub async fn plan(&mut self) -> Vec<String> {
        let plans: Vec<String> = self
            .hypothesize()
            .await
            .into_iter()
            .filter(|(_, p)| *p > self.planning_threshold)
            .map(|(hyp, _)| format!("Design plan to test hypothesis: {}", hyp))
            .collect();

        // Emit planning events
        if let Some(event_bus) = &self.event_bus {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(SystemTime::UNIX_EPOCH.duration_since(UNIX_EPOCH).unwrap())
                .as_secs();

            for plan in &plans {
                let _ = event_bus
                    .emit_agent_event(
                        AgentEvent::PlanGenerated {
                            agent_id: self.id.clone(),
                            plan: plan.clone(),
                            timestamp: current_time,
                        },
                        &format!("agent:{}", self.id),
                    )
                    .await;
            }
        }

        plans
    }

    /// Process feedback as a new observation
    pub async fn outcome(&mut self, feedback: Observation) -> Result<()> {
        self.observe(feedback).await
    }

    /// Set context information for the agent
    #[allow(dead_code)]
    pub fn set_context(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
    }

    /// Get context information
    #[allow(dead_code)]
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.context.get(key)
    }

    /// Get a concise summary of the agent's state
    pub fn summarize(&self) -> String {
        format!(
            "SymbolicAgent {{ id: {}, beliefs: {} entries, context: {} keys }}",
            self.id,
            self.beliefs.len(),
            self.context.len()
        )
    }

    /// Update the threshold for triggering actions
    pub fn set_action_threshold(&mut self, threshold: f64) {
        self.action_threshold = threshold;
    }

    /// Update the threshold for planning
    pub fn set_planning_threshold(&mut self, threshold: f64) {
        self.planning_threshold = threshold;
    }

    /// Get current action threshold
    pub fn get_action_threshold(&self) -> f64 {
        self.action_threshold
    }

    /// Get current planning threshold
    pub fn get_planning_threshold(&self) -> f64 {
        self.planning_threshold
    }
}

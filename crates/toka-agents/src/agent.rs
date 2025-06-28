#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::reasoning::symbolic::SymbolicReasoner;
use crate::{AgentEvent, EventBus};
use toka_bus::EventBusExt as _;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub probability: f64,
    pub last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub key: String,
    pub evidence_strength: f64,
    pub supports: bool,
}

/// Generic event-driven agent shell that embeds a reasoning engine.
///
/// The default engine is `SymbolicReasoner`, but you can replace it at runtime
/// (future phases will make this a boxed trait to allow heterogenous engines).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseAgent {
    pub id: String,
    pub reasoner: SymbolicReasoner,
    pub context: HashMap<String, String>,
    #[serde(skip)]
    pub event_bus: Option<EventBus>,
}

impl BaseAgent {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            reasoner: SymbolicReasoner::new(),
            context: HashMap::new(),
            event_bus: None,
        }
    }

    pub fn new_with_thresholds(id: &str, action_threshold: f64, planning_threshold: f64) -> Self {
        let mut reasoner = SymbolicReasoner::new();
        reasoner.action_threshold = action_threshold;
        reasoner.planning_threshold = planning_threshold;
        Self {
            id: id.to_string(),
            reasoner,
            context: HashMap::new(),
            event_bus: None,
        }
    }

    pub fn set_event_bus(&mut self, bus: EventBus) {
        self.event_bus = Some(bus);
    }

    pub async fn observe(&mut self, obs: Observation) -> Result<()> {
        self.reasoner
            .observe(obs, self.event_bus.as_ref(), &self.id)
            .await
    }

    pub async fn hypothesize(&self) -> Vec<(String, f64)> {
        self.reasoner.hypotheses()
    }

    pub async fn act(&self) -> Vec<String> {
        let actions = self.reasoner.actions();
        if let Some(bus) = &self.event_bus {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            for action in &actions {
                let _ = bus
                    .emit_agent_event(
                        AgentEvent::ActionTriggered {
                            agent_id: self.id.clone(),
                            action: action.clone(),
                            timestamp: ts,
                        },
                        &format!("agent:{}", self.id),
                    )
                    .await;
            }
        }
        actions
    }

    pub async fn plan(&mut self) -> Vec<String> {
        let plans = self.reasoner.plans();
        if let Some(bus) = &self.event_bus {
            let ts = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            for p in &plans {
                let _ = bus
                    .emit_agent_event(
                        AgentEvent::PlanGenerated {
                            agent_id: self.id.clone(),
                            plan: p.clone(),
                            timestamp: ts,
                        },
                        &format!("agent:{}", self.id),
                    )
                    .await;
            }
        }
        plans
    }

    // thresholds helpers ----------------------------------------------------
    pub fn set_action_threshold(&mut self, t: f64) {
        self.reasoner.action_threshold = t;
    }
    pub fn set_planning_threshold(&mut self, t: f64) {
        self.reasoner.planning_threshold = t;
    }
    pub fn get_action_threshold(&self) -> f64 {
        self.reasoner.action_threshold
    }
    pub fn get_planning_threshold(&self) -> f64 {
        self.reasoner.planning_threshold
    }

    pub fn beliefs(&self) -> &HashMap<String, Belief> {
        &self.reasoner.beliefs
    }

    pub fn summarize(&self) -> String {
        format!(
            "BaseAgent {{ id: {}, beliefs: {} entries, context: {} keys }}",
            self.id,
            self.reasoner.beliefs.len(),
            self.context.len()
        )
    }

    async fn save_state(&self, adapter: &dyn toka_memory::MemoryAdapter) -> Result<()> {
        let key = format!("agent:{}", self.id);
        let bytes = serde_json::to_vec(self)?;
        adapter.put(&key, bytes, 0).await
    }

    async fn load_state(&mut self, adapter: &dyn toka_memory::MemoryAdapter) -> Result<()> {
        let key = format!("agent:{}", self.id);
        if let Some(bytes) = adapter.get(&key).await? {
            if let Ok(saved) = serde_json::from_slice::<BaseAgent>(&bytes) {
                self.context = saved.context;
                self.reasoner = saved.reasoner;
            }
        }
        Ok(())
    }
}

// Temporary alias for backward compatibility – will be deprecated in a future release.
#[allow(deprecated)]
#[deprecated(note = "Use BaseAgent instead; SymbolicAgent will be removed in a future release.")]
pub type SymbolicAgent = BaseAgent;

// -------------------------------------------------------------------------
// Toolkit bridge (only when `toolkit` feature enabled)
// -------------------------------------------------------------------------
#[cfg(feature = "toolkit")]
impl BaseAgent {
    /// Invoke a registered tool via the runtime's `ToolRegistry` and emit
    /// appropriate `ToolEvent`s on success or error.
    pub async fn invoke_tool(
        &self,
        registry: &toka_toolkit_core::ToolRegistry,
        params: toka_toolkit_core::ToolParams,
    ) -> anyhow::Result<toka_toolkit_core::ToolResult> {
        use toka_toolkit_core::ToolResult;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Emit Invoked event
        if let Some(bus) = &self.event_bus {
            let _ = bus
                .emit_tool_event(
                    toka_bus::ToolEvent::Invoked {
                        tool_name: params.name.clone(),
                        user_id: self.id.clone(),
                        timestamp: now,
                    },
                    &format!("agent:{}", self.id),
                )
                .await;
        }

        // Execute tool
        let result: anyhow::Result<ToolResult> = registry.execute_tool(&params.name, &params).await;

        // Emit completion/error event
        if let Some(bus) = &self.event_bus {
            match &result {
                Ok(r) => {
                    let _ = bus
                        .emit_tool_event(
                            toka_bus::ToolEvent::Completed {
                                tool_name: params.name.clone(),
                                user_id: self.id.clone(),
                                duration_ms: r.metadata.execution_time_ms,
                                success: r.success,
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            },
                            &format!("agent:{}", self.id),
                        )
                        .await;
                }
                Err(e) => {
                    let _ = bus
                        .emit_tool_event(
                            toka_bus::ToolEvent::Error {
                                tool_name: params.name.clone(),
                                user_id: self.id.clone(),
                                error: e.to_string(),
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            },
                            &format!("agent:{}", self.id),
                        )
                        .await;
                }
            }
        }

        result
    }
}

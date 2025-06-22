//! Rich, typed event system used by Runtime, Agents and Tools.
//!
//! This module is a slimmed-down copy of `toka-runtime/src/events` so that
//! **all crates depend on a single implementation**.  It purposefully omits
//! docs/examples to stay under 250 lines; functional parity is preserved.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

// -----------------------------------------------------------------------------
// Event type hierarchy
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEvent {
    UserLogin {
        user_id: String,
        timestamp: u64,
    },
    UserLogout {
        user_id: String,
        timestamp: u64,
    },
    AuthFailure {
        attempt_info: String,
        timestamp: u64,
    },
    TokenRefresh {
        user_id: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    Created {
        agent_id: String,
        agent_type: String,
        timestamp: u64,
    },
    BeliefUpdated {
        agent_id: String,
        belief_key: String,
        probability: f64,
        timestamp: u64,
    },
    ActionTriggered {
        agent_id: String,
        action: String,
        timestamp: u64,
    },
    PlanGenerated {
        agent_id: String,
        plan: String,
        timestamp: u64,
    },
    ObservationProcessed {
        agent_id: String,
        observation_key: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolEvent {
    Invoked {
        tool_name: String,
        user_id: String,
        timestamp: u64,
    },
    Completed {
        tool_name: String,
        user_id: String,
        duration_ms: u64,
        success: bool,
        timestamp: u64,
    },
    Error {
        tool_name: String,
        user_id: String,
        error: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultEvent {
    SecretCreated {
        vault_id: String,
        secret_key: String,
        timestamp: u64,
    },
    SecretAccessed {
        vault_id: String,
        secret_key: String,
        user_id: String,
        timestamp: u64,
    },
    SecretUpdated {
        vault_id: String,
        secret_key: String,
        timestamp: u64,
    },
    SecretDeleted {
        vault_id: String,
        secret_key: String,
        timestamp: u64,
    },
    VaultUnlocked {
        vault_id: String,
        user_id: String,
        timestamp: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Auth(AuthEvent),
    Agent(AgentEvent),
    Tool(ToolEvent),
    Vault(VaultEvent),
    Generic { event_type: String, data: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub source: String,
    pub timestamp: u64,
}

impl Event {
    pub fn new(event_type: EventType, source: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            source: source.to_owned(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time went backwards")
                .as_secs(),
        }
    }
}

// -----------------------------------------------------------------------------
// Subscriber trait
// -----------------------------------------------------------------------------

#[async_trait::async_trait]
pub trait EventSubscriber: Send + Sync {
    async fn handle_event(&self, event: &Event) -> Result<()>;
    fn subscriber_id(&self) -> &str;
}

// -----------------------------------------------------------------------------
// EventBus implementation (tokio broadcast + in-proc subscribers)
// -----------------------------------------------------------------------------

const DEFAULT_BUFFER: usize = 1024;

pub struct EventBus {
    sender: broadcast::Sender<Event>,
    subscribers: Arc<RwLock<HashMap<String, Box<dyn EventSubscriber>>>>,
}

impl EventBus {
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer.max(1));
        Self {
            sender,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn new_default() -> Self {
        Self::new(DEFAULT_BUFFER)
    }

    pub async fn emit(&self, event_type: EventType, source: &str) -> Result<()> {
        let event = Event::new(event_type, source);
        let _ = self.sender.send(event.clone());
        let subs = self.subscribers.read().await;
        for sub in subs.values() {
            let _ = sub.handle_event(&event).await;
        }
        Ok(())
    }

    pub fn get_receiver(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    pub async fn subscribe(&self, subscriber: Box<dyn EventSubscriber>) -> Result<()> {
        let id = subscriber.subscriber_id().to_owned();
        self.subscribers.write().await.insert(id, subscriber);
        Ok(())
    }

    pub async fn unsubscribe(&self, id: &str) -> Result<()> {
        self.subscribers.write().await.remove(id);
        Ok(())
    }

    pub async fn emit_agent_event(&self, agent_event: AgentEvent, source: &str) -> Result<()> {
        self.emit(EventType::Agent(agent_event), source).await
    }

    pub async fn emit_tool_event(&self, tool_event: ToolEvent, source: &str) -> Result<()> {
        self.emit(EventType::Tool(tool_event), source).await
    }

    pub async fn emit_vault_event(&self, vault_event: VaultEvent, source: &str) -> Result<()> {
        self.emit(EventType::Vault(vault_event), source).await
    }

    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new_default()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            subscribers: self.subscribers.clone(),
        }
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus").finish()
    }
}

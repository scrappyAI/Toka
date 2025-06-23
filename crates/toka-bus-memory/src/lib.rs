//! In-memory event bus implementation for the Toka platform.
//!
//! This crate provides a memory-based event bus using tokio::broadcast for
//! efficient event distribution within a single process.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

// Re-export core types for convenience
pub use toka_events_core::{causal_hash, create_event_header, CausalDigest, IntentId};

/// Default buffer size for the broadcast channel
const DEFAULT_BUFFER: usize = 1024;

// -----------------------------------------------------------------------------
// Event type hierarchy (migrated from rich.rs)
// -----------------------------------------------------------------------------

/// Authentication-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthEvent {
    // Login events
    UserLogin {
        user_id: String,
        ip_address: String,
        device_info: String,
        timestamp: u64,
    },
    UserLogout {
        user_id: String,
        session_duration: u64,
        timestamp: u64,
    },
    
    // Authentication failures
    AuthFailure {
        attempt_info: String,
        failure_type: String, // e.g. "invalid_password", "expired_token", etc.
        ip_address: String,
        user_agent: String,
        timestamp: u64,
    },
    
    // Token management
    TokenRefresh {
        user_id: String,
        token_type: String, // e.g. "access", "refresh"
        previous_token_age: u64,
        timestamp: u64,
    },
    TokenRevoked {
        user_id: String,
        token_id: String,
        reason: String,
        timestamp: u64,
    },
    
    // Permission changes
    PermissionGranted {
        user_id: String,
        permission: String,
        granted_by: String,
        timestamp: u64,
    },
    PermissionRevoked {
        user_id: String,
        permission: String,
        revoked_by: String,
        reason: String,
        timestamp: u64,
    },
    
    // Security events
    SuspiciousActivity {
        user_id: String,
        activity_type: String,
        details: String,
        ip_address: String,
        timestamp: u64,
    },
    AccountLocked {
        user_id: String,
        reason: String,
        lock_duration: u64,
        timestamp: u64,
    },
    AccountUnlocked {
        user_id: String,
        unlocked_by: String,
        timestamp: u64,
    }
}

/// Agent-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentEvent {
    // Creation events
    Created { agent_id: String, agent_type: String, timestamp: u64 },
    CreationFailed { agent_id: String, error: String, timestamp: u64 },
    
    // Belief events
    BeliefUpdated { agent_id: String, belief_key: String, probability: f64, timestamp: u64 },
    BeliefUpdateFailed { agent_id: String, belief_key: String, error: String, timestamp: u64 },
    
    // Action events
    ActionTriggered { agent_id: String, action: String, timestamp: u64 },
    ActionStarted { agent_id: String, action: String, timestamp: u64 },
    ActionCompleted { agent_id: String, action: String, result: String, timestamp: u64 },
    ActionFailed { agent_id: String, action: String, error: String, timestamp: u64 },
    ActionTimedOut { agent_id: String, action: String, timeout_ms: u64, timestamp: u64 },
    
    // Planning events
    PlanningStarted { agent_id: String, timestamp: u64 },
    PlanGenerated { agent_id: String, plan: String, timestamp: u64 },
    PlanningFailed { agent_id: String, error: String, timestamp: u64 },
    PlanningTimedOut { agent_id: String, timeout_ms: u64, timestamp: u64 },
    
    // Observation events
    ObservationReceived { agent_id: String, observation_key: String, timestamp: u64 },
    ObservationProcessing { agent_id: String, observation_key: String, timestamp: u64 },
    ObservationProcessed { agent_id: String, observation_key: String, timestamp: u64 },
    ObservationFailed { agent_id: String, observation_key: String, error: String, timestamp: u64 },
    
    // Thinking/Processing events
    ThinkingStarted { agent_id: String, task: String, timestamp: u64 },
    ThinkingCompleted { agent_id: String, task: String, timestamp: u64 },
    ThinkingFailed { agent_id: String, task: String, error: String, timestamp: u64 },
    
    // System events
    AgentPaused { agent_id: String, reason: String, timestamp: u64 },
    AgentResumed { agent_id: String, timestamp: u64 },
    AgentTerminated { agent_id: String, reason: String, timestamp: u64 }
}

/// Tool-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolEvent {
    Invoked { tool_name: String, user_id: String, timestamp: u64 },
    Completed { tool_name: String, user_id: String, duration_ms: u64, success: bool, timestamp: u64 },
    Error { tool_name: String, user_id: String, error: String, timestamp: u64 },
}

/// Vault-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VaultEvent {
    SecretCreated { vault_id: String, secret_key: String, timestamp: u64 },
    SecretAccessed { vault_id: String, secret_key: String, user_id: String, timestamp: u64 },
    SecretUpdated { vault_id: String, secret_key: String, timestamp: u64 },
    SecretDeleted { vault_id: String, secret_key: String, timestamp: u64 },
    VaultUnlocked { vault_id: String, user_id: String, timestamp: u64 },
}

/// Memory-related events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MemoryEvent {
    MemoryAccessed { memory_id: String, user_id: String, timestamp: u64 },
    MemoryUpdated { memory_id: String, user_id: String, timestamp: u64 },
    MemoryDeleted { memory_id: String, user_id: String, timestamp: u64 },
    /// Memory has been successfully persisted to durable storage
    MemoryPersisted { memory_id: String, user_id: String, timestamp: u64 },
    MemoryAccessFailed { memory_id: String, user_id: String, error: String, timestamp: u64 },
    MemoryUpdateFailed { memory_id: String, user_id: String, error: String, timestamp: u64 },
    MemoryDeleteFailed { memory_id: String, user_id: String, error: String, timestamp: u64 },
}

/// Legacy event type hierarchy for backward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    Auth(AuthEvent),
    Agent(AgentEvent),
    Tool(ToolEvent),
    Vault(VaultEvent),
    Memory(MemoryEvent),
    Generic { event_type: String, data: String },
}

/// Legacy event structure for backward compatibility
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

// Event types automatically implement EventPayload via the blanket impl in toka-events-core

// -----------------------------------------------------------------------------
// Subscriber trait
// -----------------------------------------------------------------------------

/// Trait for handling events asynchronously
#[async_trait]
pub trait EventSubscriber: Send + Sync {
    /// Handle an incoming event
    async fn handle_event(&self, event: &Event) -> Result<()>;
    
    /// Unique identifier for this subscriber
    fn subscriber_id(&self) -> &str;
}

// -----------------------------------------------------------------------------
// EventBus trait definition
// -----------------------------------------------------------------------------

/// Generic trait for event bus implementations
#[async_trait]
pub trait EventBus: Send + Sync + Clone {
    /// Emit an event to all subscribers
    async fn emit(&self, event_type: EventType, source: &str) -> Result<()>;
    
    /// Get a receiver for broadcast events
    fn get_receiver(&self) -> broadcast::Receiver<Event>;
    
    /// Subscribe a handler for events
    async fn subscribe(&self, subscriber: Box<dyn EventSubscriber>) -> Result<()>;
    
    /// Unsubscribe a handler
    async fn unsubscribe(&self, id: &str) -> Result<()>;
    
    /// Get the number of active subscribers
    async fn subscriber_count(&self) -> usize;
    
    // Convenience methods
    async fn emit_agent_event(&self, agent_event: AgentEvent, source: &str) -> Result<()> {
        self.emit(EventType::Agent(agent_event), source).await
    }
    
    async fn emit_tool_event(&self, tool_event: ToolEvent, source: &str) -> Result<()> {
        self.emit(EventType::Tool(tool_event), source).await
    }
    
    async fn emit_vault_event(&self, vault_event: VaultEvent, source: &str) -> Result<()> {
        self.emit(EventType::Vault(vault_event), source).await
    }
    
    async fn emit_auth_event(&self, auth_event: AuthEvent, source: &str) -> Result<()> {
        self.emit(EventType::Auth(auth_event), source).await
    }

    async fn emit_memory_event(&self, memory_event: MemoryEvent, source: &str) -> Result<()> {
        self.emit(EventType::Memory(memory_event), source).await
    }
}

// -----------------------------------------------------------------------------
// In-memory EventBus implementation
// -----------------------------------------------------------------------------

/// In-memory event bus using tokio::broadcast
#[derive(Clone)]
pub struct MemoryEventBus {
    sender: broadcast::Sender<Event>,
    subscribers: Arc<RwLock<HashMap<String, Box<dyn EventSubscriber>>>>,
}

impl std::fmt::Debug for MemoryEventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryEventBus").finish()
    }
}

impl MemoryEventBus {
    /// Create a new memory event bus with specified buffer size
    pub fn new(buffer: usize) -> Self {
        let (sender, _) = broadcast::channel(buffer.max(1));
        Self {
            sender,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Create a new memory event bus with default buffer size
    pub fn new_default() -> Self {
        Self::new(DEFAULT_BUFFER)
    }
}

#[async_trait]
impl EventBus for MemoryEventBus {
    async fn emit(&self, event_type: EventType, source: &str) -> Result<()> {
        let event = Event::new(event_type, source);
        
        // Broadcast to all receivers
        let _ = self.sender.send(event.clone());
        
        // Notify direct subscribers
        let subs = self.subscribers.read().await;
        for sub in subs.values() {
            let _ = sub.handle_event(&event).await;
        }
        
        Ok(())
    }
    
    fn get_receiver(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
    
    async fn subscribe(&self, subscriber: Box<dyn EventSubscriber>) -> Result<()> {
        let id = subscriber.subscriber_id().to_owned();
        self.subscribers.write().await.insert(id, subscriber);
        Ok(())
    }
    
    async fn unsubscribe(&self, id: &str) -> Result<()> {
        self.subscribers.write().await.remove(id);
        Ok(())
    }
    
    async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }
}

impl Default for MemoryEventBus {
    fn default() -> Self {
        Self::new_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct CountingSub {
        id: String,
        count: tokio::sync::Mutex<usize>,
    }

    #[async_trait]
    impl EventSubscriber for CountingSub {
        async fn handle_event(&self, _e: &Event) -> Result<()> {
            let mut c = self.count.lock().await;
            *c += 1;
            Ok(())
        }
        
        fn subscriber_id(&self) -> &str {
            &self.id
        }
    }

    #[tokio::test]
    async fn test_event_bus_creation() {
        let bus = MemoryEventBus::new_default();
        let _receiver = bus.get_receiver();
        // Should not panic
    }

    #[tokio::test]
    async fn test_event_emission() -> Result<()> {
        let bus = MemoryEventBus::new_default();
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        
        bus.emit(
            EventType::Auth(AuthEvent::UserLogin {
                user_id: "alice".to_string(),
                ip_address: "127.0.0.1".to_string(),
                device_info: "test-device".to_string(),
                timestamp: ts,
            }),
            "test",
        ).await?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_subscriber_functionality() -> Result<()> {
        let bus = MemoryEventBus::new_default();
        let sub = Box::new(CountingSub {
            id: "test".into(),
            count: tokio::sync::Mutex::new(0),
        });
        
        bus.subscribe(sub).await?;
        
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        bus.emit_tool_event(
            ToolEvent::Invoked {
                tool_name: "dummy".into(),
                user_id: "u".into(),
                timestamp: ts,
            },
            "src",
        ).await?;
        
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(bus.subscriber_count().await, 1);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_broadcast_receiver() -> Result<()> {
        let bus = MemoryEventBus::new_default();
        let mut rx = bus.get_receiver();
        
        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        bus.emit_vault_event(
            VaultEvent::SecretCreated {
                vault_id: "v".into(),
                secret_key: "k".into(),
                timestamp: ts,
            },
            "svc",
        ).await?;
        
        let ev = rx.recv().await?;
        if let EventType::Vault(VaultEvent::SecretCreated { vault_id, .. }) = ev.event_type {
            assert_eq!(vault_id, "v");
        } else {
            panic!("unexpected event");
        }
        
        Ok(())
    }

    #[tokio::test]
    async fn test_unsubscribe() -> Result<()> {
        let bus = MemoryEventBus::new_default();
        let sub = Box::new(CountingSub {
            id: "test".into(),
            count: tokio::sync::Mutex::new(0),
        });
        
        bus.subscribe(sub).await?;
        assert_eq!(bus.subscriber_count().await, 1);
        
        bus.unsubscribe("test").await?;
        assert_eq!(bus.subscriber_count().await, 0);
        
        Ok(())
    }

    #[tokio::test]
    async fn test_memory_event_bus() -> Result<()> {
        let bus = MemoryEventBus::new_default();
        let sub = Box::new(CountingSub {
            id: "test".into(),
            count: tokio::sync::Mutex::new(0),
        });

        bus.subscribe(sub).await?;

        let ts = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        bus.emit_memory_event(
            MemoryEvent::MemoryAccessed {
                memory_id: "m".into(),
                user_id: "u".into(),
                timestamp: ts,
            },
            "svc",
        ).await?;

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        assert_eq!(bus.subscriber_count().await, 1);

        Ok(())
    }
}
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};

/// Represents different types of events in the system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventType {
    /// Authentication-related events
    Auth(AuthEvent),
    /// Agent-related events
    Agent(AgentEvent),
    /// Tool-related events
    Tool(ToolEvent),
    /// Vault-related events
    Vault(VaultEvent),
    /// Generic event for simple string-based messages
    Generic { event_type: String, data: String },
}

/// Authentication events
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

/// Agent events
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

/// Tool events
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

/// Vault events
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

/// Event wrapper with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: EventType,
    pub source: String,
    pub timestamp: u64,
}

/// Event subscriber trait for handling events
#[async_trait::async_trait]
pub trait EventSubscriber: Send + Sync {
    async fn handle_event(&self, event: &Event) -> Result<()>;
    fn subscriber_id(&self) -> &str;
}

/// Enhanced event bus with multiple subscribers and typed events
pub struct EventBus {
    sender: broadcast::Sender<Event>,
    subscribers: Arc<RwLock<HashMap<String, Box<dyn EventSubscriber>>>>,
}

impl EventBus {
    /// Create a new event bus with specified channel capacity
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            subscribers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new event bus with default capacity of 1000
    pub fn new_default() -> Self {
        Self::new(1000)
    }

    /// Emit an event to all subscribers
    pub async fn emit(&self, event_type: EventType, source: &str) -> Result<()> {
        let event = Event {
            id: uuid::Uuid::new_v4().to_string(),
            event_type,
            source: source.to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        // Send to broadcast channel (fire and forget for performance)
        let _ = self.sender.send(event.clone());

        // Notify subscribers
        let subscribers = self.subscribers.read().await;
        for subscriber in subscribers.values() {
            if let Err(e) = subscriber.handle_event(&event).await {
                eprintln!(
                    "Subscriber {} failed to handle event: {}",
                    subscriber.subscriber_id(),
                    e
                );
            }
        }

        Ok(())
    }

    /// Subscribe to events with a custom handler
    pub async fn subscribe(&self, subscriber: Box<dyn EventSubscriber>) -> Result<()> {
        let subscriber_id = subscriber.subscriber_id().to_string();
        let mut subscribers = self.subscribers.write().await;
        subscribers.insert(subscriber_id, subscriber);
        Ok(())
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscriber_id: &str) -> Result<()> {
        let mut subscribers = self.subscribers.write().await;
        subscribers.remove(subscriber_id);
        Ok(())
    }

    /// Get a receiver for listening to events directly
    pub fn get_receiver(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Get current subscriber count
    pub async fn subscriber_count(&self) -> usize {
        self.subscribers.read().await.len()
    }

    /// Convenience methods for common event types

    /// Emit an auth event
    pub async fn emit_auth_event(&self, auth_event: AuthEvent, source: &str) -> Result<()> {
        self.emit(EventType::Auth(auth_event), source).await
    }

    /// Emit an agent event
    pub async fn emit_agent_event(&self, agent_event: AgentEvent, source: &str) -> Result<()> {
        self.emit(EventType::Agent(agent_event), source).await
    }

    /// Emit a tool event
    pub async fn emit_tool_event(&self, tool_event: ToolEvent, source: &str) -> Result<()> {
        self.emit(EventType::Tool(tool_event), source).await
    }

    /// Emit a vault event
    pub async fn emit_vault_event(&self, vault_event: VaultEvent, source: &str) -> Result<()> {
        self.emit(EventType::Vault(vault_event), source).await
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
        let subscriber_count = self.subscribers.blocking_read().len();
        f.debug_struct("EventBus")
            .field("sender", &self.sender)
            .field("subscriber_count", &subscriber_count)
            .finish()
    }
}

/// Example subscriber implementation for logging
pub struct LoggingSubscriber {
    id: String,
}

impl LoggingSubscriber {
    pub fn new(id: &str) -> Self {
        Self { id: id.to_string() }
    }
}

#[async_trait::async_trait]
impl EventSubscriber for LoggingSubscriber {
    async fn handle_event(&self, event: &Event) -> Result<()> {
        println!(
            "[{}] Event {}: {:?} from {}",
            self.id, event.id, event.event_type, event.source
        );
        Ok(())
    }

    fn subscriber_id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_event_bus_creation() {
        let event_bus = EventBus::new_default();
        assert_eq!(event_bus.subscriber_count().await, 0);
    }

    #[tokio::test]
    async fn test_event_emission() -> Result<()> {
        let event_bus = EventBus::new_default();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        event_bus
            .emit_auth_event(
                AuthEvent::UserLogin {
                    user_id: "test_user".to_string(),
                    timestamp,
                },
                "test_source",
            )
            .await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_subscriber_functionality() -> Result<()> {
        let event_bus = EventBus::new_default();
        let subscriber = Box::new(LoggingSubscriber::new("test_logger"));

        event_bus.subscribe(subscriber).await?;
        assert_eq!(event_bus.subscriber_count().await, 1);

        event_bus.unsubscribe("test_logger").await?;
        assert_eq!(event_bus.subscriber_count().await, 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_broadcast_receiver() -> Result<()> {
        let event_bus = EventBus::new_default();
        let mut receiver = event_bus.get_receiver();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Emit an event
        event_bus
            .emit_vault_event(
                VaultEvent::SecretCreated {
                    vault_id: "test_vault".to_string(),
                    secret_key: "test_secret".to_string(),
                    timestamp,
                },
                "test_vault_service",
            )
            .await?;

        // Receive the event
        let received_event = receiver.recv().await?;
        match received_event.event_type {
            EventType::Vault(VaultEvent::SecretCreated { vault_id, .. }) => {
                assert_eq!(vault_id, "test_vault");
            }
            _ => panic!("Expected VaultEvent::SecretCreated"),
        }

        Ok(())
    }
}

/// Usage examples and patterns for the event bus system
#[cfg(test)]
mod examples {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    /// Example: Custom audit subscriber that counts security events
    pub struct SecurityAuditSubscriber {
        id: String,
        event_count: Arc<AtomicUsize>,
    }

    impl SecurityAuditSubscriber {
        pub fn new(id: &str) -> Self {
            Self {
                id: id.to_string(),
                event_count: Arc::new(AtomicUsize::new(0)),
            }
        }

        pub fn get_event_count(&self) -> usize {
            self.event_count.load(Ordering::Relaxed)
        }
    }

    #[async_trait::async_trait]
    impl EventSubscriber for SecurityAuditSubscriber {
        async fn handle_event(&self, event: &Event) -> Result<()> {
            match &event.event_type {
                EventType::Auth(AuthEvent::AuthFailure { .. })
                | EventType::Vault(VaultEvent::SecretAccessed { .. }) => {
                    self.event_count.fetch_add(1, Ordering::Relaxed);
                    println!("🔒 SECURITY AUDIT: {} - {:?}", event.id, event.event_type);
                }
                _ => {} // Ignore non-security events
            }
            Ok(())
        }

        fn subscriber_id(&self) -> &str {
            &self.id
        }
    }

    #[tokio::test]
    async fn example_comprehensive_event_system() -> Result<()> {
        // Create event bus
        let event_bus = EventBus::new_default();

        // Create and subscribe to various event handlers
        let logger = Box::new(LoggingSubscriber::new("system_logger"));
        let auditor = Box::new(SecurityAuditSubscriber::new("security_audit"));

        event_bus.subscribe(logger).await?;
        event_bus.subscribe(auditor).await?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Simulate various system events

        // 1. User authentication events
        event_bus
            .emit_auth_event(
                AuthEvent::UserLogin {
                    user_id: "alice".to_string(),
                    timestamp,
                },
                "auth_service",
            )
            .await?;

        event_bus
            .emit_auth_event(
                AuthEvent::AuthFailure {
                    attempt_info: "Invalid password for user 'bob'".to_string(),
                    timestamp,
                },
                "auth_service",
            )
            .await?;

        // 2. Agent events
        event_bus
            .emit_agent_event(
                AgentEvent::Created {
                    agent_id: "agent_001".to_string(),
                    agent_type: "SymbolicAgent".to_string(),
                    timestamp,
                },
                "agent_manager",
            )
            .await?;

        event_bus
            .emit_agent_event(
                AgentEvent::BeliefUpdated {
                    agent_id: "agent_001".to_string(),
                    belief_key: "market_will_rise".to_string(),
                    probability: 0.85,
                    timestamp,
                },
                "agent_001",
            )
            .await?;

        // 3. Tool events
        event_bus
            .emit_tool_event(
                ToolEvent::Invoked {
                    tool_name: "data_analyzer".to_string(),
                    user_id: "alice".to_string(),
                    timestamp,
                },
                "tool_runtime",
            )
            .await?;

        event_bus
            .emit_tool_event(
                ToolEvent::Completed {
                    tool_name: "data_analyzer".to_string(),
                    user_id: "alice".to_string(),
                    duration_ms: 1250,
                    success: true,
                    timestamp,
                },
                "tool_runtime",
            )
            .await?;

        // 4. Vault events
        event_bus
            .emit_vault_event(
                VaultEvent::VaultUnlocked {
                    vault_id: "personal_vault_alice".to_string(),
                    user_id: "alice".to_string(),
                    timestamp,
                },
                "vault_service",
            )
            .await?;

        event_bus
            .emit_vault_event(
                VaultEvent::SecretAccessed {
                    vault_id: "personal_vault_alice".to_string(),
                    secret_key: "api_key_openai".to_string(),
                    user_id: "alice".to_string(),
                    timestamp,
                },
                "vault_service",
            )
            .await?;

        // Give events time to process
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Verify that subscribers are working correctly
        assert_eq!(event_bus.subscriber_count().await, 2);

        Ok(())
    }

    #[tokio::test]
    async fn example_direct_event_listening() -> Result<()> {
        let event_bus = EventBus::new_default();
        let mut receiver = event_bus.get_receiver();

        // Spawn a task to listen for events
        let listening_task = tokio::spawn(async move {
            let mut count = 0;
            while let Ok(event) = receiver.recv().await {
                count += 1;
                println!("📨 Received event {}: {:?}", count, event.event_type);
                if count >= 3 {
                    break;
                }
            }
            count
        });

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        // Emit some events
        for i in 1..=3 {
            event_bus
                .emit_auth_event(
                    AuthEvent::UserLogin {
                        user_id: format!("user_{}", i),
                        timestamp,
                    },
                    "auth_service",
                )
                .await?;
        }

        // Wait for the listening task to complete
        let events_received = listening_task.await?;
        assert_eq!(events_received, 3);

        Ok(())
    }
}

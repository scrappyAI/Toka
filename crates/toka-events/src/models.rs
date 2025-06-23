use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Re-export modern event types from toka-events-core
pub use toka_events_core::DomainEvent;

/// Legacy event categories for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
#[deprecated(note = "Use DomainEvent from toka-events-core instead")]
pub enum EventKind {
    /// Generic log / debug message emitted by subsystems.
    Log,
    /// Resource was created (`payload` often contains resource descriptor).
    ResourceCreated,
    /// Resource was updated or modified.
    ResourceUpdated,
    /// Resource was deleted.
    ResourceDeleted,
    /// Custom / user-defined event.
    Custom(String),
}

/// Legacy event structure for backward compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[deprecated(note = "Use DomainEvent from toka-events-core instead")]
pub struct Event {
    /// Unique identifier for the event.
    pub id: Uuid,
    /// Event category.
    pub kind: EventKind,
    /// Free-form JSON payload. Keep it loosely typed to avoid coupling.
    pub payload: serde_json::Value,
    /// UTC timestamp when the event was published.
    pub timestamp: DateTime<Utc>,
}

impl Event {
    /// Create a new event with current timestamp and random `Uuid`.
    pub fn new<K: Into<EventKind>, P: Serialize>(kind: K, payload: &P) -> Self {
        Self {
            id: Uuid::new_v4(),
            kind: kind.into(),
            payload: serde_json::to_value(payload).unwrap_or(serde_json::Value::Null),
            timestamp: Utc::now(),
        }
    }

    /// Convert this legacy event to a modern DomainEvent.
    pub fn to_domain_event(&self) -> DomainEvent {
        match &self.kind {
            EventKind::Log => DomainEvent::System {
                component: "legacy-log".to_string(),
                payload: self.payload.clone(),
            },
            EventKind::ResourceCreated => DomainEvent::Resource {
                resource_id: self.id,
                action: "created".to_string(),
                payload: self.payload.clone(),
            },
            EventKind::ResourceUpdated => DomainEvent::Resource {
                resource_id: self.id,
                action: "updated".to_string(),
                payload: self.payload.clone(),
            },
            EventKind::ResourceDeleted => DomainEvent::Resource {
                resource_id: self.id,
                action: "deleted".to_string(),
                payload: self.payload.clone(),
            },
            EventKind::Custom(event_type) => DomainEvent::Custom {
                event_type: event_type.clone(),
                payload: self.payload.clone(),
            },
        }
    }
}

impl From<DomainEvent> for Event {
    /// Convert a modern DomainEvent to a legacy Event for backward compatibility.
    fn from(domain_event: DomainEvent) -> Self {
        let (kind, payload) = match domain_event {
            DomainEvent::Agent { agent_id, payload } => {
                (EventKind::Custom("agent".to_string()), 
                 serde_json::json!({
                     "agent_id": agent_id,
                     "payload": payload
                 }))
            },
            DomainEvent::Ledger { transaction_id, payload } => {
                (EventKind::Custom("ledger".to_string()),
                 serde_json::json!({
                     "transaction_id": transaction_id,
                     "payload": payload
                 }))
            },
            DomainEvent::Resource { resource_id, action, payload } => {
                let kind = match action.as_str() {
                    "created" => EventKind::ResourceCreated,
                    "updated" => EventKind::ResourceUpdated,
                    "deleted" => EventKind::ResourceDeleted,
                    _ => EventKind::Custom(format!("resource-{}", action)),
                };
                (kind, serde_json::json!({
                    "resource_id": resource_id,
                    "action": action,
                    "payload": payload
                }))
            },
            DomainEvent::Security { principal_id, event_type, payload } => {
                (EventKind::Custom("security".to_string()),
                 serde_json::json!({
                     "principal_id": principal_id,
                     "event_type": event_type,
                     "payload": payload
                 }))
            },
            DomainEvent::System { component, payload } => {
                if component == "legacy-log" {
                    (EventKind::Log, payload)
                } else {
                    (EventKind::Custom("system".to_string()),
                     serde_json::json!({
                         "component": component,
                         "payload": payload
                     }))
                }
            },
            DomainEvent::Custom { event_type, payload } => {
                (EventKind::Custom(event_type), payload)
            },
            _ => {
                // Handle any future DomainEvent variants
                (EventKind::Custom("unknown".to_string()), serde_json::Value::Null)
            },
        };

        Self {
            id: Uuid::new_v4(),
            kind,
            payload,
            timestamp: Utc::now(),
        }
    }
}

// Allow easy `Into` of &str for custom events
impl From<&str> for EventKind {
    fn from(s: &str) -> Self {
        EventKind::Custom(s.to_string())
    }
}

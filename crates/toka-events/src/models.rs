use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Enumerates high-level categories of events on the platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[non_exhaustive]
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

/// A single immutable event entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

// Allow easy `Into` of &str for custom events
impl From<&str> for EventKind {
    fn from(s: &str) -> Self {
        EventKind::Custom(s.to_string())
    }
}

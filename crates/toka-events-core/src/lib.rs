//! Core event primitives and types for the Toka platform.
//!
//! This crate provides the foundational types and traits for event handling
//! across the Toka ecosystem, including causal hashing and event headers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use uuid::Uuid;

/// Unique identifier for an event in the vault.
pub type EventId = Uuid;

/// Identifier for an intent cluster.
pub type IntentId = Uuid;

/// 32-byte Blake3 digest used for causal hashing.
pub type CausalDigest = [u8; 32];

/// Trait implemented by all event payload structures that can be committed
/// to the vault.
pub trait EventPayload: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

impl<T> EventPayload for T where T: Serialize + for<'de> Deserialize<'de> + Send + Sync {}

/// Minimal header stored inline with every event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventHeader {
    /// Event identifier (UUID v4).
    pub id: EventId,
    /// Parent event IDs this event causally depends on (can be empty).
    pub parents: SmallVec<[EventId; 4]>,
    /// Wall-clock timestamp when the event was committed.
    pub timestamp: DateTime<Utc>,
    /// Blake3 digest of the event payload and its causal parent digests.
    pub digest: CausalDigest,
    /// Semantic intent bucket this event belongs to.
    /// For the *core* crate we don't try to cluster; callers can set it to
    /// whatever value they need (e.g. `Uuid::nil()` when unknown).
    pub intent: IntentId,
    /// Application-defined kind, e.g. `ledger.mint` or `chat.msg`.
    pub kind: String,
}

/// Domain events that can occur within the Toka platform.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[non_exhaustive]
pub enum DomainEvent {
    /// Agent-related events
    Agent {
        /// Agent ID
        agent_id: Uuid,
        /// Event payload
        payload: serde_json::Value,
    },
    /// Ledger transaction events
    Ledger {
        /// Transaction ID
        transaction_id: Uuid,
        /// Event payload
        payload: serde_json::Value,
    },
    /// Resource lifecycle events
    Resource {
        /// Resource ID
        resource_id: Uuid,
        /// Action performed (created, updated, deleted)
        action: String,
        /// Event payload
        payload: serde_json::Value,
    },
    /// Security and authentication events
    Security {
        /// User or principal ID
        principal_id: Uuid,
        /// Security event type
        event_type: String,
        /// Event payload
        payload: serde_json::Value,
    },
    /// System-level events
    System {
        /// System component
        component: String,
        /// Event payload
        payload: serde_json::Value,
    },
    /// Custom domain events
    Custom {
        /// Event type identifier
        event_type: String,
        /// Event payload
        payload: serde_json::Value,
    },
}

/// Compute Blake3 causal hash for an event.
///
/// This function takes an event payload and its causal parent digests,
/// and produces a deterministic hash that captures the causal relationship.
///
/// # Arguments
/// * `payload_bytes` - Serialized event payload
/// * `parent_digests` - Digests of causally dependent parent events
///
/// # Returns
/// A 32-byte Blake3 digest
pub fn causal_hash(payload_bytes: &[u8], parent_digests: &[CausalDigest]) -> CausalDigest {
    let mut hasher = blake3::Hasher::new();
    
    // Hash the payload
    hasher.update(payload_bytes);
    
    // Hash parent digests in sorted order for determinism
    let mut sorted_parents = parent_digests.to_vec();
    sorted_parents.sort_unstable();
    
    for parent_digest in sorted_parents {
        hasher.update(&parent_digest);
    }
    
    hasher.finalize().into()
}

/// Utility function to create an EventHeader with causal hash.
///
/// # Arguments
/// * `id` - Event ID
/// * `parents` - Parent event IDs
/// * `intent` - Intent ID
/// * `kind` - Event kind string
/// * `payload` - Event payload to hash
///
/// # Returns
/// A complete EventHeader with computed causal hash
pub fn create_event_header<P: EventPayload>(
    id: EventId,
    parents: SmallVec<[EventId; 4]>,
    intent: IntentId,
    kind: String,
    payload: &P,
    parent_digests: &[CausalDigest],
) -> Result<EventHeader, serde_json::Error> {
    let payload_bytes = serde_json::to_vec(payload)?;
    let digest = causal_hash(&payload_bytes, parent_digests);
    
    Ok(EventHeader {
        id,
        parents,
        timestamp: Utc::now(),
        digest,
        intent,
        kind,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_causal_hash_deterministic() {
        let payload = b"test_payload";
        let parent1 = [1u8; 32];
        let parent2 = [2u8; 32];
        
        let hash1 = causal_hash(payload, &[parent1, parent2]);
        let hash2 = causal_hash(payload, &[parent2, parent1]); // Different order
        
        // Should be the same due to sorting
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_domain_event_serialization() {
        let event = DomainEvent::Agent {
            agent_id: Uuid::new_v4(),
            payload: json!({"action": "started"}),
        };
        
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: DomainEvent = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_create_event_header() {
        let event = DomainEvent::System {
            component: "test".to_string(),
            payload: json!({"test": true}),
        };
        
        let header = create_event_header(
            Uuid::new_v4(),
            SmallVec::new(),
            Uuid::nil(),
            "system.test".to_string(),
            &event,
            &[],
        ).unwrap();
        
        assert_eq!(header.kind, "system.test");
        assert_eq!(header.parents.len(), 0);
    }
}
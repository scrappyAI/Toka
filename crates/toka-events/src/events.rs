//! Event façade for `toka-events` default crate.
//!
//! This module simply re-exports the canonical data types from the new
//! `toka-events-api` crate while keeping the local `DomainEvent` helper used by
//! some high-level tests.  All production code should rely on the contracts
//! defined in `toka-events-api`.

// Re-export the primitives so existing downstreams using `crate::events::*`
// continue to compile unchanged.
pub use toka_events_api::{causal_hash, create_event_header, CausalDigest, EventHeader, EventId, EventPayload, IntentId};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A standard set of high-level domain events that may occur across the Toka
/// platform.  This enum is **non-exhaustive** and intended mainly for demos and
/// tests.  Production systems are free to define their own specific payload
/// types implementing [`EventPayload`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

        let serialized = rmp_serde::to_vec_named(&event).unwrap();
        let deserialized: DomainEvent = rmp_serde::from_slice(&serialized).unwrap();

        assert_eq!(event, deserialized);
    }

    #[test]
    fn test_create_event_header() {
        let event = DomainEvent::System {
            component: "test".to_string(),
            payload: json!({"test": true}),
        };

        let header =
            create_event_header(&[], Uuid::nil(), "system.test".to_string(), &event).unwrap();

        assert_eq!(header.kind, "system.test");
        assert_eq!(header.parents.len(), 0);
    }
} 
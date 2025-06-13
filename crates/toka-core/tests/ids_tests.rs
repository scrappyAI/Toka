//! # IDs Module Tests
//!
//! This file contains tests for the `toka-core::ids` module, ensuring that
//! ID creation, parsing, serialization, and type safety are all working correctly.

use toka_core::ids::*;
use uuid::Uuid;

// --- Test Helpers ---

/// A helper macro to test the basics of a given ID type.
/// It checks for correct prefix, parsing, and serialization.
macro_rules! test_id_type {
    ($test_name:ident, $id_type:ty, $prefix:expr) => {
        #[test]
        fn $test_name() {
            // 1. Test creation and prefix
            let id = <$id_type>::new();
            let serialized = id.to_string();
            assert!(serialized.starts_with(concat!($prefix, "_")));

            // 2. Test parsing from string with prefix
            let reparsed: $id_type = serialized.parse().expect("Failed to parse prefixed ID");
            assert_eq!(id, reparsed);

            // 3. Test parsing from bare UUID string
            let bare_uuid = Uuid::new_v4();
            let bare_uuid_str = bare_uuid.to_string();
            let parsed_from_bare: $id_type = bare_uuid_str.parse().expect("Failed to parse bare UUID");
            assert_eq!(parsed_from_bare.as_uuid(), &bare_uuid);

            // 4. Test serialization/deserialization with serde
            let serialized_json = serde_json::to_string(&id).expect("Serialization failed");
            let deserialized: $id_type = serde_json::from_str(&serialized_json).expect("Deserialization failed");
            assert_eq!(id, deserialized);

            // 5. Check display format
            assert_eq!(serialized, format!("{}_{}", $prefix, id.as_uuid()));
        }
    };
}

// --- Test Cases for Each ID Type ---

test_id_type!(test_user_id, UserID, "user");
test_id_type!(test_agent_id, AgentID, "agnt");
test_id_type!(test_model_id, ModelID, "mdl");
test_id_type!(test_transaction_id, TransactionID, "txn");
test_id_type!(test_resource_id, ResourceID, "rsrc");
test_id_type!(test_product_id, ProductID, "prod");
test_id_type!(test_vault_id, VaultID, "vlt");


// --- Error Handling and Edge Cases ---

#[test]
fn test_parsing_error_invalid_prefix() {
    let id_str = "wrongprefix_123e4567-e89b-12d3-a456-426614174000";
    let result: Result<UserID, _> = id_str.parse();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid prefix: expected 'user', got 'wrongprefix'"));
}

#[test]
fn test_parsing_error_invalid_uuid() {
    let id_str = "user_not-a-valid-uuid";
    let result: Result<UserID, _> = id_str.parse();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid UUID part in ID"));
}

#[test]
fn test_parsing_error_bare_invalid_uuid() {
    let id_str = "not-a-valid-uuid";
    let result: Result<UserID, _> = id_str.parse();
    assert!(result.is_err());
    let error = result.unwrap_err();
    assert!(error.to_string().contains("Invalid UUID string"));
}

#[test]
fn test_from_and_into_uuid() {
    let original_uuid = Uuid::new_v4();
    
    // Test From<Uuid> for Id<T>
    let user_id = UserID::from(original_uuid);
    assert_eq!(user_id.as_uuid(), &original_uuid);

    // Test From<Id<T>> for Uuid
    let converted_uuid: Uuid = user_id.into();
    assert_eq!(converted_uuid, original_uuid);
}

#[test]
fn test_id_equality_and_hashing() {
    use std::collections::HashSet;

    let uuid1 = Uuid::new_v4();
    let uuid2 = Uuid::new_v4();

    let id1_a = UserID::from_uuid(uuid1);
    let id1_b = UserID::from_uuid(uuid1); // Same UUID, should be equal
    let id2 = UserID::from_uuid(uuid2);   // Different UUID

    assert_eq!(id1_a, id1_b);
    assert_ne!(id1_a, id2);

    let mut set = HashSet::new();
    set.insert(id1_a);
    
    // Should be true because id1_b is considered the same
    assert!(set.contains(&id1_b));
    // Should be false because id2 is different
    assert!(!set.contains(&id2));
} 
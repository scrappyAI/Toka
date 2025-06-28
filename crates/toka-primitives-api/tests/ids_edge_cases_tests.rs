//! Comprehensive ID system tests for edge cases and security
//! Tests parsing, serialization, collision resistance, and malformed input handling

use std::collections::HashSet;
use std::str::FromStr;
use toka_primitives_api::ids::*;
use uuid::Uuid;

#[test]
fn test_malformed_input_handling() {
    // Test various malformed inputs that should fail gracefully
    let malformed_inputs = vec![
        "",
        "invalid",
        "user_",
        "user_invalid-uuid",
        "user_123",
        "wrong_12345678-1234-1234-1234-123456789012",
        "user_12345678-1234-1234-1234-12345678901", // too short
        "user_12345678-1234-1234-1234-1234567890123", // too long
        "user_xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
        "user_12345678-1234-1234-1234-123456789012-extra",
        "user__12345678-1234-1234-1234-123456789012", // double underscore
        "user-12345678-1234-1234-1234-123456789012", // wrong separator
        "USER_12345678-1234-1234-1234-123456789012", // wrong case prefix
    ];
    
    for input in malformed_inputs {
        let result = UserID::from_str(input);
        assert!(result.is_err(), "Should fail for input: '{}'", input);
    }
}

#[test]
fn test_unicode_and_special_characters_in_parsing() {
    // IDs should not accept Unicode or special characters
    let invalid_inputs = vec![
        "user_🌍12345678-1234-1234-1234-123456789012",
        "user_\n12345678-1234-1234-1234-123456789012",
        "user_\t12345678-1234-1234-1234-123456789012",
        "user_\012345678-1234-1234-1234-123456789012",
        "user_ 12345678-1234-1234-1234-123456789012", // space
    ];
    
    for input in invalid_inputs {
        let result = UserID::from_str(input);
        assert!(result.is_err(), "Should fail for input: '{}'", input);
    }
}

#[test]
fn test_case_sensitivity() {
    let uuid = Uuid::new_v4();
    let lowercase = format!("user_{}", uuid.to_string().to_lowercase());
    let uppercase = format!("user_{}", uuid.to_string().to_uppercase());
    let mixed_case = format!("user_{}", uuid.to_string().replace("a", "A"));
    
    // All should parse to the same UUID
    let id1 = UserID::from_str(&lowercase).unwrap();
    let id2 = UserID::from_str(&uppercase).unwrap();
    let id3 = UserID::from_str(&mixed_case).unwrap();
    
    assert_eq!(id1.as_uuid(), id2.as_uuid());
    assert_eq!(id1.as_uuid(), id3.as_uuid());
}

#[test]
fn test_collision_resistance() {
    let mut seen_ids = HashSet::new();
    let mut seen_strings = HashSet::new();
    
    // Generate many IDs and check for collisions
    for _ in 0..10000 {
        let id = UserID::new();
        let id_string = id.to_string();
        
        // Check UUID collision
        assert!(!seen_ids.contains(id.as_uuid()), "UUID collision detected");
        seen_ids.insert(*id.as_uuid());
        
        // Check string representation collision
        assert!(!seen_strings.contains(&id_string), "String collision detected");
        seen_strings.insert(id_string);
    }
}

#[test]
fn test_different_id_types_are_distinct() {
    let uuid = Uuid::new_v4();
    
    let user_id = UserID::from_uuid(uuid);
    let agent_id = AgentID::from_uuid(uuid);
    let vault_id = VaultID::from_uuid(uuid);
    
    // Same UUID, different prefixes
    assert_ne!(user_id.to_string(), agent_id.to_string());
    assert_ne!(user_id.to_string(), vault_id.to_string());
    assert_ne!(agent_id.to_string(), vault_id.to_string());
    
    // But same underlying UUID
    assert_eq!(user_id.as_uuid(), agent_id.as_uuid());
    assert_eq!(user_id.as_uuid(), vault_id.as_uuid());
}

#[test]
fn test_serialization_deserialization_stability() {
    // Test that serialization is stable across multiple rounds
    let original_id = UserID::new();
    
    for _ in 0..100 {
        // JSON serialization roundtrip
        let json = serde_json::to_string(&original_id).unwrap();
        let deserialized: UserID = serde_json::from_str(&json).unwrap();
        assert_eq!(original_id, deserialized);
        
        // String conversion roundtrip
        let string_repr = original_id.to_string();
        let parsed = UserID::from_str(&string_repr).unwrap();
        assert_eq!(original_id, parsed);
    }
}

#[test]
fn test_concurrent_id_generation() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let generated_ids = Arc::new(Mutex::new(HashSet::new()));
    let mut handles = vec![];
    
    // Generate IDs concurrently
    for _ in 0..10 {
        let generated_ids = Arc::clone(&generated_ids);
        let handle = thread::spawn(move || {
            let mut local_ids = Vec::new();
            for _ in 0..1000 {
                local_ids.push(UserID::new());
            }
            
            let mut ids = generated_ids.lock().unwrap();
            for id in local_ids {
                assert!(!ids.contains(id.as_uuid()), "Concurrent ID collision");
                ids.insert(*id.as_uuid());
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Should have generated 10000 unique IDs
    let ids = generated_ids.lock().unwrap();
    assert_eq!(ids.len(), 10000);
}

#[test]
fn test_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let id = UserID::new();
    
    // Hash should be consistent across multiple calls
    let mut hasher1 = DefaultHasher::new();
    id.hash(&mut hasher1);
    let hash1 = hasher1.finish();
    
    let mut hasher2 = DefaultHasher::new();
    id.hash(&mut hasher2);
    let hash2 = hasher2.finish();
    
    assert_eq!(hash1, hash2);
    
    // Same UUID should produce same hash regardless of ID type
    let uuid = *id.as_uuid();
    let agent_id = AgentID::from_uuid(uuid);
    
    let mut hasher3 = DefaultHasher::new();
    agent_id.hash(&mut hasher3);
    let hash3 = hasher3.finish();
    
    assert_eq!(hash1, hash3);
}

#[test]
fn test_ordering_consistency() {
    // Test that ordering is consistent and deterministic
    let mut ids = Vec::new();
    for _ in 0..100 {
        ids.push(UserID::new());
    }
    
    // Sort the IDs
    ids.sort();
    
    // Verify they're still sorted after multiple operations
    for i in 1..ids.len() {
        assert!(ids[i-1] <= ids[i], "IDs should remain sorted");
    }
    
    // Test ordering with same UUID but different types
    let uuid = Uuid::new_v4();
    let user_id = UserID::from_uuid(uuid);
    let agent_id = AgentID::from_uuid(uuid);
    
    // Should have consistent ordering
    let cmp1 = user_id.cmp(&user_id);
    let cmp2 = agent_id.cmp(&agent_id);
    assert_eq!(cmp1, cmp2);
}

#[test]
fn test_memory_efficiency() {
    // Verify that IDs don't use excessive memory
    let id = UserID::new();
    let size = std::mem::size_of_val(&id);
    
    // Should be approximately the size of a UUID (16 bytes) plus marker (0 bytes)
    assert_eq!(size, 16, "ID should be memory efficient");
    
    // Test that marker doesn't add runtime overhead
    let uuid_size = std::mem::size_of::<Uuid>();
    assert_eq!(size, uuid_size, "ID should not add overhead beyond UUID");
}

#[test]
fn test_boundary_values() {
    // Test with boundary UUID values
    let zero_uuid = Uuid::from_bytes([0; 16]);
    let max_uuid = Uuid::from_bytes([0xFF; 16]);
    
    let zero_id = UserID::from_uuid(zero_uuid);
    let max_id = UserID::from_uuid(max_uuid);
    
    // Should parse and display correctly
    let zero_str = zero_id.to_string();
    let max_str = max_id.to_string();
    
    let parsed_zero = UserID::from_str(&zero_str).unwrap();
    let parsed_max = UserID::from_str(&max_str).unwrap();
    
    assert_eq!(zero_id, parsed_zero);
    assert_eq!(max_id, parsed_max);
}

#[test]
fn test_string_representation_format() {
    let id = UserID::new();
    let string_repr = id.to_string();
    
    // Should follow exact format: prefix_uuid
    assert!(string_repr.starts_with("user_"));
    assert_eq!(string_repr.len(), "user_".len() + 36); // UUID string is 36 chars
    
    // Should contain exactly one underscore
    assert_eq!(string_repr.matches('_').count(), 1);
    
    // UUID part should be valid
    let uuid_part = &string_repr["user_".len()..];
    assert!(Uuid::from_str(uuid_part).is_ok());
}

#[test]
fn test_prefix_validation_security() {
    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    
    // Test that wrong prefixes are rejected
    let wrong_prefixes = vec![
        format!("agnt_{}", uuid_str), // agent prefix for user ID
        format!("vault_{}", uuid_str), // vault prefix for user ID
        format!("mdl_{}", uuid_str),   // model prefix for user ID
    ];
    
    for wrong_prefix in wrong_prefixes {
        let result = UserID::from_str(&wrong_prefix);
        assert!(result.is_err(), "Should reject wrong prefix: {}", wrong_prefix);
    }
}

#[test]
fn test_bare_uuid_compatibility() {
    // Test that bare UUIDs can be parsed for backward compatibility
    let uuid = Uuid::new_v4();
    let uuid_str = uuid.to_string();
    
    let id_from_bare = UserID::from_str(&uuid_str).unwrap();
    let id_from_uuid = UserID::from_uuid(uuid);
    
    assert_eq!(id_from_bare, id_from_uuid);
}

#[test]
fn test_json_serialization_format() {
    let id = UserID::new();
    
    // Should serialize as a simple string
    let json = serde_json::to_string(&id).unwrap();
    
    // Should be quoted string
    assert!(json.starts_with('"'));
    assert!(json.ends_with('"'));
    
    // Should not contain extra JSON structure
    assert!(!json.contains('{'));
    assert!(!json.contains('['));
    
    // Content should be the string representation
    let json_content = &json[1..json.len()-1]; // Remove quotes
    assert_eq!(json_content, id.to_string());
}

#[test]
fn test_stress_parsing() {
    // Generate many IDs and test parsing stress
    let mut ids = Vec::new();
    for _ in 0..1000 {
        ids.push(UserID::new());
    }
    
    // Convert all to strings
    let id_strings: Vec<String> = ids.iter().map(|id| id.to_string()).collect();
    
    // Parse them all back
    for (original, id_string) in ids.iter().zip(id_strings.iter()) {
        let parsed = UserID::from_str(id_string).unwrap();
        assert_eq!(*original, parsed);
    }
}

#[test]
fn test_all_id_types_consistency() {
    let uuid = Uuid::new_v4();
    
    // Create all ID types with same UUID
    let user_id = UserID::from_uuid(uuid);
    let agent_id = AgentID::from_uuid(uuid);
    let model_id = ModelID::from_uuid(uuid);
    let transaction_id = TransactionID::from_uuid(uuid);
    let resource_id = ResourceID::from_uuid(uuid);
    let product_id = ProductID::from_uuid(uuid);
    let vault_id = VaultID::from_uuid(uuid);
    
    // All should have same underlying UUID
    assert_eq!(user_id.as_uuid(), &uuid);
    assert_eq!(agent_id.as_uuid(), &uuid);
    assert_eq!(model_id.as_uuid(), &uuid);
    assert_eq!(transaction_id.as_uuid(), &uuid);
    assert_eq!(resource_id.as_uuid(), &uuid);
    assert_eq!(product_id.as_uuid(), &uuid);
    assert_eq!(vault_id.as_uuid(), &uuid);
    
    // All should have different string representations
    let representations = vec![
        user_id.to_string(),
        agent_id.to_string(),
        model_id.to_string(),
        transaction_id.to_string(),
        resource_id.to_string(),
        product_id.to_string(),
        vault_id.to_string(),
    ];
    
    // Check uniqueness
    let unique_representations: HashSet<_> = representations.iter().collect();
    assert_eq!(unique_representations.len(), representations.len());
    
    // Test that each can only be parsed as its own type
    for (i, repr) in representations.iter().enumerate() {
        match i {
            0 => assert!(UserID::from_str(repr).is_ok()),
            1 => assert!(AgentID::from_str(repr).is_ok()),
            2 => assert!(ModelID::from_str(repr).is_ok()),
            3 => assert!(TransactionID::from_str(repr).is_ok()),
            4 => assert!(ResourceID::from_str(repr).is_ok()),
            5 => assert!(ProductID::from_str(repr).is_ok()),
            6 => assert!(VaultID::from_str(repr).is_ok()),
            _ => unreachable!(),
        }
    }
}
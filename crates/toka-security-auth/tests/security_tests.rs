//! Security-focused tests for CapabilityToken
//! These tests focus on potential vulnerabilities and edge cases

use std::time::{SystemTime, UNIX_EPOCH};
use toka_security_auth::CapabilityToken;

const TEST_SECRET: &str = "test_secret_key_256_bits_long_for_testing";

#[test]
fn test_token_timing_attack_resistance() {
    // Test that signature verification time doesn't leak information
    let secret = TEST_SECRET;
    let token = CapabilityToken::new("user", "vault", vec!["read".into()], secret, 3600);
    
    // Valid signature
    let start = std::time::Instant::now();
    assert!(token.is_valid(secret));
    let valid_time = start.elapsed();
    
    // Invalid signature (completely wrong)
    let mut invalid_token = token.clone();
    invalid_token.signature = "completely_wrong_signature".to_string();
    let start = std::time::Instant::now();
    assert!(!invalid_token.is_valid(secret));
    let invalid_time = start.elapsed();
    
    // The times should be similar (within reasonable bounds for timing attack resistance)
    // This is a basic check - in practice, constant-time comparison would be better
    let time_diff = if valid_time > invalid_time {
        valid_time - invalid_time
    } else {
        invalid_time - valid_time
    };
    
    // Allow for some variance but should be within same order of magnitude
    assert!(time_diff.as_nanos() < 1_000_000); // 1ms tolerance
}

#[test]
fn test_token_replay_attack_prevention() {
    let secret = TEST_SECRET;
    
    // Create token with short TTL
    let token = CapabilityToken::new("user", "vault", vec!["read".into()], secret, 1);
    
    // Token should initially be valid
    assert!(token.is_valid(secret));
    
    // Wait for expiration (this is a fast test, so we simulate)
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Token should now be expired and invalid (preventing replay)
    assert!(!token.is_valid(secret));
}

#[test]
fn test_malicious_input_handling() {
    let secret = TEST_SECRET;
    
    // Test with extremely long inputs
    let long_subject = "a".repeat(10000);
    let long_vault_id = "b".repeat(10000);
    let long_permissions = vec!["c".repeat(10000)];
    
    // Should handle large inputs without panicking
    let token = CapabilityToken::new(&long_subject, &long_vault_id, long_permissions, secret, 3600);
    assert!(token.is_valid(secret));
    
    // Test with empty inputs
    let empty_token = CapabilityToken::new("", "", vec![], secret, 3600);
    assert!(empty_token.is_valid(secret));
    
    // Test with special characters and Unicode
    let special_subject = "user-123_@#$%^&*()";
    let unicode_vault = "vault-🔐-test";
    let special_permissions = vec!["read/write".into(), "admin*".into()];
    
    let special_token = CapabilityToken::new(special_subject, unicode_vault, special_permissions, secret, 3600);
    assert!(special_token.is_valid(secret));
}

#[test]
fn test_secret_key_sensitivity() {
    let secret1 = "secret1";
    let secret2 = "secret2";
    
    let token = CapabilityToken::new("user", "vault", vec!["read".into()], secret1, 3600);
    
    // Token should only be valid with the correct secret
    assert!(token.is_valid(secret1));
    assert!(!token.is_valid(secret2));
    
    // Even slight modifications to secret should invalidate
    assert!(!token.is_valid("secret1 ")); // trailing space
    assert!(!token.is_valid("Secret1")); // case change
    assert!(!token.is_valid("secret11")); // extra character
}

#[test]
fn test_permission_tampering_detection() {
    let secret = TEST_SECRET;
    let mut token = CapabilityToken::new(
        "user", 
        "vault", 
        vec!["read".into()], 
        secret, 
        3600
    );
    
    // Original token should be valid
    assert!(token.is_valid(secret));
    
    // Modify permissions without updating signature
    token.permissions.push("admin".into());
    
    // Token should now be invalid due to tampered permissions
    assert!(!token.is_valid(secret));
}

#[test]
fn test_time_manipulation_resistance() {
    let secret = TEST_SECRET;
    let mut token = CapabilityToken::new("user", "vault", vec!["read".into()], secret, 3600);
    
    // Store original times
    let original_issued = token.issued_at;
    let original_expires = token.expires_at;
    
    // Try to extend validity by manipulating times
    token.expires_at = original_expires + 7200; // Add 2 hours
    
    // Token should be invalid due to signature mismatch
    assert!(!token.is_valid(secret));
    
    // Try to backdate issued time
    token.issued_at = original_issued - 3600; // Subtract 1 hour
    token.expires_at = original_expires; // Reset expiry
    
    // Still should be invalid
    assert!(!token.is_valid(secret));
}

#[test]
fn test_zero_ttl_edge_case() {
    let secret = TEST_SECRET;
    
    // Token with zero TTL should immediately expire
    let token = CapabilityToken::new("user", "vault", vec!["read".into()], secret, 0);
    
    // Might be valid for a split second, but let's check it's handled properly
    // The main point is that it doesn't panic or cause undefined behavior
    let _ = token.is_valid(secret);
    
    // After any delay, should definitely be invalid
    std::thread::sleep(std::time::Duration::from_millis(1));
    assert!(!token.is_valid(secret));
}

#[test]
fn test_signature_collision_resistance() {
    let secret = TEST_SECRET;
    
    // Create two tokens with different data
    let token1 = CapabilityToken::new("user1", "vault1", vec!["read".into()], secret, 3600);
    let token2 = CapabilityToken::new("user2", "vault2", vec!["write".into()], secret, 3600);
    
    // Signatures should be different
    assert_ne!(token1.signature, token2.signature);
    
    // Each token should only validate with its own data
    assert!(token1.is_valid(secret));
    assert!(token2.is_valid(secret));
    
    // Cross-validation should fail
    let mut cross_token = token1.clone();
    cross_token.signature = token2.signature.clone();
    assert!(!cross_token.is_valid(secret));
}

#[test]
fn test_concurrent_token_validation() {
    use std::sync::Arc;
    use std::thread;
    
    let secret = Arc::new(TEST_SECRET.to_string());
    let token = Arc::new(CapabilityToken::new("user", "vault", vec!["read".into()], &secret, 3600));
    
    // Spawn multiple threads to validate the same token concurrently
    let handles: Vec<_> = (0..10).map(|_| {
        let token = Arc::clone(&token);
        let secret = Arc::clone(&secret);
        thread::spawn(move || {
            for _ in 0..100 {
                assert!(token.is_valid(&secret));
            }
        })
    }).collect();
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread should complete successfully");
    }
}

#[test]
fn test_integer_overflow_resistance() {
    let secret = TEST_SECRET;
    
    // Create token with maximum timestamp values
    let mut token = CapabilityToken::new("user", "vault", vec!["read".into()], secret, 3600);
    
    // Test with extreme timestamp values
    token.issued_at = u64::MAX - 1;
    token.expires_at = u64::MAX;
    token.signature = token.compute_signature(secret);
    
    // Should handle large values without overflow
    let _ = token.is_valid(secret);
    
    // Test with zero values
    token.issued_at = 0;
    token.expires_at = 0;
    token.signature = token.compute_signature(secret);
    
    let _ = token.is_valid(secret);
}
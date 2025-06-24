//! Security-oriented tests for the new JWT-based `CapabilityToken` implementation.

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use toka_security_auth::CapabilityToken;

const SECRET: &str = "test_secret_key_256_bits_long_for_testing";

#[test]
fn timing_attack_resistance() {
    let token = CapabilityToken::new("user", "vault", vec!["read".into()], SECRET, 3600).unwrap();

    let start_valid = Instant::now();
    assert!(token.is_valid(SECRET));
    let valid_duration = start_valid.elapsed();

    let start_invalid = Instant::now();
    assert!(!token.is_valid("wrong_secret"));
    let invalid_duration = start_invalid.elapsed();

    let diff = if valid_duration > invalid_duration {
        valid_duration - invalid_duration
    } else {
        invalid_duration - valid_duration
    };
    // Allow small variance (< 2 ms)
    assert!(diff < Duration::from_millis(2));
}

#[test]
fn replay_attack_prevention() {
    let token = CapabilityToken::new("user", "vault", vec!["read".into()], SECRET, 1).unwrap();
    assert!(token.is_valid(SECRET));
    thread::sleep(Duration::from_secs(2));
    assert!(!token.is_valid(SECRET));
}

#[test]
fn large_input_handling() {
    let long_subject = "a".repeat(10_000);
    let long_vault = "b".repeat(10_000);
    let long_perm = vec!["c".repeat(10_000)];
    let token = CapabilityToken::new(&long_subject, &long_vault, long_perm, SECRET, 3600).unwrap();
    assert!(token.is_valid(SECRET));
}

#[test]
fn concurrent_validation() {
    let secret = Arc::new(SECRET.to_string());
    let token = Arc::new(CapabilityToken::new("user", "vault", vec!["read".into()], secret.as_str(), 3600).unwrap());

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let t = Arc::clone(&token);
            let s = Arc::clone(&secret);
            thread::spawn(move || {
                for _ in 0..100 {
                    assert!(t.is_valid(s.as_str()));
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }
}
use std::thread;
use std::time::Duration;
use toka_security_auth::CapabilityToken;

const SECRET: &str = "super_secret_key";

#[test]
fn token_roundtrip() {
    let token = CapabilityToken::new("alice", "vault1", vec!["read".into()], SECRET, 3600).unwrap();
    assert!(token.is_valid(SECRET));
}

#[test]
fn invalid_secret_fails() {
    let token = CapabilityToken::new("bob", "vault2", vec!["write".into()], SECRET, 3600).unwrap();
    assert!(!token.is_valid("incorrect_secret"));
}

#[test]
fn token_expires() {
    let token = CapabilityToken::new("carol", "vault3", vec!["read".into()], SECRET, 1).unwrap();
    assert!(token.is_valid(SECRET));
    // Wait just over a second to ensure expiry has passed.
    thread::sleep(Duration::from_millis(1100));
    assert!(!token.is_valid(SECRET));
}

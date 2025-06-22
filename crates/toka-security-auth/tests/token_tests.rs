use std::env;
use toka_security_auth::CapabilityToken;

const FALLBACK_SECRET: &str = "super_secret_key";

fn get_secret() -> String {
    env::var("TOKA_TEST_SECRET").unwrap_or_else(|_| FALLBACK_SECRET.to_string())
}

#[test]
fn test_token_creation_and_signature() {
    let secret = get_secret();
    let token = CapabilityToken::new(
        "alice",
        "vault1",
        vec!["read".into()],
        &secret,
        3600,
    );

    assert!(token.is_valid(&secret));
}

#[test]
fn test_token_signature_tampering() {
    let secret = get_secret();
    let mut token =
        CapabilityToken::new("user123", "vaultABC", vec!["read".to_string()], &secret, 60);

    // Tamper with the signature
    token.signature = "invalidsignature".into();

    assert!(!token.is_valid(&secret));
}

#[test]
fn test_token_expiration() {
    let secret = get_secret();
    let mut token =
        CapabilityToken::new("user123", "vaultABC", vec!["read".to_string()], &secret, 60);

    // Manually expire the token
    token.expires_at = token.issued_at.saturating_sub(1);
    token.signature = token.compute_signature(&secret);

    assert!(!token.is_valid(&secret));
}

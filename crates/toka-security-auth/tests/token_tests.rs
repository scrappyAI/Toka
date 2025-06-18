use toka_security_auth::CapabilityToken;

const SECRET: &str = "super_secret_key";

#[test]
fn test_token_creation_and_signature() {
    let token = CapabilityToken::new(
        "user123",
        "vaultABC",
        vec!["read".to_string(), "write".to_string()],
        SECRET,
        60, // 1-minute TTL
    );

    // Signature should validate immediately after creation
    assert!(token.is_valid(SECRET));
}

#[test]
fn test_token_signature_tampering() {
    let mut token =
        CapabilityToken::new("user123", "vaultABC", vec!["read".to_string()], SECRET, 60);

    // Tamper with the signature
    token.signature = "invalidsignature".into();

    assert!(!token.is_valid(SECRET));
}

#[test]
fn test_token_expiration() {
    let mut token =
        CapabilityToken::new("user123", "vaultABC", vec!["read".to_string()], SECRET, 60);

    // Force expiration by setting expires_at in the past and recomputing signature
    token.expires_at = token.issued_at.saturating_sub(1);
    token.signature = token.compute_signature(SECRET);

    assert!(!token.is_valid(SECRET));
}

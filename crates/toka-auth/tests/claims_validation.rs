use toka_auth::{Claims, Error, MAX_TOKEN_LIFETIME_SECS, MAX_PERMISSIONS_COUNT};

#[test]
fn test_claims_validation_success() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string(), "write".to_string()],
        iat: now,
        exp: now + 3600, // 1 hour later
        jti: "test-token-id".to_string(),
    };
    
    assert!(claims.validate().is_ok());
}

#[test]
fn test_claims_validation_empty_subject() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Subject identifier cannot be empty"));
}

#[test]
fn test_claims_validation_whitespace_subject() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "   ".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Subject identifier cannot be empty"));
}

#[test]
fn test_claims_validation_long_subject() {
    let now = chrono::Utc::now().timestamp() as u64;
    let long_subject = "x".repeat(257); // 256 + 1
    let claims = Claims {
        sub: long_subject,
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Subject identifier too long"));
}

#[test]
fn test_claims_validation_empty_vault() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Vault identifier cannot be empty"));
}

#[test]
fn test_claims_validation_long_vault() {
    let now = chrono::Utc::now().timestamp() as u64;
    let long_vault = "x".repeat(257); // 256 + 1
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: long_vault,
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Vault identifier too long"));
}

#[test]
fn test_claims_validation_too_many_permissions() {
    let now = chrono::Utc::now().timestamp() as u64;
    let permissions: Vec<String> = (0..MAX_PERMISSIONS_COUNT + 1)
        .map(|i| format!("permission-{}", i))
        .collect();
    
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions,
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Too many permissions in token"));
}

#[test]
fn test_claims_validation_empty_permission() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string(), "".to_string(), "write".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Permission cannot be empty"));
}

#[test]
fn test_claims_validation_long_permission() {
    let now = chrono::Utc::now().timestamp() as u64;
    let long_permission = "x".repeat(65); // 64 + 1
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string(), long_permission, "write".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Permission name too long"));
}

#[test]
fn test_claims_validation_expiry_before_issuance() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now - 1, // Expiry before issuance
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Token expiry must be after issuance"));
}

#[test]
fn test_claims_validation_same_issuance_and_expiry() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now, // Same as issuance
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Token expiry must be after issuance"));
}

#[test]
fn test_claims_validation_lifetime_too_long() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + MAX_TOKEN_LIFETIME_SECS + 1, // Exceeds max lifetime
        jti: "test-token-id".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Token lifetime exceeds maximum allowed"));
}

#[test]
fn test_claims_validation_empty_jti() {
    let now = chrono::Utc::now().timestamp() as u64;
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600,
        jti: "".to_string(),
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Token ID (jti) cannot be empty"));
}

#[test]
fn test_claims_validation_long_jti() {
    let now = chrono::Utc::now().timestamp() as u64;
    let long_jti = "x".repeat(257); // 256 + 1
    let claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600,
        jti: long_jti,
    };
    
    let result = claims.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Token ID (jti) too long"));
}

#[test]
fn test_claims_is_expired() {
    let now = chrono::Utc::now().timestamp() as u64;
    
    // Expired token
    let expired_claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now - 3600,
        exp: now - 1, // Expired
        jti: "test-token-id".to_string(),
    };
    assert!(expired_claims.is_expired());
    
    // Valid token
    let valid_claims = Claims {
        sub: "test-user".to_string(),
        vault: "test-vault".to_string(),
        permissions: vec!["read".to_string()],
        iat: now,
        exp: now + 3600, // Valid for 1 hour
        jti: "test-token-id".to_string(),
    };
    assert!(!valid_claims.is_expired());
}

#[test]
fn test_claims_boundary_values() {
    let now = chrono::Utc::now().timestamp() as u64;
    
    // Test boundary values for string lengths
    let max_subject = "x".repeat(256);
    let max_vault = "x".repeat(256);
    let max_jti = "x".repeat(256);
    let max_permission = "x".repeat(64);
    
    let claims = Claims {
        sub: max_subject,
        vault: max_vault,
        permissions: vec![max_permission],
        iat: now,
        exp: now + MAX_TOKEN_LIFETIME_SECS, // Exactly at max lifetime
        jti: max_jti,
    };
    
    // Should be valid at boundaries
    assert!(claims.validate().is_ok());
} 
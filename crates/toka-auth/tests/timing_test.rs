use toka_auth::{Claims, CapabilityToken, TokenValidator};
use toka_auth::hs256::{JwtHs256Token, JwtHs256Validator};

#[tokio::test]
async fn test_token_expiry_validation() {
    let secret = "test_secret";
    let now = chrono::Utc::now().timestamp() as u64;
    
    // Test with expired token (-1 second)
    let exp = (now as i64 - 1) as u64;
    let claims = Claims {
        sub: "tester".into(),
        vault: "demo".into(),
        permissions: vec![],
        iat: now,
        exp,
        jti: "test".into(),
    };
    
    let token = JwtHs256Token::mint(&claims, secret.as_bytes()).await.unwrap();
    let validator = JwtHs256Validator::new(secret);
    
    // Expired token should be rejected
    let result = validator.validate(token.as_str()).await;
    assert!(result.is_err());
    
    // Test with more expired token (-10 seconds)
    let exp2 = (now as i64 - 10) as u64;
    let claims2 = Claims {
        sub: "tester".into(),
        vault: "demo".into(), 
        permissions: vec![],
        iat: now,
        exp: exp2,
        jti: "test2".into(),
    };
    
    let token2 = JwtHs256Token::mint(&claims2, secret.as_bytes()).await.unwrap();
    let result2 = validator.validate(token2.as_str()).await;
    assert!(result2.is_err());
}

#[tokio::test]
async fn test_valid_token_validation() {
    let secret = "test_secret";
    let now = chrono::Utc::now().timestamp() as u64;
    
    // Test with valid token (+1 hour)
    let exp = now + 3600;
    let claims = Claims {
        sub: "tester".into(),
        vault: "demo".into(),
        permissions: vec!["read".into(), "write".into()],
        iat: now,
        exp,
        jti: "test".into(),
    };
    
    let token = JwtHs256Token::mint(&claims, secret.as_bytes()).await.unwrap();
    let validator = JwtHs256Validator::new(secret);
    
    // Valid token should be accepted
    let result = validator.validate(token.as_str()).await;
    assert!(result.is_ok());
    
    let validated_claims = result.unwrap();
    assert_eq!(validated_claims.sub, "tester");
    assert_eq!(validated_claims.vault, "demo");
    assert_eq!(validated_claims.permissions, vec!["read", "write"]);
}

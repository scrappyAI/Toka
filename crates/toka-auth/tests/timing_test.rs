use tokio_test;
use toka_auth::{Claims, CapabilityToken, TokenValidator};
use toka_auth::hs256::{JwtHs256Token, JwtHs256Validator};

#[tokio::main]
async fn main() {
    let secret = "test_secret";
    let now = chrono::Utc::now().timestamp() as u64;
    
    println!("Current timestamp: {}", now);
    
    // Test with -1 second (the failing case)
    let exp = (now as i64 - 1) as u64;
    println!("Expiry timestamp: {}", exp);
    println!("Token expired: {}", exp < now);
    
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
    
    println!("Token: {}", token.as_str());
    
    let result = validator.validate(token.as_str()).await;
    println!("Validation result: {:?}", result);
    
    // Also test with -10 seconds to be sure
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
    println!("Validation result for -10s: {:?}", result2);
}

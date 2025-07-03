use toka_auth::hs256::{build_claims, JwtHs256Token, JwtHs256Validator};

#[tokio::test]
async fn test_jwt_hs256_mint_and_validate() {
    let secret = "super_secret_key";
    let claims = build_claims("agent-1", "vault-a", vec!["read".into(), "write".into()], 3600).unwrap();

    let token = JwtHs256Token::mint(&claims, secret.as_bytes()).await.unwrap();
    let validator = JwtHs256Validator::new(secret);

    let validated = validator.validate(token.as_str()).await.unwrap();
    assert_eq!(validated.sub, claims.sub);
    assert_eq!(validated.vault, claims.vault);
    assert_eq!(validated.permissions, claims.permissions);
}

#[tokio::test]
async fn test_jwt_hs256_invalid_token_is_rejected() {
    let validator = JwtHs256Validator::new("some_secret");
    assert!(validator.validate("not.a.valid.jwt").await.is_err());
}
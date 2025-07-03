use proptest::prelude::*;
use toka_auth::{Claims};
use toka_auth::hs256::{JwtHs256Token, JwtHs256Validator};

proptest! {
    #[test]
    fn jwt_expiry_respected(delta_secs in -3600i64..3600i64) {
        let rt = tokio_test::block_on(async {
            let secret = "prop_secret";
            let now = chrono::Utc::now().timestamp() as u64;
            let exp = (now as i64 + delta_secs) as u64;
            let claims = Claims {
                sub: "tester".into(),
                vault: "demo".into(),
                permissions: vec![],
                iat: now,
                exp,
                jti: "prop-test".into(),
            };
            let token = JwtHs256Token::mint(&claims, secret.as_bytes()).await.unwrap();
            let validator = JwtHs256Validator::new(secret);
            (validator, token)
        });
        let validator = rt.0;
        let token = rt.1;
        let res = tokio_test::block_on(validator.validate(token.as_str()));
        if delta_secs >= 0 {
            prop_assert!(res.is_ok());
        } else {
            prop_assert!(res.is_err());
        }
    }
}
use std::str::FromStr;
use toka_primitives::ids::UserID;

#[test]
fn user_id_string_roundtrip() {
    let id = UserID::new();
    let s = id.to_string();
    assert!(s.starts_with("user_"));
    let parsed = UserID::from_str(&s).expect("parse");
    assert_eq!(id.as_uuid(), parsed.as_uuid());
}

#[test]
fn user_id_from_bare_uuid() {
    let id1 = UserID::new();
    let uuid_str = id1.as_uuid().to_string();
    let parsed = UserID::from_str(&uuid_str).expect("parse bare uuid");
    assert_eq!(id1.as_uuid(), parsed.as_uuid());
}

use toka_types::{AgentSpec, EntityId, Operation};

#[test]
fn test_operation_serde_roundtrip() {
    let original = Operation::SpawnSubAgent {
        parent: EntityId(1),
        spec: AgentSpec {
            name: "child-agent".into(),
        },
    };

    let json = serde_json::to_string(&original).expect("serialization failed");
    let decoded: Operation = serde_json::from_str(&json).expect("deserialization failed");

    assert_eq!(original, decoded);
}
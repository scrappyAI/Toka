//! Tests for the unified Vault implementation.

use toka_vault::prelude::*;
use toka_vault::Vault;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct TestPayload {
    id: u32,
    msg: String,
}

async fn run_vault_tests(vault: Vault) {
    // 1. Test basic commit and retrieve
    let payload1 = TestPayload {
        id: 1,
        msg: "hello".to_string(),
    };
    let header1 = vault
        .commit(&payload1, &[], "test.one", &[])
        .await
        .unwrap();

    // Verify header fields
    assert_eq!(header1.kind, "test.one");
    assert_eq!(header1.parents.len(), 0);

    // Retrieve and verify
    let retrieved_header = vault.get_header(&header1.id).await.unwrap().unwrap();
    assert_eq!(retrieved_header, header1);

    let retrieved_payload: TestPayload = vault
        .get_payload(&header1.digest)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(retrieved_payload, payload1);

    // 2. Test causal relationships
    let payload2 = TestPayload {
        id: 2,
        msg: "world".to_string(),
    };
    let header2 = vault
        .commit(&payload2, &[header1.clone()], "test.two", &[])
        .await
        .unwrap();

    assert_eq!(header2.parents.len(), 1);
    assert_eq!(header2.parents[0], header1.id);
    assert_ne!(header2.digest, header1.digest);

    // 3. Test subscription
    let mut sub = vault.subscribe();

    let payload3 = TestPayload { id: 3, msg: "sub".to_string() };
    let header3 = vault.commit(&payload3, &[], "test.three", &[]).await.unwrap();

    let received = sub.recv().await.unwrap();
    assert_eq!(received, header3);
}

#[cfg(feature = "memory-vault")]
#[tokio::test]
async fn test_memory_vault() {
    let vault = Vault::new_memory();
    run_vault_tests(vault).await;
}

#[cfg(feature = "persist-sled")]
#[tokio::test]
async fn test_persistent_vault() {
    let temp_dir = tempdir().unwrap();
    let vault = Vault::open_persistent(temp_dir.path().to_str().unwrap()).unwrap();
    run_vault_tests(vault).await;
} 
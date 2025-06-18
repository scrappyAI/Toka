use toka_security_vault::Vault;
use anyhow::Result;
use std::path::PathBuf;

/// Generate a unique path inside the system temp directory.
fn tmp_vault_path() -> PathBuf {
    let mut dir = std::env::temp_dir();
    let unique = format!("toka_vault_test_{}", rand::random::<u64>());
    dir.push(unique);
    dir
}

#[tokio::test]
async fn test_basic_crud_flow() -> Result<()> {
    let path = tmp_vault_path();

    let vault = Vault::new(path.to_str().unwrap())?;

    // --- Create & Insert ---------------------------------------------------
    let entry = Vault::create_entry("my_key", "secret-data");
    vault.insert(&entry).await?;

    // --- Read --------------------------------------------------------------
    let retrieved = vault.get("my_key").await?.expect("entry missing");
    assert_eq!(retrieved.data, "secret-data");
    assert_eq!(retrieved.metadata.version, 1);

    // --- Update ------------------------------------------------------------
    vault.update_entry("my_key", "new-data").await?;
    let updated = vault.get("my_key").await?.expect("entry missing");
    assert_eq!(updated.data, "new-data");
    assert_eq!(updated.metadata.version, 2);

    // --- List --------------------------------------------------------------
    let keys = vault.list().await?;
    assert_eq!(keys, vec!["my_key".to_string()]);

    // --- Remove ------------------------------------------------------------
    vault.remove("my_key").await?;
    assert!(vault.get("my_key").await?.is_none());

    Ok(())
} 
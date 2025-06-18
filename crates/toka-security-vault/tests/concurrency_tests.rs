use anyhow::Result;
use futures::future::join_all;
use tempfile::tempdir;
use toka_security_vault::Vault;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn concurrent_inserts_are_visible() -> Result<()> {
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;

    // Spawn 100 concurrent insert operations
    let futures = (0..100).map(|i| {
        let vault = &vault;
        async move {
            let key = format!("k{}", i);
            let entry = Vault::create_entry(&key, "payload");
            vault.insert(&entry).await
        }
    });
    join_all(futures)
        .await
        .into_iter()
        .collect::<Result<()>>()?;

    // Verify list length matches
    let keys = vault.list().await?;
    assert_eq!(keys.len(), 100);
    Ok(())
}

use anyhow::Result;
use async_trait::async_trait;
use std::sync::Arc;
use crate::{Vault, VaultEntry};
use toka_storage::StorageAdapter;
use base64::Engine;

/// Adapter that exposes a Vault as a generic `StorageAdapter`.
///
/// All keys are stored under a common namespace (`blob://`). The plain URI is
/// used as the vault entry key.  Payloads are stored as base64 in the `data`
/// field, leaving metadata encryption & persistence to the Vault.
pub struct VaultBlobAdapter {
    vault: Arc<Vault>,
}

impl VaultBlobAdapter {
    pub fn new(vault: Arc<Vault>) -> Self { Self { vault } }
}

#[async_trait]
impl StorageAdapter for VaultBlobAdapter {
    async fn put(&self, uri: &str, bytes: &[u8]) -> Result<()> {
        let key = format!("blob://{}", uri);
        let entry = VaultEntry {
            key,
            data: base64::engine::general_purpose::STANDARD.encode(bytes),
            metadata: crate::VaultMetadata { created_at: 0, updated_at: 0, version: 1 },
        };
        self.vault.insert(&entry).await
    }

    async fn get(&self, uri: &str) -> Result<Vec<u8>> {
        let key = format!("blob://{}", uri);
        if let Some(entry) = self.vault.get(&key).await? {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(entry.data.as_bytes())
                .map_err(|e| anyhow::anyhow!(e))?;
            Ok(bytes)
        } else {
            Err(anyhow::anyhow!("Blob not found"))
        }
    }

    async fn delete(&self, uri: &str) -> Result<()> {
        let key = format!("blob://{}", uri);
        self.vault.remove(&key).await
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        // Prefix listing by scanning vault keys; simple implementation.
        let entries = self.vault.list().await?;
        Ok(entries
            .into_iter()
            .filter(|k| k.starts_with(&format!("blob://{}", prefix)))
            .collect())
    }
} 
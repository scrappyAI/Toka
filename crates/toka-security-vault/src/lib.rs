//! Toka Vault Core
//!
//! Lightweight, AES-GCM–encrypted key⁄value store built on `sled`.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use rand::{rngs::OsRng, Rng};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::Path;
use std::sync::Arc;

pub mod prelude;

pub use prelude::*; // convenient

/// Represents a vault entry with metadata and data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VaultEntry {
    pub key: String,
    pub data: String,
    pub metadata: VaultMetadata,
}

/// Metadata for vault entries
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VaultMetadata {
    pub created_at: u64,
    pub updated_at: u64,
    pub version: u32,
}

/// Secure key/value vault
pub struct Vault {
    #[allow(dead_code)]
    path: String,
    db: Arc<Db>,
    cipher: Arc<Aes256Gcm>,
}

impl Vault {
    /// Create (or open) a vault at the given path
    pub fn new(path: &str) -> Result<Self> {
        let path = Path::new(path);
        std::fs::create_dir_all(path)
            .with_context(|| format!("Failed to create vault directory: {}", path.display()))?;

        let db = sled::open(path)
            .with_context(|| format!("Failed to open vault database at {}", path.display()))?;

        let key = Self::get_or_create_key(&db)?;
        let cipher = Aes256Gcm::new(&key);

        Ok(Self {
            path: path.to_string_lossy().into_owned(),
            db: Arc::new(db),
            cipher: Arc::new(cipher),
        })
    }

    const KEY_TREE: &'static str = "__keys";
    const ENCRYPTION_KEY: &'static [u8] = b"encryption_key";

    fn get_or_create_key(db: &Db) -> Result<Key<Aes256Gcm>> {
        let tree = db.open_tree(Self::KEY_TREE)?;
        if let Some(key_data) = tree.get(Self::ENCRYPTION_KEY)? {
            let key_bytes = BASE64
                .decode(key_data.as_ref())
                .context("Failed to decode stored encryption key")?;
            Ok(Key::<Aes256Gcm>::from_slice(&key_bytes).clone())
        } else {
            let mut key_bytes = [0u8; 32];
            OsRng.fill(&mut key_bytes);
            let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
            let encoded = BASE64.encode(key_bytes);
            tree.insert(Self::ENCRYPTION_KEY, encoded.as_bytes())?;
            Ok(key.clone())
        }
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, data)
            .map_err(|e| anyhow::anyhow!("Failed to encrypt data: {:?}", e))?;
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            return Err(anyhow::anyhow!("Invalid encrypted data length"));
        }
        let (nonce_bytes, ciphertext) = data.split_at(12);
        let nonce = Nonce::from_slice(nonce_bytes);
        self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Failed to decrypt data: {:?}", e))
    }

    /// Insert or update an entry
    pub async fn insert(&self, entry: &VaultEntry) -> Result<()> {
        let serialized = serde_json::to_vec(entry).context("Failed to serialize vault entry")?;
        let encrypted = self.encrypt(&serialized)?;
        self.db.insert(entry.key.as_bytes(), encrypted)?;
        Ok(())
    }

    /// Retrieve an entry by key
    pub async fn get(&self, key: &str) -> Result<Option<VaultEntry>> {
        if let Some(encrypted) = self.db.get(key.as_bytes())? {
            let decrypted = self.decrypt(&encrypted)?;
            let entry =
                serde_json::from_slice(&decrypted).context("Failed to deserialize entry")?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    /// List user (non-internal) keys
    pub async fn list(&self) -> Result<Vec<String>> {
        Ok(self
            .db
            .iter()
            .filter_map(|res| res.ok())
            .filter_map(|(k, _)| String::from_utf8(k.to_vec()).ok())
            .filter(|k| !k.starts_with("__"))
            .collect())
    }

    pub async fn remove(&self, key: &str) -> Result<()> {
        self.db.remove(key.as_bytes())?;
        Ok(())
    }

    pub fn create_entry(key: &str, data: &str) -> VaultEntry {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        VaultEntry {
            key: key.to_string(),
            data: data.to_string(),
            metadata: VaultMetadata {
                created_at: now,
                updated_at: now,
                version: 1,
            },
        }
    }

    pub async fn update_entry(&self, key: &str, data: &str) -> Result<()> {
        let mut entry = self
            .get(key)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Entry not found"))?;
        entry.data = data.to_string();
        entry.metadata.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        entry.metadata.version += 1;
        self.insert(&entry).await
    }
}

impl Drop for Vault {
    fn drop(&mut self) {
        let _ = self.db.flush();
    }
}

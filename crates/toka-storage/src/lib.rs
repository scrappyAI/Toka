//! Minimal storage abstraction used by `toka-runtime`.
//!
//! This *placeholder* implementation exists to keep the workspace compiling
//! after Slice-5 refactors.  It will be replaced by a fully-fledged storage
//! crate in a later slice.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Extremely lightweight artefact store (blocking API kept async for symmetry).
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    /// Store raw bytes under `key` (opaque path segment).
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<()>;
    /// Retrieve bytes previously stored under `key`.
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    /// Remove the file at `key` – returns `Ok(false)` if not found.
    async fn delete(&self, key: &str) -> Result<()>;
}

/// Local-filesystem implementation used by the runtime until a richer backend
/// lands.  *Not* optimised and ignores concurrency holes.
#[derive(Debug, Clone)]
pub struct LocalFsAdapter {
    root: PathBuf,
}

impl LocalFsAdapter {
    /// Initialise a new adapter rooted at `root` (created if absent).
    pub fn new(root: &str) -> Result<Self> {
        let path = PathBuf::from(root);
        std::fs::create_dir_all(&path)?;
        Ok(Self { root: path })
    }

    fn path_for(&self, key: &str) -> PathBuf {
        self.root.join(key)
    }
}

#[async_trait]
impl StorageAdapter for LocalFsAdapter {
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<()> {
        let path = self.path_for(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        let mut file = fs::File::create(path).await?;
        file.write_all(bytes).await?;
        file.flush().await?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let path = self.path_for(key);
        match fs::read(path).await {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.path_for(key);
        match fs::remove_file(path).await {
            Ok(_) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
} 
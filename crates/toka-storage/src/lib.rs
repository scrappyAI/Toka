//! Toka Storage – pluggable blob storage layer
//!
//! This crate defines the `StorageAdapter` trait alongside a default
//! `LocalFsAdapter` implementation that stores objects on the local
//! filesystem.  Additional back-ends (S3, GCS, IPFS, …) can implement the
//! same trait and be registered in the runtime.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;
use walkdir::WalkDir;

/// Unified, async interface for reading/writing arbitrary byte payloads.
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    /// Store a blob at the given URI (upsert).
    async fn put(&self, uri: &str, bytes: &[u8]) -> Result<()>;

    /// Fetch the full contents of a blob.
    async fn get(&self, uri: &str) -> Result<Vec<u8>>;

    /// Remove a blob if it exists.
    async fn delete(&self, uri: &str) -> Result<()>;

    /// List all object URIs starting with the provided prefix.
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}

/// Local-filesystem implementation.  URIs must start with the `local://`
/// scheme or be relative paths which are resolved against `root`.
///
/// Path traversal outside `root` is prevented.
#[deprecated(
    note = "Prefer the encrypted `VaultBlobAdapter` in `toka-security-vault`. Local filesystem adapter will move to a separate crate."
)]
#[derive(Debug, Clone)]
pub struct LocalFsAdapter {
    root: PathBuf,
}

impl LocalFsAdapter {
    /// Create a new adapter rooted at `root`.
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        let path = root.as_ref();
        std::fs::create_dir_all(path).context("Failed to create local storage root")?;
        Ok(Self {
            root: path.to_path_buf(),
        })
    }

    /// Resolve a URI (or bare path) to an absolute path under `root`.
    fn resolve(&self, uri: &str) -> Result<PathBuf> {
        let trimmed = uri.strip_prefix("local://").unwrap_or(uri);
        // Normalise separators and trim leading slashes so that "../" cannot hop above root.
        let normalised = Path::new(trimmed)
            .components()
            .filter(|c| !matches!(c, std::path::Component::ParentDir))
            .collect::<PathBuf>();

        let joined = self.root.join(normalised);

        // Simple prefix check is enough because we disallow ParentDir above.
        if !joined.starts_with(&self.root) {
            return Err(anyhow::anyhow!("Path escapes storage root"));
        }
        Ok(joined)
    }
}

#[async_trait]
impl StorageAdapter for LocalFsAdapter {
    async fn put(&self, uri: &str, bytes: &[u8]) -> Result<()> {
        let path = self.resolve(uri)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, bytes).await?;
        Ok(())
    }

    async fn get(&self, uri: &str) -> Result<Vec<u8>> {
        let path = self.resolve(uri)?;
        let data = fs::read(path).await?;
        Ok(data)
    }

    async fn delete(&self, uri: &str) -> Result<()> {
        let path = self.resolve(uri)?;
        if fs::metadata(&path).await.is_ok() {
            fs::remove_file(path).await?;
        }
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let prefix_path = self.resolve(prefix)?;
        let mut uris = Vec::new();
        if !prefix_path.exists() {
            return Ok(uris);
        }

        for entry in WalkDir::new(&prefix_path)
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(&self.root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .replace('\\', "/");
                uris.push(format!("local://{}", rel));
            }
        }
        Ok(uris)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn local_fs_roundtrip() -> Result<()> {
        let dir = tempdir()?;
        let adapter = LocalFsAdapter::new(dir.path())?;

        let uri = "local://nested/hello.txt";
        let bytes = b"hello toka";
        adapter.put(uri, bytes).await?;

        let fetched = adapter.get(uri).await?;
        assert_eq!(fetched, bytes);

        let listed = adapter.list("local://nested").await?;
        assert_eq!(listed, vec![uri.to_string()]);

        adapter.delete(uri).await?;
        let listed_after = adapter.list("local://nested").await?;
        assert!(listed_after.is_empty());

        Ok(())
    }
}

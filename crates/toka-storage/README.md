# Toka Storage

This crate provides a pluggable storage layer for the Toka platform. It defines a `StorageAdapter` trait that can be implemented to support various storage backends, such as a local filesystem, S3, or GCS.

## The `StorageAdapter` Trait

The core of this crate is the `StorageAdapter` trait, which defines a simple, asynchronous interface for a key-value blob store.

```rust
#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn put(&self, uri: &str, bytes: &[u8]) -> Result<()>;
    async fn get(&self, uri: &str) -> Result<Vec<u8>>;
    async fn delete(&self, uri: &str) -> Result<()>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
}
```

## Implementations

This crate includes one default implementation:

- `LocalFsAdapter`: Stores blobs on the local filesystem.

**Note:** The `LocalFsAdapter` is deprecated and will be moved to a separate crate in a future release. The recommended approach is to use the encrypted `VaultBlobAdapter` from the `toka-security-vault` crate for most use cases.

## Usage

To use the `LocalFsAdapter`, create a new instance with a root directory:

```rust
use toka_storage::{LocalFsAdapter, StorageAdapter};

# async fn run() -> anyhow::Result<()> {
let adapter = LocalFsAdapter::new("/tmp/toka-storage")?;

let uri = "local://my-data/blob.txt";
let data = b"some bytes";

adapter.put(uri, data).await?;

let retrieved_data = adapter.get(uri).await?;

assert_eq!(data, retrieved_data.as_slice());
# Ok(())
# } 
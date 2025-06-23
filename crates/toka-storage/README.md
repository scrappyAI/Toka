# Toka Storage

Generic storage abstractions and interfaces for the Toka platform.

## Overview

This crate provides generic storage abstractions that can be implemented by various storage backends. It defines the core interfaces for storing and retrieving data in a platform-agnostic way.

## Features

- Generic storage trait definitions
- Multiple storage backend support
- Async/await interface
- Transaction support
- Key-value and document storage patterns
- Pluggable storage implementations

## Usage

Add the following to your `Cargo.toml`:

```toml
[dependencies]
toka-storage = "0.1.0"
```

### Example

```rust
use toka_storage::{Storage, StorageBackend};

// Use any storage backend that implements the Storage trait
let storage: Box<dyn Storage> = StorageBackend::new(config).await?;

// Store data
storage.put("key", "value").await?;

// Retrieve data
let value = storage.get("key").await?;

// Delete data
storage.delete("key").await?;
```

## Design Philosophy

- **Abstraction**: Platform-agnostic storage interfaces
- **Flexibility**: Support for multiple storage backends
- **Performance**: Async-first design for high-throughput operations
- **Reliability**: Transaction support for data consistency

## License

This project is licensed under either of:
- MIT License
- Apache License 2.0

at your option.

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
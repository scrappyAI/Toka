// crates/ledger/src/lib.rs

// Publicly expose the modules for the ledger crate.

// Core ledger functionality, manages accounts and transactions.
pub mod ledger;

// Defines events that occur within the ledger.
pub mod event;

// Storage abstraction for persisting ledger state.
pub mod storage;

// Potentially a Readme or further documentation entry point if moved from root.
// pub mod readme_module; // If README.md is substantial and treated as a module.

// Re-export key types for convenience
pub use ledger::{Ledger, LedgerError, Transaction};
pub use event::{LedgerEvent, LedgerEventKind, LedgerEntry, EntryType, EventStatus, WALEntry};
pub use storage::{Storage, WALStorage, StorageError, MemoryStorage, FileStorage, NoOpStorage}; 
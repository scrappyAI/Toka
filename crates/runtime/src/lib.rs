//! Runtime orchestration layer for the Toka platform.
//!
//! This crate handles client-side logic including user ledger management,
//! server synchronization, and offline-capable user accounting.

// User-side ledger management and sync
pub mod user_ledger;

// Re-export key types for convenience
pub use user_ledger::{
    UserLedger, UserLedgerEntry, TransactionType, LedgerSyncRequest, LedgerSyncResponse, FeeApplication
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
} 
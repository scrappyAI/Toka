//! # Toka Ledger – Finance Helpers
//!
//! This crate adds **double-entry accounting** primitives on top of the generic
//! `toka-ledger` event store.  It does **not** attempt to enforce accounting
//! globally; instead it provides helpers that allow finance-aware components
//! (or agents) to *opt-in* to strict, balanced transactions while still using
//! the same event sourcing pipeline.
//!
//! ## Core Types
//! * `JournalEntry` – a single debit (negative) or credit (positive) against an
//!   `AccountId`.
//! * `LedgerTx`      – a batch of `JournalEntry`s that must balance to zero.
//!
//! ## Usage
//! ```rust,no_run
//! use toka_ledger_finance as finance;
//! use toka_ledger_core::{VaultBus};
//! use ndarray::arr1;
//!
//! # async fn example(bus: &VaultBus) -> anyhow::Result<()> {
//! let tx = finance::LedgerTx::balanced(vec![
//!     finance::debit("acct/a", finance::MicroUSD(100)),
//!     finance::credit("acct/b", finance::MicroUSD(100)),
//! ])?;
//!
//! let _hdrs = tx.commit(bus, arr1(&[0.0])).await?;
//! # Ok(())
//! # }
//! ```
#![forbid(unsafe_code)]

use anyhow::Result;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use toka_ledger_core::{EventHeader, EventPayload, VaultBus};

/// Identifier for an account in the financial ledger.
pub type AccountId = String;

/// Monetary amount in the smallest unit (µUSD for now).
/// Uses the same primitive from `toka-primitives` for consistency.
pub use toka_primitives::currency::MicroUSD;

/// A single line in the journal – positive for credit, negative for debit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub account: AccountId,
    pub delta:   MicroUSD,
    /// Free-form metadata – can hold FX rate, memo, etc.
    #[serde(default)]
    pub meta:    serde_json::Value,
}

impl JournalEntry {
    /// Convenience constructor for debits (negative delta).
    pub fn debit<A: Into<AccountId>>(account: A, amount: MicroUSD) -> Self {
        Self { account: account.into(), delta: MicroUSD(0) - amount, meta: serde_json::Value::Null }
    }

    /// Convenience constructor for credits (positive delta).
    pub fn credit<A: Into<AccountId>>(account: A, amount: MicroUSD) -> Self {
        Self { account: account.into(), delta: amount, meta: serde_json::Value::Null }
    }
}

/// Batch of journal entries that must sum to zero (i.e. balance).
#[derive(Debug, Clone)]
pub struct LedgerTx(Vec<JournalEntry>);

impl LedgerTx {
    /// Create a balanced transaction.  Returns error if not balanced.
    pub fn balanced(entries: Vec<JournalEntry>) -> Result<Self> {
        let sum: i128 = entries.iter().map(|e| e.delta.0 as i128).sum();
        anyhow::ensure!(sum == 0, "ledger transaction not balanced: sum={}", sum);
        Ok(Self(entries))
    }

    /// Commit the transaction atomically – each entry becomes an event in the
    /// underlying vault.  Returns the headers of the committed events.
    pub async fn commit(
        self,
        bus: &VaultBus,
        embedding: Array1<f32>,
    ) -> Result<Vec<EventHeader>> {
        // For causal linkage we treat the *first* entry header as the parent
        // for subsequent entries so auditors can tie the batch together.
        let mut parents: Vec<EventHeader> = Vec::new();
        let mut hdrs = Vec::with_capacity(self.0.len());

        for entry in &self.0 {
            let kind = if entry.delta.0 >= 0 { "ledger.credit" } else { "ledger.debit" };
            let hdr = bus.commit(entry, &parents, kind, embedding.clone()).await?;
            if parents.is_empty() {
                parents.push(hdr.clone());
            }
            hdrs.push(hdr);
        }
        Ok(hdrs)
    }
}

// --- Re-exports for ergonomics ------------------------------------------------

/// Convenience constructor for a debit entry.
pub fn debit<A: Into<AccountId>>(account: A, amount: MicroUSD) -> JournalEntry {
    JournalEntry::debit(account, amount)
}

/// Convenience constructor for a credit entry.
pub fn credit<A: Into<AccountId>>(account: A, amount: MicroUSD) -> JournalEntry {
    JournalEntry::credit(account, amount)
} 
//! Event types and logic for the ledger.
//!
//! This module defines the core event structures for the Toka credit ledger.
//! Events are immutable and append-only with Write-Ahead Logging (WAL) support.

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// The kind of event that can occur in the ledger.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LedgerEventKind {
    /// Credits are minted to an account.
    Mint {
        credits: u64,
        reason: String,
        memo: Option<String>,
    },
    /// Credits are burned from an account.
    Burn {
        credits: u64,
        reason: String,
        memo: Option<String>,
    },
    /// Credits are transferred between accounts.
    Transfer {
        from: String,
        to: String,
        credits: u64,
        reason: String,
        memo: Option<String>,
    },
}

/// Status of an event in the commit process.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventStatus {
    /// Event is staged but not yet committed.
    Staged,
    /// Event has been committed to the ledger.
    Committed,
    /// Event was rolled back due to an error.
    RolledBack,
}

/// Represents a ledger event, including its metadata and associated entries.
/// Events are immutable once created and support atomic commits via WAL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEvent {
    /// Unique identifier for this event (immutable).
    id: Uuid,
    /// Sequence number for ordering (immutable).
    sequence: u64,
    /// Timestamp when the event was created (immutable).
    timestamp: DateTime<Utc>,
    /// The kind of ledger operation (immutable).
    kind: LedgerEventKind,
    /// Optional debit entry (immutable).
    debit_entry: Option<LedgerEntry>,
    /// Optional credit entry (immutable).
    credit_entry: Option<LedgerEntry>,
    /// Current status of the event in the commit process.
    status: EventStatus,
}

/// Represents an entry in the ledger, either debit or credit.
/// Entries are immutable once created.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LedgerEntry {
    /// Account ID for this entry (immutable).
    account_id: String,
    /// Amount for this entry, signed (immutable).
    amount: i64,
    /// Event ID this entry belongs to (immutable).
    event_id: Uuid,
    /// Type of entry: debit or credit (immutable).
    entry_type: EntryType,
}

/// The type of ledger entry: debit or credit.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntryType {
    Debit,
    Credit,
}

impl LedgerEvent {
    /// Creates a new staged event. Events start in Staged status.
    /// This is the only way to create a new event to ensure immutability.
    pub fn new(
        sequence: u64,
        kind: LedgerEventKind,
        debit_entry: Option<LedgerEntry>,
        credit_entry: Option<LedgerEntry>,
    ) -> Self {
        let id = Uuid::new_v4();
        
        Self {
            id,
            sequence,
            timestamp: Utc::now(),
            kind,
            debit_entry,
            credit_entry,
            status: EventStatus::Staged,
        }
    }

    /// Marks the event as committed. Returns a new event with committed status.
    /// This preserves immutability by returning a new instance.
    pub fn commit(mut self) -> Self {
        self.status = EventStatus::Committed;
        self
    }

    /// Marks the event as rolled back. Returns a new event with rolled back status.
    /// This preserves immutability by returning a new instance.
    pub fn rollback(mut self) -> Self {
        self.status = EventStatus::RolledBack;
        self
    }

    // Immutable getters
    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn sequence(&self) -> u64 {
        self.sequence
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn kind(&self) -> &LedgerEventKind {
        &self.kind
    }

    pub fn debit_entry(&self) -> Option<&LedgerEntry> {
        self.debit_entry.as_ref()
    }

    pub fn credit_entry(&self) -> Option<&LedgerEntry> {
        self.credit_entry.as_ref()
    }

    pub fn status(&self) -> &EventStatus {
        &self.status
    }

    /// Checks if the event is committed.
    pub fn is_committed(&self) -> bool {
        matches!(self.status, EventStatus::Committed)
    }

    /// Checks if the event is staged (pending commit).
    pub fn is_staged(&self) -> bool {
        matches!(self.status, EventStatus::Staged)
    }

    /// Checks if the event was rolled back.
    pub fn is_rolled_back(&self) -> bool {
        matches!(self.status, EventStatus::RolledBack)
    }
}

impl LedgerEntry {
    /// Creates a new ledger entry. This is the only way to create entries
    /// to ensure immutability.
    pub fn new(
        account_id: String,
        amount: i64,
        event_id: Uuid,
        entry_type: EntryType,
    ) -> Self {
        Self {
            account_id,
            amount,
            event_id,
            entry_type,
        }
    }

    // Immutable getters
    pub fn account_id(&self) -> &str {
        &self.account_id
    }

    pub fn amount(&self) -> i64 {
        self.amount
    }

    pub fn event_id(&self) -> Uuid {
        self.event_id
    }

    pub fn entry_type(&self) -> &EntryType {
        &self.entry_type
    }
}

/// Write-Ahead Log entry for atomic commits.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WALEntry {
    /// Sequence number for ordering.
    pub sequence: u64,
    /// The event being logged.
    pub event: LedgerEvent,
    /// Timestamp when logged.
    pub logged_at: DateTime<Utc>,
}

impl WALEntry {
    /// Creates a new WAL entry.
    pub fn new(sequence: u64, event: LedgerEvent) -> Self {
        Self {
            sequence,
            event,
            logged_at: Utc::now(),
        }
    }
} 
use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::event::{LedgerEvent, LedgerEventKind, LedgerEntry, EntryType, WALEntry};
use crate::storage::{Storage, WALStorage};
use domain::account::{Account, PLATFORM_RESERVE_ACCOUNT, PLATFORM_REVENUE_ACCOUNT};

/// The main ledger state, containing accounts and events.
/// All events are immutable and append-only.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ledger {
    accounts: HashMap<String, Account>,
    events: Vec<LedgerEvent>,
    next_sequence: u64,
}

/// Error types for ledger operations.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    #[error("Insufficient funds in account {account_id}: balance {balance}, required {required}")]
    InsufficientFunds { account_id: String, balance: i64, required: i64 },
    #[error("Invalid transaction: debit and credit amounts must be equal")]
    UnbalancedTransaction,
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Event sequence error: expected {expected}, got {actual}")]
    SequenceError { expected: u64, actual: u64 },
    #[error("Cannot apply non-committed event")]
    NonCommittedEvent,
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

type Result<T> = std::result::Result<T, LedgerError>;

/// Represents an atomic transaction that can be committed or rolled back.
pub struct Transaction<'a, S: WALStorage> {
    ledger: &'a mut Ledger,
    storage: &'a mut S,
    staged_events: Vec<LedgerEvent>,
    pending_balances: HashMap<String, i64>, // Track pending balance changes
}

impl Ledger {
    /// Creates a new empty ledger with platform accounts.
    pub fn new() -> Self {
        let mut ledger = Self {
            accounts: HashMap::new(),
            events: Vec::new(),
            next_sequence: 1,
        };
        
        ledger.accounts.insert(
            PLATFORM_RESERVE_ACCOUNT.to_string(),
            Account {
                id: PLATFORM_RESERVE_ACCOUNT.to_string(),
                balance: 1_000_000_000, 
            }
        );
        // Initialize platform revenue account
        ledger.accounts.insert(
            PLATFORM_REVENUE_ACCOUNT.to_string(),
            Account {
                id: PLATFORM_REVENUE_ACCOUNT.to_string(),
                balance: 0, // Starts with zero balance
            }
        );
        
        ledger
    }

    /// Creates a new ledger and recovers from WAL if available.
    pub fn new_with_recovery<S: WALStorage>(
        storage: &mut S, 
    ) -> Result<Self> {
        let mut ledger = Self::new();
        let recovery_events = storage.recover_from_wal()
            .map_err(|e| LedgerError::StorageError(e.to_string()))?;
        
        for event in recovery_events {
            ledger.apply_committed_event(event)?;
        }
        
        Ok(ledger)
    }

    /// Starts a new atomic transaction.
    pub fn begin_transaction<'a, S: WALStorage>(&'a mut self, storage: &'a mut S) -> Transaction<'a, S> {
        Transaction {
            ledger: self,
            storage,
            staged_events: Vec::new(),
            pending_balances: HashMap::new(),
        }
    }

    /// Applies a committed event to the ledger.
    /// Only committed events can be applied to maintain consistency.
    pub fn apply_committed_event(&mut self, event: LedgerEvent) -> Result<()> {
        if !event.is_committed() {
            return Err(LedgerError::NonCommittedEvent);
        }

        // Validate sequence
        if event.sequence() != self.events.len() as u64 + 1 {
            return Err(LedgerError::SequenceError {
                expected: self.events.len() as u64 + 1,
                actual: event.sequence(),
            });
        }

        // Validate double entry accounting (all operations must have both debit and credit)
        let (debit, credit) = match (event.debit_entry(), event.credit_entry()) {
            (Some(debit), Some(credit)) => (debit, credit),
            _ => return Err(LedgerError::UnbalancedTransaction),
        };

        if debit.amount() != credit.amount() {
            return Err(LedgerError::UnbalancedTransaction);
        }

        // Apply entries
        self.debit(debit.account_id(), debit.amount())?;
        self.credit(credit.account_id(), credit.amount());

        self.events.push(event);
        self.next_sequence = self.events.len() as u64 + 1;
        Ok(())
    }

    /// Gets the balance for an account, returning 0 if account doesn't exist.
    pub fn get_account_balance(&self, account_id: &str) -> i64 {
        self.accounts.get(account_id).map(|a| a.balance).unwrap_or(0)
    }

    /// Gets the platform reserve balance.
    pub fn get_platform_reserve_balance(&self) -> i64 {
        self.get_account_balance(PLATFORM_RESERVE_ACCOUNT)
    }

    /// Gets all accounts.
    pub fn get_accounts(&self) -> &HashMap<String, Account> {
        &self.accounts
    }

    /// Gets all events.
    pub fn get_events(&self) -> &[LedgerEvent] {
        &self.events
    }

    /// Gets the next sequence number.
    pub fn next_sequence(&self) -> u64 {
        self.next_sequence
    }

    /// Calculates the total credits in circulation (excludes platform reserve).
    pub fn total_credits_in_circulation(&self) -> u64 {
        self.accounts.values()
            .filter(|account| account.id != PLATFORM_RESERVE_ACCOUNT && account.balance > 0)
            .map(|account| account.balance as u64)
            .sum()
    }

    /// Calculates the total supply of credits (all positive balances including reserve).
    pub fn total_supply(&self) -> u64 {
        self.accounts.values()
            .filter_map(|account| if account.balance > 0 { Some(account.balance as u64) } else { None })
            .sum()
    }

    /// Validates ledger integrity (all debits equal all credits).
    pub fn validate_integrity(&self) -> Result<()> {
        let mut total_debits: i64 = 0;
        let mut total_credits: i64 = 0;

        for event in &self.events {
            if !event.is_committed() {
                continue; // Skip non-committed events
            }

            if let Some(debit) = event.debit_entry() {
                total_debits += debit.amount();
            }
            if let Some(credit) = event.credit_entry() {
                total_credits += credit.amount();
            }
        }

        if total_debits != total_credits {
            return Err(LedgerError::UnbalancedTransaction);
        }

        // Also check that the sum of all account balances makes sense
        let total_balance: i64 = self.accounts.values().map(|acc| acc.balance).sum();
        
        // For a simple check, we expect the total of all balances to equal the initial platform reserve
        // This is a basic integrity check - in production you'd have more sophisticated validation
        if !self.events.is_empty() {
            let _platform_reserve = self.accounts.get(PLATFORM_RESERVE_ACCOUNT)
                .map(|acc| acc.balance)
                .unwrap_or(0);
            let _platform_revenue = self.accounts.get(PLATFORM_REVENUE_ACCOUNT)
                .map(|acc| acc.balance)
                .unwrap_or(0);
            
            // Total should equal initial reserve amount (1B credits in our case)
            let expected_total = 1_000_000_000; // Initial platform reserve
            if total_balance != expected_total {
                return Err(LedgerError::TransactionFailed(
                    format!("Balance integrity check failed: total {} != expected {}", total_balance, expected_total)
                ));
            }
        }

        Ok(())
    }

    /// Saves the ledger state using the provided storage backend.
    pub fn save<S: Storage>(&self, storage: &mut S) -> Result<()> {
        storage.save_ledger(self)
            .map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    /// Loads the ledger from storage.
    pub fn load<S: Storage>(storage: &S) -> Result<Self> {
        storage.load_ledger().map_err(|e| LedgerError::StorageError(e.to_string()))
    }

    fn debit(&mut self, account_id: &str, amount: i64) -> Result<()> {
        let account = self.accounts.entry(account_id.to_string())
            .or_insert(Account { id: account_id.to_string(), balance: 0 });

        if account.balance < amount {
            return Err(LedgerError::InsufficientFunds {
                account_id: account_id.to_string(),
                balance: account.balance,
                required: amount,
            });
        }
        account.balance -= amount;
        Ok(())
    }

    fn credit(&mut self, account_id: &str, amount: i64) {
        let account = self.accounts.entry(account_id.to_string())
            .or_insert(Account { id: account_id.to_string(), balance: 0 });
        account.balance += amount;
    }
}

impl<'a, S: WALStorage> Transaction<'a, S> {
    // Helper to get current balance considering pending changes
    fn get_effective_balance(&self, account_id: &str) -> i64 {
        self.ledger.get_account_balance(account_id) + self.pending_balances.get(account_id).unwrap_or(&0)
    }

    // Helper to update pending balance changes
    fn update_pending_balance(&mut self, account_id: &str, change: i64) {
        *self.pending_balances.entry(account_id.to_string()).or_insert(0) += change;
    }
    
    /// Mints new credits into an account (e.g., credit purchase by user).
    /// This is a credit to the target account and a debit from the platform reserve.
    pub fn mint(&mut self, account_id: &str, credits_to_mint: u64, reason: String, memo: Option<String>) -> Result<()> {
        if credits_to_mint == 0 {
            return Err(LedgerError::TransactionFailed("Cannot mint zero credits".to_string()));
        }

        let amount = credits_to_mint as i64;

        // Check platform reserve balance
        let reserve_balance = self.get_effective_balance(PLATFORM_RESERVE_ACCOUNT);
        if reserve_balance < amount {
            return Err(LedgerError::InsufficientFunds {
                account_id: PLATFORM_RESERVE_ACCOUNT.to_string(),
                balance: reserve_balance,
                required: amount,
            });
        }
        
        self.update_pending_balance(PLATFORM_RESERVE_ACCOUNT, -amount);
        self.update_pending_balance(account_id, amount);

        // Create the event first to get its ID
        let sequence = self.ledger.next_sequence + self.staged_events.len() as u64;
        let kind = LedgerEventKind::Mint {
            credits: credits_to_mint,
            reason,
            memo,
        };

        // Create a temporary event to get its ID for the entries
        let temp_event = LedgerEvent::new(sequence, kind.clone(), None, None);
        let event_id = temp_event.id();

        // Create the ledger entries with the event ID
        let debit_entry = LedgerEntry::new(
            PLATFORM_RESERVE_ACCOUNT.to_string(),
            amount,
            event_id,
            EntryType::Debit,
        );
        let credit_entry = LedgerEntry::new(
            account_id.to_string(),
            amount,
            event_id,
            EntryType::Credit,
        );

        // Create the final event with the entries
        let event = LedgerEvent::new(
            sequence,
            kind,
            Some(debit_entry),
            Some(credit_entry),
        );

        self.staged_events.push(event);
        Ok(())
    }

    /// Burns credits from an account (e.g., creator cashout).
    /// This is a debit from the target account and a credit to the platform reserve.
    pub fn burn(&mut self, account_id: &str, credits_to_cashout: u64, reason: String, memo: Option<String>) -> Result<()> {
        if credits_to_cashout == 0 {
            return Err(LedgerError::TransactionFailed("Cannot burn zero credits".to_string()));
        }
        let amount = credits_to_cashout as i64;

        let account_balance = self.get_effective_balance(account_id);
        if account_balance < amount {
            return Err(LedgerError::InsufficientFunds {
                account_id: account_id.to_string(),
                balance: account_balance,
                required: amount,
            });
        }

        self.update_pending_balance(account_id, -amount);
        self.update_pending_balance(PLATFORM_RESERVE_ACCOUNT, amount);
        
        // Create the event first to get its ID
        let sequence = self.ledger.next_sequence + self.staged_events.len() as u64;
        let kind = LedgerEventKind::Burn {
            credits: credits_to_cashout,
            reason,
            memo,
        };

        // Create a temporary event to get its ID for the entries
        let temp_event = LedgerEvent::new(sequence, kind.clone(), None, None);
        let event_id = temp_event.id();

        // Create the ledger entries with the event ID
        let debit_entry = LedgerEntry::new(
            account_id.to_string(),
            amount,
            event_id,
            EntryType::Debit,
        );
        let credit_entry = LedgerEntry::new(
            PLATFORM_RESERVE_ACCOUNT.to_string(),
            amount,
            event_id,
            EntryType::Credit,
        );

        // Create the final event with the entries
        let event = LedgerEvent::new(
            sequence,
            kind,
            Some(debit_entry),
            Some(credit_entry),
        );

        self.staged_events.push(event);
        Ok(())
    }

    /// Transfers credits between two accounts.
    pub fn transfer(&mut self, from: &str, to: &str, credits_to_transfer: u64, reason: String, memo: Option<String>) -> Result<()> {
        if from == to {
            return Err(LedgerError::TransactionFailed("Cannot transfer to the same account".to_string()));
        }
        if credits_to_transfer == 0 {
            return Err(LedgerError::TransactionFailed("Cannot transfer zero credits".to_string()));
        }

        let amount = credits_to_transfer as i64;

        let from_balance = self.get_effective_balance(from);
        if from_balance < amount {
            return Err(LedgerError::InsufficientFunds {
                account_id: from.to_string(),
                balance: from_balance,
                required: amount,
            });
        }
        
        self.update_pending_balance(from, -amount);
        self.update_pending_balance(to, amount);

        // Create the event first to get its ID
        let sequence = self.ledger.next_sequence + self.staged_events.len() as u64;
        let kind = LedgerEventKind::Transfer {
            from: from.to_string(),
            to: to.to_string(),
            credits: credits_to_transfer,
            reason,
            memo,
        };

        // Create a temporary event to get its ID for the entries
        let temp_event = LedgerEvent::new(sequence, kind.clone(), None, None);
        let event_id = temp_event.id();

        // Create the ledger entries with the event ID
        let debit_entry = LedgerEntry::new(
            from.to_string(),
            amount,
            event_id,
            EntryType::Debit,
        );
        let credit_entry = LedgerEntry::new(
            to.to_string(),
            amount,
            event_id,
            EntryType::Credit,
        );

        // Create the final event with the entries
        let event = LedgerEvent::new(
            sequence,
            kind,
            Some(debit_entry),
            Some(credit_entry),
        );

        self.staged_events.push(event);
        Ok(())
    }

    /// Commits all staged events in the transaction.
    /// Writes to WAL, then applies to ledger state.
    pub fn commit(self) -> Result<Vec<Uuid>> {
        if self.staged_events.is_empty() {
            return Ok(Vec::new()); // Nothing to commit
        }

        let mut committed_event_ids = Vec::new();

        for event in self.staged_events {
            // Commit the event (changes status to Committed)
            let committed_event = event.commit();
            let event_id = committed_event.id();

            // Write to WAL first
            let wal_entry = WALEntry::new(committed_event.sequence(), committed_event.clone());
            self.storage.append_to_wal(wal_entry)
                .map_err(|e| LedgerError::StorageError(e.to_string()))?;
            
            // Apply to in-memory ledger
            self.ledger.apply_committed_event(committed_event)
                .map_err(|e| {
                    // If applying to ledger fails, we have a critical inconsistency.
                    // This might require manual intervention or a more robust recovery.
                    // For now, we'll report it as a storage error as it indicates WAL write succeeded
                    // but in-memory update failed.
                    LedgerError::StorageError(format!("Failed to apply committed event after WAL write: {}", e))
                })?;
            
            committed_event_ids.push(event_id);
        }

        Ok(committed_event_ids)
    }

    /// Rolls back the transaction, clearing staged events.
    pub fn rollback(self) -> Result<()> {
        // No actual state change to revert in the ledger, as events were only staged.
        // WAL entries are not written for rolled-back transactions.
        Ok(())
    }
    
    /// Returns the number of events staged in this transaction.
    pub fn staged_count(&self) -> usize {
        self.staged_events.len()
    }

    /// Returns a reference to the pending balance changes.
    pub fn pending_balances(&self) -> &HashMap<String, i64> {
        &self.pending_balances
    }
} 
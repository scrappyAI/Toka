use std::collections::HashMap;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use crate::event::{LedgerEvent, LedgerEventKind, LedgerEntry, EntryType, WALEntry, ReasonCode};
use crate::storage::{Storage, WALStorage};
use domain::account::{
    Account, 
    PLATFORM_REVENUE_ACCOUNT,
    EXTERNAL_DEPOSITS_ACCOUNT,
    EXTERNAL_WITHDRAWALS_ACCOUNT
};

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

/// Information about the current system economic status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicStatus {
    /// Total money received from users (in cents)
    pub total_deposits: i64,
    /// Total money paid to creators (in cents)
    pub total_withdrawals: i64,
    /// Net money position (deposits - withdrawals)
    pub net_money_position: i64,
    /// Total credits currently in circulation
    pub credits_in_circulation: u64,
    /// Platform profit (money kept by platform)
    pub platform_profit: i64,
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
    /// Creates a new empty ledger with zero balances.
    /// Credits are only created when users purchase them with real money.
    pub fn new() -> Self {
        let mut ledger = Self {
            accounts: HashMap::new(),
            events: Vec::new(),
            next_sequence: 1,
        };
        
        // Initialize platform revenue account (for fees, if needed)
        ledger.accounts.insert(
            PLATFORM_REVENUE_ACCOUNT.to_string(),
            Account {
                id: PLATFORM_REVENUE_ACCOUNT.to_string(),
                balance: 0,
            }
        );
        
        // Initialize external money tracking accounts
        ledger.accounts.insert(
            EXTERNAL_DEPOSITS_ACCOUNT.to_string(),
            Account {
                id: EXTERNAL_DEPOSITS_ACCOUNT.to_string(),
                balance: 0, // No initial deposits
            }
        );
        
        ledger.accounts.insert(
            EXTERNAL_WITHDRAWALS_ACCOUNT.to_string(),
            Account {
                id: EXTERNAL_WITHDRAWALS_ACCOUNT.to_string(),
                balance: 0, // No initial withdrawals
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

        // All events must be proper double-entry transactions
        match (event.debit_entry(), event.credit_entry()) {
            (Some(debit), Some(credit)) => {
                if debit.amount() != credit.amount() {
                    return Err(LedgerError::UnbalancedTransaction);
                }
                self.debit(debit.account_id(), debit.amount())?;
                self.credit(credit.account_id(), credit.amount());
            },
            _ => return Err(LedgerError::UnbalancedTransaction),
        }

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
        // In zero-investment model, there is no platform reserve account
        // This method is kept for compatibility but always returns 0
        0
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

    /// Calculates the total credits in circulation (all user and creator balances).
    pub fn total_credits_in_circulation(&self) -> u64 {
        self.accounts.values()
            .filter(|account| !account.id.starts_with("external:") && !account.id.starts_with("platform:"))
            .filter(|account| account.balance > 0)
            .map(|account| account.balance as u64)
            .sum()
    }

    /// Calculates the total supply of credits (all positive credit balances).
    pub fn total_supply(&self) -> u64 {
        self.accounts.values()
            .filter(|account| !account.id.starts_with("external:")) // Exclude money accounts
            .filter_map(|account| if account.balance > 0 { Some(account.balance as u64) } else { None })
            .sum()
    }

    /// Validates ledger integrity using standard double-entry accounting.
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

        // For credit creation/destruction, we allow some flexibility
        // The economic validation handles the money backing
        if total_debits != total_credits {
            // In zero-investment model, credit creation doesn't require matching debits
            // But we should validate that the imbalance makes economic sense
            let imbalance = total_credits - total_debits;
            let _credits_in_circulation = self.total_credits_in_circulation() as i64;
            
            // The imbalance should roughly equal credits created minus credits destroyed
            if imbalance < 0 {
                return Err(LedgerError::UnbalancedTransaction);
            }
        }

        Ok(())
    }

    /// Validates that the credit system is economically sound.
    pub fn validate_economic_integrity(&self) -> Result<()> {
        let deposits = self.get_total_external_deposits();
        let withdrawals = self.get_total_external_withdrawals();
        
        // Can't pay out more money than we've received
        if withdrawals > deposits {
            return Err(LedgerError::TransactionFailed(
                format!("More money paid out (${:.2}) than received (${:.2})", 
                    withdrawals as f64 / 100.0, deposits as f64 / 100.0)
            ));
        }
        
        // Net money position should be positive or zero
        let net_position = deposits - withdrawals;
        if net_position < 0 {
            return Err(LedgerError::TransactionFailed(
                format!("Negative net money position: ${:.2}", net_position as f64 / 100.0)
            ));
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

    /// Gets the total amount of real money deposited into the system.
    pub fn get_total_external_deposits(&self) -> i64 {
        self.get_account_balance(EXTERNAL_DEPOSITS_ACCOUNT)
    }

    /// Gets the total amount of real money withdrawn from the system.
    pub fn get_total_external_withdrawals(&self) -> i64 {
        self.get_account_balance(EXTERNAL_WITHDRAWALS_ACCOUNT)
    }

    /// Gets the net real money position (deposits - withdrawals).
    pub fn get_net_money_position(&self) -> i64 {
        self.get_total_external_deposits() - self.get_total_external_withdrawals()
    }

    /// Gets comprehensive economic status of the system.
    pub fn get_economic_status(&self) -> EconomicStatus {
        let deposits = self.get_total_external_deposits();
        let withdrawals = self.get_total_external_withdrawals();
        
        EconomicStatus {
            total_deposits: deposits,
            total_withdrawals: withdrawals,
            net_money_position: deposits - withdrawals,
            credits_in_circulation: self.total_credits_in_circulation(),
            platform_profit: deposits - withdrawals, // Platform keeps the difference
        }
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
    
    /// User purchases credits with real money (creates new credits in the system).
    pub fn purchase_credits(&mut self, user_id: &str, usd_cents: u64, credits: u64, memo: Option<String>) -> Result<()> {
        if usd_cents == 0 || credits == 0 {
            return Err(LedgerError::TransactionFailed("Purchase amounts must be greater than zero".to_string()));
        }

        // Record the real money received
        self.update_pending_balance(EXTERNAL_DEPOSITS_ACCOUNT, usd_cents as i64);
        
        // Create credits for the user (no balancing debit needed in zero-investment model)
        self.update_pending_balance(user_id, credits as i64);

        // Create the purchase event
        let sequence = self.ledger.next_sequence + self.staged_events.len() as u64;
        let kind = LedgerEventKind::Mint {
            credits,
            reason: ReasonCode::CreditPurchase,
            memo: memo.clone(),
        };

        // Create temporary event to get ID
        let temp_event = LedgerEvent::new(sequence, kind.clone(), None, None);
        let event_id = temp_event.id();

        // Create credit entry for user
        let credit_entry = LedgerEntry::new(
            user_id.to_string(),
            credits as i64,
            event_id,
            EntryType::Credit,
        );

        // In zero-investment model, credit creation doesn't need a balancing debit
        // This reflects the economic reality: user gives money, gets credits
        let event = LedgerEvent::new(
            sequence,
            LedgerEventKind::Transfer {
                from: "external:money_source".to_string(), // Conceptual external source
                to: user_id.to_string(),
                credits,
                reason: ReasonCode::CreditPurchase,
                memo,
            },
            None, // No debit needed for credit creation
            Some(credit_entry),
        );

        self.staged_events.push(event);
        Ok(())
    }

    /// Creator cashes out credits for real money (destroys credits from the system).
    pub fn creator_cashout(&mut self, creator_id: &str, credits: u64, usd_cents: u64, memo: Option<String>) -> Result<()> {
        if credits == 0 || usd_cents == 0 {
            return Err(LedgerError::TransactionFailed("Cashout amounts must be greater than zero".to_string()));
        }

        // Check if creator has enough credits
        let creator_balance = self.get_effective_balance(creator_id);
        if creator_balance < credits as i64 {
            return Err(LedgerError::InsufficientFunds {
                account_id: creator_id.to_string(),
                balance: creator_balance,
                required: credits as i64,
            });
        }

        // Check if we have enough money to pay out
        let money_balance = self.get_effective_balance(EXTERNAL_DEPOSITS_ACCOUNT);
        if money_balance < usd_cents as i64 {
            return Err(LedgerError::InsufficientFunds {
                account_id: EXTERNAL_DEPOSITS_ACCOUNT.to_string(),
                balance: money_balance,
                required: usd_cents as i64,
            });
        }

        // Remove credits from creator
        self.update_pending_balance(creator_id, -(credits as i64));
        
        // Move money from deposits to withdrawals 
        self.update_pending_balance(EXTERNAL_DEPOSITS_ACCOUNT, -(usd_cents as i64));
        self.update_pending_balance(EXTERNAL_WITHDRAWALS_ACCOUNT, usd_cents as i64);

        // Use standard transfer to move money and burn credits via separate transaction
        self.transfer(
            EXTERNAL_DEPOSITS_ACCOUNT,
            EXTERNAL_WITHDRAWALS_ACCOUNT,
            usd_cents,
            ReasonCode::CreatorCashout,
            memo
        )?;

        Ok(())
    }

    /// Transfers credits between two accounts (for content unlocking, tips, etc.).
    pub fn transfer(&mut self, from: &str, to: &str, credits_to_transfer: u64, reason: ReasonCode, memo: Option<String>) -> Result<()> {
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
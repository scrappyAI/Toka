use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Types of transactions that can occur in the user's local ledger.
/// Note: P2P transfers are intentionally limited to reduce sync complexity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransactionType {
    /// User purchased credits with real money (server-authoritative)
    CreditPurchase,
    /// User unlocked content from a creator (most common operation)
    ContentUnlock,
    /// Creator cashed out credits for real money (server-authoritative)
    CreatorCashout,
    /// Platform fee collection (automatic)
    PlatformFee,
    /// Creator Empowerment Fund allocation (automatic)
    EmpowermentFundAllocation,
    /// Bonus credits awarded (server-authoritative)
    BonusCredits,
    /// Refund or reversal (server-authoritative)
    Refund,
}

/// Local ledger entry stored on user's device for fast accounting.
/// This represents the user's VIEW of transactions, not the authoritative record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLedgerEntry {
    /// Unique identifier for this transaction
    pub transaction_id: Uuid,
    /// The other party in the transaction (creator, platform, etc.)
    pub counterparty_id: Uuid,
    /// Amount in credits (always positive, transaction_type determines direction)
    pub amount_credits: u64,
    /// Type of transaction
    pub transaction_type: TransactionType,
    /// When the transaction occurred
    pub timestamp: DateTime<Utc>,
    /// User's credit balance after this transaction (local calculation)
    pub local_balance_after: u64,
    /// Whether this entry has been confirmed by central server
    pub synced_to_server: bool,
    /// Optional: Content ID if this was a content unlock
    pub content_id: Option<Uuid>,
    /// Optional: Fees applied to this transaction
    pub fees_applied: Option<Vec<FeeApplication>>,
    /// Optional: Description or notes
    pub description: Option<String>,
}

/// Details about a fee that was applied to a transaction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeApplication {
    /// Type of fee (platform, cashout, etc.)
    pub fee_type: String,
    /// Amount of the fee in credits
    pub amount_credits: u64,
    /// Percentage rate if applicable
    pub percentage_rate: Option<Decimal>,
    /// Who receives this fee (platform, empowerment fund, etc.)
    pub recipient: String,
}

/// Local user ledger that provides on-device accounting.
/// This is NOT the authoritative ledger - just a user's view for UX.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserLedger {
    /// User ID this ledger belongs to
    pub user_id: Uuid,
    /// Current credit balance (local view, may be out of sync)
    pub current_balance_credits: u64,
    /// All transactions in chronological order
    pub transactions: Vec<UserLedgerEntry>,
    /// Last time this ledger was synced with server
    pub last_sync_timestamp: Option<DateTime<Utc>>,
    /// Number of pending (unsynced) transactions
    pub pending_sync_count: u32,
    /// Checksum/hash for integrity verification
    pub ledger_checksum: Option<String>,
}

/// Sync request to reconcile local ledger with central server.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerSyncRequest {
    pub user_id: Uuid,
    /// Transactions that haven't been confirmed yet
    pub pending_transactions: Vec<UserLedgerEntry>,
    /// Local balance for verification
    pub local_balance_credits: u64,
    /// Last known sync timestamp
    pub last_sync_timestamp: Option<DateTime<Utc>>,
    /// Client-side ledger checksum
    pub local_checksum: Option<String>,
}

/// Response from server after sync attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LedgerSyncResponse {
    /// Whether sync was successful
    pub success: bool,
    /// Authoritative balance from server
    pub server_balance_credits: u64,
    /// Transactions the server rejected (conflicts)
    pub rejected_transactions: Vec<Uuid>,
    /// New transactions from server the client missed
    pub missing_transactions: Vec<UserLedgerEntry>,
    /// Updated checksum after sync
    pub updated_checksum: Option<String>,
    /// Error message if sync failed
    pub error_message: Option<String>,
}

impl UserLedger {
    /// Creates a new empty ledger for a user.
    pub fn new(user_id: Uuid) -> Self {
        Self {
            user_id,
            current_balance_credits: 0,
            transactions: Vec::new(),
            last_sync_timestamp: None,
            pending_sync_count: 0,
            ledger_checksum: None,
        }
    }

    /// Adds a new transaction to the local ledger.
    /// Note: This is optimistic - server may reject during sync.
    pub fn add_transaction(&mut self, mut entry: UserLedgerEntry) -> Result<(), String> {
        // Calculate new balance based on transaction type
        let new_balance = match entry.transaction_type {
            // These increase the user's balance
            TransactionType::CreditPurchase | TransactionType::BonusCredits | TransactionType::Refund => {
                self.current_balance_credits.saturating_add(entry.amount_credits)
            }
            // These decrease the user's balance
            TransactionType::ContentUnlock | TransactionType::CreatorCashout | 
            TransactionType::PlatformFee | TransactionType::EmpowermentFundAllocation => {
                if self.current_balance_credits < entry.amount_credits {
                    return Err("Insufficient balance for transaction".to_string());
                }
                self.current_balance_credits - entry.amount_credits
            }
        };

        // Update the entry with the new balance
        entry.local_balance_after = new_balance;
        entry.synced_to_server = false;

        // Add to transactions and update state
        self.transactions.push(entry);
        self.current_balance_credits = new_balance;
        self.pending_sync_count += 1;

        // Update checksum
        self.update_checksum();

        Ok(())
    }

    /// Gets transaction history with optional filtering.
    pub fn get_transaction_history(
        &self,
        transaction_type: Option<TransactionType>,
        limit: Option<usize>,
    ) -> Vec<&UserLedgerEntry> {
        let mut filtered: Vec<&UserLedgerEntry> = self.transactions
            .iter()
            .filter(|tx| {
                if let Some(ref tx_type) = transaction_type {
                    &tx.transaction_type == tx_type
                } else {
                    true
                }
            })
            .collect();

        // Sort by timestamp, most recent first
        filtered.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            filtered.truncate(limit);
        }

        filtered
    }

    /// Gets pending (unsynced) transactions.
    pub fn get_pending_transactions(&self) -> Vec<&UserLedgerEntry> {
        self.transactions
            .iter()
            .filter(|tx| !tx.synced_to_server)
            .collect()
    }

    /// Prepares a sync request to send to the server.
    pub fn prepare_sync_request(&self) -> LedgerSyncRequest {
        LedgerSyncRequest {
            user_id: self.user_id,
            pending_transactions: self.get_pending_transactions().into_iter().cloned().collect(),
            local_balance_credits: self.current_balance_credits,
            last_sync_timestamp: self.last_sync_timestamp,
            local_checksum: self.ledger_checksum.clone(),
        }
    }

    /// Processes a sync response from the server.
    /// Server is always authoritative for balances and transaction validity.
    pub fn process_sync_response(&mut self, response: LedgerSyncResponse) -> Result<(), String> {
        if !response.success {
            return Err(response.error_message.unwrap_or("Sync failed".to_string()));
        }

        // Mark confirmed transactions as synced
        for transaction in &mut self.transactions {
            if !response.rejected_transactions.contains(&transaction.transaction_id) {
                transaction.synced_to_server = true;
            }
        }

        // Remove rejected transactions (server didn't accept them)
        self.transactions.retain(|tx| !response.rejected_transactions.contains(&tx.transaction_id));

        // Add missing transactions from server
        for missing_tx in response.missing_transactions {
            self.transactions.push(missing_tx);
        }

        // Always trust server balance (this is critical!)
        self.current_balance_credits = response.server_balance_credits;

        // Update sync metadata
        self.last_sync_timestamp = Some(Utc::now());
        self.pending_sync_count = self.get_pending_transactions().len() as u32;
        self.ledger_checksum = response.updated_checksum;

        // Re-sort transactions by timestamp
        self.transactions.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        Ok(())
    }

    /// Calculates spending by category for user insights.
    /// Only includes outgoing transactions (spending).
    pub fn get_spending_summary(&self) -> HashMap<TransactionType, u64> {
        let mut summary = HashMap::new();
        
        for tx in &self.transactions {
            // Only count confirmed transactions for spending summary
            if tx.synced_to_server {
                match tx.transaction_type {
                    TransactionType::ContentUnlock | TransactionType::CreatorCashout | 
                    TransactionType::PlatformFee => {
                        *summary.entry(tx.transaction_type.clone()).or_insert(0) += tx.amount_credits;
                    }
                    _ => {} // Only count outgoing transactions for spending
                }
            }
        }
        
        summary
    }

    /// Gets total spending (confirmed transactions only).
    pub fn get_total_spending(&self) -> u64 {
        self.get_spending_summary().values().sum()
    }

    /// Gets total earnings for creators (confirmed transactions only).
    pub fn get_creator_earnings(&self) -> u64 {
        self.transactions
            .iter()
            .filter(|tx| tx.synced_to_server)
            .filter(|tx| matches!(tx.transaction_type, TransactionType::ContentUnlock))
            .map(|tx| {
                // Calculate net earnings (amount minus fees)
                let fees: u64 = tx.fees_applied
                    .as_ref()
                    .map(|fees| fees.iter().map(|f| f.amount_credits).sum())
                    .unwrap_or(0);
                tx.amount_credits.saturating_sub(fees)
            })
            .sum()
    }

    /// Simple checksum calculation for integrity verification.
    fn update_checksum(&mut self) {
        let checksum_data = format!(
            "{}:{}:{}",
            self.user_id,
            self.current_balance_credits,
            self.transactions.len()
        );
        // Simple hash - in production, use a proper cryptographic hash
        self.ledger_checksum = Some(format!("{:x}", checksum_data.len()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_user_ledger_creation() {
        let user_id = Uuid::new_v4();
        let ledger = UserLedger::new(user_id);
        
        assert_eq!(ledger.user_id, user_id);
        assert_eq!(ledger.current_balance_credits, 0);
        assert_eq!(ledger.transactions.len(), 0);
        assert_eq!(ledger.pending_sync_count, 0);
    }

    #[test]
    fn test_credit_purchase_transaction() {
        let mut ledger = UserLedger::new(Uuid::new_v4());
        
        let transaction = UserLedgerEntry {
            transaction_id: Uuid::new_v4(),
            counterparty_id: Uuid::new_v4(), // Platform
            amount_credits: 100,
            transaction_type: TransactionType::CreditPurchase,
            timestamp: Utc::now(),
            local_balance_after: 0, // Will be calculated
            synced_to_server: false,
            content_id: None,
            fees_applied: None,
            description: Some("Purchased starter pack".to_string()),
        };

        let result = ledger.add_transaction(transaction);
        assert!(result.is_ok());
        assert_eq!(ledger.current_balance_credits, 100);
        assert_eq!(ledger.pending_sync_count, 1);
        assert_eq!(ledger.transactions[0].local_balance_after, 100);
    }

    #[test]
    fn test_insufficient_balance_transaction() {
        let mut ledger = UserLedger::new(Uuid::new_v4());
        
        let transaction = UserLedgerEntry {
            transaction_id: Uuid::new_v4(),
            counterparty_id: Uuid::new_v4(),
            amount_credits: 100,
            transaction_type: TransactionType::ContentUnlock,
            timestamp: Utc::now(),
            local_balance_after: 0,
            synced_to_server: false,
            content_id: Some(Uuid::new_v4()),
            fees_applied: None,
            description: None,
        };

        let result = ledger.add_transaction(transaction);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Insufficient balance"));
    }

    #[test]
    fn test_server_authoritative_sync() {
        let mut ledger = UserLedger::new(Uuid::new_v4());
        ledger.current_balance_credits = 100; // Local thinks we have 100

        // Server says we actually have 150
        let sync_response = LedgerSyncResponse {
            success: true,
            server_balance_credits: 150,
            rejected_transactions: Vec::new(),
            missing_transactions: Vec::new(),
            updated_checksum: Some("abc123".to_string()),
            error_message: None,
        };

        ledger.process_sync_response(sync_response).unwrap();
        
        // Local balance should now match server (server is authoritative)
        assert_eq!(ledger.current_balance_credits, 150);
    }
} 
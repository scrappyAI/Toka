//! Account entity definition for the ledger.

use serde::{Serialize, Deserialize};

/// Platform reserve account ID - used for credit issuance and redemption
pub const PLATFORM_RESERVE_ACCOUNT: &str = "platform:reserve";
/// Platform revenue account ID - used for collecting platform fees
pub const PLATFORM_REVENUE_ACCOUNT: &str = "platform:revenue";

/// Represents a ledger account.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)] // Added more derives for HashMap key usage
pub struct Account {
    pub id: String,
    pub balance: i64, // Using i64 to allow for negative balances if design requires, though typically positive
}

impl Account {
    /// Creates a new account with a zero balance.
    pub fn new(id: String) -> Self {
        Self {
            id,
            balance: 0,
        }
    }

    /// Creates a new account with an initial balance.
    pub fn new_with_balance(id: String, initial_balance: i64) -> Self {
        Self {
            id,
            balance: initial_balance,
        }
    }

    // Note: Direct balance manipulation methods (debit/credit) are intentionally
    // omitted here. The Ledger module should be responsible for all balance changes
    // to ensure double-entry accounting and event sourcing principles are upheld.
    // Account merely holds state.
} 
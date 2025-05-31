# Hybrid Ledger: Local Accounting with Central Sync

This document describes the hybrid ledger implementation that provides **on-device accounting** with **periodic synchronization** to central servers.

## Architecture Overview

The hybrid approach combines the best of both centralized and peer-to-peer ledgers:

- **Local Ledger**: Each user maintains their own transaction history on-device
- **Central Authority**: Server maintains authoritative balance and handles fees
- **Periodic Sync**: Local ledgers sync periodically with central server for consistency

## Clean Separation of Concerns

### 1. `pricing_model.rs` - Economic Rules Only
- Credit packages and pricing
- Creator tier fee schedules
- Platform take rates
- Creator Empowerment Fund configuration
- Payout settings and KYC thresholds

### 2. `user_ledger.rs` - Client-Side Accounting Only  
- Local transaction storage
- Balance calculations (optimistic)
- Sync protocol with server
- User analytics and insights

### 3. `account.rs` - Account Entity Abstraction
- Account struct definition and constructors
- Platform account constants
- Pure entity state (no operations)

### 4. `ledger.rs` - Server-Side Authoritative Ledger
- Double-entry accounting
- Authoritative balances
- Atomic transactions
- Account management and operations
- Compliance and audit trails

## Key Components

### 1. `UserLedger` - On-Device Accounting
```rust
use ledger::user_ledger::{UserLedger, UserLedgerEntry, TransactionType};
use uuid::Uuid;
use chrono::Utc;

// Create a new ledger for a user
let mut ledger = UserLedger::new(user_id);

// Add transactions locally
let transaction = UserLedgerEntry {
    transaction_id: Uuid::new_v4(),
    counterparty_id: creator_id,
    amount_credits: 25,
    transaction_type: TransactionType::ContentUnlock,
    timestamp: Utc::now(),
    local_balance_after: 0, // Calculated automatically
    synced_to_server: false,
    content_id: Some(content_id),
    fees_applied: Some(vec![/* fee details */]),
    description: Some("Unlocked premium content".to_string()),
};

ledger.add_transaction(transaction)?;
```

### 2. Transaction Types (Sync-Safe)
**Note: P2P transfers have been removed to prevent sync complexity**

- `CreditPurchase` - User bought credits (server-authoritative)
- `ContentUnlock` - User unlocked creator content (most common)
- `CreatorCashout` - Creator cashed out credits (server-authoritative)
- `PlatformFee` - Platform fee collection (automatic)
- `BonusCredits` - Bonus credits awarded (server-authoritative)
- `Refund` - Refunds and reversals (server-authoritative)

### 3. Sync Mechanism
```rust
// Prepare sync request
let sync_request = ledger.prepare_sync_request();

// Send to server and get response
let sync_response = send_to_server(sync_request).await?;

// Process server response (server is always authoritative)
ledger.process_sync_response(sync_response)?;
```

## Why No P2P Transfers?

You were absolutely right to be concerned about P2P transfers. Here's why they're problematic:

### Sync Complexity Issues:
1. **Race Conditions**: Two users' local ledgers updating simultaneously
2. **Double Spending**: User A sends to User B, but also spends the same credits elsewhere
3. **Conflict Resolution**: When User A thinks they sent 100 credits but User B thinks they received 50
4. **Orphaned Transactions**: Network issues causing incomplete P2P transfers

### Instead, Use Server-Mediated Transfers:
```rust
// Server handles the transfer atomically
POST /api/transfer {
    from_user: user_a_id,
    to_user: user_b_id, 
    amount: 100,
    reason: "tip for great content"
}

// Both users get notifications to update their local ledgers
// Server ensures atomic debit/credit and proper fee handling
```

## Benefits for MVP

### 1. **Fast Content Unlocks**
- Instant local transaction recording
- No server round-trip for basic operations
- Better user experience

### 2. **Native Accounting Features**
```rust
// Get spending by category
let spending = ledger.get_spending_summary();
println!("Content unlocks: {} credits", spending.get(&TransactionType::ContentUnlock));

// Get transaction history
let recent_unlocks = ledger.get_transaction_history(
    Some(TransactionType::ContentUnlock), 
    Some(10) // Last 10 transactions
);
```

### 3. **Server Authority Maintained**
- **Server is always authoritative** for balances
- **Client optimistically updates** for UX
- **Sync reconciles differences** and rejects invalid transactions
- **Compliance simplified** with centralized control

### 4. **Clean Architecture**
- **Pricing logic** separate from accounting logic
- **User experience** separate from business rules  
- **Easy to test** and maintain each component

## Sync Strategy

### When to Sync
1. **App startup** - Sync any pending transactions
2. **Credit purchase** - Immediate sync required 
3. **Periodic background** - Every 5-10 minutes
4. **Manual user action** - Refresh button
5. **Before cashout** - Ensure server has latest state

### Conflict Resolution (Server Wins)
```rust
pub fn process_sync_response(&mut self, response: LedgerSyncResponse) -> Result<(), String> {
    // Remove rejected transactions (server didn't accept them)
    self.transactions.retain(|tx| !response.rejected_transactions.contains(&tx.transaction_id));
    
    // Always trust server balance (this is critical!)
    self.current_balance_credits = response.server_balance_credits;
    
    // Add missing transactions from server
    for missing_tx in response.missing_transactions {
        self.transactions.push(missing_tx);
    }
}
```

## Usage Examples

### Content Creator Unlocking Content
```rust
// User unlocks creator content locally
let unlock_tx = UserLedgerEntry {
    transaction_id: Uuid::new_v4(),
    counterparty_id: creator_id,
    amount_credits: 20,
    transaction_type: TransactionType::ContentUnlock,
    timestamp: Utc::now(),
    local_balance_after: 0,
    synced_to_server: false,
    content_id: Some(content_id),
    fees_applied: Some(vec![
        FeeApplication {
            fee_type: "Platform Fee".to_string(),
            amount_credits: 4,
            percentage_rate: Some(dec!(20.0)),
            recipient: "Platform".to_string(),
        }
    ]),
    description: Some("Premium tutorial unlock".to_string()),
};

user_ledger.add_transaction(unlock_tx)?;

// Server processes the unlock and updates creator's account
// Creator gets a server-mediated credit (not P2P)
```

### Using Pricing Model for Fee Calculations
```rust
use ledger::pricing_model::PlatformPricingConfig;

let pricing_config = PlatformPricingConfig::load();

// Calculate platform fee using pricing rules
let platform_fee = pricing_config.calculate_content_unlock_fee(
    &CreatorTier::Micro,
    unlock_amount
)?;

// Calculate empowerment fund allocation
let fund_allocation = pricing_config.calculate_empowerment_fund_allocation(platform_fee);
```

## Integration with Existing Systems

### Your Existing Infrastructure Stays:
- **Pricing rules** in `PlatformPricingConfig` 
- **Central ledger** for authoritative accounting
- **Tremendous API** for redemptions
- **Fee schedules** by creator tier

### New User Experience Layer:
- **Local transaction history** for instant feedback
- **Spending analytics** for user insights
- **Offline transaction viewing** when network is poor
- **Optimistic updates** with server reconciliation

This architecture gives you the speed and UX benefits of local accounting while maintaining the security and compliance benefits of centralized control. No P2P complexity, clean separation of concerns, and easy to implement for MVP. 
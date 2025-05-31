# Toka Ledger Action Mapping

This document describes how user-facing actions map to core ledger primitives in the Toka system, following the Toka Credit Ledger Spec.

## Core Primitives

- **Mint**: Creates new credits in a user account
- **Burn**: Destroys credits (e.g. upon redemption or exit)
- **Transfer**: Moves credits from one account to another

## Ledger Event Structure

```rust
pub enum LedgerEventKind {
    Mint {
        credits: u64,
        reason: String,
        memo: Option<String>,
    },
    Burn {
        credits: u64,
        reason: String,
        memo: Option<String>,
    },
    Transfer {
        from: String,
        to: String,
        credits: u64,
        reason: String,
        memo: Option<String>,
    },
}

pub enum EntryType {
    Debit,
    Credit,
}

pub struct LedgerEntry {
    pub account_id: String,
    pub amount: i64,
    pub event_id: Uuid,
    pub entry_type: EntryType,
}

pub struct LedgerEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub kind: LedgerEventKind,
    pub debit_entry: Option<LedgerEntry>,
    pub credit_entry: Option<LedgerEntry>,
}
```

### Field Explanation

- **reason**: A structured field indicating the purpose of the transaction (e.g., "credit_purchase", "content_unlock", "creator_payout")
- **memo**: An optional free-form field for additional human-readable context or notes

## Action to Primitive Mapping

| App Action        | LedgerEventKind | Notes                        |
|-------------------|-----------------|------------------------------|
| CreditPurchase    | Mint            | Platform → user              |
| Content Unlock    | Transfer        | User → creator               |
| Creator Payout    | Burn            | Destroy credits on exit      |
| Refund            | Mint + Burn     | Issue + negate original tx   |

- **CreditPurchase**: Platform mints credits to the user account.
- **Content Unlock**: User transfers credits to a creator.
- **Creator Payout**: Credits are burned when a creator exits.
- **Refund**: A refund is represented as a Mint (to issue credits) and a Burn (to negate the original transaction). 
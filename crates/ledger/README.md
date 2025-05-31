# Toka Ledger Crate

This crate implements a secure, double-entry, append-only credit ledger with immutable events, atomic transactions, and pluggable storage (including Write-Ahead Logging).

## Architecture

The ledger system follows a clean separation of concerns with three core modules:

### Core Modules

- **`account.rs`** - Account entity abstraction. Defines the `Account` struct and related constants.
- **`ledger.rs`** - Core ledger system that manages accounts, processes transactions, and maintains integrity.
- **`event.rs`** - Defines immutable events and their lifecycle (staged → committed/rolled back).
- **`storage.rs`** - Pluggable storage backends with Write-Ahead Logging (WAL) support.
- **`user_ledger.rs`** - Client-side local ledger for offline-capable user accounting with server sync.

### Design Principles

- **Entity Separation**: `Account` is a pure entity abstraction - the `Ledger` manages all operations.
- **Immutable Events**: All events are append-only and immutable once created.
- **Double-Entry Accounting**: Every transaction has balanced debits and credits.
- **Atomic Transactions**: Operations are staged, then committed or rolled back atomically.
- **WAL Support**: Write-Ahead Logging ensures data integrity and recovery.

## Atomic Transactions & Event Commit

**Important:** Creating an event (e.g., mint, transfer, burn) does **not** update account balances or ledger state until the event is committed via a transaction. All state changes must go through atomic transactions:

```rust
use ledger::{Ledger, MemoryStorage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger = Ledger::new();
    let mut storage = MemoryStorage::new();

    // Start a transaction
    let mut tx = ledger.begin_transaction(&mut storage);

    // Stage operations
    tx.mint("user123", 1000, "credit_purchase".to_string(), Some("Initial credit purchase".to_string()))?;
    tx.transfer("user123", "creator456", 250, "content_unlock".to_string(), Some("Premium article unlock".to_string()))?;
    tx.burn("creator456", 100, "creator_payout".to_string(), Some("Creator cashing out".to_string()))?;

    // Commit all staged events atomically
    let event_ids = tx.commit()?;

    println!("Committed events: {:?}", event_ids);
    println!("User123 balance: {}", ledger.get_account_balance("user123"));
    println!("Creator456 balance: {}", ledger.get_account_balance("creator456"));
    Ok(())
}
```

- Use a transaction (`begin_transaction`) to stage all operations.
- Call `commit()` to apply all staged events and update balances.
- After commit, balances will reflect all operations.

## Module Usage Examples

### Working with Accounts (Entity)

```rust
use ledger::account::{Account, PLATFORM_RESERVE_ACCOUNT, PLATFORM_REVENUE_ACCOUNT};

// Create accounts
let user_account = Account::new("user123".to_string());
let creator_account = Account::new_with_balance("creator456".to_string(), 500);

// Platform constants are available
println!("Platform reserve: {}", PLATFORM_RESERVE_ACCOUNT);
println!("Platform revenue: {}", PLATFORM_REVENUE_ACCOUNT);
```

### Core Ledger Operations

```rust
use ledger::{Ledger, LedgerError, MemoryStorage};

// Initialize ledger with platform accounts
let mut ledger = Ledger::new();
let mut storage = MemoryStorage::new();

// Get balances
let balance = ledger.get_account_balance("user123");
let reserve_balance = ledger.get_platform_reserve_balance();

// Validate integrity
ledger.validate_integrity()?;

// Get statistics
let total_supply = ledger.total_supply();
let credits_in_circulation = ledger.total_credits_in_circulation();
```

### Working with Events

```rust
use ledger::event::{LedgerEvent, LedgerEventKind, LedgerEntry, EntryType, EventStatus};

// Events are created through the ledger transaction system
// but you can examine their structure:

let event_kind = LedgerEventKind::Mint {
    credits: 1000,
    reason: "Credit purchase".to_string(),
    memo: Some("User onboarding package".to_string()),
};

// Events start as Staged, then become Committed
// (This is handled automatically by the transaction system)
```

## Running Example Programs

The `examples/` directory contains runnable example programs that demonstrate how to use the ledger, including:
- **basic_usage.rs**: Shows basic mint, transfer, burn, and storage operations (using transactions and commit).
- **atomic_transactions.rs**: Demonstrates atomic transactions, WAL, commit/rollback, and error handling.

### How to Run Examples

From the root of the workspace or the `ledger` crate directory, use the following command:

```sh
cargo run --example <example_name>
```

Replace `<example_name>` with the file name (without `.rs`). For example:

```sh
# Run the basic usage example
cargo run --example basic_usage

# Run the atomic transactions example
cargo run --example atomic_transactions
```

You can also list all available examples:

```sh
cargo run --example
```

### Notes
- Examples are a great way to learn the API and see best practices in action.
- You can modify or add your own examples in the `examples/` directory.
- If you encounter build errors, ensure you are in the correct directory and have all dependencies installed.

---

For more details, see the crate documentation and inline comments in each example file. 
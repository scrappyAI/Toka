//! Atomic transactions example for the Toka ledger system.
//!
//! This example demonstrates immutable events, Write-Ahead Logging (WAL),
//! and atomic transactions with commit and rollback scenarios.

use ledger::{Ledger, MemoryStorage, WALStorage, WALEntry};
use ledger::event::ReasonCode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Toka Ledger Atomic Transactions Example");
    println!("==========================================");

    // Create a new ledger and storage with WAL support
    let mut ledger = Ledger::new();
    let mut storage = MemoryStorage::new();

    println!("\n📋 Initial Setup");
    println!("   Next sequence: {}", ledger.next_sequence());
    println!("   Total events: {}", ledger.get_events().len());

    // Demonstrate atomic transaction with commit
    println!("\n✅ Atomic Transaction - Success Case");
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        
        // Stage multiple operations
        tx.mint("user123", 1000, ReasonCode::Custom("initial_mint".to_string()), Some("First mint".to_string()))?;
        tx.transfer("user123", "creator456", 250, ReasonCode::ContentUnlock, None)?;
        tx.burn("creator456", 50, ReasonCode::CreatorCashout, None)?;
        
        println!("   Staged {} operations", tx.staged_count());
        
        // Commit atomically
        let event_ids = tx.commit()?;
        println!("   ✅ Committed {} events atomically", event_ids.len());
        
        for (i, id) in event_ids.iter().enumerate() {
            println!("      Event {}: {}", i + 1, id);
        }
    }

    println!("\n📊 Ledger State After Commit");
    println!("   User123 balance: {}", ledger.get_account_balance("user123"));
    println!("   Creator456 balance: {}", ledger.get_account_balance("creator456"));
    println!("   Total supply: {}", ledger.total_supply());
    println!("   Total events: {}", ledger.get_events().len());

    // Demonstrate event immutability
    println!("\n🔒 Event Immutability");
    for (i, event) in ledger.get_events().iter().enumerate() {
        println!("   Event {}: seq={}, id={}, status={:?}", 
                 i + 1, event.sequence(), event.id(), event.status());
        println!("            kind={:?}", event.kind());
    }

    // Demonstrate atomic transaction with rollback
    println!("\n❌ Atomic Transaction - Rollback Case");
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        
        // Stage operations that will be rolled back
        tx.mint("user456", 500, ReasonCode::Custom("test_mint".to_string()), None)?;
        tx.transfer("user456", "creator789", 300, ReasonCode::Custom("test_transfer".to_string()), None)?;
        
        println!("   Staged {} operations for rollback test", tx.staged_count());
        
        // Rollback instead of commit
        tx.rollback()?;
        println!("   ❌ Rolled back transaction");
    }

    println!("\n📊 Ledger State After Rollback (unchanged)");
    println!("   User123 balance: {}", ledger.get_account_balance("user123"));
    println!("   Creator456 balance: {}", ledger.get_account_balance("creator456"));
    println!("   User456 balance: {}", ledger.get_account_balance("user456"));
    println!("   Creator789 balance: {}", ledger.get_account_balance("creator789"));
    println!("   Total supply: {}", ledger.total_supply());
    println!("   Total events: {}", ledger.get_events().len());

    // Demonstrate WAL recovery
    println!("\n🔄 WAL Recovery Simulation");
    
    // Get the next sequence number before the transaction
    let recovery_sequence = ledger.next_sequence();
    // Create a new event using transaction, then add to WAL manually for simulation
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.mint("recovery_account", 100, ReasonCode::Custom("recovery_test".to_string()), None)?;
        // Don't commit or use the staged event, just drop tx
        tx.rollback()?;
    }
    // Now create the event after tx is dropped
    use ledger::{LedgerEvent, LedgerEventKind};
    let recovery_event = LedgerEvent::new(
        recovery_sequence,
        LedgerEventKind::Mint {
            credits: 100,
            reason: ReasonCode::Custom("recovery_test".to_string()),
            memo: None,
        },
        None,
        None,
    );
    let wal_entry = WALEntry::new(recovery_event.sequence(), recovery_event.clone());
    storage.append_to_wal(wal_entry)?;
    
    println!("   Added event to WAL (sequence: {})", recovery_event.sequence());
    println!("   WAL sequence: {}", storage.get_wal_sequence());
    
    // Simulate recovery
    let recovery_events = storage.recover_from_wal()?;
    println!("   Found {} events in WAL for recovery", recovery_events.len());

    // Demonstrate ledger integrity validation
    println!("\n🔍 Ledger Integrity Validation");
    match ledger.validate_integrity() {
        Ok(()) => println!("   ✅ Ledger integrity validated successfully"),
        Err(e) => println!("   ❌ Ledger integrity validation failed: {}", e),
    }

    // Show final WAL state
    println!("\n📈 Final WAL State");
    println!("   WAL sequence: {}", storage.get_wal_sequence());
    
    // Truncate WAL to clean up
    storage.truncate_wal()?;
    println!("   ✅ WAL truncated");

    // Demonstrate error handling with insufficient funds
    println!("\n⚠️  Error Handling - Insufficient Funds");
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        
        match tx.transfer("user123", "creator456", 10000, ReasonCode::Custom("impossible_transfer".to_string()), None) {
            Ok(()) => println!("   Unexpected success"),
            Err(e) => println!("   ✅ Correctly caught error: {}", e),
        }
        
        // Even if we had other valid operations, we can rollback the entire transaction
        tx.rollback()?;
        println!("   ✅ Transaction rolled back due to error");
    }

    println!("\n📈 Final Summary");
    println!("   Total accounts: {}", ledger.get_accounts().len());
    println!("   Total events: {}", ledger.get_events().len());
    println!("   Total supply: {}", ledger.total_supply());
    println!("   Next sequence: {}", ledger.next_sequence());

    Ok(())
} 
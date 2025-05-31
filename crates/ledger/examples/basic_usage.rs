//! Basic usage example for the Toka ledger system.
//!
//! This example demonstrates proper double-entry accounting with a platform reserve,
//! credit purchase/redemption flow, and internal transfers.

use ledger::{Ledger, MemoryStorage, FileStorage, Storage};
use ledger::event::ReasonCode;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏦 Toka Credit Ledger Example");
    println!("=============================");
    println!("Demonstrating proper double-entry accounting");

    // Create a new ledger and storage
    let mut ledger = Ledger::new();
    let mut storage = MemoryStorage::new();

    println!("\n📋 Initial State");
    println!("   Platform reserve: {} credits", ledger.get_platform_reserve_balance());
    println!("   Credits in circulation: {} credits", ledger.total_credits_in_circulation());

    // Start a transaction and stage operations
    let mut tx = ledger.begin_transaction(&mut storage);

    // 1. Credit Purchase: User buys credits (Platform Reserve -> User)
    tx.mint(
        "user123", 
        1000, 
        ReasonCode::CreditPurchase, 
        Some("User purchased $10 worth of credits".to_string())
    )?;
    println!("\n✅ Staged credit purchase: 1000 credits to user123");
    println!("   (Debit: Platform Reserve, Credit: User123)");

    // 2. Content Unlock: User pays creator (User -> Creator)
    tx.transfer(
        "user123", 
        "creator456", 
        250, 
        ReasonCode::ContentUnlock,
        Some("Premium article unlock".to_string())
    )?;
    println!("✅ Staged content unlock: 250 credits from user123 to creator456");
    println!("   (Debit: User123, Credit: Creator456)");

    // 3. Creator Payout: Creator cashes out (Creator -> Platform Reserve)
    tx.burn(
        "creator456", 
        100, 
        ReasonCode::CreatorCashout,
        Some("Creator cashing out $1 worth of credits".to_string())
    )?;
    println!("✅ Staged creator payout: 100 credits from creator456");
    println!("   (Debit: Creator456, Credit: Platform Reserve)");

    // Commit all staged events atomically
    let event_ids = tx.commit()?;
    println!("\n✅ Committed {} events atomically", event_ids.len());

    // Print balances after commit
    println!("\n📊 Final Balances");
    println!("   Platform reserve: {} credits", ledger.get_platform_reserve_balance());
    println!("   User123: {} credits", ledger.get_account_balance("user123"));
    println!("   Creator456: {} credits", ledger.get_account_balance("creator456"));
    println!("   Credits in circulation: {} credits", ledger.total_credits_in_circulation());

    // Show the accounting equation
    let reserve = ledger.get_platform_reserve_balance();
    let circulation = ledger.total_credits_in_circulation() as i64;
    println!("\n🧮 Accounting Verification");
    println!("   Platform Reserve + Credits in Circulation = Total Supply");
    println!("   {} + {} = {}", reserve, circulation, reserve + circulation);
    println!("   ✅ Equation balances: {}", reserve + circulation == 1_000_000_000);

    // 4. Validate ledger integrity
    println!("\n🔍 Ledger Validation");
    ledger.validate_integrity()?;
    println!("✅ Ledger integrity validated successfully");
    println!("   - All debits equal all credits");
    println!("   - Event sequence is correct");
    println!("   - Total balances equal initial reserve");

    // 5. Demonstrate pluggable storage
    println!("\n💾 Storage Backends");

    // Memory storage
    ledger.save(&mut storage)?;
    println!("✅ Saved ledger to memory storage");

    let loaded_ledger = Ledger::load(&storage)?;
    println!("✅ Loaded ledger from memory storage");
    println!("   Loaded credits in circulation: {}", loaded_ledger.total_credits_in_circulation());

    // File storage
    let mut file_storage = FileStorage::new("ledger_backup.json");
    if file_storage.is_available() {
        ledger.save(&mut file_storage)?;
        println!("✅ Saved ledger to file storage");
        
        let file_loaded_ledger = Ledger::load(&file_storage)?;
        println!("✅ Loaded ledger from file storage");
        println!("   File loaded credits in circulation: {}", file_loaded_ledger.total_credits_in_circulation());
    } else {
        println!("⚠️  File storage not available");
    }

    // 6. Display detailed transaction flow
    println!("\n📈 Transaction Flow Summary");
    println!("   1. User bought 1000 credits (paid external $10)");
    println!("      Platform Reserve: 1,000,000,000 → 999,999,000");
    println!("      User123: 0 → 1000");
    println!();
    println!("   2. User unlocked content for 250 credits");
    println!("      User123: 1000 → 750");
    println!("      Creator456: 0 → 250");
    println!();
    println!("   3. Creator cashed out 100 credits (received external $1)");
    println!("      Creator456: 250 → 150");
    println!("      Platform Reserve: 999,999,000 → 999,999,100");
    println!();
    println!("   Net result:");
    println!("   - Platform collected $10, paid out $1 = $9 profit");
    println!("   - 900 credits remain in circulation");
    println!("   - All transactions follow double-entry accounting");

    Ok(())
} 
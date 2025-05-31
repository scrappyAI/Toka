//! Zero Investment Economic Model example for the Toka ledger system.
//!
//! This example demonstrates how the platform can operate with zero upfront investment,
//! where credits are only created when users purchase them with real money.

use ledger::{Ledger, MemoryStorage, WALStorage, EconomicStatus};
use ledger::event::ReasonCode;
use domain::account::{EXTERNAL_DEPOSITS_ACCOUNT, EXTERNAL_WITHDRAWALS_ACCOUNT};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Toka Ledger Zero Investment Model Example");
    println!("============================================");

    let mut ledger = Ledger::new();
    let mut storage = MemoryStorage::new();

    println!("\n📋 Initial State (Everything Starts at Zero)");
    let status = ledger.get_economic_status();
    println!("   Money deposits: ${:.2}", status.total_deposits as f64 / 100.0);
    println!("   Money withdrawals: ${:.2}", status.total_withdrawals as f64 / 100.0);
    println!("   Credits in circulation: {}", status.credits_in_circulation);
    println!("   Platform profit: ${:.2}", status.platform_profit as f64 / 100.0);

    // First user purchase creates the first credits in the system
    println!("\n💰 First User Purchase (Creates First Credits)");
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.purchase_credits("user1", 1000, 1000, Some("$10 purchase".to_string()))?; // $10.00 in cents
        tx.commit()?;
        println!("   User1 purchased 1000 credits for $10.00");
    }

    println!("\n📊 After First Purchase");
    let status = ledger.get_economic_status();
    println!("   Money deposits: ${:.2}", status.total_deposits as f64 / 100.0);
    println!("   Credits in circulation: {}", status.credits_in_circulation);
    println!("   User1 balance: {} credits", ledger.get_account_balance("user1"));

    // More users join the platform
    println!("\n🎯 More Users Join");
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.purchase_credits("user2", 2500, 2500, Some("$25 purchase".to_string()))?; // $25.00
        tx.commit()?;
        println!("   User2 purchased 2500 credits for $25.00");
    }

    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.purchase_credits("user3", 500, 500, Some("$5 purchase".to_string()))?; // $5.00
        tx.commit()?;
        println!("   User3 purchased 500 credits for $5.00");
    }

    println!("\n📊 Growing Economy");
    let status = ledger.get_economic_status();
    println!("   Total money in: ${:.2}", status.total_deposits as f64 / 100.0);
    println!("   Total credits in circulation: {}", status.credits_in_circulation);
    println!("   Platform has: ${:.2} in deposits", status.total_deposits as f64 / 100.0);

    // Users start using credits for content
    println!("\n🔓 Content Economy Begins");
    
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.transfer("user1", "creator1", 250, ReasonCode::ContentUnlock, Some("Premium video".to_string()))?;
        tx.commit()?;
        println!("   User1 unlocked content from Creator1 for 250 credits");
    }

    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.transfer("user2", "creator1", 500, ReasonCode::ContentUnlock, Some("Course bundle".to_string()))?;
        tx.commit()?;
        println!("   User2 unlocked content from Creator1 for 500 credits");
    }

    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.transfer("user3", "creator2", 200, ReasonCode::ContentUnlock, Some("Tutorial".to_string()))?;
        tx.commit()?;
        println!("   User3 unlocked content from Creator2 for 200 credits");
    }

    println!("\n💰 Account Balances After Trading");
    for (account_id, account) in ledger.get_accounts() {
        if account.balance > 0 && !account_id.starts_with("external:") && !account_id.starts_with("platform:") {
            println!("   {}: {} credits", account_id, account.balance);
        }
    }

    // Creator cashes out
    println!("\n💸 Creator Cashouts");
    
    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.creator_cashout("creator1", 600, 600, Some("Weekly payout".to_string()))?; // $6.00
        tx.commit()?;
        println!("   Creator1 cashed out 600 credits for $6.00");
    }

    {
        let mut tx = ledger.begin_transaction(&mut storage);
        tx.creator_cashout("creator2", 150, 150, Some("Monthly payout".to_string()))?; // $1.50
        tx.commit()?;
        println!("   Creator2 cashed out 150 credits for $1.50");
    }

    println!("\n📊 Final Economic State");
    let status = ledger.get_economic_status();
    println!("   Money received from users: ${:.2}", status.total_deposits as f64 / 100.0);
    println!("   Money paid to creators: ${:.2}", status.total_withdrawals as f64 / 100.0);
    println!("   Platform profit: ${:.2}", status.platform_profit as f64 / 100.0);
    println!("   Credits still in circulation: {}", status.credits_in_circulation);

    // Show final account balances
    println!("\n💰 Final Account Balances");
    for (account_id, account) in ledger.get_accounts() {
        if account.balance != 0 {
            if account_id.starts_with("external:") {
                println!("   {}: ${:.2}", account_id, account.balance as f64 / 100.0);
            } else if !account_id.starts_with("platform:") {
                println!("   {}: {} credits", account_id, account.balance);
            }
        }
    }

    // Economic analysis
    println!("\n📈 Economic Analysis");
    let total_user_spend = 1000 + 2500 + 500; // $40 total
    let total_creator_payout = 600 + 150; // $7.50 total
    let platform_profit = total_user_spend - total_creator_payout; // $32.50
    println!("   Users spent: ${:.2}", total_user_spend as f64 / 100.0);
    println!("   Creators received: ${:.2}", total_creator_payout as f64 / 100.0);
    println!("   Platform keeps: ${:.2}", platform_profit as f64 / 100.0);
    println!("   Platform profit margin: {:.1}%", (platform_profit as f64 / total_user_spend as f64) * 100.0);

    // Validate system integrity
    println!("\n🔍 System Validation");
    match ledger.validate_integrity() {
        Ok(()) => println!("   ✅ Ledger integrity validated"),
        Err(e) => println!("   ❌ Integrity validation failed: {}", e),
    }

    match ledger.validate_economic_integrity() {
        Ok(()) => println!("   ✅ Economic integrity validated"),
        Err(e) => println!("   ❌ Economic validation failed: {}", e),
    }

    println!("\n🎯 Key Benefits of Zero Investment Model");
    println!("   • No upfront capital required");
    println!("   • System grows organically with user demand");
    println!("   • All credits backed by real money received");
    println!("   • Platform profit comes from facilitating value exchange");
    println!("   • Creators can cash out earned credits immediately");
    println!("   • Simple and sustainable economic model");

    println!("\n📋 Transaction Summary");
    println!("   Total events: {}", ledger.get_events().len());
    for (i, event) in ledger.get_events().iter().enumerate() {
        match event.kind() {
            ledger::LedgerEventKind::Mint { credits, reason, .. } => {
                println!("   {}: PURCHASE {} credits (reason: {:?})", i + 1, credits, reason);
            },
            ledger::LedgerEventKind::Transfer { from, to, credits, reason, .. } => {
                if from.starts_with("external:") && to.starts_with("external:") {
                    println!("   {}: CASHOUT money transfer (reason: {:?})", i + 1, reason);
                } else {
                    println!("   {}: TRANSFER {} credits from {} to {} (reason: {:?})", 
                        i + 1, credits, from, to, reason);
                }
            },
            _ => {}
        }
    }

    Ok(())
} 
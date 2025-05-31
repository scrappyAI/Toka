//! Example demonstrating the complete content unlock flow with pricing.

use std::collections::HashMap;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use uuid::Uuid;

use domain::{
    pricing_model::{
        CreditPackage, DefaultPricingPolicy, PricingResult, TierPricingConfig,
        CreatorTier, PlatformConfig, ProcessingResult, TransactionDetails,
    },
    account::Account,
};

// Note: In a real implementation, these would be obtained from the ledger
// This example focuses on the pricing and domain logic only
fn create_mock_accounts() -> HashMap<String, Account> {
    let mut accounts = HashMap::new();
    
    // Creator account
    accounts.insert(
        "creator_alice".to_string(), 
        Account::new_with_balance("creator_alice".to_string(), 0)
    );
    
    // Platform accounts  
    accounts.insert(
        "platform:reserve".to_string(),
        Account::new_with_balance("platform:reserve".to_string(), 1000000)
    );
    
    accounts.insert(
        "platform:revenue".to_string(),
        Account::new_with_balance("platform:revenue".to_string(), 0)
    );
    
    accounts
}

fn main() {
    println!("🎯 Toka Content Unlock Flow Example");
    println!("=====================================\\n");

    // Initialize accounts (mock data for this example)
    let accounts = create_mock_accounts();
    
    // Setup pricing policy
    let pricing_policy = DefaultPricingPolicy::new(
        PlatformConfig {
            platform_fee_percentage: dec!(15.0),
            empowerment_fund_percentage: dec!(5.0),
            min_payout_threshold_credits: 1000,
            kyc_required_threshold_credits: 5000,
        }
    );

    // Creator configuration
    let creator_config = TierPricingConfig {
        tier: CreatorTier::Standard,
        per_unlock_fee_credits: 50,
        processing_fee_percentage: dec!(2.5),
        max_daily_processing_fee_credits: 500,
    };

    println!("💰 Credit Package Purchase");
    println!("---------------------------");
    
    let starter_pack = CreditPackage {
        id: "starter".to_string(),
        credits: 1000,
        price_usd: dec!(9.99),
        bonus_credits: 100,
        description: "Perfect for getting started".to_string(),
    };

    println!("📦 Package: {} - {} credits (+ {} bonus) for ${}", 
        starter_pack.description, starter_pack.credits, starter_pack.bonus_credits, starter_pack.price_usd);

    let user_credits = starter_pack.credits + starter_pack.bonus_credits;
    println!("✅ User now has {} credits\\n", user_credits);

    println!("🔓 Content Unlock Transaction");
    println!("------------------------------");
    
    let content_price_credits = 200;
    let creator_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap();
    
    println!("📄 Content price: {} credits", content_price_credits);
    println!("👤 Creator: {}", creator_id);

    // Calculate pricing breakdown
    match pricing_policy.calculate_unlock_pricing(content_price_credits, &creator_config) {
        Ok(pricing_result) => {
            println!("\\n💡 Pricing Breakdown:");
            println!("   Content Price: {} credits", pricing_result.content_price_credits);
            println!("   Platform Fee ({}%): {} credits", 
                pricing_policy.platform_config.platform_fee_percentage, 
                pricing_result.platform_fee_credits);
            println!("   Creator Processing Fee: {} credits", pricing_result.creator_processing_fee_credits);
            println!("   Empowerment Fund ({}%): {} credits", 
                pricing_policy.platform_config.empowerment_fund_percentage,
                pricing_result.empowerment_fund_credits);
            println!("   Creator Net Earnings: {} credits", pricing_result.creator_net_credits);
            println!("   TOTAL USER COST: {} credits", pricing_result.total_user_cost_credits);

            // Simulate the transaction processing
            let transaction_details = TransactionDetails {
                content_id: Uuid::new_v4(),
                creator_id,
                user_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
            };

            match pricing_policy.process_content_unlock(pricing_result, transaction_details) {
                Ok(processing_result) => {
                    println!("\\n✅ Transaction Processed Successfully!");
                    println!("   Transaction ID: {}", processing_result.transaction_id);
                    println!("   Ledger entries created: {}", processing_result.ledger_entries.len());
                    
                    for (i, entry) in processing_result.ledger_entries.iter().enumerate() {
                        println!("   Entry {}: {} {} credits from {} to {}", 
                            i + 1,
                            if entry.entry_type.to_string() == "Credit" { "Credit" } else { "Debit" },
                            entry.amount_credits,
                            entry.from_account_id,
                            entry.to_account_id
                        );
                    }

                    let remaining_credits = user_credits - pricing_result.total_user_cost_credits;
                    println!("\\n🎉 Content unlocked! User has {} credits remaining.", remaining_credits);
                }
                Err(e) => {
                    println!("❌ Transaction failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Pricing calculation failed: {}", e);
        }
    }

    println!("\\n🏁 Example completed!");
} 
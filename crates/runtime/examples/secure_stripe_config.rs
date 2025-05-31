//! Example: Secure Stripe Configuration
//!
//! This example demonstrates how to securely load and validate Stripe configuration
//! from environment variables, following security best practices.
//!
//! To run this example:
//! 1. Set up your .env file (see CREDENTIALS_SETUP.md)
//! 2. Run: cargo run --example secure_stripe_config

use runtime::{RuntimeConfig, StripeClient, Environment};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Toka Runtime - Secure Stripe Configuration Example");
    println!("====================================================\n");

    // Load configuration from environment variables
    println!("📋 Loading configuration from environment...");
    let config = match RuntimeConfig::from_env() {
        Ok(config) => {
            println!("✅ Configuration loaded successfully!");
            config
        }
        Err(e) => {
            eprintln!("❌ Failed to load configuration: {}", e);
            eprintln!("\n💡 Make sure you have set up your .env file.");
            eprintln!("   See CREDENTIALS_SETUP.md for instructions.");
            return Err(e.into());
        }
    };

    // Display environment information (without sensitive data)
    println!("\n🌍 Environment Information:");
    println!("   Environment: {:?}", config.environment);
    println!("   Test Mode: {}", config.stripe.test_mode);
    println!("   Server: {}:{}", config.server.host, config.server.port);
    println!("   Webhook Base: {}", config.server.webhook_base_url);

    // Validate API key format (without exposing the key)
    let api_key_type = if config.stripe.api_key.starts_with("sk_test_") {
        "Test Key"
    } else if config.stripe.api_key.starts_with("sk_live_") {
        "Live Key"
    } else {
        "Unknown Key Format"
    };
    println!("   API Key Type: {}", api_key_type);

    // Create Stripe client with secure configuration
    println!("\n🔧 Creating Stripe client...");
    let stripe_client = StripeClient::new(config.stripe)?;
    println!("✅ Stripe client created successfully!");

    // Demonstrate test mode safety
    if stripe_client.is_test_mode() {
        println!("\n🧪 Running in TEST MODE - safe for development");
        
        // Example: Create a test payment intent
        println!("\n💳 Testing payment intent creation...");
        match stripe_client.create_payment_intent("test_user_123", 1000, "usd") {
            Ok(payment_intent_id) => {
                println!("✅ Mock payment intent created: {}", payment_intent_id);
            }
            Err(e) => {
                println!("⚠️  Payment intent creation not yet implemented: {}", e);
            }
        }

        // Example: Create a test payout
        println!("\n💰 Testing payout creation...");
        match stripe_client.create_payout("test_creator_456", 500, "usd") {
            Ok(payout_id) => {
                println!("✅ Mock payout created: {}", payout_id);
            }
            Err(e) => {
                println!("⚠️  Payout creation not yet implemented: {}", e);
            }
        }
    } else {
        println!("\n🚨 PRODUCTION MODE - Live transactions will be processed!");
        println!("   Be careful with any test operations.");
    }

    // Show next steps
    println!("\n🚀 Next Steps:");
    println!("   1. Implement actual Stripe API calls in stripe_integration.rs");
    println!("   2. Add HTTP client (reqwest) for API communication");
    println!("   3. Implement webhook signature verification");
    println!("   4. Build server endpoints for payment flows");
    println!("   5. Integrate with your ledger for credit management");

    println!("\n✨ Configuration and security setup complete!");
    Ok(())
} 
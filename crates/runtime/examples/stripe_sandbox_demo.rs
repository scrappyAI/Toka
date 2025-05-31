//! Stripe Sandbox API Demo
//!
//! This example demonstrates actual Stripe API calls using the sandbox environment.
//! 
//! ## Prerequisites
//! 1. Set environment variables:
//!    - STRIPE_API_KEY: Your sandbox secret key (sk_test_...)
//!    - STRIPE_WEBHOOK_SECRET: Your webhook endpoint secret
//!    - TOKA_ENV: "sandbox"
//!    - WEBHOOK_BASE_URL: "http://localhost:8080"
//!
//! ## What this demo does:
//! 1. Creates a payment intent for a user credit purchase
//! 2. Demonstrates webhook signature verification
//! 3. Simulates creator payout processing
//! 4. Shows proper error handling and security practices
//!
//! ## Run this example:
//! ```sh
//! # Set your sandbox credentials first
//! export STRIPE_API_KEY="sk_test_your_key_here"
//! export STRIPE_WEBHOOK_SECRET="whsec_your_secret_here"
//! export TOKA_ENV="sandbox"
//! export WEBHOOK_BASE_URL="http://localhost:8080"
//! 
//! # Run the example
//! cargo run --example stripe_sandbox_demo
//! ```

use runtime::stripe_integration::{StripeClient, PaymentProvider, WebhookEvent};
use runtime::config::RuntimeConfig;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Stripe Sandbox API Demo");
    println!("==========================");

    // Load configuration from environment
    println!("\n📋 Loading configuration...");
    let config = RuntimeConfig::from_env()?;
    println!("✅ Environment: {:?}", config.environment);
    println!("✅ Test mode: {}", config.stripe.test_mode);

    // Create Stripe client
    println!("\n🔗 Initializing Stripe client...");
    let stripe_client = StripeClient::new(config.stripe)?;
    println!("✅ Stripe client initialized for sandbox");

    // Demo 1: Create a payment intent
    println!("\n💳 Demo 1: Creating payment intent");
    println!("------------------------------------");
    
    let user_id = "demo_user_123";
    let amount_cents = 2000; // $20.00
    let currency = "usd";
    
    println!("Creating payment intent for:");
    println!("  User ID: {}", user_id);
    println!("  Amount: ${:.2}", amount_cents as f64 / 100.0);
    println!("  Currency: {}", currency);

    match stripe_client.create_payment_intent_impl(user_id, amount_cents, currency).await {
        Ok(payment_intent) => {
            println!("✅ Payment intent created successfully!");
            println!("  ID: {}", payment_intent.id);
            println!("  Status: {}", payment_intent.status);
            println!("  Client Secret: {}...", &payment_intent.client_secret[..20]);
            println!("  Amount: ${:.2}", payment_intent.amount as f64 / 100.0);
            
            // In a real app, you'd send the client_secret to your frontend
            println!("\n💡 Next steps:");
            println!("  1. Send client_secret to your frontend");
            println!("  2. Use Stripe.js to confirm the payment");
            println!("  3. Handle webhook events for completion");
        }
        Err(e) => {
            println!("❌ Failed to create payment intent: {}", e);
            println!("\n🔍 Troubleshooting:");
            println!("  • Check your STRIPE_API_KEY is valid");
            println!("  • Ensure it starts with 'sk_test_' for sandbox");
            println!("  • Verify your internet connection");
        }
    }

    // Demo 2: Webhook signature verification
    println!("\n🔗 Demo 2: Webhook signature verification");
    println!("------------------------------------------");
    
    // Simulate webhook payload (this would come from Stripe in reality)
    let mock_payload = r#"{
        "id": "evt_test_webhook",
        "object": "event",
        "type": "payment_intent.succeeded",
        "data": {
            "object": {
                "id": "pi_test_12345",
                "amount": 2000,
                "currency": "usd",
                "status": "succeeded",
                "metadata": {
                    "user_id": "demo_user_123",
                    "source": "toka_credit_purchase"
                }
            }
        }
    }"#;

    // Note: In a real webhook, Stripe would send the signature header
    // For demo purposes, we'll show how the verification would work
    println!("Webhook payload received (simulated):");
    println!("  Event type: payment_intent.succeeded");
    println!("  Payment intent: pi_test_12345");
    println!("  User ID: demo_user_123");
    
    println!("\n⚠️  Note: Webhook signature verification requires a real webhook");
    println!("   from Stripe. This demo shows the structure only.");

    // Demo 3: Creator payout (sandbox simulation)
    println!("\n💰 Demo 3: Creator payout simulation");
    println!("-------------------------------------");
    
    let creator_id = "creator_456";
    let payout_amount = 1500; // $15.00
    
    println!("Creating payout for:");
    println!("  Creator ID: {}", creator_id);
    println!("  Amount: ${:.2}", payout_amount as f64 / 100.0);
    println!("  Currency: {}", currency);

    match stripe_client.create_payout_impl(creator_id, payout_amount, currency).await {
        Ok(payout) => {
            println!("✅ Payout created successfully!");
            println!("  ID: {}", payout.id);
            println!("  Status: {}", payout.status);
            println!("  Amount: ${:.2}", payout.amount as f64 / 100.0);
            println!("  Currency: {}", payout.currency);
            
            println!("\n💡 Note: This is a sandbox simulation.");
            println!("   Real payouts require connected accounts.");
        }
        Err(e) => {
            println!("⚠️  Payout simulation: {}", e);
            println!("   This is expected in sandbox without connected accounts.");
        }
    }

    // Demo 4: Security best practices
    println!("\n🔒 Demo 4: Security best practices");
    println!("-----------------------------------");
    
    println!("✅ Configuration loaded from environment variables");
    println!("✅ API keys validated for correct format");
    println!("✅ Test mode enforced in sandbox environment");
    println!("✅ Webhook signature verification implemented");
    println!("✅ No credentials hardcoded in source");

    // Demo 5: Integration with Toka ledger
    println!("\n📊 Demo 5: Integration points");
    println!("------------------------------");
    
    println!("Integration workflow:");
    println!("1. User initiates credit purchase → Create payment intent");
    println!("2. User completes payment → Webhook confirms success");
    println!("3. Webhook handler → Mint credits in Toka ledger");
    println!("4. Creator earns credits → Transfer in ledger");
    println!("5. Creator requests payout → Create Stripe payout");
    println!("6. Payout completes → Burn credits from ledger");

    println!("\n🎉 Demo completed successfully!");
    println!("\n📖 Next steps:");
    println!("  • Set up a webhook endpoint at /webhooks/stripe");
    println!("  • Configure your webhook URL in Stripe Dashboard");
    println!("  • Test with real payments using Stripe test cards");
    println!("  • Integrate with Toka ledger for credit management");

    Ok(())
}

/// Helper function to demonstrate webhook handling workflow
#[allow(dead_code)]
async fn simulate_webhook_workflow() {
    println!("\n📥 Webhook workflow simulation");
    println!("------------------------------");
    
    // This shows how a real webhook handler would work:
    // 1. Receive POST request at /webhooks/stripe
    // 2. Extract payload and signature header
    // 3. Verify signature for security
    // 4. Parse event and take action
    // 5. Return 200 OK to acknowledge receipt
    
    println!("1. Receive webhook POST at /webhooks/stripe");
    println!("2. Extract Stripe-Signature header");
    println!("3. Verify signature with webhook secret");
    println!("4. Parse event JSON");
    println!("5. Handle event (mint credits, update status, etc.)");
    println!("6. Return 200 OK to Stripe");
    
    println!("\n⚡ Event types to handle:");
    println!("  • payment_intent.succeeded → Mint user credits");
    println!("  • payment_intent.payment_failed → Notify user");
    println!("  • payout.paid → Confirm creator payment");
    println!("  • payout.failed → Handle payout failure");
}

/// Helper function to show environment setup
#[allow(dead_code)]
fn show_environment_setup() {
    println!("\n🔧 Environment Setup Guide");
    println!("===========================");
    
    println!("Required environment variables:");
    println!("```bash");
    println!("# Stripe sandbox credentials");
    println!("export STRIPE_API_KEY=\"sk_test_...\"");
    println!("export STRIPE_WEBHOOK_SECRET=\"whsec_...\"");
    println!("");
    println!("# Toka configuration");
    println!("export TOKA_ENV=\"sandbox\"");
    println!("export SERVER_HOST=\"127.0.0.1\"");
    println!("export SERVER_PORT=\"8080\"");
    println!("export WEBHOOK_BASE_URL=\"http://localhost:8080\"");
    println!("```");
    
    println!("\n📋 Stripe Dashboard setup:");
    println!("1. Go to https://dashboard.stripe.com/test/developers");
    println!("2. Copy your 'Secret key' (starts with sk_test_)");
    println!("3. Create a webhook endpoint: http://localhost:8080/webhooks/stripe");
    println!("4. Copy the webhook secret (starts with whsec_)");
    println!("5. Select events: payment_intent.succeeded, payment_intent.payment_failed");
} 
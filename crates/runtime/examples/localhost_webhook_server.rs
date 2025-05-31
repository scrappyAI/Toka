//! Localhost Webhook Server for Stripe Integration
//!
//! This example demonstrates a localhost server that can receive and handle
//! Stripe webhooks with proper signature verification and ledger integration.
//!
//! ## Prerequisites
//! 1. Set environment variables (same as stripe_sandbox_demo)
//! 2. Install ngrok or similar for webhook testing: `npm install -g ngrok`
//! 3. Run this server, then expose it via ngrok
//!
//! ## Setup Steps
//! ```sh
//! # Terminal 1: Start the webhook server
//! cargo run --example localhost_webhook_server
//!
//! # Terminal 2: Expose localhost to internet for webhook testing
//! ngrok http 8080
//!
//! # Copy the ngrok URL and add /webhooks/stripe to your Stripe Dashboard
//! ```
//!
//! ## Security Features
//! - Webhook signature verification using HMAC SHA256
//! - Environment-based configuration
//! - Proper error handling and logging
//! - Integration with Toka ledger for credit management

use runtime::stripe_integration::{StripeClient, WebhookEvent};
use runtime::config::RuntimeConfig;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use tokio::signal;

/// Simple HTTP server for handling webhooks
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Toka Webhook Server");
    println!("===============================");

    // Load configuration
    let config = RuntimeConfig::from_env()?;
    println!("✅ Configuration loaded");
    println!("   Environment: {:?}", config.environment);
    println!("   Server: {}:{}", config.server.host, config.server.port);
    println!("   Test mode: {}", config.stripe.test_mode);

    // Create Stripe client for webhook verification
    let stripe_client = StripeClient::new(config.stripe.clone())?;
    println!("✅ Stripe client initialized");

    // Build server address
    let addr: SocketAddr = format!("{}:{}", config.server.host, config.server.port)
        .parse()
        .expect("Invalid server address");

    println!("\n🌐 Starting HTTP server on http://{}", addr);
    println!("📝 Webhook endpoint: http://{}/webhooks/stripe", addr);
    
    if config.stripe.test_mode {
        println!("\n🔗 For webhook testing with ngrok:");
        println!("   1. Install ngrok: npm install -g ngrok");
        println!("   2. Run: ngrok http {}", config.server.port);
        println!("   3. Copy the https URL from ngrok");
        println!("   4. Add to Stripe Dashboard: <ngrok-url>/webhooks/stripe");
        println!("   5. Select events: payment_intent.succeeded, payment_intent.payment_failed");
    }

    // Start the server
    let make_svc = hyper::service::make_service_fn(move |_conn| {
        let stripe_client = stripe_client.clone();
        async move {
            Ok::<_, Infallible>(hyper::service::service_fn(move |req| {
                handle_request(req, stripe_client.clone())
            }))
        }
    });

    let server = hyper::Server::bind(&addr).serve(make_svc);

    println!("\n✅ Server running! Press Ctrl+C to stop.");
    
    // Run server with graceful shutdown
    let graceful = server.with_graceful_shutdown(shutdown_signal());
    
    if let Err(e) = graceful.await {
        eprintln!("❌ Server error: {}", e);
    }

    println!("\n👋 Server shut down gracefully");
    Ok(())
}

/// Handle incoming HTTP requests
async fn handle_request(
    req: hyper::Request<hyper::Body>,
    stripe_client: StripeClient,
) -> Result<hyper::Response<hyper::Body>, Infallible> {
    let method = req.method();
    let path = req.uri().path();

    println!("\n📥 {} {}", method, path);

    match (method, path) {
        // Health check endpoint
        (&hyper::Method::GET, "/") | (&hyper::Method::GET, "/health") => {
            Ok(create_response(200, "🚀 Toka Webhook Server is running!"))
        }

        // Stripe webhook endpoint
        (&hyper::Method::POST, "/webhooks/stripe") => {
            handle_stripe_webhook(req, stripe_client).await
        }

        // 404 for everything else
        _ => {
            println!("❌ Not found: {} {}", method, path);
            Ok(create_response(404, "Not Found"))
        }
    }
}

/// Handle Stripe webhook with signature verification
async fn handle_stripe_webhook(
    req: hyper::Request<hyper::Body>,
    stripe_client: StripeClient,
) -> Result<hyper::Response<hyper::Body>, Infallible> {
    println!("🔗 Processing Stripe webhook...");

    // Extract signature header
    let signature = match req.headers().get("stripe-signature") {
        Some(sig) => match sig.to_str() {
            Ok(s) => s,
            Err(_) => {
                println!("❌ Invalid signature header format");
                return Ok(create_response(400, "Invalid signature header"));
            }
        },
        None => {
            println!("❌ Missing Stripe-Signature header");
            return Ok(create_response(400, "Missing signature header"));
        }
    };

    // Read request body
    let body_bytes = match hyper::body::to_bytes(req.into_body()).await {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("❌ Failed to read request body: {}", e);
            return Ok(create_response(400, "Failed to read body"));
        }
    };

    // Verify webhook signature and parse event
    match stripe_client.handle_webhook(&body_bytes, signature) {
        Ok(event) => {
            println!("✅ Webhook signature verified");
            
            // Process the event
            match process_webhook_event(event).await {
                Ok(_) => {
                    println!("✅ Webhook processed successfully");
                    Ok(create_response(200, "OK"))
                }
                Err(e) => {
                    println!("❌ Failed to process webhook: {}", e);
                    Ok(create_response(500, "Processing failed"))
                }
            }
        }
        Err(e) => {
            println!("❌ Webhook verification failed: {}", e);
            Ok(create_response(400, "Signature verification failed"))
        }
    }
}

/// Process webhook events and integrate with Toka ledger
async fn process_webhook_event(event: WebhookEvent) -> Result<(), Box<dyn std::error::Error>> {
    match event {
        WebhookEvent::PaymentIntentSucceeded { 
            payment_intent_id, 
            user_id, 
            amount_cents 
        } => {
            println!("💳 Payment succeeded!");
            println!("   Payment Intent: {}", payment_intent_id);
            println!("   User: {}", user_id);
            println!("   Amount: ${:.2}", amount_cents as f64 / 100.0);

            // TODO: Integrate with Toka ledger
            // Example integration:
            // 1. Create a ledger transaction
            // 2. Mint credits for the user
            // 3. Update user's credit balance
            // 4. Log the transaction for audit

            println!("🎯 Next steps:");
            println!("   • Mint {} credits for user {}", amount_cents, user_id);
            println!("   • Update user balance in database");
            println!("   • Send confirmation email/notification");
            println!("   • Log transaction for audit trail");

            // Simulate ledger integration
            simulate_ledger_mint(&user_id, amount_cents).await?;
        }

        WebhookEvent::PaymentIntentFailed { 
            payment_intent_id, 
            user_id 
        } => {
            println!("❌ Payment failed!");
            println!("   Payment Intent: {}", payment_intent_id);
            println!("   User: {}", user_id);

            println!("🎯 Next steps:");
            println!("   • Notify user of payment failure");
            println!("   • Log failed attempt");
            println!("   • Possibly retry or suggest alternative payment");

            // Handle payment failure
            simulate_payment_failure_handling(&user_id, &payment_intent_id).await?;
        }

        WebhookEvent::PayoutSucceeded { 
            payout_id, 
            creator_id, 
            amount_cents 
        } => {
            println!("💰 Payout succeeded!");
            println!("   Payout ID: {}", payout_id);
            println!("   Creator: {}", creator_id);
            println!("   Amount: ${:.2}", amount_cents as f64 / 100.0);

            // TODO: Integrate with Toka ledger
            println!("🎯 Next steps:");
            println!("   • Burn {} credits from creator {}", amount_cents, creator_id);
            println!("   • Update creator balance");
            println!("   • Send payout confirmation");

            simulate_ledger_burn(&creator_id, amount_cents).await?;
        }

        WebhookEvent::PayoutFailed { 
            payout_id, 
            creator_id 
        } => {
            println!("❌ Payout failed!");
            println!("   Payout ID: {}", payout_id);
            println!("   Creator: {}", creator_id);

            println!("🎯 Next steps:");
            println!("   • Notify creator of payout failure");
            println!("   • Investigate issue");
            println!("   • Retry payout if possible");

            simulate_payout_failure_handling(&creator_id, &payout_id).await?;
        }
    }

    Ok(())
}

/// Simulate minting credits in the Toka ledger
async fn simulate_ledger_mint(user_id: &str, amount_cents: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!("🏦 Simulating ledger mint...");
    
    // In a real implementation, this would:
    // 1. Create a ledger transaction
    // 2. Mint credits for the user
    // 3. Record the transaction
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("   ✅ Minted {} credits for user {}", amount_cents, user_id);
    println!("   📝 Transaction recorded in ledger");
    
    Ok(())
}

/// Simulate burning credits from the Toka ledger
async fn simulate_ledger_burn(creator_id: &str, amount_cents: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔥 Simulating ledger burn...");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    println!("   ✅ Burned {} credits from creator {}", amount_cents, creator_id);
    println!("   📝 Transaction recorded in ledger");
    
    Ok(())
}

/// Handle payment failure
async fn simulate_payment_failure_handling(user_id: &str, payment_intent_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📧 Simulating failure notification...");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    println!("   ✅ Notified user {} about failed payment {}", user_id, payment_intent_id);
    
    Ok(())
}

/// Handle payout failure
async fn simulate_payout_failure_handling(creator_id: &str, payout_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📧 Simulating payout failure handling...");
    
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    println!("   ✅ Notified creator {} about failed payout {}", creator_id, payout_id);
    
    Ok(())
}

/// Create an HTTP response
fn create_response(status: u16, body: &str) -> hyper::Response<hyper::Body> {
    hyper::Response::builder()
        .status(status)
        .header("content-type", "text/plain")
        .body(hyper::Body::from(body.to_string()))
        .unwrap()
}

/// Wait for shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("\n🛑 Shutdown signal received");
} 
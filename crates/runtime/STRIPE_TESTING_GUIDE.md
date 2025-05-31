# Stripe Sandbox Testing Guide

This guide walks you through testing the Toka Stripe integration in the sandbox environment using localhost and real Stripe API calls.

## 🚀 Quick Start

### 1. Prerequisites

- Rust and Cargo installed
- A Stripe account (free to create)
- ngrok installed for webhook testing: `npm install -g ngrok`

### 2. Get Your Stripe Sandbox Keys

1. Go to [Stripe Dashboard](https://dashboard.stripe.com/test/developers)
2. Copy your **Secret key** (starts with `sk_test_`)
3. Create a webhook endpoint (we'll set this up later)
4. Copy the **Webhook secret** (starts with `whsec_`)

### 3. Set Environment Variables

Create a `.env` file in the `crates/runtime` directory:

```bash
# Stripe sandbox credentials
STRIPE_API_KEY="sk_test_your_key_here"
STRIPE_WEBHOOK_SECRET="whsec_your_secret_here"

# Toka configuration
TOKA_ENV="sandbox"
SERVER_HOST="127.0.0.1"
SERVER_PORT="8080"
WEBHOOK_BASE_URL="http://localhost:8080"
```

### 4. Load Environment Variables

```bash
# Source the environment file
source crates/runtime/.env

# Or export manually
export STRIPE_API_KEY="sk_test_..."
export STRIPE_WEBHOOK_SECRET="whsec_..."
export TOKA_ENV="sandbox"
export SERVER_HOST="127.0.0.1"
export SERVER_PORT="8080"
export WEBHOOK_BASE_URL="http://localhost:8080"
```

## 🧪 Testing Scenarios

### Scenario 1: Basic API Connection Test

Test your Stripe API connection:

```bash
cd crates/runtime
cargo run --example stripe_sandbox_demo
```

This will:
- ✅ Validate your API keys
- ✅ Create a payment intent
- ✅ Show webhook structure
- ✅ Simulate payout processing

### Scenario 2: Localhost Webhook Server

Start the webhook server to handle real Stripe events:

```bash
# Terminal 1: Start the webhook server
cd crates/runtime
cargo run --example localhost_webhook_server
```

The server will start on `http://localhost:8080` with endpoints:
- `GET /` - Health check
- `POST /webhooks/stripe` - Stripe webhook handler

### Scenario 3: Webhook Testing with ngrok

To test real webhooks from Stripe:

```bash
# Terminal 2: Expose localhost to the internet
ngrok http 8080
```

ngrok will show output like:
```
Forwarding    https://abc123.ngrok.io -> http://localhost:8080
```

### Scenario 4: Configure Stripe Webhook

1. Go to [Stripe Webhooks](https://dashboard.stripe.com/test/webhooks)
2. Click "Add endpoint"
3. Set URL to: `https://abc123.ngrok.io/webhooks/stripe` (use your ngrok URL)
4. Select events:
   - `payment_intent.succeeded`
   - `payment_intent.payment_failed`
   - `payout.paid`
   - `payout.failed`
5. Copy the webhook secret and update your environment

### Scenario 5: End-to-End Payment Test

1. Start the webhook server
2. Expose with ngrok
3. Configure Stripe webhook
4. Create a payment intent using the API demo
5. Use [Stripe test cards](https://stripe.com/docs/testing#cards) to complete payment
6. Watch webhook events in your server logs

## 📋 Test Cards for Different Scenarios

Stripe provides test cards for various scenarios:

### Successful Payments
- **Visa**: `4242424242424242`
- **Visa (debit)**: `4000056655665556`
- **Mastercard**: `5555555555554444`

### Failed Payments
- **Generic decline**: `4000000000000002`
- **Insufficient funds**: `4000000000009995`
- **Lost card**: `4000000000009987`

### Authentication Required
- **Requires 3D Secure**: `4000002500003155`

Use any future expiry date, any 3-digit CVC, and any postal code.

## 🔒 Security Best Practices

### ✅ What We're Doing Right

1. **Environment Variables**: All credentials loaded from environment, never hardcoded
2. **Webhook Verification**: HMAC SHA256 signature verification for all webhooks
3. **Test Mode Validation**: API keys validated to ensure sandbox/live mode consistency
4. **Error Handling**: Comprehensive error handling with security-focused messages
5. **HTTPS Required**: Webhooks only work over HTTPS (ngrok provides this)

### ⚠️ Production Considerations

1. **Key Rotation**: Rotate API keys regularly
2. **Webhook Endpoints**: Use dedicated domains for webhook endpoints
3. **Rate Limiting**: Implement rate limiting on webhook endpoints
4. **Logging**: Log webhook events for audit and debugging
5. **Idempotency**: Handle duplicate webhook events gracefully

## 🔧 Troubleshooting

### Common Issues and Solutions

#### "Missing STRIPE_API_KEY"
- **Problem**: Environment variable not set
- **Solution**: Check your `.env` file or export statement
- **Check**: `echo $STRIPE_API_KEY` should show your key

#### "Invalid API key format"
- **Problem**: Wrong key type for environment
- **Solution**: Ensure sandbox uses `sk_test_` and production uses `sk_live_`

#### "Webhook verification failed"
- **Problem**: Webhook secret mismatch
- **Solution**: Copy the webhook secret from Stripe Dashboard webhook settings

#### "Connection refused" when testing webhooks
- **Problem**: Server not running or ngrok not forwarding
- **Solution**: 
  1. Ensure webhook server is running
  2. Check ngrok is active and forwarding
  3. Verify webhook URL in Stripe Dashboard

#### API calls fail with authentication error
- **Problem**: Invalid or expired API key
- **Solution**: 
  1. Check API key is correct
  2. Verify it's a test key for sandbox
  3. Check key hasn't been revoked in Stripe Dashboard

## 📊 Integration with Toka Ledger

The webhook handlers are designed to integrate with the Toka ledger system:

### Payment Flow
1. **User initiates payment** → Create Stripe payment intent
2. **User completes payment** → Stripe sends webhook
3. **Webhook verified** → Mint credits in Toka ledger
4. **Credits available** → User can spend on content

### Payout Flow
1. **Creator requests payout** → Check ledger balance
2. **Sufficient credits** → Create Stripe payout
3. **Payout succeeds** → Burn credits from ledger
4. **Payout confirmed** → Update creator balance

### Code Integration Points

In the webhook handlers, you'll integrate with the ledger like this:

```rust
// In process_webhook_event function
use ledger::{Ledger, MemoryStorage};

async fn mint_credits_for_payment(user_id: &str, amount_cents: u64) -> Result<(), LedgerError> {
    let mut ledger = Ledger::new();
    let mut storage = MemoryStorage::new();
    
    let mut tx = ledger.begin_transaction(&mut storage);
    tx.mint(user_id, amount_cents, "stripe_payment".to_string(), 
           Some(format!("Credit purchase via Stripe: ${:.2}", amount_cents as f64 / 100.0)))?;
    tx.commit()?;
    
    Ok(())
}
```

## 🚀 Next Steps

1. **Test the basic demo**: Run the sandbox demo to verify API connection
2. **Start webhook server**: Get familiar with the webhook handling
3. **Test with ngrok**: Set up end-to-end webhook testing
4. **Integrate with ledger**: Add real ledger operations to webhook handlers
5. **Add frontend**: Create a payment form using Stripe.js
6. **Production setup**: Move to live keys when ready for real payments

## 📚 Additional Resources

- [Stripe API Documentation](https://stripe.com/docs/api)
- [Stripe Testing Guide](https://stripe.com/docs/testing)
- [Webhook Best Practices](https://stripe.com/docs/webhooks/best-practices)
- [Stripe CLI for Testing](https://stripe.com/docs/stripe-cli)

---

🎉 **Happy testing!** You now have a complete sandbox environment for Stripe integration with secure webhook handling and proper error management. 
# Secure Credentials Setup Guide

This guide explains how to securely configure API keys and credentials for the Toka runtime environment.

## 🔐 Security Principles

1. **Never commit credentials to version control**
2. **Use environment variables for all sensitive data**
3. **Separate sandbox and production environments**
4. **Validate credentials at startup**
5. **Use proper key rotation practices**

## 📋 Required Environment Variables

### Core Environment
```bash
TOKA_ENV=sandbox                    # Environment: development, sandbox, production
```

### Stripe Configuration
```bash
STRIPE_API_KEY=sk_test_...          # Your Stripe secret key
STRIPE_WEBHOOK_SECRET=whsec_...     # Your webhook endpoint secret
```

### Server Configuration
```bash
SERVER_HOST=127.0.0.1               # Host to bind to (optional, defaults to 127.0.0.1)
SERVER_PORT=8080                    # Port to bind to (optional, defaults to 8080)
WEBHOOK_BASE_URL=https://...        # Base URL for webhooks (required)
```

## 🚀 Setup Instructions

### 1. Get Your Stripe Credentials

#### For Sandbox/Development:
1. Go to [Stripe Dashboard](https://dashboard.stripe.com/)
2. Make sure you're in **Test mode** (toggle in the left sidebar)
3. Navigate to **Developers** → **API keys**
4. Copy your **Secret key** (starts with `sk_test_...`)
5. Create a webhook endpoint and copy the **Webhook secret** (starts with `whsec_...`)

#### For Production:
1. Go to [Stripe Dashboard](https://dashboard.stripe.com/)
2. Switch to **Live mode** (toggle in the left sidebar)
3. Navigate to **Developers** → **API keys**
4. Copy your **Secret key** (starts with `sk_live_...`)
5. Create a webhook endpoint and copy the **Webhook secret** (starts with `whsec_...`)

### 2. Set Up Environment File

1. Copy the example file:
   ```bash
   cp .env.example .env
   ```

2. Edit `.env` with your actual credentials:
   ```bash
   # For sandbox development
   TOKA_ENV=sandbox
   STRIPE_API_KEY=sk_test_51ABC...your_actual_test_key
   STRIPE_WEBHOOK_SECRET=whsec_abc...your_actual_webhook_secret
   WEBHOOK_BASE_URL=https://your-app.ngrok.io
   ```

3. **NEVER commit the `.env` file** - it's already in `.gitignore`

### 3. Environment-Specific Setup

#### Development Environment
```bash
TOKA_ENV=development
STRIPE_API_KEY=sk_test_...
WEBHOOK_BASE_URL=https://localhost.ngrok.io
```

#### Sandbox Environment
```bash
TOKA_ENV=sandbox
STRIPE_API_KEY=sk_test_...
WEBHOOK_BASE_URL=https://sandbox.yourdomain.com
```

#### Production Environment
```bash
TOKA_ENV=production
STRIPE_API_KEY=sk_live_...
WEBHOOK_BASE_URL=https://api.yourdomain.com
```

## 🔧 Local Development with Webhooks

For local development, you'll need to expose your local server to the internet for Stripe webhooks:

### Using ngrok (Recommended)
1. Install ngrok: `brew install ngrok` (macOS) or download from [ngrok.com](https://ngrok.com/)
2. Start your local server: `cargo run`
3. In another terminal, expose it: `ngrok http 8080`
4. Copy the HTTPS URL (e.g., `https://abc123.ngrok.io`)
5. Set `WEBHOOK_BASE_URL=https://abc123.ngrok.io`
6. Configure this URL in your Stripe webhook settings

## ✅ Validation

The runtime will automatically validate your configuration on startup:

- ✅ **API Key Format**: Ensures test keys for sandbox, live keys for production
- ✅ **Key Length**: Validates reasonable key lengths
- ✅ **Environment Consistency**: Ensures all configs match the environment
- ✅ **Required Fields**: Fails fast if any required credential is missing

## 🔄 Key Rotation

### Regular Rotation Schedule
- **Development**: Rotate monthly or when team members change
- **Production**: Rotate quarterly or immediately if compromised

### Rotation Steps
1. Generate new keys in Stripe Dashboard
2. Update environment variables
3. Deploy the new configuration
4. Revoke old keys in Stripe Dashboard
5. Monitor for any authentication errors

## 🚨 Security Best Practices

### DO ✅
- Use environment variables for all credentials
- Separate test and live environments completely
- Rotate keys regularly
- Monitor for unusual API activity
- Use HTTPS for all webhook endpoints
- Validate webhook signatures

### DON'T ❌
- Hardcode API keys in source code
- Commit `.env` files to version control
- Use live keys in development
- Share API keys via email or chat
- Log API keys in application logs
- Use HTTP for webhook endpoints

## 🐛 Troubleshooting

### "Missing required credential" Error
- Check that all required environment variables are set
- Verify `.env` file is in the correct directory
- Ensure no extra spaces in variable values

### "Invalid credential" Error
- Verify API key format (sk_test_ vs sk_live_)
- Check that environment matches key type
- Validate key length and format

### Webhook Verification Failed
- Ensure webhook secret is correct
- Check that webhook URL is accessible
- Verify HTTPS is used for webhook endpoints

## 📞 Support

If you encounter issues:
1. Check the validation error messages
2. Verify all environment variables are set correctly
3. Test with Stripe's test keys first
4. Check Stripe Dashboard for API logs

Remember: **Security is everyone's responsibility!** 🔐 
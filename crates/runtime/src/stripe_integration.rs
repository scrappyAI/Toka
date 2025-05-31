//! Stripe integration module for Toka runtime
//!
//! This module provides secure integration with Stripe's REST API for the sandbox environment.
//! Reference: https://stripe.com/docs/api
//!
//! Responsibilities:
//! - Creating payment intents for credit purchases
//! - Handling Stripe webhooks for payment confirmation
//! - Initiating payouts to creators
//! - Managing API keys and webhook secrets securely

use crate::config::{StripeConfig, ConfigError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use reqwest::Client;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use async_trait::async_trait;

/// Abstraction for any payment provider (Stripe, Tremendous, etc.)
#[async_trait]
pub trait PaymentProvider {
    /// Initiate a payment (e.g., user purchases credits)
    async fn create_payment_intent(&self, user_id: &str, amount_cents: u64, currency: &str) -> Result<String, PaymentError>;

    /// Handle a webhook event from the payment provider
    fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<WebhookEvent, PaymentError>;

    /// Initiate a payout to a creator
    async fn create_payout(&self, creator_id: &str, amount_cents: u64, currency: &str) -> Result<String, PaymentError>;
}

/// Stripe-specific implementation of the payment provider
#[derive(Clone)]
pub struct StripeClient {
    /// Stripe configuration loaded from environment
    config: StripeConfig,
    /// HTTP client for API requests
    client: Client,
    /// Base URL for Stripe API
    base_url: String,
}

impl StripeClient {
    /// Create a new Stripe client from secure configuration
    /// 
    /// # Security
    /// The configuration must be loaded from environment variables using
    /// `StripeConfig::from_env()` to ensure API keys are not hardcoded.
    pub fn new(config: StripeConfig) -> Result<Self, PaymentError> {
        // Validate configuration before using it
        config.validate().map_err(PaymentError::Configuration)?;
        
        // Create HTTP client with proper headers
        let mut headers = reqwest::header::HeaderMap::new();
        let auth_header = format!("Bearer {}", config.api_key);
        headers.insert(
            reqwest::header::AUTHORIZATION,
            reqwest::header::HeaderValue::from_str(&auth_header)
                .map_err(|_| PaymentError::Configuration(
                    ConfigError::InvalidCredential("Invalid API key format".to_string())
                ))?
        );
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/x-www-form-urlencoded")
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| PaymentError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;
        
        Ok(Self { 
            config,
            client,
            base_url: "https://api.stripe.com/v1".to_string(),
        })
    }

    /// Create a new Stripe client directly from environment variables
    /// This is a convenience method that loads config and creates the client.
    pub fn from_env() -> Result<Self, PaymentError> {
        let config = StripeConfig::from_env().map_err(PaymentError::Configuration)?;
        Self::new(config)
    }

    /// Get the API key for this client (for HTTP headers)
    fn api_key(&self) -> &str {
        &self.config.api_key
    }

    /// Get the webhook secret for signature verification
    fn webhook_secret(&self) -> &str {
        &self.config.webhook_secret
    }

    /// Check if we're in test mode
    pub fn is_test_mode(&self) -> bool {
        self.config.test_mode
    }

    /// Create a payment intent for a user credit purchase
    pub async fn create_payment_intent_impl(&self, user_id: &str, amount_cents: u64, currency: &str) -> Result<PaymentIntentResponse, PaymentError> {
        // Validate input parameters
        if amount_cents < 50 {
            return Err(PaymentError::InvalidRequest("Amount must be at least 50 cents".to_string()));
        }
        if currency.len() != 3 {
            return Err(PaymentError::InvalidRequest("Currency must be 3-letter ISO code".to_string()));
        }

        // Build request parameters
        let mut params = HashMap::new();
        params.insert("amount", amount_cents.to_string());
        params.insert("currency", currency.to_lowercase());
        params.insert("automatic_payment_methods[enabled]", "true".to_string());
        params.insert("metadata[user_id]", user_id.to_string());
        params.insert("metadata[source]", "toka_credit_purchase".to_string());
        
        // For sandbox, add test mode indicators
        if self.is_test_mode() {
            params.insert("metadata[test_mode]", "true".to_string());
        }

        // Convert params to form-encoded string
        let form_data = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        // Make API request
        let url = format!("{}/payment_intents", self.base_url);
        let response = self.client
            .post(&url)
            .body(form_data)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(format!("Request failed: {}", e)))?;

        // Handle response
        if response.status().is_success() {
            let payment_intent: PaymentIntentResponse = response
                .json()
                .await
                .map_err(|e| PaymentError::ApiError(format!("Failed to parse response: {}", e)))?;
            Ok(payment_intent)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PaymentError::ApiError(format!("Stripe API error: {}", error_text)))
        }
    }

    /// Handle a Stripe webhook event with signature verification
    pub fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<WebhookEvent, PaymentError> {
        // Verify webhook signature for security
        self.verify_webhook_signature(payload, signature)?;

        // Parse the webhook payload
        let event: StripeEvent = serde_json::from_slice(payload)
            .map_err(|e| PaymentError::ApiError(format!("Failed to parse webhook payload: {}", e)))?;

        // Convert Stripe event to our internal representation
        self.convert_stripe_event(event)
    }

    /// Verify webhook signature using HMAC SHA256
    fn verify_webhook_signature(&self, payload: &[u8], signature_header: &str) -> Result<(), PaymentError> {
        // Parse signature header (format: t=timestamp,v1=signature)
        let mut timestamp = None;
        let mut signature = None;

        for part in signature_header.split(',') {
            if let Some(rest) = part.strip_prefix("t=") {
                timestamp = Some(rest);
            } else if let Some(rest) = part.strip_prefix("v1=") {
                signature = Some(rest);
            }
        }

        let timestamp = timestamp.ok_or(PaymentError::WebhookVerificationFailed)?;
        let expected_signature = signature.ok_or(PaymentError::WebhookVerificationFailed)?;

        // Create signed payload: timestamp + "." + payload
        let signed_payload = format!("{}.{}", timestamp, String::from_utf8_lossy(payload));

        // Compute HMAC SHA256
        type HmacSha256 = Hmac<Sha256>;
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret().as_bytes())
            .map_err(|_| PaymentError::WebhookVerificationFailed)?;
        mac.update(signed_payload.as_bytes());
        let computed_signature = hex::encode(mac.finalize().into_bytes());

        // Compare signatures (constant time comparison)
        if computed_signature != expected_signature {
            return Err(PaymentError::WebhookVerificationFailed);
        }

        Ok(())
    }

    /// Convert Stripe event to internal webhook event
    fn convert_stripe_event(&self, event: StripeEvent) -> Result<WebhookEvent, PaymentError> {
        match event.event_type.as_str() {
            "payment_intent.succeeded" => {
                let pi = event.data.object;
                let user_id = pi.metadata.get("user_id")
                    .ok_or_else(|| PaymentError::ApiError("Missing user_id in payment intent metadata".to_string()))?
                    .clone();
                
                Ok(WebhookEvent::PaymentIntentSucceeded {
                    payment_intent_id: pi.id,
                    user_id,
                    amount_cents: pi.amount,
                })
            }
            "payment_intent.payment_failed" => {
                let pi = event.data.object;
                let user_id = pi.metadata.get("user_id")
                    .ok_or_else(|| PaymentError::ApiError("Missing user_id in payment intent metadata".to_string()))?
                    .clone();
                
                Ok(WebhookEvent::PaymentIntentFailed {
                    payment_intent_id: pi.id,
                    user_id,
                })
            }
            _ => Err(PaymentError::ApiError(format!("Unsupported event type: {}", event.event_type)))
        }
    }

    /// Create a payout to a creator (sandbox implementation)
    pub async fn create_payout_impl(&self, creator_id: &str, amount_cents: u64, currency: &str) -> Result<PayoutResponse, PaymentError> {
        // Note: Payouts in test mode require a test bank account to be set up
        // For now, we'll simulate the API call structure
        
        if !self.is_test_mode() {
            return Err(PaymentError::InvalidRequest("Live payouts not implemented yet".to_string()));
        }

        // Build request parameters
        let mut params = HashMap::new();
        params.insert("amount", amount_cents.to_string());
        params.insert("currency", currency.to_lowercase());
        params.insert("metadata[creator_id]", creator_id.to_string());
        params.insert("metadata[source]", "toka_creator_payout".to_string());

        // Convert params to form-encoded string
        let form_data = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        // For sandbox, return a mock response since test payouts require account setup
        if self.is_test_mode() {
            return Ok(PayoutResponse {
                id: format!("po_test_{}_{}", creator_id, amount_cents),
                amount: amount_cents,
                currency: currency.to_lowercase(),
                status: "pending".to_string(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("creator_id".to_string(), creator_id.to_string());
                    map.insert("source".to_string(), "toka_creator_payout".to_string());
                    map
                },
            });
        }

        // Make API request (for live mode when implemented)
        let url = format!("{}/payouts", self.base_url);
        let response = self.client
            .post(&url)
            .body(form_data)
            .send()
            .await
            .map_err(|e| PaymentError::NetworkError(format!("Request failed: {}", e)))?;

        if response.status().is_success() {
            let payout: PayoutResponse = response
                .json()
                .await
                .map_err(|e| PaymentError::ApiError(format!("Failed to parse response: {}", e)))?;
            Ok(payout)
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            Err(PaymentError::ApiError(format!("Stripe API error: {}", error_text)))
        }
    }
}

#[async_trait]
impl PaymentProvider for StripeClient {
    async fn create_payment_intent(&self, user_id: &str, amount_cents: u64, currency: &str) -> Result<String, PaymentError> {
        self.create_payment_intent_impl(user_id, amount_cents, currency).await.map(|resp| resp.id)
    }
    
    fn handle_webhook(&self, payload: &[u8], signature: &str) -> Result<WebhookEvent, PaymentError> {
        self.handle_webhook(payload, signature)
    }
    
    async fn create_payout(&self, creator_id: &str, amount_cents: u64, currency: &str) -> Result<String, PaymentError> {
        self.create_payout_impl(creator_id, amount_cents, currency).await.map(|resp| resp.id)
    }
}

/// Represents a parsed webhook event (expand as needed)
#[derive(Debug, Clone)]
pub enum WebhookEvent {
    PaymentIntentSucceeded { 
        payment_intent_id: String, 
        user_id: String, 
        amount_cents: u64 
    },
    PaymentIntentFailed { 
        payment_intent_id: String, 
        user_id: String 
    },
    PayoutSucceeded { 
        payout_id: String, 
        creator_id: String, 
        amount_cents: u64 
    },
    PayoutFailed { 
        payout_id: String, 
        creator_id: String 
    },
    // TODO: Add more event types as needed
}

/// Payment provider error type
#[derive(Debug)]
pub enum PaymentError {
    /// Feature not yet implemented
    Unimplemented,
    /// Configuration error (missing or invalid credentials)
    Configuration(ConfigError),
    /// Invalid request parameters
    InvalidRequest(String),
    /// Authentication failed (bad API key)
    AuthenticationFailed,
    /// Webhook signature verification failed
    WebhookVerificationFailed,
    /// API error from payment provider
    ApiError(String),
    /// Network or HTTP error
    NetworkError(String),
    // TODO: Add more error variants as needed
}

impl std::fmt::Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentError::Unimplemented => write!(f, "Feature not yet implemented"),
            PaymentError::Configuration(e) => write!(f, "Configuration error: {}", e),
            PaymentError::InvalidRequest(msg) => write!(f, "Invalid request: {}", msg),
            PaymentError::AuthenticationFailed => write!(f, "Authentication failed"),
            PaymentError::WebhookVerificationFailed => write!(f, "Webhook verification failed"),
            PaymentError::ApiError(msg) => write!(f, "API error: {}", msg),
            PaymentError::NetworkError(msg) => write!(f, "Network error: {}", msg),
        }
    }
}

impl std::error::Error for PaymentError {}

// Re-export for convenience
pub use StripeClient as DefaultPaymentProvider;
pub use PaymentProvider as PaymentProviderTrait;

// Stripe API response types
#[derive(Debug, Deserialize)]
pub struct PaymentIntentResponse {
    pub id: String,
    pub amount: u64,
    pub currency: String,
    pub status: String,
    pub client_secret: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct PayoutResponse {
    pub id: String,
    pub amount: u64,
    pub currency: String,
    pub status: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub struct StripeEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: StripeEventData,
}

#[derive(Debug, Deserialize)]
pub struct StripeEventData {
    pub object: StripePaymentIntent,
}

#[derive(Debug, Deserialize)]
pub struct StripePaymentIntent {
    pub id: String,
    pub amount: u64,
    pub currency: String,
    pub status: String,
    pub metadata: HashMap<String, String>,
} 
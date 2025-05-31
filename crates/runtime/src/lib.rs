//! Runtime orchestration layer for the Toka platform.
//!
//! This crate handles client-side logic including user ledger management,
//! server synchronization, and offline-capable user accounting.

// User-side ledger management and sync
pub mod user_ledger;

// Secure configuration management
pub mod config;

// Stripe integration for payment and payout flows
pub mod stripe_integration;

// Re-export key types for convenience
pub use user_ledger::{
    UserLedger, UserLedgerEntry, TransactionType, LedgerSyncRequest, LedgerSyncResponse, FeeApplication
};

pub use config::{RuntimeConfig, StripeConfig, Environment, ConfigError};
pub use stripe_integration::{PaymentProviderTrait, DefaultPaymentProvider, StripeClient, PaymentError, WebhookEvent};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
} 
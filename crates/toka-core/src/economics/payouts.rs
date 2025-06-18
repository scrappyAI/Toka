//! # Payouts
//!
//! Defines settings and supported methods for creator payouts.

use crate::currency::MicroUSD;
use serde::{Deserialize, Serialize};

/// Supported payout methods.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PayoutType {
    Stripe,
    PayPal,
    Crypto(String), // e.g., "USDC"
}

/// Settings related to creator cashouts and payouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayoutSettings {
    pub min_payout_threshold_micro_usd: MicroUSD,
    pub supported_payout_types: Vec<PayoutType>,
    // Could include KYC requirements, etc.
}

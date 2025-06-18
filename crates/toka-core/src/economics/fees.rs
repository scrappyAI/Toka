//! # Fees
//!
//! Defines fee structures and calculation logic for various platform activities.
//! This includes content unlocks, cashouts, and other monetizable events.

use crate::currency::MicroUSD;
use serde::{Deserialize, Serialize};

/// Represents the tier of a creator, which can influence fee rates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatorTier {
    Standard,
    Plus,
    Premium,
}

/// Defines the fee configuration for unlocking content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentUnlockFeeConfig {
    pub percentage: f64, // e.g., 0.20 for a 20% fee
}

/// Defines the fee configuration for cashing out earnings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashoutFeeConfig {
    pub percentage: f64,           // e.g., 0.05 for a 5% fee
    pub fixed_micro_usd: MicroUSD, // e.g., 500,000 micro-USD for a $0.50 fixed fee
}

/// A comprehensive schedule of all fees applicable on the platform,
/// potentially varying by creator tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeSchedule {
    pub content_unlock_fee: std::collections::HashMap<CreatorTier, ContentUnlockFeeConfig>,
    pub cashout_fee: std::collections::HashMap<CreatorTier, CashoutFeeConfig>,
}

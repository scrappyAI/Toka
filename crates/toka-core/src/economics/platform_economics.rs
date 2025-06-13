//! # Platform Economics
//!
//! Defines platform-wide economic policies and configurations, such as take rates
//! and special funds.

use serde::{Serialize, Deserialize};

/// Configuration for the sliding window used to calculate the platform's take rate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TakeRateSlidingWindowConfig {
    pub window_days: u32,
    pub min_transactions: u32,
}

/// Configuration for a "fairness multiplier" that might adjust earnings based on
/// certain platform goals or principles.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FairnessMultiplierConfig {
    pub is_enabled: bool,
    pub factor: f64,
}

/// Configuration for a fund that supports creators, possibly funded by a portion
/// of platform revenue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatorEmpowermentFundConfig {
    pub contribution_percentage: f64, // e.g., 0.10 for 10% of fees
} 
//! # Pricing Configuration
//!
//! Orchestrates the overall pricing and economic configuration for the platform.

use serde::{Serialize, Deserialize};
use crate::economics::{FeeSchedule, CreatorEmpowermentFundConfig, TakeRateSlidingWindowConfig};
use crate::products::CreditPackage;

/// A single, unified configuration object for all platform pricing and economic rules.
/// This can be loaded from a file (e.g., a TOML or JSON) to allow for dynamic adjustments
/// without redeploying code.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformPricingConfig {
    pub fee_schedule: FeeSchedule,
    pub empowerment_fund_config: CreatorEmpowermentFundConfig,
    pub take_rate_config: TakeRateSlidingWindowConfig,
    pub credit_packages: Vec<CreditPackage>,
} 
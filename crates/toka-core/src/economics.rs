//! # Toka Economics
//!
//! Contains the fundamental rules of value flow and financial models for the platform.
//! This includes fee structures, platform-wide economic policies, and payout rules.

pub mod fees;
pub mod payouts;
pub mod platform_economics;

// Re-export key types for convenience
pub use self::fees::{CashoutFeeConfig, ContentUnlockFeeConfig, CreatorTier, FeeSchedule};
pub use self::payouts::{PayoutSettings, PayoutType};
pub use self::platform_economics::{
    CreatorEmpowermentFundConfig, FairnessMultiplierConfig, TakeRateSlidingWindowConfig,
};

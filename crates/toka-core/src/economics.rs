//! # Toka Economics
//!
//! Contains the fundamental rules of value flow and financial models for the platform.
//! This includes fee structures, platform-wide economic policies, and payout rules.

pub mod fees;
pub mod platform_economics;
pub mod payouts;

// Re-export key types for convenience
pub use self::fees::{CreatorTier, FeeSchedule, ContentUnlockFeeConfig, CashoutFeeConfig};
pub use self::platform_economics::{CreatorEmpowermentFundConfig, FairnessMultiplierConfig, TakeRateSlidingWindowConfig};
pub use self::payouts::{PayoutSettings, PayoutType}; 
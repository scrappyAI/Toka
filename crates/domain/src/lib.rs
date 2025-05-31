//! Business domain logic for the Toka project.
//!
//! This crate contains business rules, pricing models, and domain-specific
//! logic that is separate from the core ledger accounting mechanics.

pub mod pricing_model;

// Re-export key types for convenience
pub use pricing_model::{
    PlatformPricingConfig, CreditPackage, CreditPackageTier, CreatorTier, 
    FeeSchedule, ContentUnlockFeeConfig, CashoutFeeConfig, AgentPricingConfig,
    CreatorEmpowermentFundConfig, PayoutSettings, PayoutType, FairnessMultiplierConfig,
    TakeRateSlidingWindowConfig,
    // New extensible pricing API
    CreditPackageView, PricingPolicy, DefaultPricingPolicy, PricingService
};

// Account entity - shared between server and client
pub mod account;

// Re-export key types for convenience
pub use account::{Account, PLATFORM_RESERVE_ACCOUNT, PLATFORM_REVENUE_ACCOUNT};

//! # Toka Pricing
//!
//! Handles the logic for presenting prices and product offerings to users.
//! It provides a clean API for frontends to query available products and their
//! prices, and allows for different pricing strategies via the `PricingPolicy` trait.

pub mod agent_pricing;
pub mod config;
pub mod policies;

// Re-export key types for convenience
pub use self::agent_pricing::AgentPricingConfig;
pub use self::config::PlatformPricingConfig;
pub use self::policies::{
    create_standard_pricing_service, DefaultPricingPolicy, PricingPolicy, PricingService,
};

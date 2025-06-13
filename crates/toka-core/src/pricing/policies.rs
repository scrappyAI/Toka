//! # Pricing Policies
//!
//! Defines a policy-based approach to pricing, allowing for different pricing
//! strategies to be implemented and swapped out. This promotes flexibility for
//! A/B testing, regional pricing, or dynamic adjustments.

use crate::products::{CreditPackage, CreditPackageTier, CreditPackageView};
use crate::pricing::config::PlatformPricingConfig;
use std::sync::Arc;

/// A trait that defines the contract for a pricing policy.
/// It provides methods to query available products and their details.
pub trait PricingPolicy: Send + Sync {
    /// Returns a list of all credit packages available for purchase under this policy.
    fn get_available_packages(&self) -> Vec<CreditPackageView>;
    
    /// Retrieves a specific credit package by its tier.
    fn get_package_by_tier(&self, tier: &CreditPackageTier) -> Option<&CreditPackage>;
}

/// A default implementation of the `PricingPolicy` trait that reads its
/// configuration from the `PlatformPricingConfig` object.
pub struct DefaultPricingPolicy {
    config: PlatformPricingConfig,
}

impl DefaultPricingPolicy {
    pub fn new(config: PlatformPricingConfig) -> Self {
        Self { config }
    }
}

impl PricingPolicy for DefaultPricingPolicy {
    fn get_available_packages(&self) -> Vec<CreditPackageView> {
        // In a real implementation, this would format prices and descriptions nicely.
        // Here we just map the raw data.
        self.config.credit_packages.iter().map(|p| {
            CreditPackageView {
                product_id: p.product_id,
                name: format!("{:?} Package", p.tier),
                description: format!("Get {} credits", p.credits_awarded),
                credits_awarded: p.credits_awarded,
                display_price: format!("${:.2}", p.cost_micro_usd.to_usd_decimal()),
            }
        }).collect()
    }

    fn get_package_by_tier(&self, tier: &CreditPackageTier) -> Option<&CreditPackage> {
        self.config.credit_packages.iter().find(|p| &p.tier == tier)
    }
}

/// A service that provides a clean, high-level API for accessing pricing information.
/// It uses a `PricingPolicy` trait object to allow for different strategies.
pub struct PricingService {
    policy: Arc<dyn PricingPolicy>,
}

impl PricingService {
    pub fn new(policy: Arc<dyn PricingPolicy>) -> Self {
        Self { policy }
    }

    pub fn get_available_packages(&self) -> Vec<CreditPackageView> {
        self.policy.get_available_packages()
    }

    pub fn get_package_by_tier(&self, tier: &CreditPackageTier) -> Option<&CreditPackage> {
        self.policy.get_package_by_tier(tier)
    }
}

/// A factory function to create a standard pricing service with the default policy.
pub fn create_standard_pricing_service(config: PlatformPricingConfig) -> PricingService {
    let policy = Arc::new(DefaultPricingPolicy::new(config));
    PricingService::new(policy)
} 
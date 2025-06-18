//! # Pricing and Economics Integration Tests
//!
//! This file tests the interaction between the `pricing`, `products`, and `economics`
//! modules. It ensures that the high-level `PricingService` correctly uses the
//! underlying configurations to provide product information.

#![cfg(all(feature = "pricing", feature = "economics", feature = "products"))]

use std::collections::HashMap;
use std::sync::Arc;
use toka_core::currency::MicroUSD;
use toka_core::economics::{
    CashoutFeeConfig, ContentUnlockFeeConfig, CreatorEmpowermentFundConfig, CreatorTier,
    FeeSchedule, TakeRateSlidingWindowConfig,
};
use toka_core::ids::ProductID;
use toka_core::pricing::{
    create_standard_pricing_service, DefaultPricingPolicy, PlatformPricingConfig, PricingPolicy,
};
use toka_core::products::{CreditPackage, CreditPackageTier};

/// Creates a mock `PlatformPricingConfig` for use in tests.
fn create_mock_config() -> PlatformPricingConfig {
    // --- Fee Schedule ---
    let mut content_unlock_fees = HashMap::new();
    content_unlock_fees.insert(
        CreatorTier::Standard,
        ContentUnlockFeeConfig { percentage: 0.20 },
    );
    content_unlock_fees.insert(
        CreatorTier::Plus,
        ContentUnlockFeeConfig { percentage: 0.15 },
    );

    let mut cashout_fees = HashMap::new();
    cashout_fees.insert(
        CreatorTier::Standard,
        CashoutFeeConfig {
            percentage: 0.05,
            fixed_micro_usd: MicroUSD(500_000), // $0.50
        },
    );

    let fee_schedule = FeeSchedule {
        content_unlock_fee: content_unlock_fees,
        cashout_fee: cashout_fees,
    };

    // --- Credit Packages ---
    let credit_packages = vec![
        CreditPackage {
            product_id: ProductID::new(),
            tier: CreditPackageTier::Starter,
            credits_awarded: 100,
            cost_micro_usd: MicroUSD(5_000_000), // $5.00
        },
        CreditPackage {
            product_id: ProductID::new(),
            tier: CreditPackageTier::Plus,
            credits_awarded: 550,
            cost_micro_usd: MicroUSD(25_000_000), // $25.00
        },
    ];

    PlatformPricingConfig {
        fee_schedule,
        empowerment_fund_config: CreatorEmpowermentFundConfig {
            contribution_percentage: 0.10,
        },
        take_rate_config: TakeRateSlidingWindowConfig {
            window_days: 30,
            min_transactions: 100,
        },
        credit_packages,
    }
}

#[test]
fn test_default_pricing_policy_get_available_packages() {
    let config = create_mock_config();
    let policy = DefaultPricingPolicy::new(config);

    let packages_view = policy.get_available_packages();

    assert_eq!(packages_view.len(), 2);

    // Check the starter package view
    let starter_view = packages_view
        .iter()
        .find(|p| p.name == "Starter Package")
        .unwrap();
    assert_eq!(starter_view.credits_awarded, 100);
    assert_eq!(starter_view.display_price, "$5.00");
    assert_eq!(starter_view.description, "Get 100 credits");

    // Check the plus package view
    let plus_view = packages_view
        .iter()
        .find(|p| p.name == "Plus Package")
        .unwrap();
    assert_eq!(plus_view.credits_awarded, 550);
    assert_eq!(plus_view.display_price, "$25.00");
}

#[test]
fn test_default_pricing_policy_get_package_by_tier() {
    let config = create_mock_config();
    let policy = DefaultPricingPolicy::new(config);

    // Retrieve an existing package
    let starter_package = policy
        .get_package_by_tier(&CreditPackageTier::Starter)
        .unwrap();
    assert_eq!(starter_package.credits_awarded, 100);
    assert_eq!(starter_package.cost_micro_usd, MicroUSD(5_000_000));

    // Attempt to retrieve a non-existent package
    let pro_package = policy.get_package_by_tier(&CreditPackageTier::Pro);
    assert!(pro_package.is_none());
}

#[test]
fn test_pricing_service_integration() {
    let config = create_mock_config();
    // The service can be created with the factory function
    let pricing_service = create_standard_pricing_service(config);

    // Test getting all packages through the service
    let packages_view = pricing_service.get_available_packages();
    assert_eq!(packages_view.len(), 2);
    assert!(packages_view.iter().any(|p| p.name == "Starter Package"));

    // Test getting a specific package by tier through the service
    let plus_package = pricing_service
        .get_package_by_tier(&CreditPackageTier::Plus)
        .unwrap();
    assert_eq!(plus_package.tier, CreditPackageTier::Plus);
    assert_eq!(plus_package.credits_awarded, 550);
}

#[test]
fn test_pricing_service_with_arc_policy() {
    let config = create_mock_config();
    let policy = Arc::new(DefaultPricingPolicy::new(config));
    // The service can also be created by injecting a policy directly
    let pricing_service = toka_core::pricing::PricingService::new(policy);

    let packages_view = pricing_service.get_available_packages();
    assert_eq!(packages_view.len(), 2);
}

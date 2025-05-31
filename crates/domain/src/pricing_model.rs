// crates/domain/src/pricing_model.rs

use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents the different tiers for credit packages.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreditPackageTier {
    Starter,
    Pro,
    Creator,
    Custom(String), // For bespoke packages
}

/// Defines a purchasable credit package.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditPackage {
    pub tier: CreditPackageTier,
    pub package_id: Uuid,
    pub display_name: String,
    /// Number of credits included in the package.
    pub credits_amount: u64,
    /// Price of the package in USD.
    pub price_usd: Decimal,
    /// Optional: Bonus credits offered with the package.
    pub bonus_credits: Option<u64>,
    /// Optional: Discount percentage applied to this package.
    pub discount_percentage: Option<Decimal>,
    /// Optional: Badge or label like "Most Popular".
    pub badge: Option<String>,
    /// Is this package currently available for purchase?
    pub is_active: bool,
}

/// Represents different tiers of creators on the platform.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreatorTier {
    Micro,
    MidTier,
    Top,
    Emerging,
    Custom(String), // For specific categorizations
}

/// Configuration for fees related to content unlocking or purchases.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContentUnlockFeeConfig {
    /// Percentage-based fee.
    pub percentage: Option<Decimal>,
    /// Fixed fee in USD.
    pub fixed_usd: Option<Decimal>,
    /// Describes when this fee applies (e.g., "Standard content unlock").
    pub description: Option<String>,
}

/// Configuration for fees related to creator cashouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CashoutFeeConfig {
    /// Percentage-based fee.
    pub percentage: Option<Decimal>,
    /// Fixed fee in USD.
    pub fixed_usd: Option<Decimal>,
    /// Minimum transaction amount in USD for this fee to apply.
    pub minimum_transaction_for_fee_usd: Option<Decimal>,
    /// Describes when this fee applies (e.g., "Standard cashout").
    pub description: Option<String>,
}

/// Defines the overall fee schedule for a specific creator tier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeeSchedule {
    pub tier: CreatorTier,
    /// List of applicable fees for content unlocking.
    /// Allows multiple types of content fees if needed (e.g., standard, premium).
    pub content_unlock_fees: Vec<ContentUnlockFeeConfig>,
    /// List of applicable fees for cashing out credits.
    /// Allows multiple types of cashout fees (e.g., standard, expedited).
    pub cashout_fees: Vec<CashoutFeeConfig>,
}

/// Configuration for AI agent-specific pricing and fees.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AgentPricingConfig {
    /// Additional platform fee percentage for AI-owned agent transactions.
    pub ai_owned_additional_fee_percentage: Option<Decimal>,
    /// Discount percentage for human-AI collaborative transactions.
    pub hybrid_collaboration_discount_percentage: Option<Decimal>,
}

/// Configuration for the Creator Empowerment Fund.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreatorEmpowermentFundConfig {
    /// Percentage of platform fees allocated to the fund.
    pub allocation_percentage: Decimal,
    /// Minimum platform revenue share before fund allocation kicks in (in USD).
    pub minimum_platform_revenue_share_for_allocation_usd: Option<Decimal>,
    /// Purposes for which the fund can be used.
    pub usage_guidelines: Vec<String>,
}

/// Represents different types of payouts available.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PayoutType {
    GiftCard(String), // e.g., "Amazon", "Walmart"
    PrepaidVisa,
    PayPal,
    Venmo,
    AchTransfer,
    Other(String),
}

/// Contains settings related to payout processing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PayoutSettings {
    /// Minimum amount for a creator to request a payout in credits.
    pub minimum_payout_credits: u64,
    /// Maximum amount for a single payout transaction in credits.
    pub maximum_single_payout_credits: Option<u64>,
    /// List of supported payout types.
    pub supported_payout_types: Vec<PayoutType>,
    /// Estimated processing time for payouts in days.
    pub estimated_processing_time_days: Option<u8>,
    /// KYC (Know Your Customer) threshold in USD equivalent.
    /// Payouts exceeding this value might require additional verification.
    pub kyc_threshold_usd: Option<Decimal>,
}

/// The "Fairness Multiplier" configuration.
/// This allows dynamic adjustments to pricing or fees based on certain factors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FairnessMultiplierConfig {
    pub is_enabled: bool,
    /// Factor based on user's predicted lifetime value.
    pub user_ltv_adjustment_factor: Option<Decimal>,
    /// Factor based on regional purchasing power parity.
    pub region_ppp_adjustment_factor: Option<Decimal>,
    /// Factor based on creator supply/demand dynamics.
    pub creator_supply_demand_adjustment_factor: Option<Decimal>,
}

/// Configuration for a dynamic sliding window for the platform's take rate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TakeRateSlidingWindowConfig {
    pub is_enabled: bool,
    /// Minimum allowed take rate percentage.
    pub min_take_rate_percentage: Decimal,
    /// Maximum allowed take rate percentage.
    pub max_take_rate_percentage: Decimal,
    /// The current take rate suggested by an optimizer or manual setting, before clamping.
    pub current_suggested_take_rate_percentage: Decimal,
}

/// Top-level configuration for the platform's pricing model.
/// This struct consolidates all pricing and economic settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlatformPricingConfig {
    pub credit_packages: Vec<CreditPackage>,
    pub fee_schedules: Vec<FeeSchedule>,
    pub agent_pricing: AgentPricingConfig,
    pub creator_empowerment_fund: CreatorEmpowermentFundConfig,
    pub payout_settings: PayoutSettings,
    pub fairness_multiplier_config: FairnessMultiplierConfig,
    /// Optional configuration for a dynamic take rate sliding window.
    pub take_rate_sliding_window: Option<TakeRateSlidingWindowConfig>,
    /// Default currency for the platform, e.g., "USD".
    pub default_currency: String,
    /// The platform's general take rate percentage on transactions where specific fees don't apply directly,
    /// or when the dynamic sliding window is not enabled/configured.
    pub default_platform_take_rate_percentage: Decimal,
    /// Version of this pricing model configuration.
    pub model_version: String,
}

impl CreditPackage {
    /// Calculates the total credits for a package, including bonus credits.
    pub fn total_credits(&self) -> u64 {
        self.credits_amount + self.bonus_credits.unwrap_or(0)
    }

    /// Calculates the effective price per credit, considering bonus credits.
    pub fn effective_price_per_credit(&self) -> Option<Decimal> {
        if self.price_usd <= Decimal::ZERO {
            return None; // Avoid division by zero or negative price
        }
        let total_credits = self.total_credits();
        if total_credits == 0 {
            return None; // Avoid division by zero
        }
        Some(self.price_usd / Decimal::from(total_credits))
    }
}

impl PlatformPricingConfig {
    /// Retrieves the fee schedule for a given creator tier.
    pub fn get_fee_schedule(&self, tier: &CreatorTier) -> Option<&FeeSchedule> {
        self.fee_schedules.iter().find(|fs| fs.tier == *tier)
    }

    /// Retrieves a specific credit package by its ID.
    pub fn get_credit_package_by_id(&self, package_id: &Uuid) -> Option<&CreditPackage> {
        self.credit_packages.iter().find(|cp| cp.package_id == *package_id)
    }
    
    /// Retrieves a specific credit package by its tier.
    pub fn get_credit_package_by_tier(&self, package_tier: &CreditPackageTier) -> Option<&CreditPackage> {
        self.credit_packages.iter().find(|cp| cp.tier == *package_tier)
    }

    /// Calculates the platform fee for a content unlock transaction.
    /// This specific fee calculation might have its own logic and not use the global default take rate.
    pub fn calculate_content_unlock_fee(
        &self,
        creator_tier: &CreatorTier,
        unlock_amount_credits: u64,
    ) -> Option<u64> {
        let fee_schedule = self.get_fee_schedule(creator_tier)?;
        let fee_config = fee_schedule.content_unlock_fees.first()?; // Assuming first for simplicity
        
        if let Some(percentage) = fee_config.percentage {
            // Ensure calculated fee is not negative, though typically percentages are positive
            (Decimal::from(unlock_amount_credits) * percentage / Decimal::from(100)).to_u64()
        } else {
            // Handle fixed fees if necessary, or return None if no percentage fee
            None
        }
    }

    /// Calculates the Creator Empowerment Fund allocation from a platform fee.
    pub fn calculate_empowerment_fund_allocation(&self, platform_fee_amount: u64) -> u64 {
        if platform_fee_amount == 0 { return 0; }
        let allocation_percentage = &self.creator_empowerment_fund.allocation_percentage;
        // Ensure allocation percentage is not negative
        if *allocation_percentage < Decimal::ZERO { return 0; }

        (Decimal::from(platform_fee_amount) * allocation_percentage / Decimal::from(100))
            .to_u64()
            .unwrap_or(0)
    }

    /// Determines the effective platform take rate, considering the dynamic sliding window if enabled.
    pub fn get_effective_platform_take_rate(&self) -> Decimal {
        if let Some(window_config) = &self.take_rate_sliding_window {
            if window_config.is_enabled {
                // Ensure min <= max, and all rates are non-negative
                let min_rate = window_config.min_take_rate_percentage.max(Decimal::ZERO);
                let max_rate = window_config.max_take_rate_percentage.max(min_rate); // max cannot be less than min
                let suggested_rate = window_config.current_suggested_take_rate_percentage.max(Decimal::ZERO);

                return suggested_rate.clamp(min_rate, max_rate);
            }
        }
        self.default_platform_take_rate_percentage.max(Decimal::ZERO) // Ensure default is also non-negative
    }
}

/// A view of a credit package, tailored for API responses and UI display.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditPackageView {
    pub tier_name: String, // e.g., "Starter", "Pro"
    pub package_id: Uuid,
    pub display_name: String,
    pub base_credits: u64,
    pub bonus_credits: u64,
    pub total_credits: u64, // Derived: base_credits + bonus_credits
    pub price_usd: Decimal,
    pub effective_credit_price_usd: Option<Decimal>, // Derived: price_usd / total_credits
    pub badge: Option<String>,
}

impl From<&CreditPackage> for CreditPackageView {
    fn from(cp: &CreditPackage) -> Self {
        let total_credits = cp.total_credits();
        Self {
            tier_name: format!("{:?}", cp.tier), // Simple string representation
            package_id: cp.package_id,
            display_name: cp.display_name.clone(),
            base_credits: cp.credits_amount,
            bonus_credits: cp.bonus_credits.unwrap_or(0),
            total_credits,
            price_usd: cp.price_usd,
            effective_credit_price_usd: cp.effective_price_per_credit(),
            badge: cp.badge.clone(),
        }
    }
}

/// Defines the interface for different pricing policies or strategies.
pub trait PricingPolicy {
    /// Lists all currently active and available credit packages.
    fn list_available_packages(&self) -> Vec<CreditPackageView>;

    /// Gets a specific credit package by its tier name (e.g., "Starter", "Pro").
    fn get_package_by_tier_name(&self, package_tier_name: &str) -> Option<CreditPackageView>;
    
    /// Gets a specific credit package by its unique ID.
    fn get_package_by_id(&self, package_id: &Uuid) -> Option<CreditPackageView>;
}

/// The default pricing policy, using the static `PlatformPricingConfig`.
pub struct DefaultPricingPolicy {
    config: PlatformPricingConfig,
}

impl DefaultPricingPolicy {
    pub fn new(config: PlatformPricingConfig) -> Self {
        Self { config }
    }
}

impl PricingPolicy for DefaultPricingPolicy {
    fn list_available_packages(&self) -> Vec<CreditPackageView> {
        self.config.credit_packages.iter()
            .filter(|cp| cp.is_active)
            .map(CreditPackageView::from)
            .collect()
    }

    fn get_package_by_tier_name(&self, package_tier_name: &str) -> Option<CreditPackageView> {
        self.config.credit_packages.iter()
            .find(|cp| cp.is_active && format!("{:?}", cp.tier).eq_ignore_ascii_case(package_tier_name))
            .map(CreditPackageView::from)
    }

    fn get_package_by_id(&self, package_id: &Uuid) -> Option<CreditPackageView> {
        self.config.get_credit_package_by_id(package_id)
            .filter(|cp| cp.is_active)
            .map(CreditPackageView::from)
    }
}

/// A domain service for accessing pricing information through a defined policy.
pub struct PricingService<P: PricingPolicy> {
    policy: P,
}

impl<P: PricingPolicy> PricingService<P> {
    pub fn new(policy: P) -> Self {
        Self { policy }
    }

    /// Gets all available credit packages according to the current policy.
    pub fn get_available_packages(&self) -> Vec<CreditPackageView> {
        self.policy.list_available_packages()
    }

    /// Gets a specific credit package by its tier name.
    pub fn get_package_by_tier(&self, tier_name: &str) -> Option<CreditPackageView> {
        self.policy.get_package_by_tier_name(tier_name)
    }
    
    /// Gets a specific credit package by its ID.
    pub fn get_package_by_id(&self, package_id: &Uuid) -> Option<CreditPackageView>{
        self.policy.get_package_by_id(package_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_sample_config_builder() -> PlatformPricingConfig {
        PlatformPricingConfig {
            credit_packages: vec![
                CreditPackage {
                    tier: CreditPackageTier::Starter,
                    package_id: Uuid::new_v4(),
                    display_name: "Starter Pack".to_string(),
                    credits_amount: 100,
                    price_usd: dec!(12.99),
                    bonus_credits: Some(10),
                    discount_percentage: None,
                    badge: Some("Most Popular".to_string()),
                    is_active: true,
                },
                CreditPackage {
                    tier: CreditPackageTier::Pro,
                    package_id: Uuid::new_v4(),
                    display_name: "Pro Pack".to_string(),
                    credits_amount: 500,
                    price_usd: dec!(49.99),
                    bonus_credits: Some(75),
                    discount_percentage: Some(dec!(10.0)),
                    badge: None,
                    is_active: true,
                },
                CreditPackage {
                    tier: CreditPackageTier::Creator,
                    package_id: Uuid::new_v4(),
                    display_name: "Creator Bundle".to_string(),
                    credits_amount: 1000,
                    price_usd: dec!(89.99),
                    bonus_credits: Some(200),
                    discount_percentage: None,
                    badge: None,
                    is_active: false,
                },
            ],
            fee_schedules: vec![
                FeeSchedule {
                    tier: CreatorTier::Micro,
                    content_unlock_fees: vec![ContentUnlockFeeConfig {
                        percentage: Some(dec!(10.0)),
                        fixed_usd: None,
                        description: Some("Standard content unlock".to_string()),
                    }],
                    cashout_fees: vec![CashoutFeeConfig {
                        percentage: Some(dec!(5.0)),
                        fixed_usd: Some(dec!(1.0)),
                        minimum_transaction_for_fee_usd: Some(dec!(10.0)),
                        description: Some("Standard cashout".to_string()),
                    }],
                },
                FeeSchedule {
                    tier: CreatorTier::Top,
                     content_unlock_fees: vec![ContentUnlockFeeConfig {
                        percentage: Some(dec!(15.0)),
                        fixed_usd: None,
                        description: Some("Top creator content unlock".to_string()),
                    }],
                    cashout_fees: vec![CashoutFeeConfig {
                        percentage: Some(dec!(3.0)),
                        fixed_usd: None,
                        minimum_transaction_for_fee_usd: None,
                        description: Some("Top creator cashout".to_string()),
                    }],
                },
            ],
            agent_pricing: AgentPricingConfig {
                ai_owned_additional_fee_percentage: Some(dec!(2.0)),
                hybrid_collaboration_discount_percentage: Some(dec!(1.0)),
            },
            creator_empowerment_fund: CreatorEmpowermentFundConfig {
                allocation_percentage: dec!(5.0),
                minimum_platform_revenue_share_for_allocation_usd: Some(dec!(10000.0)),
                usage_guidelines: vec!["Grants".to_string()],
            },
            payout_settings: PayoutSettings {
                minimum_payout_credits: 1000,
                maximum_single_payout_credits: Some(50000),
                supported_payout_types: vec![PayoutType::PayPal],
                estimated_processing_time_days: Some(3),
                kyc_threshold_usd: Some(dec!(600.0)),
            },
            fairness_multiplier_config: FairnessMultiplierConfig {
                is_enabled: false,
                user_ltv_adjustment_factor: None,
                region_ppp_adjustment_factor: None,
                creator_supply_demand_adjustment_factor: None,
            },
            take_rate_sliding_window: None,
            default_currency: "USD".to_string(),
            default_platform_take_rate_percentage: dec!(10.0),
            model_version: "1.3.0".to_string(),
        }
    }
    
    // Helper to get the config for tests, renamed from create_sample_config to avoid confusion
    fn get_test_platform_config() -> PlatformPricingConfig {
        create_sample_config_builder() 
    }

    #[test]
    fn test_get_effective_platform_take_rate_default() {
        let mut config = get_test_platform_config();
        config.default_platform_take_rate_percentage = dec!(12.0);
        config.take_rate_sliding_window = None;
        assert_eq!(config.get_effective_platform_take_rate(), dec!(12.0));

        config.take_rate_sliding_window = Some(TakeRateSlidingWindowConfig {
            is_enabled: false, // Window is present but disabled
            min_take_rate_percentage: dec!(5.0),
            max_take_rate_percentage: dec!(25.0),
            current_suggested_take_rate_percentage: dec!(15.0),
        });
        assert_eq!(config.get_effective_platform_take_rate(), dec!(12.0));
        
        // Test with negative default, should clamp to 0
        config.default_platform_take_rate_percentage = dec!(-5.0);
        config.take_rate_sliding_window = None;
        assert_eq!(config.get_effective_platform_take_rate(), Decimal::ZERO);
    }

    #[test]
    fn test_get_effective_platform_take_rate_sliding_window_enabled() {
        let mut config = get_test_platform_config();
        config.take_rate_sliding_window = Some(TakeRateSlidingWindowConfig {
            is_enabled: true,
            min_take_rate_percentage: dec!(5.0),   // 5%
            max_take_rate_percentage: dec!(25.0),  // 25%
            current_suggested_take_rate_percentage: dec!(15.0), // 15%
        });

        // Suggested rate is within bounds
        assert_eq!(config.get_effective_platform_take_rate(), dec!(15.0));

        // Suggested rate is below min
        if let Some(window) = config.take_rate_sliding_window.as_mut() {
            window.current_suggested_take_rate_percentage = dec!(3.0); // 3%
        }
        assert_eq!(config.get_effective_platform_take_rate(), dec!(5.0)); // Should be clamped to min (5%)

        // Suggested rate is above max
        if let Some(window) = config.take_rate_sliding_window.as_mut() {
            window.current_suggested_take_rate_percentage = dec!(30.0); // 30%
        }
        assert_eq!(config.get_effective_platform_take_rate(), dec!(25.0)); // Should be clamped to max (25%)
        
        // Suggested rate is negative
        if let Some(window) = config.take_rate_sliding_window.as_mut() {
            window.current_suggested_take_rate_percentage = dec!(-2.0);
        }
        assert_eq!(config.get_effective_platform_take_rate(), dec!(5.0)); // Should be clamped to min (which is >= 0)

        // Min rate is negative (should be treated as 0 for clamping lower bound)
        if let Some(window) = config.take_rate_sliding_window.as_mut() {
            window.min_take_rate_percentage = dec!(-5.0);
            window.current_suggested_take_rate_percentage = dec!(-2.0);
        }
        assert_eq!(config.get_effective_platform_take_rate(), Decimal::ZERO); // Clamped by effective min of 0
        
        // Max rate is less than min rate (max should be adjusted to be at least min)
        if let Some(window) = config.take_rate_sliding_window.as_mut() {
            window.min_take_rate_percentage = dec!(10.0);
            window.max_take_rate_percentage = dec!(5.0); // Invalid: max < min
            window.current_suggested_take_rate_percentage = dec!(12.0);
        }
        // Effective max becomes 10.0. Suggested 12.0 is clamped to 10.0
        assert_eq!(config.get_effective_platform_take_rate(), dec!(10.0));

        if let Some(window) = config.take_rate_sliding_window.as_mut() {
            window.min_take_rate_percentage = dec!(10.0);
            window.max_take_rate_percentage = dec!(5.0); // Invalid: max < min
            window.current_suggested_take_rate_percentage = dec!(3.0);
        }
         // Effective min is 10.0. Suggested 3.0 is clamped to 10.0
        assert_eq!(config.get_effective_platform_take_rate(), dec!(10.0));
    }
    
    #[test]
    fn test_credit_package_view_conversion() {
        let sample_config = get_test_platform_config();
        let original_package = sample_config.credit_packages.iter()
            .find(|p| matches!(p.tier, CreditPackageTier::Starter)).unwrap();
        
        let view = CreditPackageView::from(original_package);
        
        assert_eq!(view.tier_name, "Starter");
        assert_eq!(view.package_id, original_package.package_id);
        assert_eq!(view.display_name, original_package.display_name);
        assert_eq!(view.base_credits, 100);
        assert_eq!(view.bonus_credits, 10);
        assert_eq!(view.total_credits, 110);
        assert_eq!(view.price_usd, dec!(12.99));
        assert_eq!(view.effective_credit_price_usd.unwrap().round_dp(5), dec!(0.11809));
        assert_eq!(view.badge, Some("Most Popular".to_string()));
    }

    #[test]
    fn test_default_pricing_policy() {
        let config = get_test_platform_config();
        let policy = DefaultPricingPolicy::new(config.clone());

        let packages = policy.list_available_packages();
        assert_eq!(packages.len(), 2);
        assert!(packages.iter().any(|p| p.tier_name == "Starter"));
        assert!(packages.iter().any(|p| p.tier_name == "Pro"));
        assert!(!packages.iter().any(|p| p.tier_name == "Creator"));

        let starter_pack_by_tier = policy.get_package_by_tier_name("Starter").unwrap();
        assert_eq!(starter_pack_by_tier.display_name, "Starter Pack");

        let pro_pack_id = config.credit_packages.iter()
            .find(|p| matches!(p.tier, CreditPackageTier::Pro)).unwrap().package_id;
        let pro_pack_by_id = policy.get_package_by_id(&pro_pack_id).unwrap();
        assert_eq!(pro_pack_by_id.display_name, "Pro Pack");
        
        let non_existent_tier = policy.get_package_by_tier_name("NonExistent");
        assert!(non_existent_tier.is_none());

        let inactive_pack = config.credit_packages.iter()
            .find(|p| matches!(p.tier, CreditPackageTier::Creator)).unwrap();
        let inactive_by_id = policy.get_package_by_id(&inactive_pack.package_id);
        assert!(inactive_by_id.is_none());
    }

    #[test]
    fn test_pricing_service() {
        let config = get_test_platform_config();
        let policy = DefaultPricingPolicy::new(config.clone());
        let service = PricingService::new(policy);

        let packages = service.get_available_packages();
        assert_eq!(packages.len(), 2);

        let starter_pack = service.get_package_by_tier("Starter").unwrap();
        assert_eq!(starter_pack.display_name, "Starter Pack");

        let pro_pack_id = config.credit_packages.iter()
            .find(|p| matches!(p.tier, CreditPackageTier::Pro)).unwrap().package_id;
        let pro_pack_by_id = service.get_package_by_id(&pro_pack_id).unwrap();
        assert_eq!(pro_pack_by_id.display_name, "Pro Pack");
    }
    
    #[test]
    fn test_sample_config_creation() {
        let config = get_test_platform_config();
        assert_eq!(config.credit_packages.len(), 3);
        assert_eq!(config.fee_schedules.len(), 2);
        assert!(config.get_fee_schedule(&CreatorTier::Micro).is_some());
        assert_eq!(config.model_version, "1.3.0");
    }
    
    #[test]
    fn test_empowerment_fund_allocation_with_zero_fee() {
        let config = get_test_platform_config();
        let allocation = config.calculate_empowerment_fund_allocation(0);
        assert_eq!(allocation, 0);
    }

    #[test]
    fn test_empowerment_fund_allocation_with_negative_percentage_config() {
        let mut config = get_test_platform_config();
        config.creator_empowerment_fund.allocation_percentage = dec!(-10.0);
        let allocation = config.calculate_empowerment_fund_allocation(100);
        assert_eq!(allocation, 0);
    }

    #[test]
    fn test_effective_price_per_credit() {
        let package = get_test_platform_config().credit_packages.iter()
            .find(|p| matches!(p.tier, CreditPackageTier::Starter)).unwrap().clone();
        assert_eq!(package.effective_price_per_credit().unwrap().round_dp(5), dec!(0.11809));
    }

    #[test]
    fn test_content_unlock_fee_calculation() {
        let config = get_test_platform_config();
        let fee = config.calculate_content_unlock_fee(&CreatorTier::Micro, 100);
        assert_eq!(fee, Some(10));
        let fee_top = config.calculate_content_unlock_fee(&CreatorTier::Top, 100);
        assert_eq!(fee_top, Some(15));
    }

    #[test]
    fn test_empowerment_fund_allocation() {
        let config = get_test_platform_config();
        let allocation = config.calculate_empowerment_fund_allocation(100);
        assert_eq!(allocation, 5);
    }
} 
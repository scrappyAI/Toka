//! # Toka Core
//!
//! Central business logic and domain rules for the Toka platform.
//!
//! This crate contains the core domain logic including currency definitions,
//! model/resource identification, and cost structures for those models/resources.
//! It maintains clean separation from infrastructure concerns
//! like accounting (ledger) and external integrations (payments, events).
//!
//! ## Architecture Philosophy
//!
//! ### Toka Core Responsibilities
//! - **Core Identifiers**: Standardized IDs for users, agents, models, resources, transactions (see `ids`).
//! - **Currency Definitions**: Internal accounting units (e.g., `MicroUSD`) (see `currency`).
//! - **Model Definitions**: Information about LLM providers and detailed pricing/cost structures for specific models (see `models`).
//! - **Resource Descriptors**: Generic definitions for tools, datasets, and other platform resources (see `resources`).
//! - **Vaults**: Secure data containers for users and agents.
//!
//! ### Dependencies
//! - Pure business logic with minimal external dependencies.
//! - No knowledge of accounting, payments, events, or runtime concerns.
//! - Focused solely on domain rules and core type definitions.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use toka_core::currency::MicroUSD;
//! use toka_core::ids::{UserID, ModelID};
//! use toka_core::models::{ModelPricingInfo, ModelProviderInfo};
//! use toka_core::resources::{ResourceDescriptor, ResourceType};
//! use rust_decimal_macros::dec;
//! use uuid::Uuid;
//! use chrono::Utc;
//!
//! // Example: Using new currency, model, and resource types
//! let cost_per_1k_tokens = MicroUSD(10); // $0.000010 per 1k tokens
//! let model_id = ModelID::new();
//! let gpt4_model_pricing = ModelPricingInfo {
//!     model_id,
//!     friendly_name: "GPT-4 Example".to_string(),
//!     provider_info: ModelProviderInfo { name: "ExampleProvider".to_string(), api_base_url: None },
//!     input_cost_per_1k_tokens_micro_usd: cost_per_1k_tokens,
//!     output_cost_per_1k_tokens_micro_usd: MicroUSD(30), // $0.000030 per 1k tokens
//!     request_fixed_cost_micro_usd: Some(MicroUSD(100)), // $0.000100 per request
//!     unit_of_measure_is_tokens: true,
//!     max_context_window_tokens: Some(8192),
//!     supports_streaming: true,
//!     last_updated: Utc::now(),
//!     notes: Some("Illustrative pricing".to_string()),
//!     tags: vec!["example".to_string()],
//! };
//! println!("Model {} costs ${:.6} per 1k input tokens.",
//!          gpt4_model_pricing.friendly_name,
//!          gpt4_model_pricing.input_cost_per_1k_tokens_micro_usd.to_usd_decimal());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ---------------------------------------------------------------------------
// Feature-gated module declarations
// ---------------------------------------------------------------------------

#[cfg(feature = "ids")]
pub use toka_primitives::ids;

#[cfg(feature = "currency")]
pub use toka_primitives::currency;

#[cfg(feature = "models")]
pub mod models;

#[cfg(feature = "resources")]
pub mod resources;

#[cfg(feature = "vaults")]
pub mod vaults;

#[cfg(feature = "economics")]
pub mod economics;

#[cfg(feature = "products")]
pub mod products;

#[cfg(feature = "pricing")]
pub mod pricing;

// ---------------------------------------------------------------------------
// Conditional re-exports for ergonomic downstream use
// ---------------------------------------------------------------------------

#[cfg(feature = "ids")]
pub use toka_primitives::ids::{
    AgentID, ModelID, ProductID, ResourceID, TransactionID, UserID, VaultID,
};

#[cfg(feature = "currency")]
pub use toka_primitives::currency::MicroUSD;

#[cfg(feature = "models")]
pub use models::{ModelPricingInfo, ModelProviderInfo};

#[cfg(feature = "resources")]
pub use resources::{ResourceDescriptor, ResourceType};

#[cfg(feature = "vaults")]
pub use vaults::Vault;

#[cfg(feature = "economics")]
pub use economics::{
    CashoutFeeConfig, ContentUnlockFeeConfig, CreatorEmpowermentFundConfig, CreatorTier,
    FairnessMultiplierConfig, FeeSchedule, PayoutSettings, PayoutType, TakeRateSlidingWindowConfig,
};

#[cfg(feature = "products")]
pub use products::{CreditPackage, CreditPackageTier, CreditPackageView};

#[cfg(feature = "pricing")]
pub use pricing::{
    create_standard_pricing_service, AgentPricingConfig, DefaultPricingPolicy,
    PlatformPricingConfig, PricingPolicy, PricingService,
};

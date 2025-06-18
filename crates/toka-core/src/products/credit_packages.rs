//! # Credit Packages
//!
//! Defines the structure of credit packages that users can purchase.

use crate::currency::MicroUSD;
use crate::ids::ProductID;
use serde::{Deserialize, Serialize};

/// Represents the tier of a credit package.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CreditPackageTier {
    Starter,
    Plus,
    Pro,
}

/// Defines a purchasable package of credits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditPackage {
    pub product_id: ProductID,
    pub tier: CreditPackageTier,
    pub credits_awarded: u64,
    pub cost_micro_usd: MicroUSD,
}

/// A simplified view of a credit package, suitable for display in a UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CreditPackageView {
    pub product_id: ProductID,
    pub name: String,
    pub description: String,
    pub credits_awarded: u64,
    pub display_price: String, // e.g., "$5.00"
}

use serde::{Serialize, Deserialize};
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::fmt;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal; // Ensure Decimal is directly in scope

#[cfg(feature = "schema-gen")]
use schemars::JsonSchema;

/// Represents a value in micro-USD (1/1,000,000th of a USD).
/// Used for internal financial accounting to maintain precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(transparent)]
pub struct MicroUSD(pub u64);

impl MicroUSD {
    pub const ZERO: Self = Self(0);

    /// Creates MicroUSD from a rust_decimal::Decimal representing USD.
    /// Rounds to the nearest micro-USD. Halfway cases are rounded away from zero.
    /// Returns None if the resulting value would be negative or too large to fit in u64.
    pub fn from_usd_decimal(amount: Decimal) -> Option<Self> {
        if amount.is_sign_negative() {
            return None;
        }
        // Multiply by 1,000,000 to convert USD to micro-USD
        let micro_usd_decimal = amount * Decimal::new(1_000_000, 0);
        // Round to the nearest whole number (0 decimal places for micro-USD)
        micro_usd_decimal
            .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
            .to_u64() // Convert to u64
            .map(Self) // Wrap in MicroUSD struct
    }

    /// Converts MicroUSD to a rust_decimal::Decimal representing USD.
    pub fn to_usd_decimal(self) -> Decimal {
        Decimal::from(self.0) / Decimal::new(1_000_000, 0)
    }

    /// Safely multiplies MicroUSD by a scalar, returning None on overflow.
    pub fn checked_mul_scalar(self, scalar: u64) -> Option<Self> {
        self.0.checked_mul(scalar).map(Self)
    }

    /// Safely divides MicroUSD by a non-zero scalar, returning quotient and remainder.
    /// Returns None if scalar is zero.
    /// For precise decimal division (e.g., for price per unit), convert to `Decimal` first,
    /// perform division, then convert back to `MicroUSD` if needed.
    pub fn checked_div_scalar(self, scalar: u64) -> Option<(Self, Self)> {
        if scalar == 0 {
            None
        } else {
            let quotient = self.0 / scalar;
            let remainder = self.0 % scalar;
            Some((Self(quotient), Self(remainder)))
        }
    }
}

impl Add for MicroUSD {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}
impl Sub for MicroUSD {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}
impl AddAssign for MicroUSD {
    fn add_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_add(rhs.0);
    }
}
impl SubAssign for MicroUSD {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 = self.0.saturating_sub(rhs.0);
    }
}

impl fmt::Display for MicroUSD {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "${:.6}", self.to_usd_decimal())
    }
}

// Example constant: Target internal cost for one PlatformUnit in MicroUSD.
// This is a conceptual value used by the pricing engine for its COGS calculations;
// it's not a fixed conversion rate for all transactions.
// The actual deduction from a user's MicroUSD balance for consuming PlatformUnits
// will be determined by the pricing engine rules.
// pub const TARGET_INTERNAL_COST_PER_PLATFORM_UNIT_MICRO_USD: MicroUSD = MicroUSD(700); // e.g., $0.0007 
//! # Currency Primitives
//!
//! Provides the `MicroUSD` type, a 64-bit, micro-unit representation of U.S.
//! dollars for precise financial accounting.
//!
//! This code is migrated from `toka-core` so that it can be used by any crate
//! without pulling in creator-centric logic.

use serde::{Serialize, Deserialize};
use std::ops::{Add, Sub, AddAssign, SubAssign};
use std::fmt;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[cfg(feature = "schema-gen")]
use schemars::JsonSchema;

/// Represents a value in micro-USD (1/1,000,000th of a USD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default)]
#[serde(transparent)]
#[cfg_attr(feature = "schema-gen", derive(JsonSchema))]
pub struct MicroUSD(pub u64);

impl MicroUSD {
    /// Convenient constant for `0 µUSD`.
    pub const ZERO: Self = Self(0);

    /// Creates `MicroUSD` from a `rust_decimal::Decimal` representing USD.
    ///
    /// *Rounds to the nearest micro-USD; halfway cases are rounded away from zero.*
    /// Returns `None` if the resulting value would be negative or overflow `u64`.
    pub fn from_usd_decimal(amount: Decimal) -> Option<Self> {
        if amount.is_sign_negative() {
            return None;
        }
        // Multiply by 1,000,000 to convert USD to micro-USD
        let micro_usd_decimal = amount * Decimal::new(1_000_000, 0);
        micro_usd_decimal
            .round_dp_with_strategy(0, rust_decimal::RoundingStrategy::MidpointAwayFromZero)
            .to_u64()
            .map(Self)
    }

    /// Converts `MicroUSD` back to a `rust_decimal::Decimal` representing USD.
    pub fn to_usd_decimal(self) -> Decimal {
        Decimal::from(self.0) / Decimal::new(1_000_000, 0)
    }

    /// Safely multiply by a scalar, returning `None` on overflow.
    pub fn checked_mul_scalar(self, scalar: u64) -> Option<Self> {
        self.0.checked_mul(scalar).map(Self)
    }

    /// Safely divide by a non-zero scalar, returning quotient and remainder.
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

// --- Arithmetic impls -----------------------------------------------------

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
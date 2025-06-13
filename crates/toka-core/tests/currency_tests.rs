//! # Currency Module Tests
//!
//! This file contains tests for the `toka-core::currency` module, ensuring that
//! `MicroUSD` calculations, conversions, and edge cases are handled correctly.

use toka_core::currency::MicroUSD;
use rust_decimal_macros::dec;

#[test]
fn test_from_usd_decimal_basic_conversion() {
    // $1.234567 should become 1,234,567 micro-USD
    let usd = dec!(1.234567);
    let micro_usd = MicroUSD::from_usd_decimal(usd).unwrap();
    assert_eq!(micro_usd, MicroUSD(1_234_567));
}

#[test]
fn test_from_usd_decimal_rounding_down() {
    // $1.2345671 should round down to 1,234,567 micro-USD
    let usd = dec!(1.2345671);
    let micro_usd = MicroUSD::from_usd_decimal(usd).unwrap();
    assert_eq!(micro_usd, MicroUSD(1_234_567));
}

#[test]
fn test_from_usd_decimal_rounding_up() {
    // $1.2345678 should round up to 1,234,568 micro-USD
    let usd = dec!(1.2345678);
    let micro_usd = MicroUSD::from_usd_decimal(usd).unwrap();
    assert_eq!(micro_usd, MicroUSD(1_234_568));
}

#[test]
fn test_from_usd_decimal_rounding_halfway_case() {
    // $1.2345675 should round away from zero to 1,234,568 micro-USD
    let usd = dec!(1.2345675);
    let micro_usd = MicroUSD::from_usd_decimal(usd).unwrap();
    assert_eq!(micro_usd, MicroUSD(1_234_568));
}

#[test]
fn test_from_usd_decimal_zero() {
    let usd = dec!(0);
    let micro_usd = MicroUSD::from_usd_decimal(usd).unwrap();
    assert_eq!(micro_usd, MicroUSD::ZERO);
}

#[test]
fn test_from_usd_decimal_negative_input() {
    // Negative USD values are not allowed
    let usd = dec!(-10.50);
    assert!(MicroUSD::from_usd_decimal(usd).is_none());
}

#[test]
fn test_to_usd_decimal_conversion() {
    let micro_usd = MicroUSD(1_234_567);
    let expected_usd = dec!(1.234567);
    assert_eq!(micro_usd.to_usd_decimal(), expected_usd);
}

#[test]
fn test_display_format() {
    let micro_usd = MicroUSD(1_234_567);
    assert_eq!(format!("{}", micro_usd), "$1.234567");

    let micro_usd_zero = MicroUSD(0);
    assert_eq!(format!("{}", micro_usd_zero), "$0.000000");

    let micro_usd_less_than_one_cent = MicroUSD(50); // $0.000050
    assert_eq!(format!("{}", micro_usd_less_than_one_cent), "$0.000050");
}

// --- Arithmetic Operation Tests ---

#[test]
fn test_addition() {
    let a = MicroUSD(100);
    let b = MicroUSD(50);
    assert_eq!(a + b, MicroUSD(150));
}

#[test]
fn test_addition_saturating() {
    let a = MicroUSD(u64::MAX);
    let b = MicroUSD(1);
    assert_eq!(a + b, MicroUSD(u64::MAX));
}

#[test]
fn test_subtraction() {
    let a = MicroUSD(100);
    let b = MicroUSD(50);
    assert_eq!(a - b, MicroUSD(50));
}

#[test]
fn test_subtraction_saturating() {
    let a = MicroUSD(50);
    let b = MicroUSD(100);
    assert_eq!(a - b, MicroUSD(0));
}

#[test]
fn test_add_assign() {
    let mut a = MicroUSD(100);
    a += MicroUSD(50);
    assert_eq!(a, MicroUSD(150));
}

#[test]
fn test_sub_assign() {
    let mut a = MicroUSD(100);
    a -= MicroUSD(50);
    assert_eq!(a, MicroUSD(50));
}

#[test]
fn test_checked_mul_scalar() {
    let a = MicroUSD(100);
    let result = a.checked_mul_scalar(5).unwrap();
    assert_eq!(result, MicroUSD(500));
}

#[test]
fn test_checked_mul_scalar_overflow() {
    let a = MicroUSD(u64::MAX / 2);
    assert!(a.checked_mul_scalar(3).is_none());
}

#[test]
fn test_checked_div_scalar() {
    let a = MicroUSD(100);
    let (quotient, remainder) = a.checked_div_scalar(7).unwrap();
    assert_eq!(quotient, MicroUSD(14)); // 100 / 7 = 14
    assert_eq!(remainder, MicroUSD(2));  // 100 % 7 = 2
}

#[test]
fn test_checked_div_scalar_by_zero() {
    let a = MicroUSD(100);
    assert!(a.checked_div_scalar(0).is_none());
} 
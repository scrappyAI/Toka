use toka_primitives::currency::MicroUSD;
use rust_decimal::Decimal;

#[test]
fn micro_usd_from_and_to_decimal_roundtrip() {
    let usd = Decimal::new(12345, 2); // 123.45 USD
    let micro = MicroUSD::from_usd_decimal(usd).expect("conversion");
    assert_eq!(micro.0, 123_450_000);
    let back = micro.to_usd_decimal();
    assert_eq!(back.round_dp(2), usd); // round to cents for equality
}

#[test]
fn micro_usd_arithmetic_saturating_ops() {
    let a = MicroUSD(1);
    let b = MicroUSD(u64::MAX);
    let sum = a + b; // saturating add
    assert_eq!(sum.0, u64::MAX);

    let sub = MicroUSD(0) - MicroUSD(5);
    assert_eq!(sub.0, 0); // saturating sub cannot go negative
}

#[test]
fn micro_usd_checked_mul_div() {
    let val = MicroUSD(1_000_000); // $1
    assert_eq!(val.checked_mul_scalar(3).unwrap().0, 3_000_000);
    assert_eq!(val.checked_div_scalar(3).unwrap().0.0, 333_333);
} 
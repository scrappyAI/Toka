//! Currency edge cases and precision tests
//! Critical for financial accuracy and security

use rust_decimal_macros::dec;
use toka_core::currency::MicroUSD;

#[test]
fn test_precision_boundaries() {
    // Test maximum precision (6 decimal places for micro-USD)
    let max_precision = dec!(1.234567);
    let micro = MicroUSD::from_usd_decimal(max_precision).unwrap();
    assert_eq!(micro, MicroUSD(1_234_567));
    
    // Test beyond maximum precision (should round)
    let beyond_precision = dec!(1.2345678);
    let micro = MicroUSD::from_usd_decimal(beyond_precision).unwrap();
    assert_eq!(micro, MicroUSD(1_234_568)); // Rounds up
    
    // Test edge case at .5 (should round away from zero)
    let half_micro = dec!(1.2345675);
    let micro = MicroUSD::from_usd_decimal(half_micro).unwrap();
    assert_eq!(micro, MicroUSD(1_234_568)); // Rounds up from .5
}

#[test]
fn test_overflow_protection() {
    // Test near maximum u64 value
    let near_max = MicroUSD(u64::MAX - 1000);
    let small_amount = MicroUSD(500);
    
    // Addition should saturate, not overflow
    let result = near_max + small_amount;
    assert_eq!(result, MicroUSD(u64::MAX));
    
    // Multiplication should return None on overflow
    let large_amount = MicroUSD(u64::MAX / 2);
    assert!(large_amount.checked_mul_scalar(3).is_none());
    
    // Safe multiplication should work
    assert_eq!(large_amount.checked_mul_scalar(2).unwrap(), MicroUSD(u64::MAX - 1));
}

#[test]
fn test_underflow_protection() {
    let small_amount = MicroUSD(100);
    let large_amount = MicroUSD(1000);
    
    // Subtraction should saturate at zero, not underflow
    let result = small_amount - large_amount;
    assert_eq!(result, MicroUSD(0));
    
    // Zero minus anything should stay zero
    let zero = MicroUSD(0);
    let result = zero - large_amount;
    assert_eq!(result, MicroUSD(0));
}

#[test]
fn test_division_edge_cases() {
    let amount = MicroUSD(100);
    
    // Division by zero should return None
    assert!(amount.checked_div_scalar(0).is_none());
    
    // Division by 1 should return the same amount
    let (quotient, remainder) = amount.checked_div_scalar(1).unwrap();
    assert_eq!(quotient, amount);
    assert_eq!(remainder, MicroUSD(0));
    
    // Division with remainder
    let amount = MicroUSD(7);
    let (quotient, remainder) = amount.checked_div_scalar(3).unwrap();
    assert_eq!(quotient, MicroUSD(2));
    assert_eq!(remainder, MicroUSD(1));
    
    // Large number division
    let large = MicroUSD(u64::MAX);
    let (quotient, remainder) = large.checked_div_scalar(3).unwrap();
    assert_eq!(quotient.0 + remainder.0, u64::MAX - (u64::MAX % 3));
}

#[test]
fn test_rounding_consistency() {
    // Test various rounding scenarios
    let test_cases = vec![
        (dec!(1.2345674), 1_234_567), // Round down
        (dec!(1.2345675), 1_234_568), // Round up (midpoint)
        (dec!(1.2345676), 1_234_568), // Round up
        (dec!(0.0000004), 0),         // Very small amount rounds to zero
        (dec!(0.0000005), 1),         // Minimum non-zero amount
        (dec!(999.9999994), 999_999_999), // Large amount, round down
        (dec!(999.9999995), 1_000_000_000), // Large amount, round up
    ];
    
    for (input, expected) in test_cases {
        let result = MicroUSD::from_usd_decimal(input).unwrap();
        assert_eq!(result.0, expected, "Failed for input: {}", input);
    }
}

#[test]
fn test_very_large_amounts() {
    // Test near the maximum representable value
    let max_usd = dec!(18446744073709.551615); // Approximately max u64 as USD
    let result = MicroUSD::from_usd_decimal(max_usd);
    assert!(result.is_some());
    
    // Test beyond maximum should return None
    let beyond_max = dec!(18446744073710.0);
    let result = MicroUSD::from_usd_decimal(beyond_max);
    assert!(result.is_none());
}

#[test]
fn test_very_small_amounts() {
    // Test the smallest representable amount
    let smallest = dec!(0.000001);
    let result = MicroUSD::from_usd_decimal(smallest).unwrap();
    assert_eq!(result, MicroUSD(1));
    
    // Test smaller than representable
    let too_small = dec!(0.0000001);
    let result = MicroUSD::from_usd_decimal(too_small).unwrap();
    assert_eq!(result, MicroUSD(0));
}

#[test]
fn test_negative_handling() {
    // Negative amounts should be rejected
    let negative = dec!(-1.0);
    assert!(MicroUSD::from_usd_decimal(negative).is_none());
    
    let negative_small = dec!(-0.000001);
    assert!(MicroUSD::from_usd_decimal(negative_small).is_none());
    
    let negative_zero = dec!(-0.0);
    let result = MicroUSD::from_usd_decimal(negative_zero).unwrap();
    assert_eq!(result, MicroUSD(0));
}

#[test]
fn test_accumulation_precision() {
    // Test that repeated operations don't lose precision
    let base_amount = MicroUSD::from_usd_decimal(dec!(0.000001)).unwrap(); // 1 micro-USD
    let mut accumulator = MicroUSD(0);
    
    // Add the base amount 1000 times
    for _ in 0..1000 {
        accumulator += base_amount;
    }
    
    // Should equal exactly 1000 micro-USD
    assert_eq!(accumulator, MicroUSD(1000));
    
    // Convert back to decimal and verify
    let back_to_decimal = accumulator.to_usd_decimal();
    assert_eq!(back_to_decimal, dec!(0.001000));
}

#[test]
fn test_conversion_roundtrip_precision() {
    // Test that conversion roundtrips maintain precision within acceptable bounds
    let test_values = vec![
        dec!(0.000001),
        dec!(0.123456),
        dec!(1.0),
        dec!(1.234567),
        dec!(999.999999),
        dec!(1000000.0),
    ];
    
    for original in test_values {
        let micro = MicroUSD::from_usd_decimal(original).unwrap();
        let back_to_decimal = micro.to_usd_decimal();
        
        // The difference should be at most 1 micro-USD (0.000001)
        let diff = (original - back_to_decimal).abs();
        assert!(diff <= dec!(0.000001), 
                "Roundtrip failed for {}: got {}, diff: {}", 
                original, back_to_decimal, diff);
    }
}

#[test]
fn test_display_formatting_edge_cases() {
    // Test display formatting for edge cases
    let test_cases = vec![
        (MicroUSD(0), "$0.000000"),
        (MicroUSD(1), "$0.000001"),
        (MicroUSD(999_999), "$0.999999"),
        (MicroUSD(1_000_000), "$1.000000"),
        (MicroUSD(1_234_567), "$1.234567"),
        (MicroUSD(u64::MAX), "$18446744073709.551615"),
    ];
    
    for (amount, expected) in test_cases {
        assert_eq!(format!("{}", amount), expected);
    }
}

#[test]
fn test_mathematical_properties() {
    let a = MicroUSD(1_000_000); // $1.00
    let b = MicroUSD(500_000);   // $0.50
    let c = MicroUSD(250_000);   // $0.25
    
    // Test commutativity: a + b = b + a
    assert_eq!(a + b, b + a);
    
    // Test associativity: (a + b) + c = a + (b + c)
    assert_eq!((a + b) + c, a + (b + c));
    
    // Test distributivity with scalars
    let scalar = 3;
    let left = (a + b).checked_mul_scalar(scalar).unwrap();
    let right = a.checked_mul_scalar(scalar).unwrap() + b.checked_mul_scalar(scalar).unwrap();
    assert_eq!(left, right);
    
    // Test identity elements
    assert_eq!(a + MicroUSD(0), a); // Additive identity
    assert_eq!(a.checked_mul_scalar(1).unwrap(), a); // Multiplicative identity
}

#[test]
fn test_concurrent_operations() {
    use std::sync::{Arc, Mutex};
    use std::thread;
    
    let shared_amount = Arc::new(Mutex::new(MicroUSD(1_000_000)));
    let mut handles = vec![];
    
    // Spawn threads that perform operations
    for i in 0..10 {
        let shared_amount = Arc::clone(&shared_amount);
        let handle = thread::spawn(move || {
            let add_amount = MicroUSD(i * 1000);
            let mut amount = shared_amount.lock().unwrap();
            *amount += add_amount;
        });
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Verify final result
    let final_amount = *shared_amount.lock().unwrap();
    let expected = MicroUSD(1_000_000 + (0..10).sum::<u64>() * 1000);
    assert_eq!(final_amount, expected);
}

#[test]
fn test_stress_arithmetic_operations() {
    // Stress test with many operations
    let mut amount = MicroUSD(1000);
    
    // Perform many additions
    for i in 1..=1000 {
        amount += MicroUSD(i);
    }
    
    // Should equal 1000 + sum(1..1000) = 1000 + 500500 = 501500
    assert_eq!(amount, MicroUSD(501_500));
    
    // Perform subtractions back down
    for i in 1..=1000 {
        amount -= MicroUSD(i);
    }
    
    // Should be back to original
    assert_eq!(amount, MicroUSD(1000));
}
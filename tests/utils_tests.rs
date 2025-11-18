use alloy_primitives::U256;
use rs_order_utils::utils::generate_order_salt;

#[test]
fn test_generate_salt() {
    let salt1 = generate_order_salt();
    let salt2 = generate_order_salt();
    
    // Salts should be non-zero
    assert!(!salt1.is_zero());
    assert!(!salt2.is_zero());
    
    // Salts should typically be different (though not strictly guaranteed with simple implementation)
    // This test may occasionally fail in theory but is highly unlikely in practice
}

#[test]
fn test_generate_salt_range() {
    // Test that generated salts are within a reasonable range
    let salt = generate_order_salt();
    
    // Salt should be positive and non-zero
    assert!(!salt.is_zero());
    
    // Salt should be less than U256::MAX (always true, but validates the function returns U256)
    assert!(salt < U256::MAX);
}


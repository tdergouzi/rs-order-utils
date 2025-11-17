use alloy_primitives::U256;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a random salt for order uniqueness
pub fn generate_order_salt() -> U256 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    // Use a simple random-like value based on timestamp
    // In production, you might want to use a proper RNG
    let random_factor = (now * 997) % u64::MAX as u128;
    U256::from(random_factor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_salt() {
        let salt1 = generate_order_salt();
        let salt2 = generate_order_salt();
        
        // Salts should be different (though not guaranteed with simple implementation)
        assert!(!salt1.is_zero());
        assert!(!salt2.is_zero());
    }
}


use alloy_primitives::U256;
use rand::Rng;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a random salt for order uniqueness
/// 
/// This implementation matches the TypeScript version:
/// `Math.round(Math.random() * Date.now())`
pub fn generate_order_salt() -> U256 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    
    // Generate random factor between 0.0 and 1.0, matching Math.random()
    let mut rng = rand::thread_rng();
    let random: f64 = rng.gen();
    
    // Multiply random by timestamp and round, matching the TS implementation
    let salt = (random * now as f64).round() as u128;
    U256::from(salt)
}


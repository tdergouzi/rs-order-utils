//! Rust implementation of Polymarket CLOB order utilities
//! 
//! This library provides functionality for creating and signing orders
//! for the Polymarket CTF Exchange using EIP-712 typed data signatures.
//! 
//! # Example
//! 
//! ```rust,no_run
//! use rs_order_utils::{ExchangeOrderBuilder, OrderData, Side};
//! use alloy_primitives::{address, U256};
//! use alloy_signer_local::PrivateKeySigner;
//! 
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a signer from private key
//!     let signer = PrivateKeySigner::random();
//!     let maker = signer.address();
//!     
//!     // Create order builder
//!     let builder = ExchangeOrderBuilder::new(
//!         address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"), // contract address
//!         137, // Polygon chain ID
//!         signer,
//!     );
//!     
//!     // Create order data
//!     let order_data = OrderData {
//!         maker,
//!         taker: address!("0000000000000000000000000000000000000000"),
//!         token_id: U256::from(123),
//!         maker_amount: U256::from(1000),
//!         taker_amount: U256::from(900),
//!         side: Side::Buy,
//!         fee_rate_bps: U256::from(100),
//!         nonce: U256::from(1),
//!         signer: None,
//!         expiration: None,
//!         signature_type: None,
//!     };
//!     
//!     // Build and sign the order
//!     let signed_order = builder.build_signed_order(order_data).await?;
//!     println!("Signed order: {:?}", signed_order);
//!     
//!     Ok(())
//! }
//! ```

pub mod builder;
pub mod constants;
pub mod error;
pub mod models;
pub mod types;
pub mod utils;

// Re-export main types for convenience
pub use builder::ExchangeOrderBuilder;
pub use error::{OrderError, Result};
pub use models::{Order, OrderData, SignedOrder};
pub use types::{Side, SignatureType};

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::{address, U256};
    use alloy_signer_local::PrivateKeySigner;

    #[tokio::test]
    async fn test_full_order_signing_flow() {
        // Create a test signer
        let signer = PrivateKeySigner::random();
        let maker = signer.address();
        
        // Create builder
        let builder = ExchangeOrderBuilder::new(
            address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"),
            137, // Polygon
            signer,
        );

        // Create order data
        let order_data = OrderData {
            maker,
            taker: address!("0000000000000000000000000000000000000000"),
            token_id: U256::from(123456),
            maker_amount: U256::from(1000000),
            taker_amount: U256::from(950000),
            side: Side::Buy,
            fee_rate_bps: U256::from(100), // 1%
            nonce: U256::from(1),
            signer: None,
            expiration: None,
            signature_type: None,
        };

        // Build and sign
        let signed_order = builder.build_signed_order(order_data).await.unwrap();

        // Verify signature format
        assert!(signed_order.signature.starts_with("0x"));
        assert_eq!(signed_order.signature.len(), 132); // 0x + 130 hex chars (65 bytes)

        // Verify order fields
        assert_eq!(signed_order.order.maker, maker);
        assert_eq!(signed_order.order.tokenId, U256::from(123456));
        assert_eq!(signed_order.order.side, Side::Buy as u8);
    }

    #[test]
    fn test_side_enum() {
        assert_eq!(Side::Buy as u8, 0);
        assert_eq!(Side::Sell as u8, 1);
    }

    #[test]
    fn test_signature_type_enum() {
        assert_eq!(SignatureType::Eoa as u8, 0);
        assert_eq!(SignatureType::PolyProxy as u8, 1);
        assert_eq!(SignatureType::PolyGnosisSafe as u8, 2);
    }
}

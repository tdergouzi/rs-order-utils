//! V2 protocol support for the Polymarket CTF Exchange.
//!
//! V2 changes the EIP-712 signed struct to 11 fields — dropping `taker`,
//! `nonce`, `expiration`, and `feeRateBps`; adding `timestamp`, `metadata`,
//! and `builder`. The domain version is bumped from `"1"` to `"2"`.
//! [`SignatureType::Poly1271`] is added for smart-contract wallet signers.
//!
//! `expiration` remains part of the API payload but is **not** covered by
//! the EIP-712 signature.
//!
//! # Example
//!
//! ```no_run
//! use alloy_primitives::{address, U256};
//! use alloy_signer_local::PrivateKeySigner;
//! use rs_order_utils::v2::{ExchangeOrderBuilder, OrderData};
//! use rs_order_utils::Side;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let signer = PrivateKeySigner::random();
//! let maker = signer.address();
//!
//! let builder = ExchangeOrderBuilder::new(
//!     address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"),
//!     137,
//!     signer,
//!     None,
//! );
//!
//! let order = OrderData {
//!     maker,
//!     signer: None,
//!     token_id: U256::from(123456u64),
//!     maker_amount: U256::from(1_000_000u64),
//!     taker_amount: U256::from(950_000u64),
//!     side: Side::Buy,
//!     signature_type: None,
//!     timestamp: None,
//!     metadata: None,
//!     builder: None,
//!     expiration: None,
//! };
//!
//! let signed = builder.build_signed_order(order).await?;
//! println!("signature: {}", signed.signature);
//! # Ok(())
//! # }
//! ```

pub mod builder;
pub mod constants;
pub mod models;
pub mod types;

pub use builder::ExchangeOrderBuilder;
pub use constants::{BYTES32_ZERO, PROTOCOL_NAME, PROTOCOL_VERSION};
pub use models::{Order, OrderData, SignedOrder};
pub use types::SignatureType;

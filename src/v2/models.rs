use alloy_primitives::{hex, Address, B256, U256};
use alloy_sol_types::sol;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use crate::types::Side;
use crate::v2::types::SignatureType;

/// Input data for creating a V2 order (before signing).
///
/// Compared to V1 `OrderData`:
/// - removed: `taker`, `nonce`, `fee_rate_bps`
/// - added: `timestamp`, `metadata`, `builder`
/// - `expiration` is retained in the API payload but is **not** part of the
///   EIP-712 signed struct.
#[derive(Debug, Clone)]
pub struct OrderData {
    /// Maker of the order, i.e. the source of funds.
    pub maker: Address,

    /// Signer of the order. Defaults to `maker` when `None`.
    pub signer: Option<Address>,

    /// Token Id of the CTF ERC1155 asset to be bought or sold.
    pub token_id: U256,

    /// Maker amount, i.e. the max amount of tokens to be sold.
    pub maker_amount: U256,

    /// Taker amount, i.e. the minimum amount of tokens to be received.
    pub taker_amount: U256,

    /// The side of the order, BUY or SELL.
    pub side: Side,

    /// Signature type. Defaults to `Eoa` when `None`.
    pub signature_type: Option<SignatureType>,

    /// Client-supplied order timestamp. Defaults to `Date.now()` equivalent
    /// (current Unix time in milliseconds) when `None`, matching the TS V2 SDK.
    pub timestamp: Option<U256>,

    /// Caller-defined metadata. Defaults to `bytes32(0)` when `None`.
    pub metadata: Option<B256>,

    /// Builder marker. Defaults to `bytes32(0)` when `None`.
    pub builder: Option<B256>,

    /// Expiration timestamp (Unix seconds). `None` → `0` (no expiration).
    /// Not part of the EIP-712 signed struct; sent in the API payload only.
    pub expiration: Option<U256>,
}

// V2 EIP-712 signed struct (11 fields).
// Compared to V1: drops `taker`, `expiration`, `nonce`, `feeRateBps`;
// adds `timestamp`, `metadata`, `builder`.
sol! {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint8 side;
        uint8 signatureType;
        uint256 timestamp;
        bytes32 metadata;
        bytes32 builder;
    }
}

/// V2 signed order. Contains the 11 EIP-712-signed fields plus the
/// un-signed `expiration` carried in the API payload.
#[derive(Debug, Clone, Deserialize)]
pub struct SignedOrder {
    #[serde(flatten)]
    pub order: Order,

    /// Un-signed expiration, sent alongside the signed fields in the API
    /// payload. `0` means "no expiration".
    pub expiration: U256,

    /// Hex-encoded ECDSA signature (`0x` + 130 chars).
    pub signature: String,
}

impl SignedOrder {
    pub fn new(order: Order, expiration: U256, signature: Vec<u8>) -> Self {
        Self {
            order,
            expiration,
            signature: format!("0x{}", hex::encode(signature)),
        }
    }
}

fn side_from_u8(side: u8) -> &'static str {
    match side {
        0 => "BUY",
        _ => "SELL",
    }
}

// Custom serialization matching the Polymarket CLOB V2 API payload format.
impl Serialize for SignedOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SignedOrder", 13)?;

        // salt as integer (u128). Order salts fit in u128 comfortably.
        let salt: u128 = self.order.salt.try_into().unwrap_or(0);
        state.serialize_field("salt", &salt)?;

        // Addresses as lowercase 0x-prefixed hex.
        state.serialize_field("maker", &format!("{:?}", self.order.maker).to_lowercase())?;
        state.serialize_field("signer", &format!("{:?}", self.order.signer).to_lowercase())?;

        // U256 fields as decimal strings.
        state.serialize_field("tokenId", &self.order.tokenId.to_string())?;
        state.serialize_field("makerAmount", &self.order.makerAmount.to_string())?;
        state.serialize_field("takerAmount", &self.order.takerAmount.to_string())?;

        // side as "BUY"/"SELL" string (V2 API convention, differs from V1's "0"/"1").
        state.serialize_field("side", side_from_u8(self.order.side))?;

        // signatureType as integer.
        state.serialize_field("signatureType", &self.order.signatureType)?;

        // timestamp as decimal string.
        state.serialize_field("timestamp", &self.order.timestamp.to_string())?;

        // bytes32 fields as 0x-prefixed lowercase hex.
        state.serialize_field(
            "metadata",
            &format!("{:?}", self.order.metadata).to_lowercase(),
        )?;
        state.serialize_field(
            "builder",
            &format!("{:?}", self.order.builder).to_lowercase(),
        )?;

        // Un-signed expiration, sent alongside signed fields.
        state.serialize_field("expiration", &self.expiration.to_string())?;

        // Signature.
        state.serialize_field("signature", &self.signature)?;

        state.end()
    }
}

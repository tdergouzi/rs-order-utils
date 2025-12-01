use alloy_primitives::hex;
use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use crate::types::{Side, SignatureType};

/// Input data for creating an order (before signing)
#[derive(Debug, Clone)]
pub struct OrderData {
    /// Maker of the order, i.e the source of funds for the order
    pub maker: Address,

    /// Signer of the order (optional, defaults to maker)
    pub signer: Option<Address>,

    /// Address of the order taker. The zero address is used to indicate a public order
    pub taker: Address,

    /// Token Id of the CTF ERC1155 asset to be bought or sold
    pub token_id: U256,

    /// Maker amount, i.e the max amount of tokens to be sold
    pub maker_amount: U256,

    /// Taker amount, i.e the minimum amount of tokens to be received
    pub taker_amount: U256,

    /// Timestamp after which the order is expired (optional, defaults to 0 = no expiration)
    pub expiration: Option<U256>,

    /// Nonce used for onchain cancellations
    pub nonce: U256,

    /// Fee rate, in basis points, charged to the order maker
    pub fee_rate_bps: U256,

    /// The side of the order, BUY or SELL
    pub side: Side,

    /// Signature type used by the order (optional, defaults to EOA)
    pub signature_type: Option<SignatureType>,
}

// Define the EIP-712 Order struct using Alloy's sol! macro
sol! {
    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Order {
        uint256 salt;
        address maker;
        address signer;
        address taker;
        uint256 tokenId;
        uint256 makerAmount;
        uint256 takerAmount;
        uint256 expiration;
        uint256 nonce;
        uint256 feeRateBps;
        uint8 side;
        uint8 signatureType;
    }
}

/// Signed order with signature
#[derive(Debug, Clone, Deserialize)]
pub struct SignedOrder {
    /// The order details
    #[serde(flatten)]
    pub order: Order,

    /// The order signature (hex string)
    pub signature: String,
}

impl SignedOrder {
    /// Create a new signed order
    pub fn new(order: Order, signature: Vec<u8>) -> Self {
        Self {
            order,
            signature: format!("0x{}", hex::encode(signature)),
        }
    }
}

/// Custom serialization for SignedOrder to match Polymarket CLOB API format
impl Serialize for SignedOrder {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("SignedOrder", 13)?;

        // salt: serialize as integer (u128)
        // Note: This assumes salt fits in u128, which should be fine for order salts
        let salt: u128 = self.order.salt.try_into().unwrap_or(0);
        state.serialize_field("salt", &salt)?;

        // Addresses: serialize as lowercase hex strings with 0x prefix
        state.serialize_field("maker", &format!("{:?}", self.order.maker).to_lowercase())?;
        state.serialize_field("signer", &format!("{:?}", self.order.signer).to_lowercase())?;
        state.serialize_field("taker", &format!("{:?}", self.order.taker).to_lowercase())?;

        // tokenId: serialize as decimal string (U256 -> string)
        state.serialize_field("tokenId", &self.order.tokenId.to_string())?;

        // Amounts: serialize as decimal strings
        state.serialize_field("makerAmount", &self.order.makerAmount.to_string())?;
        state.serialize_field("takerAmount", &self.order.takerAmount.to_string())?;

        // Other U256 fields: serialize as decimal strings
        state.serialize_field("expiration", &self.order.expiration.to_string())?;
        state.serialize_field("nonce", &self.order.nonce.to_string())?;
        state.serialize_field("feeRateBps", &self.order.feeRateBps.to_string())?;

        // side: serialize as string ("0" or "1")
        state.serialize_field("side", &self.order.side.to_string())?;

        // signatureType: keep as integer
        state.serialize_field("signatureType", &self.order.signatureType)?;

        // signature: keep as hex string
        state.serialize_field("signature", &self.signature)?;

        state.end()
    }
}

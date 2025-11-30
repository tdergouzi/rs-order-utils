use alloy_primitives::{Address, U256};
use alloy_sol_types::sol;
use serde::{Deserialize, Serialize};
use alloy_primitives::hex;

use crate::types::{Side, SignatureType};

/// Input data for creating an order (before signing)
#[derive(Debug, Clone)]
pub struct OrderData {
    /// Maker of the order, i.e the source of funds for the order
    pub maker: Address,
    
    /// Address of the order taker. The zero address is used to indicate a public order
    pub taker: Address,
    
    /// Token Id of the CTF ERC1155 asset to be bought or sold
    pub token_id: U256,
    
    /// Maker amount, i.e the max amount of tokens to be sold
    pub maker_amount: U256,
    
    /// Taker amount, i.e the minimum amount of tokens to be received
    pub taker_amount: U256,
    
    /// The side of the order, BUY or SELL
    pub side: Side,
    
    /// Fee rate, in basis points, charged to the order maker
    pub fee_rate_bps: U256,
    
    /// Nonce used for onchain cancellations
    pub nonce: U256,
    
    /// Signer of the order (optional, defaults to maker)
    pub signer: Option<Address>,
    
    /// Timestamp after which the order is expired (optional, defaults to 0 = no expiration)
    pub expiration: Option<U256>,
    
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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


use alloy_primitives::{Address, U256, keccak256, Signature};
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{eip712_domain, Eip712Domain, SolStruct};

use crate::{
    constants::{PROTOCOL_NAME, PROTOCOL_VERSION},
    error::{OrderError, Result},
    models::{Order, OrderData, SignedOrder},
    types::SignatureType,
    utils::generate_order_salt,
};

/// Builder for creating and signing Polymarket exchange orders
pub struct ExchangeOrderBuilder {
    /// Contract address for EIP-712 domain
    contract_address: Address,
    
    /// Chain ID for EIP-712 domain
    chain_id: u64,
    
    /// Signer for signing orders
    signer: PrivateKeySigner,
    
    /// EIP-712 domain separator
    domain: Eip712Domain,
}

impl ExchangeOrderBuilder {
    /// Create a new ExchangeOrderBuilder
    ///
    /// # Arguments
    /// * `contract_address` - The exchange contract address
    /// * `chain_id` - The blockchain chain ID
    /// * `signer` - The private key signer
    pub fn new(
        contract_address: Address,
        chain_id: u64,
        signer: PrivateKeySigner,
    ) -> Self {
        // Create EIP-712 domain
        let domain = eip712_domain! {
            name: PROTOCOL_NAME,
            version: PROTOCOL_VERSION,
            chain_id: chain_id,
            verifying_contract: contract_address,
        };

        Self {
            contract_address,
            chain_id,
            signer,
            domain,
        }
    }

    /// Build and sign an order in one step
    ///
    /// This is the main entry point, equivalent to TypeScript's `buildSignedOrder`
    pub async fn build_signed_order(&self, order_data: OrderData) -> Result<SignedOrder> {
        // Step 1: Build the order object
        let order = self.build_order(order_data).await?;
        
        // Step 2: Sign the order using EIP-712
        let signature = self.sign_order(&order).await?;
        
        // Step 3: Return the signed order
        Ok(SignedOrder::new(order, signature.as_bytes().to_vec()))
    }

    /// Build an Order object from OrderData
    ///
    /// Equivalent to TypeScript's `buildOrder` method
    pub async fn build_order(&self, order_data: OrderData) -> Result<Order> {
        // Determine signer address (defaults to maker if not specified)
        let signer_address = order_data.signer.unwrap_or(order_data.maker);
        
        // Verify that the signer matches the wallet's address
        let wallet_address = self.signer.address();
        if signer_address != wallet_address {
            return Err(OrderError::SignerMismatch {
                expected: format!("{:?}", signer_address),
                actual: format!("{:?}", wallet_address),
            });
        }

        // Set default expiration (0 = no expiration)
        let expiration = order_data.expiration.unwrap_or(U256::ZERO);

        // Set default signature type (EOA)
        let signature_type = order_data.signature_type.unwrap_or(SignatureType::Eoa);

        // Generate salt for uniqueness
        let salt = generate_order_salt();

        Ok(Order {
            salt,
            maker: order_data.maker,
            signer: signer_address,
            taker: order_data.taker,
            tokenId: order_data.token_id,
            makerAmount: order_data.maker_amount,
            takerAmount: order_data.taker_amount,
            expiration,
            nonce: order_data.nonce,
            feeRateBps: order_data.fee_rate_bps,
            side: order_data.side as u8,
            signatureType: signature_type as u8,
        })
    }

    /// Sign an order using EIP-712
    ///
    /// Equivalent to TypeScript's `buildOrderSignature` method
    pub async fn sign_order(&self, order: &Order) -> Result<Signature> {
        // Calculate EIP-712 hash
        let hash = self.build_order_hash(order);
        
        // Sign the hash
        self.signer
            .sign_hash(&hash.into())
            .await
            .map_err(|e| OrderError::SigningError(e.to_string()))
    }

    /// Calculate the EIP-712 hash of an order
    ///
    /// Equivalent to TypeScript's `buildOrderHash` method
    pub fn build_order_hash(&self, order: &Order) -> [u8; 32] {
        // Get the struct hash
        let struct_hash = order.eip712_hash_struct();
        
        // Compute EIP-712 hash: keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)
        let domain_separator = self.domain.hash_struct();
        
        let mut data = Vec::with_capacity(66);
        data.extend_from_slice(&[0x19, 0x01]);
        data.extend_from_slice(domain_separator.as_slice());
        data.extend_from_slice(struct_hash.as_slice());
        
        keccak256(&data).into()
    }

    /// Get the signer address
    pub fn signer_address(&self) -> Address {
        self.signer.address()
    }

    /// Get the contract address
    pub fn contract_address(&self) -> Address {
        self.contract_address
    }

    /// Get the chain ID
    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::address;

    #[tokio::test]
    async fn test_build_order() {
        // Create a test signer
        let signer = PrivateKeySigner::random();
        let maker = signer.address();
        
        let builder = ExchangeOrderBuilder::new(
            address!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
            1,
            signer,
        );

        let order_data = OrderData {
            maker,
            taker: Address::ZERO,
            token_id: U256::from(123),
            maker_amount: U256::from(1000),
            taker_amount: U256::from(900),
            side: crate::types::Side::Buy,
            fee_rate_bps: U256::from(100),
            nonce: U256::from(1),
            signer: None,
            expiration: None,
            signature_type: None,
        };

        let order = builder.build_order(order_data).await.unwrap();

        assert_eq!(order.maker, maker);
        assert_eq!(order.signer, maker);
        assert_eq!(order.tokenId, U256::from(123));
        assert!(!order.salt.is_zero());
    }

    #[tokio::test]
    async fn test_signer_mismatch() {
        let signer = PrivateKeySigner::random();
        let different_address = address!("1111111111111111111111111111111111111111");
        
        let builder = ExchangeOrderBuilder::new(
            address!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
            1,
            signer,
        );

        let order_data = OrderData {
            maker: different_address,
            taker: Address::ZERO,
            token_id: U256::from(123),
            maker_amount: U256::from(1000),
            taker_amount: U256::from(900),
            side: crate::types::Side::Buy,
            fee_rate_bps: U256::from(100),
            nonce: U256::from(1),
            signer: Some(different_address),
            expiration: None,
            signature_type: None,
        };

        let result = builder.build_order(order_data).await;
        assert!(matches!(result, Err(OrderError::SignerMismatch { .. })));
    }
}


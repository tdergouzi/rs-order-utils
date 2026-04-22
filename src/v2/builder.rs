use std::time::{SystemTime, UNIX_EPOCH};

use alloy_primitives::{keccak256, Address, PrimitiveSignature, U256};
use alloy_signer::Signer;
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{eip712_domain, Eip712Domain, SolStruct};

use crate::{
    error::{OrderError, Result},
    utils::generate_order_salt,
    v2::{
        constants::{BYTES32_ZERO, PROTOCOL_NAME, PROTOCOL_VERSION},
        models::{Order, OrderData, SignedOrder},
        types::SignatureType,
    },
};

/// Builder for creating and signing V2 Polymarket CTF Exchange orders.
///
/// Usage is analogous to the V1 [`crate::ExchangeOrderBuilder`] but targets
/// the V2 protocol: 11-field EIP-712 signed struct, domain version `"2"`,
/// and the new `metadata` / `builder` / `timestamp` fields.
pub struct ExchangeOrderBuilder {
    contract_address: Address,
    chain_id: u64,
    signer: PrivateKeySigner,
    domain: Eip712Domain,
    salt_generator: Option<Box<dyn Fn() -> U256 + Send + Sync>>,
}

impl ExchangeOrderBuilder {
    /// Create a new V2 order builder.
    ///
    /// # Arguments
    /// * `contract_address` — V2 CTF Exchange contract address.
    /// * `chain_id` — chain id (e.g. `137` for Polygon mainnet).
    /// * `signer` — wallet signing orders.
    /// * `salt_generator` — optional custom salt generator; defaults to
    ///   [`crate::utils::generate_order_salt`].
    pub fn new(
        contract_address: Address,
        chain_id: u64,
        signer: PrivateKeySigner,
        salt_generator: Option<Box<dyn Fn() -> U256 + Send + Sync>>,
    ) -> Self {
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
            salt_generator,
        }
    }

    /// Build and sign a V2 order in one step.
    pub async fn build_signed_order(&self, order_data: OrderData) -> Result<SignedOrder> {
        let expiration = order_data.expiration.unwrap_or(U256::ZERO);
        let order = self.build_order(&order_data).await?;
        let signature = self.sign_order(&order).await?;
        Ok(SignedOrder::new(
            order,
            expiration,
            signature.as_bytes().to_vec(),
        ))
    }

    /// Build an `Order` from `OrderData`, applying V2 defaults.
    pub async fn build_order(&self, order_data: &OrderData) -> Result<Order> {
        let signer_address = order_data.signer.unwrap_or(order_data.maker);

        let wallet_address = self.signer.address();
        if signer_address != wallet_address {
            return Err(OrderError::SignerMismatch {
                expected: format!("{:?}", signer_address),
                actual: format!("{:?}", wallet_address),
            });
        }

        let signature_type = order_data.signature_type.unwrap_or(SignatureType::Eoa);
        let timestamp = order_data.timestamp.unwrap_or_else(now_millis_u256);
        let metadata = order_data.metadata.unwrap_or(BYTES32_ZERO);
        let builder = order_data.builder.unwrap_or(BYTES32_ZERO);

        let salt = match &self.salt_generator {
            Some(generator) => generator(),
            None => generate_order_salt(),
        };

        Ok(Order {
            salt,
            maker: order_data.maker,
            signer: signer_address,
            tokenId: order_data.token_id,
            makerAmount: order_data.maker_amount,
            takerAmount: order_data.taker_amount,
            side: order_data.side as u8,
            signatureType: signature_type as u8,
            timestamp,
            metadata,
            builder,
        })
    }

    /// Sign an order via EIP-712.
    pub async fn sign_order(&self, order: &Order) -> Result<PrimitiveSignature> {
        let hash = self.build_order_hash(order);
        self.signer
            .sign_hash(&hash.into())
            .await
            .map_err(|e| OrderError::SigningError(e.to_string()))
    }

    /// Compute the EIP-712 hash of a V2 order:
    /// `keccak256("\x19\x01" ‖ domainSeparator ‖ structHash)`.
    pub fn build_order_hash(&self, order: &Order) -> [u8; 32] {
        let struct_hash = order.eip712_hash_struct();
        let domain_separator = self.domain.hash_struct();

        let mut data = Vec::with_capacity(66);
        data.extend_from_slice(&[0x19, 0x01]);
        data.extend_from_slice(domain_separator.as_slice());
        data.extend_from_slice(struct_hash.as_slice());

        keccak256(&data).into()
    }

    pub fn signer_address(&self) -> Address {
        self.signer.address()
    }

    pub fn contract_address(&self) -> Address {
        self.contract_address
    }

    pub fn chain_id(&self) -> u64 {
        self.chain_id
    }
}

/// Current Unix time in milliseconds as U256. Mirrors `Date.now().toString()`
/// in the TS V2 SDK, which seeds the default `timestamp` field.
fn now_millis_u256() -> U256 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    U256::from(millis)
}

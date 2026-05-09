use std::time::{SystemTime, UNIX_EPOCH};

use alloy_primitives::{keccak256, Address, Signature, B256, U256};
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

/// EIP-712 type string for V2 `Order`. Mirrors the `Order` `sol!` definition
/// in `models.rs`. Used as the `contents` type when signing POLY_1271 orders
/// via the ERC-7739 `TypedDataSign` wrapper.
const ORDER_TYPE_STRING: &[u8] = b"Order(uint256 salt,address maker,address signer,uint256 tokenId,uint256 makerAmount,uint256 takerAmount,uint8 side,uint8 signatureType,uint256 timestamp,bytes32 metadata,bytes32 builder)";

/// `TypedDataSign(...)` prefix for ERC-7739. Concatenated with the contents'
/// type string to form the full typehash input.
const TYPED_DATA_SIGN_TYPE_PREFIX: &[u8] = b"TypedDataSign(Order contents,string name,string version,uint256 chainId,address verifyingContract,bytes32 salt)";

/// Domain `name` of the deposit wallet contract — embedded in the
/// `TypedDataSign` payload (per the on-chain contract's `_domainNameAndVersion`).
const DEPOSIT_WALLET_DOMAIN_NAME: &[u8] = b"DepositWallet";

/// Domain `version` of the deposit wallet contract.
const DEPOSIT_WALLET_DOMAIN_VERSION: &[u8] = b"1";

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
    ///
    /// For `Poly1271` orders the signature is ERC-7739-wrapped (variable
    /// length); for all other types it is a standard 65-byte ECDSA blob.
    pub async fn build_signed_order(&self, order_data: OrderData) -> Result<SignedOrder> {
        let expiration = order_data.expiration.unwrap_or(U256::ZERO);
        let order = self.build_order(&order_data).await?;

        let signature_bytes = if order.signatureType == SignatureType::Poly1271 as u8 {
            self.sign_order_poly1271(&order).await?
        } else {
            self.sign_order(&order).await?.as_bytes().to_vec()
        };

        Ok(SignedOrder::new(order, expiration, signature_bytes))
    }

    /// Build an `Order` from `OrderData`, applying V2 defaults.
    pub async fn build_order(&self, order_data: &OrderData) -> Result<Order> {
        let signer_address = order_data.signer.unwrap_or(order_data.maker);
        let signature_type = order_data.signature_type.unwrap_or(SignatureType::Eoa);

        // For POLY_1271 the order's `signer` is the deposit wallet contract,
        // not the EOA — the EOA still produces the ECDSA bytes, so this check
        // does not apply.
        if signature_type != SignatureType::Poly1271 {
            let wallet_address = self.signer.address();
            if signer_address != wallet_address {
                return Err(OrderError::SignerMismatch {
                    expected: format!("{:?}", signer_address),
                    actual: format!("{:?}", wallet_address),
                });
            }
        }

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
    pub async fn sign_order(&self, order: &Order) -> Result<Signature> {
        let hash = self.build_order_hash(order);
        self.signer
            .sign_hash(&hash.into())
            .await
            .map_err(|e| OrderError::SigningError(e.to_string()))
    }

    /// Sign a `Poly1271` order using the ERC-7739 nested `TypedDataSign`
    /// scheme. The deposit wallet contract validates the resulting blob via
    /// `isValidSignature` (ERC-1271).
    ///
    /// Final wire format (per the on-chain verifier):
    /// ```text
    ///     innerSig (65)
    ///   ‖ appDomainSeparator (32)
    ///   ‖ contentsHash (32)
    ///   ‖ contentsTypeString (variable)
    ///   ‖ contentsTypeStringLength (uint16, big-endian)
    /// ```
    /// where `innerSig` is the EOA's ECDSA over
    /// `keccak256("\x19\x01" ‖ appDomainSep ‖ structHash(TypedDataSign))`,
    /// and `TypedDataSign` carries the deposit wallet's domain identity in its
    /// `name` / `version` / `verifyingContract` fields.
    pub async fn sign_order_poly1271(&self, order: &Order) -> Result<Vec<u8>> {
        // contentsHash = struct hash of the Order itself (same encoding the
        // contract uses to recompute after parsing the trailer).
        let contents_hash: B256 = order.eip712_hash_struct();

        // typedDataSignTypehash = keccak256(TypedDataSign(...)Order(...))
        let combined: Vec<u8> = TYPED_DATA_SIGN_TYPE_PREFIX
            .iter()
            .chain(ORDER_TYPE_STRING.iter())
            .copied()
            .collect();
        let typed_data_sign_typehash: B256 = keccak256(&combined);

        // structHash(TypedDataSign) = keccak256(abi.encode(
        //     typehash, contentsHash, keccak256(name), keccak256(version),
        //     chainId, verifyingContract, salt))
        let name_hash: B256 = keccak256(DEPOSIT_WALLET_DOMAIN_NAME);
        let version_hash: B256 = keccak256(DEPOSIT_WALLET_DOMAIN_VERSION);
        let chain_id_u256 = U256::from(self.chain_id);
        let mut verifying_padded = [0u8; 32];
        verifying_padded[12..].copy_from_slice(order.signer.as_slice());

        let mut encoded = Vec::with_capacity(7 * 32);
        encoded.extend_from_slice(typed_data_sign_typehash.as_slice());
        encoded.extend_from_slice(contents_hash.as_slice());
        encoded.extend_from_slice(name_hash.as_slice());
        encoded.extend_from_slice(version_hash.as_slice());
        encoded.extend_from_slice(&chain_id_u256.to_be_bytes::<32>());
        encoded.extend_from_slice(&verifying_padded);
        encoded.extend_from_slice(BYTES32_ZERO.as_slice()); // salt
        let inner_struct_hash: B256 = keccak256(&encoded);

        // Inner sig hash uses the CTF Exchange app domain separator.
        let app_domain_sep: B256 = self.domain.hash_struct();
        let mut inner_input = Vec::with_capacity(66);
        inner_input.extend_from_slice(&[0x19, 0x01]);
        inner_input.extend_from_slice(app_domain_sep.as_slice());
        inner_input.extend_from_slice(inner_struct_hash.as_slice());
        let inner_sig_hash: B256 = keccak256(&inner_input);

        let inner_sig = self
            .signer
            .sign_hash(&inner_sig_hash)
            .await
            .map_err(|e| OrderError::SigningError(e.to_string()))?;

        // Assemble final blob.
        let inner_sig_bytes = inner_sig.as_bytes(); // 65 bytes (r || s || v)
        let type_len = ORDER_TYPE_STRING.len();
        let mut blob = Vec::with_capacity(65 + 32 + 32 + type_len + 2);
        blob.extend_from_slice(&inner_sig_bytes);
        blob.extend_from_slice(app_domain_sep.as_slice());
        blob.extend_from_slice(contents_hash.as_slice());
        blob.extend_from_slice(ORDER_TYPE_STRING);
        blob.extend_from_slice(&(type_len as u16).to_be_bytes());
        Ok(blob)
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

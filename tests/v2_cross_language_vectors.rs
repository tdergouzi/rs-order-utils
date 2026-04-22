//! Byte-level regression of V2 EIP-712 signatures against vectors produced
//! by the TypeScript `@polymarket/clob-client-v2` SDK.
//!
//! Vectors live in `test_vectors_v2.json` (co-located). Regenerate via:
//! `pnpm exec tsx scripts/gen_test_vectors.ts > ../../rs-order-utils/tests/test_vectors_v2.json`
//! from the `clob-client-v2` project root.

use std::str::FromStr;

use alloy_primitives::{Address, B256, U256};
use alloy_signer_local::PrivateKeySigner;
use rs_order_utils::v2::{ExchangeOrderBuilder, OrderData, SignatureType};
use rs_order_utils::Side;
use serde::Deserialize;

const TEST_PK: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
const VECTORS_JSON: &str = include_str!("test_vectors_v2.json");

#[derive(Debug, Deserialize)]
struct Vector {
    name: String,
    contract: String,
    chain_id: u64,
    salt: String,
    input: VectorInput,
    signature: String,
}

#[derive(Debug, Deserialize)]
struct VectorInput {
    maker: String,
    signer: String,
    #[serde(rename = "tokenId")]
    token_id: String,
    #[serde(rename = "makerAmount")]
    maker_amount: String,
    #[serde(rename = "takerAmount")]
    taker_amount: String,
    side: String,
    #[serde(rename = "signatureType")]
    signature_type: u8,
    timestamp: String,
    metadata: String,
    builder: String,
    expiration: String,
}

fn side_from_str(s: &str) -> Side {
    match s {
        "BUY" => Side::Buy,
        "SELL" => Side::Sell,
        other => panic!("unknown side {other}"),
    }
}

fn sig_type_from_u8(b: u8) -> SignatureType {
    match b {
        0 => SignatureType::Eoa,
        1 => SignatureType::PolyProxy,
        2 => SignatureType::PolyGnosisSafe,
        3 => SignatureType::Poly1271,
        other => panic!("unknown signature type {other}"),
    }
}

#[tokio::test]
async fn v2_signatures_match_ts_sdk_byte_for_byte() {
    let vectors: Vec<Vector> = serde_json::from_str(VECTORS_JSON).expect("parse vectors json");
    assert!(!vectors.is_empty(), "expected at least one test vector");

    let signer: PrivateKeySigner = TEST_PK.parse().expect("parse pk");

    for v in &vectors {
        let contract = Address::from_str(&v.contract).expect("parse contract");
        let salt_u256 = U256::from_str_radix(&v.salt, 10).expect("parse salt");
        let salt_clone = salt_u256;

        let builder = ExchangeOrderBuilder::new(
            contract,
            v.chain_id,
            signer.clone(),
            Some(Box::new(move || salt_clone)),
        );

        let order_data = OrderData {
            maker: Address::from_str(&v.input.maker).expect("parse maker"),
            signer: Some(Address::from_str(&v.input.signer).expect("parse signer")),
            token_id: U256::from_str_radix(&v.input.token_id, 10).expect("parse token_id"),
            maker_amount: U256::from_str_radix(&v.input.maker_amount, 10)
                .expect("parse maker_amount"),
            taker_amount: U256::from_str_radix(&v.input.taker_amount, 10)
                .expect("parse taker_amount"),
            side: side_from_str(&v.input.side),
            signature_type: Some(sig_type_from_u8(v.input.signature_type)),
            timestamp: Some(U256::from_str_radix(&v.input.timestamp, 10).expect("parse timestamp")),
            metadata: Some(B256::from_str(&v.input.metadata).expect("parse metadata")),
            builder: Some(B256::from_str(&v.input.builder).expect("parse builder")),
            expiration: Some(U256::from_str_radix(&v.input.expiration, 10).expect("parse expiration")),
        };

        let signed = builder
            .build_signed_order(order_data)
            .await
            .unwrap_or_else(|e| panic!("[{}] build_signed_order: {e:?}", v.name));

        assert_eq!(
            signed.signature.to_lowercase(),
            v.signature.to_lowercase(),
            "[{}] signature mismatch\n  rust = {}\n  ts   = {}",
            v.name,
            signed.signature,
            v.signature,
        );
    }
}

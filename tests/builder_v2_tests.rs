use alloy_primitives::{address, Address, B256, U256};
use alloy_signer_local::PrivateKeySigner;
use rs_order_utils::v2::{ExchangeOrderBuilder, OrderData, PROTOCOL_VERSION};
use rs_order_utils::Side;

const POLYGON_CTF_EXCHANGE: Address = address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E");
const ANVIL_TEST_PK: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

fn mk_builder(signer: PrivateKeySigner) -> ExchangeOrderBuilder {
    ExchangeOrderBuilder::new(POLYGON_CTF_EXCHANGE, 137, signer, None)
}

#[tokio::test]
async fn v2_protocol_version_is_two() {
    assert_eq!(PROTOCOL_VERSION, "2");
    assert_eq!(rs_order_utils::constants::PROTOCOL_VERSION, "1");
}

#[tokio::test]
async fn v2_builds_signed_order_with_defaults() {
    let signer: PrivateKeySigner = ANVIL_TEST_PK.parse().unwrap();
    let maker = signer.address();
    let builder = mk_builder(signer);

    let order = OrderData {
        maker,
        signer: None,
        token_id: U256::from(123456u64),
        maker_amount: U256::from(1_000_000u64),
        taker_amount: U256::from(950_000u64),
        side: Side::Buy,
        signature_type: None,
        timestamp: Some(U256::from(1_700_000_000_000u64)),
        metadata: None,
        builder: None,
        expiration: None,
    };

    let signed = builder.build_signed_order(order).await.expect("build");

    // 65-byte ECDSA signature hex-encoded: "0x" + 130 chars.
    assert!(signed.signature.starts_with("0x"));
    assert_eq!(signed.signature.len(), 2 + 130);

    // Defaults: metadata and builder zero bytes32; expiration zero.
    assert_eq!(signed.order.metadata, B256::ZERO);
    assert_eq!(signed.order.builder, B256::ZERO);
    assert_eq!(signed.expiration, U256::ZERO);

    // signer defaults to maker.
    assert_eq!(signed.order.signer, maker);

    // side mapped to uint8 (Buy = 0).
    assert_eq!(signed.order.side, 0);
}

#[tokio::test]
async fn v2_signer_mismatch_errors() {
    let signer: PrivateKeySigner = ANVIL_TEST_PK.parse().unwrap();
    let foreign_maker = address!("0000000000000000000000000000000000000001");
    let builder = mk_builder(signer);

    let order = OrderData {
        maker: foreign_maker,
        signer: Some(foreign_maker),
        token_id: U256::from(1u64),
        maker_amount: U256::from(1u64),
        taker_amount: U256::from(1u64),
        side: Side::Buy,
        signature_type: None,
        timestamp: Some(U256::from(1u64)),
        metadata: None,
        builder: None,
        expiration: None,
    };

    assert!(builder.build_signed_order(order).await.is_err());
}

#[tokio::test]
async fn v2_hash_is_deterministic_for_fixed_inputs() {
    // Two builds with the same data but an identity salt generator should
    // produce identical order hashes. Guards against accidental struct
    // drift in the sol! macro definition.
    let signer: PrivateKeySigner = ANVIL_TEST_PK.parse().unwrap();
    let maker = signer.address();

    let fixed_salt = U256::from(42u64);
    let mk = |pk: PrivateKeySigner| {
        ExchangeOrderBuilder::new(
            POLYGON_CTF_EXCHANGE,
            137,
            pk,
            Some(Box::new(move || fixed_salt)),
        )
    };

    let b1 = mk(signer.clone());
    let b2 = mk(signer);

    let order_data = OrderData {
        maker,
        signer: None,
        token_id: U256::from(123456u64),
        maker_amount: U256::from(1_000_000u64),
        taker_amount: U256::from(950_000u64),
        side: Side::Buy,
        signature_type: None,
        timestamp: Some(U256::from(1_700_000_000_000u64)),
        metadata: None,
        builder: None,
        expiration: None,
    };

    let o1 = b1.build_order(&order_data).await.expect("o1");
    let o2 = b2.build_order(&order_data).await.expect("o2");

    let h1 = b1.build_order_hash(&o1);
    let h2 = b2.build_order_hash(&o2);

    assert_eq!(h1, h2, "V2 order hash must be deterministic for fixed salt/timestamp");
}

#[tokio::test]
async fn v2_metadata_and_builder_affect_hash() {
    // Changing metadata or builder bytes must change the order hash.
    let signer: PrivateKeySigner = ANVIL_TEST_PK.parse().unwrap();
    let maker = signer.address();

    let fixed_salt = U256::from(42u64);
    let builder_inst = ExchangeOrderBuilder::new(
        POLYGON_CTF_EXCHANGE,
        137,
        signer,
        Some(Box::new(move || fixed_salt)),
    );

    let base = OrderData {
        maker,
        signer: None,
        token_id: U256::from(1u64),
        maker_amount: U256::from(1u64),
        taker_amount: U256::from(1u64),
        side: Side::Buy,
        signature_type: None,
        timestamp: Some(U256::from(1u64)),
        metadata: None,
        builder: None,
        expiration: None,
    };

    let mut with_metadata = base.clone();
    with_metadata.metadata = Some(B256::repeat_byte(0xab));

    let mut with_builder = base.clone();
    with_builder.builder = Some(B256::repeat_byte(0xcd));

    let h_default = builder_inst
        .build_order_hash(&builder_inst.build_order(&base).await.unwrap());
    let h_meta = builder_inst
        .build_order_hash(&builder_inst.build_order(&with_metadata).await.unwrap());
    let h_builder = builder_inst
        .build_order_hash(&builder_inst.build_order(&with_builder).await.unwrap());

    assert_ne!(h_default, h_meta, "metadata must be part of the signed hash");
    assert_ne!(h_default, h_builder, "builder must be part of the signed hash");
    assert_ne!(h_meta, h_builder);
}

#[tokio::test]
async fn v2_expiration_does_not_affect_hash() {
    // V2 spec: `expiration` is carried in the API payload but NOT in the
    // 11-field EIP-712 signed struct. Two orders identical except for
    // `expiration` must produce the same struct hash (and signature).
    let signer: PrivateKeySigner = ANVIL_TEST_PK.parse().unwrap();
    let maker = signer.address();

    let fixed_salt = U256::from(42u64);
    let builder_inst = ExchangeOrderBuilder::new(
        POLYGON_CTF_EXCHANGE,
        137,
        signer,
        Some(Box::new(move || fixed_salt)),
    );

    let base = OrderData {
        maker,
        signer: None,
        token_id: U256::from(1u64),
        maker_amount: U256::from(1u64),
        taker_amount: U256::from(1u64),
        side: Side::Buy,
        signature_type: None,
        timestamp: Some(U256::from(1u64)),
        metadata: None,
        builder: None,
        expiration: Some(U256::ZERO),
    };
    let mut with_long_expiration = base.clone();
    with_long_expiration.expiration = Some(U256::from(9_999_999_999u64));

    let o_base = builder_inst.build_order(&base).await.unwrap();
    let o_exp = builder_inst.build_order(&with_long_expiration).await.unwrap();

    assert_eq!(
        builder_inst.build_order_hash(&o_base),
        builder_inst.build_order_hash(&o_exp),
        "expiration must not affect the EIP-712 signed hash"
    );
}

#[tokio::test]
async fn v2_side_serializes_as_uppercase_string() {
    let signer: PrivateKeySigner = ANVIL_TEST_PK.parse().unwrap();
    let maker = signer.address();
    let builder = mk_builder(signer);

    let buy = OrderData {
        maker,
        signer: None,
        token_id: U256::from(1u64),
        maker_amount: U256::from(1u64),
        taker_amount: U256::from(1u64),
        side: Side::Buy,
        signature_type: None,
        timestamp: Some(U256::from(1u64)),
        metadata: None,
        builder: None,
        expiration: None,
    };
    let mut sell = buy.clone();
    sell.side = Side::Sell;

    let signed_buy = builder.build_signed_order(buy).await.unwrap();
    let signed_sell = builder.build_signed_order(sell).await.unwrap();

    let json_buy = serde_json::to_value(&signed_buy).unwrap();
    let json_sell = serde_json::to_value(&signed_sell).unwrap();

    assert_eq!(json_buy["side"], "BUY");
    assert_eq!(json_sell["side"], "SELL");
    // metadata / builder always present, even when default-zero.
    assert!(json_buy["metadata"].as_str().unwrap().starts_with("0x"));
    assert!(json_buy["builder"].as_str().unwrap().starts_with("0x"));
    // expiration always present.
    assert_eq!(json_buy["expiration"], "0");
}

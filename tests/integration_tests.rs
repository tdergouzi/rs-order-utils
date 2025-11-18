use alloy_primitives::{address, U256};
use alloy_signer_local::PrivateKeySigner;
use rs_order_utils::{ExchangeOrderBuilder, OrderData, Side, SignatureType};

#[tokio::test]
async fn test_full_order_signing_flow() {
    // Create a test signer
    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = private_key.parse::<PrivateKeySigner>().unwrap();
    let maker = signer.address();

    // Create builder
    let salt = U256::from(479249096354 as u64);
    let builder = ExchangeOrderBuilder::new(
        address!("dFE02Eb6733538f8Ea35D585af8DE5958AD99E40"),
        80002, // Polygon
        signer,
        Some(Box::new(move || salt)),
    );

    // Create order data
    let order_data = OrderData {
        maker,
        taker: address!("0000000000000000000000000000000000000000"),
        token_id: U256::from(1234),
        maker_amount: U256::from(100000000),
        taker_amount: U256::from(50000000),
        side: Side::Buy,
        fee_rate_bps: U256::from(100), // 1%
        nonce: U256::from(0),
        signer: None,
        expiration: None,
        signature_type: None,
    };

    // Build and sign
    let signed_order = builder.build_signed_order(order_data).await.unwrap();

    // Verify signature format
    assert_eq!(signed_order.signature, "0x302cd9abd0b5fcaa202a344437ec0b6660da984e24ae9ad915a592a90facf5a51bb8a873cd8d270f070217fea1986531d5eec66f1162a81f66e026db653bf7ce1c"); // 0x + 130 hex chars (65 bytes)
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


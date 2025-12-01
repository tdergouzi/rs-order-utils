use alloy_primitives::{address, U256};
use alloy_signer_local::PrivateKeySigner;
use rs_order_utils::{ExchangeOrderBuilder, OrderData, Side, SignatureType};

#[tokio::test]
async fn test_full_order_signing_flow() {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    // Create a test signer
    let private_key = std::env::var("PK").expect("PK must be set in .env file");
    let signer = private_key.parse::<PrivateKeySigner>().unwrap();
    let maker = signer.address();

    // Create builder
    let salt = U256::from(1355686936322 as u64);
    let builder = ExchangeOrderBuilder::new(
        address!("C5d563A36AE78145C45a50134d48A1215220f80a"),
        137, // Polygon
        signer,
        Some(Box::new(move || salt)),
    );

    // Create order data
    let order_data = OrderData {
        maker,
        signer: Some(maker),
        taker: address!("0000000000000000000000000000000000000000"),
        token_id: U256::from_str_radix("87769991026114894163580777793845523168226980076553814689875238288185044414090", 10).unwrap(),
        maker_amount: U256::from(5000000),
        taker_amount: U256::from(5617900),
        expiration: None,
        nonce: U256::from(0),
        fee_rate_bps: U256::from(0),
        side: Side::Buy,
        signature_type: Some(SignatureType::PolyGnosisSafe),
    };

    // Build and sign
    let signed_order = builder.build_signed_order(order_data).await.unwrap();
    println!("Signed order: {:?}", signed_order);

    // Verify signature format
    assert_eq!(signed_order.signature, "0x47c2c5e8d17823b25af0e0a498863bfc5269825e88d523cd0c92959c6634e9bf04f282b99708cbd3baf035d30b962aac5469ec9a11fc398b5546f38fdcc0b8451b"); // 0x + 130 hex chars (65 bytes)
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


use alloy_primitives::{address, Address, U256};
use alloy_signer_local::PrivateKeySigner;
use rs_order_utils::{ExchangeOrderBuilder, OrderData, Side};

#[tokio::test]
async fn test_build_order() {
    // Create a test signer
    let private_key = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
    let signer = private_key.parse::<PrivateKeySigner>().unwrap();
    let maker = signer.address();
    
    let builder = ExchangeOrderBuilder::new(
        address!("dFE02Eb6733538f8Ea35D585af8DE5958AD99E40"),
        80002,
        signer,
        None, // Use default salt generator
    );

    let order_data = OrderData {
        maker,
        taker: Address::ZERO,
        token_id: U256::from(123),
        maker_amount: U256::from(1000),
        taker_amount: U256::from(900),
        side: Side::Buy,
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
        None, // Use default salt generator
    );

    let order_data = OrderData {
        maker: different_address,
        taker: Address::ZERO,
        token_id: U256::from(123),
        maker_amount: U256::from(1000),
        taker_amount: U256::from(900),
        side: Side::Buy,
        fee_rate_bps: U256::from(100),
        nonce: U256::from(1),
        signer: Some(different_address),
        expiration: None,
        signature_type: None,
    };

    let result = builder.build_order(order_data).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_custom_salt_generator() {
    // Create a test signer
    let signer = PrivateKeySigner::random();
    let maker = signer.address();
    
    // Use a custom salt generator that always returns a fixed value
    let builder = ExchangeOrderBuilder::new(
        address!("CcCCccccCCCCcCCCCCCcCcCccCcCCCcCcccccccC"),
        1,
        signer,
        Some(Box::new(|| U256::from(99999))), // Fixed salt for testing
    );

    let order_data = OrderData {
        maker,
        taker: Address::ZERO,
        token_id: U256::from(123),
        maker_amount: U256::from(1000),
        taker_amount: U256::from(900),
        side: Side::Buy,
        fee_rate_bps: U256::from(100),
        nonce: U256::from(1),
        signer: None,
        expiration: None,
        signature_type: None,
    };

    let order = builder.build_order(order_data).await.unwrap();

    // Verify custom salt is used
    assert_eq!(order.salt, U256::from(99999));
}


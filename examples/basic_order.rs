use alloy_primitives::{address, U256};
use alloy_signer_local::PrivateKeySigner;
use rs_order_utils::{ExchangeOrderBuilder, OrderData, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a signer from private key (in production, load from secure storage)
    let signer = PrivateKeySigner::random();
    let maker = signer.address();

    println!("Maker address: {:?}", maker);

    // Create order builder with contract address and chain ID
    let builder = ExchangeOrderBuilder::new(
        address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"), // Polymarket exchange contract
        137,                                                   // Polygon chain ID
        signer,
    );

    // Create order data for a BUY order
    let order_data = OrderData {
        maker,
        taker: address!("0000000000000000000000000000000000000000"), // Public order
        token_id: U256::from(123456),                                // CTF token ID
        maker_amount: U256::from(1_000_000),                         // 1 USDC (6 decimals)
        taker_amount: U256::from(950_000),                           // 0.95 tokens
        side: Side::Buy,
        fee_rate_bps: U256::from(100), // 1% fee (100 basis points)
        nonce: U256::from(1),           // Nonce for cancellation
        signer: None,                   // Use maker as signer
        expiration: None,               // No expiration (永久有效)
        signature_type: None,           // Use default EOA signature
    };

    // Build and sign the order
    println!("\nBuilding and signing order...");
    let signed_order = builder.build_signed_order(order_data).await?;

    // Print the result
    println!("\n✅ Order signed successfully!");
    println!("Order hash: 0x{}", hex::encode(builder.build_order_hash(&signed_order.order)));
    println!("Signature: {}", signed_order.signature);
    println!("\nOrder details:");
    println!("  Salt: {}", signed_order.order.salt);
    println!("  Maker: {:?}", signed_order.order.maker);
    println!("  Token ID: {}", signed_order.order.tokenId);
    println!("  Maker Amount: {}", signed_order.order.makerAmount);
    println!("  Taker Amount: {}", signed_order.order.takerAmount);
    println!("  Side: {} ({})", signed_order.order.side, if signed_order.order.side == 0 { "BUY" } else { "SELL" });
    println!("  Fee Rate: {}bps", signed_order.order.feeRateBps);

    Ok(())
}


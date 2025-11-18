# rs-order-utils

🦀 Rust implementation of Polymarket CLOB (Central Limit Order Book) order utilities with EIP-712 typed data signatures.

This library provides a complete Rust port of the TypeScript `clob-order-utils` package, enabling you to create and sign Polymarket exchange orders using EIP-712 standard.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rs_order_utils = "0.1"
alloy-primitives = "0.8"
alloy-signer-local = "0.5"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use alloy_primitives::{address, U256};
use alloy_signer_local::PrivateKeySigner;
use rs_order_utils::{ExchangeOrderBuilder, OrderData, Side};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a signer
    let signer = PrivateKeySigner::random();
    let maker = signer.address();

    // Create order builder
    let builder = ExchangeOrderBuilder::new(
        address!("4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"), // Contract address
        137, // Polygon chain ID
        signer,
    );

    // Create order data
    let order_data = OrderData {
        maker,
        taker: address!("0000000000000000000000000000000000000000"),
        token_id: U256::from(123456),
        maker_amount: U256::from(1_000_000),
        taker_amount: U256::from(950_000),
        side: Side::Buy,
        fee_rate_bps: U256::from(100),
        nonce: U256::from(1),
        signer: None,
        expiration: None,
        signature_type: None,
    };

    // Build and sign the order
    let signed_order = builder.build_signed_order(order_data).await?;
    
    println!("Order signed! Signature: {}", signed_order.signature);
    Ok(())
}
```

## Usage

```bash
# Run examples
cargo run --example basic_order

# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture
```

## Security Considerations

- 🔒 **Private Keys**: Never hardcode private keys. Use environment variables or secure key management
- 🔐 **EIP-712**: All orders are signed using EIP-712 to prevent phishing attacks
- ✅ **Signer Verification**: The builder verifies that the signer matches the wallet address
- 🛡️ **Domain Separation**: Orders are bound to specific chain and contract addresses

## Notice

⚠️ **AI-Generated Code**: This library was generated with AI assistance. While it has been tested, users should:
- Review the code thoroughly before using in production
- Conduct their own security audits
- Test extensively with their specific use cases
- Use at their own risk

## License

MIT

## Acknowledgments

- Original TypeScript implementation: [Polymarket clob-order-utils](https://github.com/Polymarket/clob-order-utils)
- Built with [Alloy](https://github.com/alloy-rs/alloy) - the modern Ethereum Rust library

---

**Made with 🦀 by the Polymarket community**


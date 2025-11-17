# rs-order-utils

🦀 Rust implementation of Polymarket CLOB (Central Limit Order Book) order utilities with EIP-712 typed data signatures.

This library provides a complete Rust port of the TypeScript `clob-order-utils` package, enabling you to create and sign Polymarket exchange orders using EIP-712 standard.

## Features

- ✅ **Complete EIP-712 Implementation**: Full support for typed data signing
- ✅ **Type-Safe**: Leveraging Rust's type system for compile-time guarantees
- ✅ **Compatible**: Produces signatures compatible with the Polymarket exchange contract
- ✅ **Async/Await**: Built on tokio for async operations
- ✅ **Well-Tested**: Comprehensive unit tests included
- ✅ **Zero-Cost Abstractions**: High performance with no runtime overhead

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

## Architecture

### Core Components

#### 1. **ExchangeOrderBuilder**

The main entry point for creating and signing orders.

```rust
pub struct ExchangeOrderBuilder {
    contract_address: Address,
    chain_id: u64,
    signer: PrivateKeySigner,
    domain: Eip712Domain,
}
```

**Methods:**
- `new()` - Create a new builder instance
- `build_signed_order()` - Build and sign an order in one step (主要方法)
- `build_order()` - Build an order object without signing
- `sign_order()` - Sign an existing order
- `build_order_hash()` - Calculate EIP-712 hash of an order

#### 2. **Order Types**

```rust
// Input data for creating an order
pub struct OrderData {
    pub maker: Address,
    pub taker: Address,
    pub token_id: U256,
    pub maker_amount: U256,
    pub taker_amount: U256,
    pub side: Side,
    pub fee_rate_bps: U256,
    pub nonce: U256,
    pub signer: Option<Address>,
    pub expiration: Option<U256>,
    pub signature_type: Option<SignatureType>,
}

// EIP-712 compliant Order struct
pub struct Order {
    pub salt: U256,
    pub maker: Address,
    pub signer: Address,
    pub taker: Address,
    pub tokenId: U256,
    pub makerAmount: U256,
    pub takerAmount: U256,
    pub expiration: U256,
    pub nonce: U256,
    pub feeRateBps: U256,
    pub side: u8,
    pub signatureType: u8,
}

// Signed order with signature
pub struct SignedOrder {
    pub order: Order,
    pub signature: String,
}
```

#### 3. **Enums**

```rust
// Order side
pub enum Side {
    Buy = 0,
    Sell = 1,
}

// Signature types
pub enum SignatureType {
    Eoa = 0,              // Standard EOA signature
    PolyProxy = 1,        // Polymarket Proxy wallet
    PolyGnosisSafe = 2,   // Gnosis Safe multisig
}
```

## Technical Details

### EIP-712 Implementation

The library implements [EIP-712](https://eips.ethereum.org/EIPS/eip-712) typed data signing:

1. **Domain Separator**:
```rust
{
    name: "Polymarket CTF Exchange",
    version: "1",
    chainId: 137,
    verifyingContract: "0x4bFb41d5B3570DeFd03C39a9A4D8dE6Bd8B8982E"
}
```

2. **Order Structure**: Defined using Alloy's `sol!` macro for automatic EIP-712 encoding

3. **Signing Process**:
   - Compute `structHash` = `keccak256(typeHash || encodeData(order))`
   - Compute `domainSeparator` = `hashStruct(domain)`
   - Compute `digest` = `keccak256("\x19\x01" || domainSeparator || structHash)`
   - Sign the digest using ECDSA

### Comparison with TypeScript Version

| Feature | TypeScript | Rust |
|---------|-----------|------|
| **Type Safety** | Runtime | Compile-time |
| **Performance** | ~slower | ~10-100x faster |
| **Memory Safety** | GC | Zero-cost abstractions |
| **EIP-712** | ethers.js | alloy-rs |
| **Async** | Promises | async/await (tokio) |

## Examples

Run the examples:

```bash
# Basic order signing
cargo run --example basic_order

# Run tests
cargo test

# Run with verbose output
cargo test -- --nocapture
```

## Testing

The library includes comprehensive tests:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_build_order

# Run with output
cargo test -- --nocapture --test-threads=1
```

## Project Structure

```
rs-order-utils/
├── Cargo.toml
├── README.md
├── src/
│   ├── lib.rs              # Library entry point
│   ├── builder.rs          # ExchangeOrderBuilder implementation
│   ├── models.rs           # Order, OrderData, SignedOrder types
│   ├── types.rs            # Side, SignatureType enums
│   ├── constants.rs        # Protocol constants
│   ├── error.rs            # Error types
│   └── utils.rs            # Utility functions
├── examples/
│   └── basic_order.rs      # Example usage
└── tests/
    └── ...                 # Integration tests
```

## Relationship to TypeScript Version

This Rust implementation is a complete port of the TypeScript `clob-order-utils` package:

**TypeScript Location**: `../clob-order-utils/`

**Key Correspondences**:

| TypeScript | Rust |
|-----------|------|
| `ExchangeOrderBuilder` class | `builder::ExchangeOrderBuilder` |
| `buildSignedOrder()` | `build_signed_order()` |
| `buildOrder()` | `build_order()` |
| `buildOrderSignature()` | `sign_order()` |
| `buildOrderHash()` | `build_order_hash()` |
| `Side` enum | `types::Side` |
| `SignatureType` enum | `types::SignatureType` |

## Security Considerations

- 🔒 **Private Keys**: Never hardcode private keys. Use environment variables or secure key management
- 🔐 **EIP-712**: All orders are signed using EIP-712 to prevent phishing attacks
- ✅ **Signer Verification**: The builder verifies that the signer matches the wallet address
- 🛡️ **Domain Separation**: Orders are bound to specific chain and contract addresses

## Dependencies

- **alloy-rs**: Modern Ethereum library (successor to ethers-rs)
  - `alloy-primitives`: Core types (Address, U256, etc.)
  - `alloy-sol-types`: Solidity type system and EIP-712
  - `alloy-signer`: Signing abstractions
  - `alloy-signer-local`: Local key signing
- **k256**: ECDSA secp256k1 cryptography
- **tokio**: Async runtime

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Original TypeScript implementation: [Polymarket clob-order-utils](https://github.com/Polymarket/clob-order-utils)
- Built with [Alloy](https://github.com/alloy-rs/alloy) - the modern Ethereum Rust library

---

**Made with 🦀 by the Polymarket community**


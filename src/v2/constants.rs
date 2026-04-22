use alloy_primitives::B256;

/// Protocol name for V2 EIP-712 domain (unchanged from V1).
pub const PROTOCOL_NAME: &str = "Polymarket CTF Exchange";

/// Protocol version for V2 EIP-712 domain (bumped from "1").
pub const PROTOCOL_VERSION: &str = "2";

/// Zero `bytes32` value. Used as default for `metadata` and `builder`
/// when the caller does not provide one.
pub const BYTES32_ZERO: B256 = B256::ZERO;

# Changelog

All notable changes to `rs_order_utils` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.4.2] — 2026-05-09

### Added

- **V2 Poly1271 signing.** Implements the ERC-7739 nested `TypedDataSign` scheme for `Poly1271` (smart-contract wallet) orders. `v2::ExchangeOrderBuilder::build_signed_order` now produces a deposit-wallet-compatible blob verified via `isValidSignature` (ERC-1271); previously emitted a raw 65-byte ECDSA that on-chain validation would reject. New public method `sign_order_poly1271` exposes the same path directly.

### Changed

- `v2::ExchangeOrderBuilder::build_order` skips the EOA-vs-`signer` mismatch check when `signature_type == Poly1271`, since the order's `signer` is the deposit-wallet contract rather than the EOA.

### Notes

- **Cross-language parity gap.** `tests/v2_cross_language_vectors.rs` skips `Poly1271` vectors: the upstream TS `@polymarket/clob-client-v2` SDK still emits raw 65-byte ECDSA. Re-enable once the TS SDK adopts the same ERC-7739 wrapping. EOA / PolyProxy / PolyGnosisSafe coverage is unchanged.

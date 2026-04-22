/// Signature type used by the V2 order.
///
/// V2 adds `Poly1271` (EIP-1271 smart-contract wallet signatures) compared
/// to V1's three variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignatureType {
    /// ECDSA EIP712 signatures signed by EOAs.
    Eoa = 0,
    /// EIP712 signatures signed by EOAs that own Polymarket Proxy wallets.
    PolyProxy = 1,
    /// EIP712 signatures signed by EOAs that own Polymarket Gnosis safes.
    PolyGnosisSafe = 2,
    /// EIP-1271 signatures produced by smart-contract wallets / vaults.
    Poly1271 = 3,
}

impl Default for SignatureType {
    fn default() -> Self {
        SignatureType::Eoa
    }
}

impl From<SignatureType> for u8 {
    fn from(sig_type: SignatureType) -> Self {
        sig_type as u8
    }
}

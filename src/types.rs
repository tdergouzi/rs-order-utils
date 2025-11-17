/// Side of the order (BUY or SELL)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    /// Buy order
    Buy = 0,
    /// Sell order
    Sell = 1,
}

impl From<Side> for u8 {
    fn from(side: Side) -> Self {
        side as u8
    }
}

/// Signature type used by the order
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SignatureType {
    /// ECDSA EIP712 signatures signed by EOAs
    Eoa = 0,
    /// EIP712 signatures signed by EOAs that own Polymarket Proxy wallets
    PolyProxy = 1,
    /// EIP712 signatures signed by EOAs that own Polymarket Gnosis safes
    PolyGnosisSafe = 2,
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


use thiserror::Error;

#[derive(Error, Debug)]
pub enum OrderError {
    #[error("Signer does not match: expected {expected}, got {actual}")]
    SignerMismatch { expected: String, actual: String },

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("EIP712 encoding error: {0}")]
    Eip712Error(String),

    #[error("Signing error: {0}")]
    SigningError(String),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
}

pub type Result<T> = std::result::Result<T, OrderError>;


pub mod builder;
pub mod constants;
pub mod error;
pub mod models;
pub mod types;
pub mod utils;  // Public module for salt generation
pub mod v2;     // V2 protocol support (additive, non-breaking)

// Re-export main types for convenience (V1 remains the default top-level API).
pub use builder::ExchangeOrderBuilder;
pub use error::{OrderError, Result};
pub use models::{Order, OrderData, SignedOrder};
pub use types::{Side, SignatureType};

pub mod builder;
pub mod constants;
pub mod error;
pub mod models;
pub mod types;
pub mod utils;  // Public module for salt generation

// Re-export main types for convenience
pub use builder::ExchangeOrderBuilder;
pub use error::{OrderError, Result};
pub use models::{Order, OrderData, SignedOrder};
pub use types::{Side, SignatureType};

//! # Kraken Client
//!
//! An async Rust client library for the Kraken exchange REST and WebSocket v2 APIs.
//!
//! ## Features
//!
//! - Full REST API support for Kraken Spot trading
//! - WebSocket v2 API with automatic reconnection
//! - Built-in rate limiting
//! - Strong typing for all request/response types
//! - Financial precision with `rust_decimal`
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use kraken_api_client::spot::rest::SpotRestClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = SpotRestClient::new();
//!     let time = client.get_server_time().await?;
//!     println!("Server time: {:?}", time);
//!     Ok(())
//! }
//! ```

pub mod auth;
pub mod error;
pub mod rate_limit;
pub mod spot;
pub mod types;

// Placeholder for future Kraken Futures API support
pub mod futures;

// Re-export commonly used types at crate root
pub use error::KrakenError;
pub use types::common::{BuySell, OrderStatus, OrderType};

/// Result type alias using KrakenError
pub type Result<T> = std::result::Result<T, KrakenError>;

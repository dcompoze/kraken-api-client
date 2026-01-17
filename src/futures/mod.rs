//! Kraken Futures API client.
//!
//! This module provides clients for the Kraken Futures (derivatives) API,
//! supporting both REST and WebSocket interfaces.
//!
//! ## Features
//!
//! - Full REST API support for Futures trading
//! - WebSocket API for real-time market data and order updates
//! - Support for perpetual and fixed-maturity contracts
//! - Position management with margin and PnL tracking
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use kraken_api_client::futures::rest::FuturesRestClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = FuturesRestClient::new();
//!     let tickers = client.get_tickers().await?;
//!     for ticker in tickers {
//!         println!("{}: {} (funding: {:?})",
//!             ticker.symbol, ticker.last, ticker.funding_rate);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Authentication
//!
//! The Futures API uses a different signature algorithm than the Spot API:
//!
//! ```text
//! Futures: HMAC-SHA512(SHA256(postData + nonce + path), secret)
//! Spot:    HMAC-SHA512(path + SHA256(nonce + postData), secret)
//! ```
//!
//! Use the same `Credentials` and `CredentialsProvider` types, but the
//! signature is computed differently.
//!
//! ## API Documentation
//!
//! - REST API: <https://docs.kraken.com/api/docs/futures-api>
//! - WebSocket: <https://docs.kraken.com/api/docs/futures-api/websocket>

mod auth;
pub mod rest;
pub mod types;
pub mod ws;

pub use auth::sign_futures_request;
pub use types::*;
pub use ws::{FuturesStream, FuturesWsClient, FuturesWsEvent};

//! Kraken Futures WebSocket API client.
//!
//! This module provides a WebSocket client for the Kraken Futures API,
//! supporting both public market data feeds and authenticated private feeds.
//!
//! ## Features
//!
//! - Real-time market data (ticker, order book, trades)
//! - Private feeds (orders, fills, positions, balances)
//! - Automatic reconnection with exponential backoff
//! - Subscription restoration after reconnect
//! - Challenge-based authentication
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use kraken_api_client::futures::ws::{FuturesWsClient, feeds};
//! use futures_util::StreamExt;
//!
//! // Connect to public feeds
//! let client = FuturesWsClient::new();
//! let mut stream = client.connect_public().await?;
//!
//! // Subscribe to ticker for BTC perpetual
//! stream.subscribe_public(feeds::TICKER, vec!["PI_XBTUSD"]).await?;
//!
//! while let Some(msg) = stream.next().await {
//!     println!("Message: {:?}", msg);
//! }
//! ```
//!
//! ## Authentication
//!
//! The Futures WebSocket API uses challenge-based authentication:
//!
//! 1. Request a challenge with your API key
//! 2. Sign the challenge using HMAC-SHA512(SHA256(challenge), secret)
//! 3. Include both original and signed challenge in private subscriptions
//!
//! ```rust,ignore
//! use kraken_api_client::futures::ws::FuturesWsClient;
//! use kraken_api_client::auth::StaticCredentials;
//! use std::sync::Arc;
//!
//! let credentials = Arc::new(StaticCredentials::new("api_key", "api_secret"));
//! let client = FuturesWsClient::new();
//!
//! // Connect and authenticate
//! let mut stream = client.connect_private(credentials).await?;
//!
//! // Subscribe to private feeds
//! stream.subscribe_private(feeds::OPEN_ORDERS).await?;
//! stream.subscribe_private(feeds::FILLS).await?;
//! ```

mod client;
mod messages;
mod stream;

pub use client::{FuturesWsClient, WsConfig, WsConfigBuilder};
pub use messages::*;
pub use stream::{FuturesStream, FuturesWsEvent};

/// WebSocket endpoint URLs.
pub mod endpoints {
    /// Public WebSocket endpoint.
    pub const WS_PUBLIC: &str = "wss://futures.kraken.com/ws/v1";
    /// Demo/testnet WebSocket endpoint.
    pub const WS_DEMO: &str = "wss://demo-futures.kraken.com/ws/v1";
}

/// Available feed names.
pub mod feeds {
    // Public feeds
    /// Order book feed - provides order book snapshots and updates.
    pub const BOOK: &str = "book";
    /// Ticker feed - price and volume information.
    pub const TICKER: &str = "ticker";
    /// Lightweight ticker feed - minimal ticker data.
    pub const TICKER_LITE: &str = "ticker_lite";
    /// Trade feed - individual trade executions.
    pub const TRADE: &str = "trade";

    // Private feeds
    /// Open orders feed - user's open orders.
    pub const OPEN_ORDERS: &str = "open_orders";
    /// Fills feed - user's trade executions.
    pub const FILLS: &str = "fills";
    /// Open positions feed - user's open positions.
    pub const OPEN_POSITIONS: &str = "open_positions";
    /// Balances feed - account balances.
    pub const BALANCES: &str = "balances";
    /// Account log feed - account activity.
    pub const ACCOUNT_LOG: &str = "account_log";
}

//! Kraken Futures WebSocket API client with public and private feeds.
//!
//! Supports automatic reconnection with subscription restoration.
//! Private feeds use challenge-based authentication (see `sign_challenge`).

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

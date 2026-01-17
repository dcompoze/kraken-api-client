//! Base WebSocket message types.

use serde::{Deserialize, Serialize};

/// WebSocket request message.
#[derive(Debug, Clone, Serialize)]
pub struct WsRequest<T> {
    /// The method to call.
    pub method: String,
    /// Request parameters.
    pub params: T,
    /// Optional request ID for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req_id: Option<u64>,
}

impl<T> WsRequest<T> {
    /// Create a new request.
    pub fn new(method: impl Into<String>, params: T) -> Self {
        Self {
            method: method.into(),
            params,
            req_id: None,
        }
    }

    /// Set the request ID.
    pub fn with_req_id(mut self, id: u64) -> Self {
        self.req_id = Some(id);
        self
    }
}

/// WebSocket response message.
#[derive(Debug, Clone, Deserialize)]
pub struct WsResponse<T> {
    /// The method that was called.
    pub method: String,
    /// Whether the request succeeded.
    pub success: bool,
    /// Result data (if successful).
    #[serde(default)]
    pub result: Option<T>,
    /// Error information (if failed).
    #[serde(default)]
    pub error: Option<String>,
    /// The request ID (if provided in request).
    #[serde(default)]
    pub req_id: Option<u64>,
    /// Timestamp when message was received by Kraken.
    #[serde(default)]
    pub time_in: Option<String>,
    /// Timestamp when message was sent by Kraken.
    #[serde(default)]
    pub time_out: Option<String>,
}

/// Channel subscription request.
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeParams {
    /// Channel name.
    pub channel: String,
    /// Symbol(s) to subscribe to (for market data channels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<Vec<String>>,
    /// Authentication token (for private channels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Snapshot flag (whether to receive initial snapshot).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot: Option<bool>,
    /// Depth for order book channel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<u32>,
}

impl SubscribeParams {
    /// Create a subscription for a public channel.
    pub fn public(channel: impl Into<String>, symbols: Vec<String>) -> Self {
        Self {
            channel: channel.into(),
            symbol: Some(symbols),
            token: None,
            snapshot: None,
            depth: None,
        }
    }

    /// Create a subscription for a private channel.
    pub fn private(channel: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            channel: channel.into(),
            symbol: None,
            token: Some(token.into()),
            snapshot: None,
            depth: None,
        }
    }

    /// Set snapshot flag.
    pub fn with_snapshot(mut self, snapshot: bool) -> Self {
        self.snapshot = Some(snapshot);
        self
    }

    /// Set order book depth.
    pub fn with_depth(mut self, depth: u32) -> Self {
        self.depth = Some(depth);
        self
    }
}

/// Channel names.
pub mod channels {
    // Public channels
    pub const TICKER: &str = "ticker";
    pub const BOOK: &str = "book";
    pub const LEVEL3: &str = "level3";
    pub const OHLC: &str = "ohlc";
    pub const TRADE: &str = "trade";
    pub const INSTRUMENT: &str = "instrument";

    // Admin channels
    pub const STATUS: &str = "status";
    pub const HEARTBEAT: &str = "heartbeat";

    // Private channels
    pub const EXECUTIONS: &str = "executions";
    pub const BALANCES: &str = "balances";
}

/// Common subscription result.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscriptionResult {
    /// Channel name.
    pub channel: String,
    /// Subscribed symbols.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Snapshot flag.
    #[serde(default)]
    pub snapshot: Option<bool>,
}

/// Error response from WebSocket.
#[derive(Debug, Clone, Deserialize)]
pub struct WsError {
    /// Error code.
    pub code: Option<i32>,
    /// Error message.
    pub message: String,
}

//! Futures WebSocket message types.
//!
//! The Futures WebSocket API uses a different message format than Spot:
//! - Subscriptions use `event` field instead of `method`
//! - Feeds use `feed` field instead of `channel`
//! - Authentication uses challenge/response instead of tokens

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};


// Request Messages


/// Challenge request for authentication.
#[derive(Debug, Clone, Serialize)]
pub struct ChallengeRequest {
    /// Event type (always "challenge").
    pub event: &'static str,
    /// API key.
    pub api_key: String,
}

impl ChallengeRequest {
    /// Create a new challenge request.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            event: "challenge",
            api_key: api_key.into(),
        }
    }
}

/// Subscribe request for a public feed.
#[derive(Debug, Clone, Serialize)]
pub struct SubscribeRequest {
    /// Event type (always "subscribe").
    pub event: &'static str,
    /// Feed name.
    pub feed: String,
    /// Product IDs to subscribe to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_ids: Option<Vec<String>>,
}

impl SubscribeRequest {
    /// Create a new public subscription request.
    pub fn public(feed: impl Into<String>, product_ids: Vec<String>) -> Self {
        Self {
            event: "subscribe",
            feed: feed.into(),
            product_ids: if product_ids.is_empty() {
                None
            } else {
                Some(product_ids)
            },
        }
    }

    /// Create a new subscription for all products.
    pub fn all(feed: impl Into<String>) -> Self {
        Self {
            event: "subscribe",
            feed: feed.into(),
            product_ids: None,
        }
    }
}

/// Subscribe request for a private feed (authenticated).
#[derive(Debug, Clone, Serialize)]
pub struct PrivateSubscribeRequest {
    /// Event type (always "subscribe").
    pub event: &'static str,
    /// Feed name.
    pub feed: String,
    /// Original challenge (from server).
    pub original_challenge: String,
    /// Signed challenge (HMAC-SHA512 of SHA256 hash).
    pub signed_challenge: String,
    /// Product IDs to subscribe to (optional for private feeds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_ids: Option<Vec<String>>,
}

impl PrivateSubscribeRequest {
    /// Create a new private subscription request.
    pub fn new(
        feed: impl Into<String>,
        original_challenge: String,
        signed_challenge: String,
    ) -> Self {
        Self {
            event: "subscribe",
            feed: feed.into(),
            original_challenge,
            signed_challenge,
            product_ids: None,
        }
    }

    /// Add product IDs filter.
    pub fn with_product_ids(mut self, product_ids: Vec<String>) -> Self {
        self.product_ids = Some(product_ids);
        self
    }
}

/// Unsubscribe request.
#[derive(Debug, Clone, Serialize)]
pub struct UnsubscribeRequest {
    /// Event type (always "unsubscribe").
    pub event: &'static str,
    /// Feed name.
    pub feed: String,
    /// Product IDs to unsubscribe from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_ids: Option<Vec<String>>,
}

impl UnsubscribeRequest {
    /// Create a new unsubscription request.
    pub fn new(feed: impl Into<String>, product_ids: Vec<String>) -> Self {
        Self {
            event: "unsubscribe",
            feed: feed.into(),
            product_ids: if product_ids.is_empty() {
                None
            } else {
                Some(product_ids)
            },
        }
    }
}


// Response Messages


/// Challenge response from the server.
#[derive(Debug, Clone, Deserialize)]
pub struct ChallengeResponse {
    /// Event type (should be "challenge").
    pub event: String,
    /// The challenge message (UUID to sign).
    pub message: String,
}

/// Subscription confirmation.
#[derive(Debug, Clone, Deserialize)]
pub struct SubscribedResponse {
    /// Event type (should be "subscribed").
    pub event: String,
    /// Feed name.
    pub feed: String,
    /// Product IDs subscribed to.
    #[serde(default)]
    pub product_ids: Option<Vec<String>>,
}

/// Unsubscription confirmation.
#[derive(Debug, Clone, Deserialize)]
pub struct UnsubscribedResponse {
    /// Event type (should be "unsubscribed").
    pub event: String,
    /// Feed name.
    pub feed: String,
    /// Product IDs unsubscribed from.
    #[serde(default)]
    pub product_ids: Option<Vec<String>>,
}

/// Error response from the server.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorResponse {
    /// Event type (should be "error").
    pub event: String,
    /// Error message.
    pub message: String,
}

/// Info/alert response from the server.
#[derive(Debug, Clone, Deserialize)]
pub struct InfoResponse {
    /// Event type (should be "info" or "alert").
    pub event: String,
    /// Info/alert message.
    pub message: String,
    /// Version info (optional).
    #[serde(default)]
    pub version: Option<String>,
}


// Feed Data Messages


/// Order book update message.
#[derive(Debug, Clone, Deserialize)]
pub struct BookMessage {
    /// Feed name.
    pub feed: String,
    /// Product ID.
    pub product_id: String,
    /// Sequence number.
    #[serde(default)]
    pub seq: Option<u64>,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub timestamp: Option<u64>,
    /// Bids (price levels).
    #[serde(default)]
    pub bids: Vec<BookLevel>,
    /// Asks (price levels).
    #[serde(default)]
    pub asks: Vec<BookLevel>,
}

/// Order book snapshot message.
#[derive(Debug, Clone, Deserialize)]
pub struct BookSnapshotMessage {
    /// Feed name.
    pub feed: String,
    /// Product ID.
    pub product_id: String,
    /// Sequence number.
    #[serde(default)]
    pub seq: Option<u64>,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub timestamp: Option<u64>,
    /// Bids (price levels).
    #[serde(default)]
    pub bids: Vec<BookLevel>,
    /// Asks (price levels).
    #[serde(default)]
    pub asks: Vec<BookLevel>,
}

/// A price level in the order book.
#[derive(Debug, Clone, Deserialize)]
pub struct BookLevel {
    /// Price.
    pub price: Decimal,
    /// Quantity.
    pub qty: Decimal,
}

/// Ticker message.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerMessage {
    /// Feed name.
    pub feed: String,
    /// Product ID.
    pub product_id: String,
    /// Timestamp in milliseconds.
    #[serde(default)]
    pub time: Option<u64>,
    /// Best bid price.
    #[serde(default)]
    pub bid: Option<Decimal>,
    /// Best bid size.
    #[serde(default)]
    pub bid_size: Option<Decimal>,
    /// Best ask price.
    #[serde(default)]
    pub ask: Option<Decimal>,
    /// Best ask size.
    #[serde(default)]
    pub ask_size: Option<Decimal>,
    /// Last trade price.
    #[serde(default)]
    pub last: Option<Decimal>,
    /// Last trade size.
    #[serde(default)]
    pub last_size: Option<Decimal>,
    /// 24h volume.
    #[serde(default)]
    pub volume: Option<Decimal>,
    /// Mark price.
    #[serde(default, rename = "markPrice")]
    pub mark_price: Option<Decimal>,
    /// Open interest.
    #[serde(default, rename = "openInterest")]
    pub open_interest: Option<Decimal>,
    /// Funding rate.
    #[serde(default)]
    pub funding_rate: Option<Decimal>,
    /// Funding rate prediction.
    #[serde(default)]
    pub funding_rate_prediction: Option<Decimal>,
    /// Change in last 24h (percentage).
    #[serde(default)]
    pub change: Option<Decimal>,
    /// Premium.
    #[serde(default)]
    pub premium: Option<Decimal>,
    /// Index price.
    #[serde(default)]
    pub index: Option<Decimal>,
    /// Post only flag.
    #[serde(default)]
    pub post_only: Option<bool>,
    /// Suspended flag.
    #[serde(default)]
    pub suspended: Option<bool>,
}

/// Trade message.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeMessage {
    /// Feed name.
    pub feed: String,
    /// Product ID.
    pub product_id: String,
    /// Trade ID.
    #[serde(default)]
    pub uid: Option<String>,
    /// Trade side ("buy" or "sell").
    #[serde(default)]
    pub side: Option<String>,
    /// Trade type.
    #[serde(rename = "type", default)]
    pub trade_type: Option<String>,
    /// Trade price.
    #[serde(default)]
    pub price: Option<Decimal>,
    /// Trade quantity.
    #[serde(default)]
    pub qty: Option<Decimal>,
    /// Trade time in milliseconds.
    #[serde(default)]
    pub time: Option<u64>,
    /// Sequence number.
    #[serde(default)]
    pub seq: Option<u64>,
}

/// Trades snapshot message.
#[derive(Debug, Clone, Deserialize)]
pub struct TradesSnapshotMessage {
    /// Feed name.
    pub feed: String,
    /// Product ID.
    pub product_id: String,
    /// List of recent trades.
    pub trades: Vec<TradeItem>,
}

/// A single trade item in the snapshot.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeItem {
    /// Trade ID.
    #[serde(default)]
    pub uid: Option<String>,
    /// Trade side.
    pub side: String,
    /// Trade type.
    #[serde(rename = "type", default)]
    pub trade_type: Option<String>,
    /// Trade price.
    pub price: Decimal,
    /// Trade quantity.
    pub qty: Decimal,
    /// Trade time in milliseconds.
    pub time: u64,
    /// Sequence number.
    #[serde(default)]
    pub seq: Option<u64>,
}


// Private Feed Messages


/// Open orders message.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenOrdersMessage {
    /// Feed name.
    pub feed: String,
    /// List of open orders (for snapshot).
    #[serde(default)]
    pub orders: Option<Vec<WsOrder>>,
    /// Single order update.
    #[serde(flatten)]
    pub order: Option<WsOrder>,
}

/// Order data from WebSocket.
#[derive(Debug, Clone, Deserialize)]
pub struct WsOrder {
    /// Order ID.
    #[serde(default)]
    pub order_id: Option<String>,
    /// Client order ID.
    #[serde(default)]
    pub cli_ord_id: Option<String>,
    /// Instrument/symbol.
    #[serde(default)]
    pub instrument: Option<String>,
    /// Order side ("buy" or "sell").
    #[serde(default)]
    pub side: Option<String>,
    /// Order type ("lmt", "mkt", "stp", "take_profit").
    #[serde(default)]
    pub order_type: Option<String>,
    /// Limit price.
    #[serde(default)]
    pub limit_price: Option<Decimal>,
    /// Stop price.
    #[serde(default)]
    pub stop_price: Option<Decimal>,
    /// Original quantity.
    #[serde(default)]
    pub qty: Option<Decimal>,
    /// Filled quantity.
    #[serde(default)]
    pub filled: Option<Decimal>,
    /// Reduce only flag.
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Timestamp.
    #[serde(default)]
    pub time: Option<u64>,
    /// Last update timestamp.
    #[serde(default)]
    pub last_update_time: Option<u64>,
    /// Order status.
    #[serde(default)]
    pub status: Option<String>,
    /// Reason (for cancellation).
    #[serde(default)]
    pub reason: Option<String>,
}

/// Fills message.
#[derive(Debug, Clone, Deserialize)]
pub struct FillsMessage {
    /// Feed name.
    pub feed: String,
    /// List of fills (for snapshot).
    #[serde(default)]
    pub fills: Option<Vec<WsFill>>,
    /// Single fill update (for realtime).
    #[serde(flatten)]
    pub fill: Option<WsFill>,
}

/// Fill data from WebSocket.
#[derive(Debug, Clone, Deserialize)]
pub struct WsFill {
    /// Fill ID.
    #[serde(default)]
    pub fill_id: Option<String>,
    /// Order ID.
    #[serde(default)]
    pub order_id: Option<String>,
    /// Client order ID.
    #[serde(default)]
    pub cli_ord_id: Option<String>,
    /// Instrument/symbol.
    #[serde(default)]
    pub instrument: Option<String>,
    /// Fill side ("buy" or "sell").
    #[serde(default)]
    pub side: Option<String>,
    /// Fill price.
    #[serde(default)]
    pub price: Option<Decimal>,
    /// Fill quantity.
    #[serde(default)]
    pub qty: Option<Decimal>,
    /// Fill type ("maker", "taker", "liquidation").
    #[serde(default)]
    pub fill_type: Option<String>,
    /// Fee paid.
    #[serde(default)]
    pub fee_paid: Option<Decimal>,
    /// Fee currency.
    #[serde(default)]
    pub fee_currency: Option<String>,
    /// Timestamp.
    #[serde(default)]
    pub time: Option<u64>,
}

/// Open positions message.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenPositionsMessage {
    /// Feed name.
    pub feed: String,
    /// Account type.
    #[serde(default)]
    pub account: Option<String>,
    /// List of positions (for snapshot).
    #[serde(default)]
    pub positions: Option<Vec<WsPosition>>,
    /// Single position update.
    #[serde(flatten)]
    pub position: Option<WsPosition>,
}

/// Position data from WebSocket.
#[derive(Debug, Clone, Deserialize)]
pub struct WsPosition {
    /// Instrument/symbol.
    #[serde(default)]
    pub instrument: Option<String>,
    /// Position balance (positive = long, negative = short).
    #[serde(default)]
    pub balance: Option<Decimal>,
    /// Entry price.
    #[serde(default)]
    pub entry_price: Option<Decimal>,
    /// Mark price.
    #[serde(default)]
    pub mark_price: Option<Decimal>,
    /// Index price.
    #[serde(default)]
    pub index_price: Option<Decimal>,
    /// PnL (unrealized).
    #[serde(default)]
    pub pnl: Option<Decimal>,
    /// Effective leverage.
    #[serde(default)]
    pub effective_leverage: Option<Decimal>,
    /// Initial margin.
    #[serde(default)]
    pub initial_margin: Option<Decimal>,
    /// Maintenance margin.
    #[serde(default)]
    pub maintenance_margin: Option<Decimal>,
    /// Return on equity.
    #[serde(default)]
    pub return_on_equity: Option<Decimal>,
}

/// Balances message.
#[derive(Debug, Clone, Deserialize)]
pub struct BalancesMessage {
    /// Feed name.
    pub feed: String,
    /// Account type.
    #[serde(default)]
    pub account: Option<String>,
    /// Sequence number.
    #[serde(default)]
    pub seq: Option<u64>,
    /// Total balance.
    #[serde(default)]
    pub balance: Option<Decimal>,
    /// Available balance.
    #[serde(default)]
    pub available: Option<Decimal>,
    /// Margin used.
    #[serde(default)]
    pub margin: Option<Decimal>,
    /// PnL.
    #[serde(default)]
    pub pnl: Option<Decimal>,
    /// Collateral balances (for flex/multi-collateral accounts).
    #[serde(default)]
    pub flex_futures: Option<FlexFuturesBalance>,
}

/// Flex/Multi-collateral account balance.
#[derive(Debug, Clone, Deserialize)]
pub struct FlexFuturesBalance {
    /// Currencies and their balances.
    #[serde(default)]
    pub currencies: Option<serde_json::Value>,
    /// Portfolio value.
    #[serde(default)]
    pub portfolio_value: Option<Decimal>,
    /// Available margin.
    #[serde(default)]
    pub available_margin: Option<Decimal>,
    /// Initial margin.
    #[serde(default)]
    pub initial_margin: Option<Decimal>,
    /// Maintenance margin.
    #[serde(default)]
    pub maintenance_margin: Option<Decimal>,
    /// Unrealized PnL.
    #[serde(default)]
    pub unrealized_pnl: Option<Decimal>,
}


// Tests


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_challenge_request_serialization() {
        let req = ChallengeRequest::new("my_api_key");
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"event\":\"challenge\""));
        assert!(json.contains("\"api_key\":\"my_api_key\""));
    }

    #[test]
    fn test_subscribe_request_serialization() {
        let req = SubscribeRequest::public("book", vec!["PI_XBTUSD".into()]);
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"event\":\"subscribe\""));
        assert!(json.contains("\"feed\":\"book\""));
        assert!(json.contains("\"product_ids\":[\"PI_XBTUSD\"]"));
    }

    #[test]
    fn test_challenge_response_deserialization() {
        let json = r#"{"event":"challenge","message":"123e4567-e89b-12d3-a456-426614174000"}"#;
        let resp: ChallengeResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.event, "challenge");
        assert_eq!(resp.message, "123e4567-e89b-12d3-a456-426614174000");
    }

    #[test]
    fn test_book_message_deserialization() {
        let json = r#"{
            "feed": "book",
            "product_id": "PI_XBTUSD",
            "seq": 1234,
            "timestamp": 1640000000000,
            "bids": [{"price": "50000.0", "qty": "1.5"}],
            "asks": [{"price": "50001.0", "qty": "2.0"}]
        }"#;
        let msg: BookMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.feed, "book");
        assert_eq!(msg.product_id, "PI_XBTUSD");
        assert_eq!(msg.seq, Some(1234));
        assert_eq!(msg.bids.len(), 1);
        assert_eq!(msg.asks.len(), 1);
    }

    #[test]
    fn test_ticker_message_deserialization() {
        let json = r#"{
            "feed": "ticker",
            "product_id": "PI_XBTUSD",
            "bid": "50000.0",
            "ask": "50001.0",
            "last": "50000.5",
            "volume": "1000.0",
            "funding_rate": "0.0001"
        }"#;
        let msg: TickerMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.feed, "ticker");
        assert_eq!(msg.product_id, "PI_XBTUSD");
        assert!(msg.bid.is_some());
        assert!(msg.ask.is_some());
    }

    #[test]
    fn test_private_subscribe_request() {
        let req = PrivateSubscribeRequest::new(
            "open_orders",
            "challenge-uuid".to_string(),
            "signed-challenge".to_string(),
        );
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"event\":\"subscribe\""));
        assert!(json.contains("\"feed\":\"open_orders\""));
        assert!(json.contains("\"original_challenge\":\"challenge-uuid\""));
        assert!(json.contains("\"signed_challenge\":\"signed-challenge\""));
    }
}

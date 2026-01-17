//! Market data WebSocket messages.

use rust_decimal::Decimal;
use serde::Deserialize;

/// Ticker update message.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerMessage {
    /// Channel name.
    pub channel: String,
    /// Message type ("snapshot" or "update").
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Ticker data.
    pub data: Vec<TickerData>,
}

/// Ticker data.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerData {
    /// Symbol.
    pub symbol: String,
    /// Best bid price.
    pub bid: Decimal,
    /// Best bid quantity.
    pub bid_qty: Decimal,
    /// Best ask price.
    pub ask: Decimal,
    /// Best ask quantity.
    pub ask_qty: Decimal,
    /// Last trade price.
    pub last: Decimal,
    /// 24h volume.
    pub volume: Decimal,
    /// Volume weighted average price.
    pub vwap: Decimal,
    /// 24h low.
    pub low: Decimal,
    /// 24h high.
    pub high: Decimal,
    /// Price change (absolute).
    pub change: Decimal,
    /// Price change (percentage).
    pub change_pct: Decimal,
}

/// Order book update message.
#[derive(Debug, Clone, Deserialize)]
pub struct BookMessage {
    /// Channel name.
    pub channel: String,
    /// Message type ("snapshot" or "update").
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Book data.
    pub data: Vec<BookData>,
}

/// Order book data.
#[derive(Debug, Clone, Deserialize)]
pub struct BookData {
    /// Symbol.
    pub symbol: String,
    /// Bid levels.
    #[serde(default)]
    pub bids: Vec<BookLevel>,
    /// Ask levels.
    #[serde(default)]
    pub asks: Vec<BookLevel>,
    /// Checksum for validation.
    #[serde(default)]
    pub checksum: Option<u32>,
    /// Timestamp.
    #[serde(default)]
    pub timestamp: Option<String>,
}

/// Single order book level.
#[derive(Debug, Clone, Deserialize)]
pub struct BookLevel {
    /// Price level.
    pub price: Decimal,
    /// Quantity at price level.
    pub qty: Decimal,
}

/// Trade message.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeMessage {
    /// Channel name.
    pub channel: String,
    /// Message type.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Trade data.
    pub data: Vec<TradeData>,
}

/// Single trade data.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeData {
    /// Symbol.
    pub symbol: String,
    /// Trade side (buy/sell).
    pub side: String,
    /// Trade price.
    pub price: Decimal,
    /// Trade quantity.
    pub qty: Decimal,
    /// Order type.
    pub ord_type: String,
    /// Trade ID.
    pub trade_id: i64,
    /// Timestamp.
    pub timestamp: String,
}

/// OHLC (candlestick) message.
#[derive(Debug, Clone, Deserialize)]
pub struct OhlcMessage {
    /// Channel name.
    pub channel: String,
    /// Message type.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// OHLC data.
    pub data: Vec<OhlcData>,
}

/// OHLC data.
#[derive(Debug, Clone, Deserialize)]
pub struct OhlcData {
    /// Symbol.
    pub symbol: String,
    /// Open price.
    pub open: Decimal,
    /// High price.
    pub high: Decimal,
    /// Low price.
    pub low: Decimal,
    /// Close price.
    pub close: Decimal,
    /// Volume.
    pub volume: Decimal,
    /// Volume weighted average price.
    pub vwap: Decimal,
    /// Number of trades.
    pub trades: u64,
    /// Interval start timestamp.
    pub interval_begin: String,
    /// Interval end timestamp.
    pub interval: u32,
    /// Timestamp.
    pub timestamp: String,
}

/// Instrument info message.
#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentMessage {
    /// Channel name.
    pub channel: String,
    /// Message type.
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Instrument data.
    pub data: InstrumentData,
}

/// Instrument data.
#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentData {
    /// List of assets.
    #[serde(default)]
    pub assets: Vec<AssetData>,
    /// List of pairs.
    #[serde(default)]
    pub pairs: Vec<PairData>,
}

/// Asset info.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetData {
    /// Asset ID.
    pub id: String,
    /// Asset status.
    pub status: String,
    /// Precision.
    #[serde(default)]
    pub precision: Option<u8>,
    /// Display precision.
    #[serde(default)]
    pub precision_display: Option<u8>,
    /// Whether borrowable.
    #[serde(default)]
    pub borrowable: Option<bool>,
    /// Collateral value.
    #[serde(default)]
    pub collateral_value: Option<Decimal>,
    /// Margin rate.
    #[serde(default)]
    pub margin_rate: Option<Decimal>,
}

/// Trading pair info.
#[derive(Debug, Clone, Deserialize)]
pub struct PairData {
    /// Symbol.
    pub symbol: String,
    /// Base asset.
    pub base: String,
    /// Quote asset.
    pub quote: String,
    /// Pair status.
    pub status: String,
    /// Whether marginable.
    #[serde(default)]
    pub marginable: Option<bool>,
    /// Whether has index.
    #[serde(default)]
    pub has_index: Option<bool>,
    /// Price precision.
    #[serde(default)]
    pub price_precision: Option<u8>,
    /// Quantity precision.
    #[serde(default)]
    pub qty_precision: Option<u8>,
    /// Minimum order quantity.
    #[serde(default)]
    pub qty_min: Option<Decimal>,
    /// Minimum order cost.
    #[serde(default)]
    pub cost_min: Option<Decimal>,
}

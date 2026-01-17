//! Types for public REST API endpoints.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::OhlcInterval;

/// Server time response.
#[derive(Debug, Clone, Deserialize)]
pub struct ServerTime {
    /// Unix timestamp.
    pub unixtime: i64,
    /// RFC 1123 formatted time string.
    pub rfc1123: String,
}

/// System status response.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemStatus {
    /// Current system status.
    pub status: String,
    /// Current timestamp.
    pub timestamp: String,
}

/// Request parameters for asset info.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AssetInfoRequest {
    /// Comma-separated list of assets to get info for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Asset class (default: "currency").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aclass: Option<String>,
}

impl AssetInfoRequest {
    /// Create a new request for specific assets.
    pub fn for_assets(assets: impl Into<String>) -> Self {
        Self {
            asset: Some(assets.into()),
            aclass: None,
        }
    }
}

/// Asset information.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetInfo {
    /// Asset class.
    pub aclass: String,
    /// Alternate name.
    pub altname: String,
    /// Number of decimals.
    pub decimals: u8,
    /// Display decimals.
    pub display_decimals: u8,
    /// Collateral value (if applicable).
    #[serde(default)]
    pub collateral_value: Option<Decimal>,
    /// Asset status.
    #[serde(default)]
    pub status: Option<String>,
}

/// Request parameters for asset pairs.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AssetPairsRequest {
    /// Comma-separated list of pairs to get info for.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pair: Option<String>,
    /// Info level: "info" (default), "leverage", "fees", or "margin".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
}

impl AssetPairsRequest {
    /// Create a new request for specific pairs.
    pub fn for_pairs(pairs: impl Into<String>) -> Self {
        Self {
            pair: Some(pairs.into()),
            info: None,
        }
    }
}

/// Tradable asset pair information.
#[derive(Debug, Clone, Deserialize)]
pub struct AssetPair {
    /// Alternate pair name.
    pub altname: String,
    /// WebSocket pair name.
    #[serde(default)]
    pub wsname: Option<String>,
    /// Asset class of base component.
    pub aclass_base: String,
    /// Base asset.
    pub base: String,
    /// Asset class of quote component.
    pub aclass_quote: String,
    /// Quote asset.
    pub quote: String,
    /// Volume lot size.
    #[serde(default)]
    pub lot: Option<String>,
    /// Scaling decimal places for cost.
    pub cost_decimals: u8,
    /// Scaling decimal places for pair.
    pub pair_decimals: u8,
    /// Scaling decimal places for volume.
    pub lot_decimals: u8,
    /// Amount to multiply lot volume by to get currency volume.
    pub lot_multiplier: u32,
    /// Array of leverage amounts available.
    #[serde(default)]
    pub leverage_buy: Vec<u32>,
    /// Array of leverage amounts available for selling.
    #[serde(default)]
    pub leverage_sell: Vec<u32>,
    /// Fee schedule array.
    #[serde(default)]
    pub fees: Vec<(u64, Decimal)>,
    /// Maker fee schedule array.
    #[serde(default)]
    pub fees_maker: Option<Vec<(u64, Decimal)>>,
    /// Minimum order size.
    #[serde(default)]
    pub ordermin: Option<Decimal>,
    /// Minimum order cost.
    #[serde(default)]
    pub costmin: Option<Decimal>,
    /// Minimum price tick size.
    #[serde(default)]
    pub tick_size: Option<Decimal>,
    /// Pair status.
    #[serde(default)]
    pub status: Option<String>,
    /// Maximum long margin position size.
    #[serde(default)]
    pub long_position_limit: Option<u64>,
    /// Maximum short margin position size.
    #[serde(default)]
    pub short_position_limit: Option<u64>,
}

/// Ticker information.
#[derive(Debug, Clone, Deserialize)]
pub struct TickerInfo {
    /// Ask price [price, whole lot volume, lot volume].
    pub a: Vec<Decimal>,
    /// Bid price [price, whole lot volume, lot volume].
    pub b: Vec<Decimal>,
    /// Last trade closed [price, lot volume].
    pub c: Vec<Decimal>,
    /// Volume [today, last 24 hours].
    pub v: Vec<Decimal>,
    /// Volume weighted average price [today, last 24 hours].
    pub p: Vec<Decimal>,
    /// Number of trades [today, last 24 hours].
    pub t: Vec<u64>,
    /// Low [today, last 24 hours].
    pub l: Vec<Decimal>,
    /// High [today, last 24 hours].
    pub h: Vec<Decimal>,
    /// Today's opening price.
    pub o: Decimal,
}

impl TickerInfo {
    /// Get the current ask price.
    pub fn ask_price(&self) -> Decimal {
        self.a.first().copied().unwrap_or_default()
    }

    /// Get the current bid price.
    pub fn bid_price(&self) -> Decimal {
        self.b.first().copied().unwrap_or_default()
    }

    /// Get the last trade price.
    pub fn last_price(&self) -> Decimal {
        self.c.first().copied().unwrap_or_default()
    }

    /// Get today's volume.
    pub fn volume_today(&self) -> Decimal {
        self.v.first().copied().unwrap_or_default()
    }

    /// Get 24-hour volume.
    pub fn volume_24h(&self) -> Decimal {
        self.v.get(1).copied().unwrap_or_default()
    }
}

/// Request parameters for OHLC data.
#[derive(Debug, Clone, Serialize)]
pub struct OhlcRequest {
    /// Asset pair.
    pub pair: String,
    /// Time frame interval in minutes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<OhlcInterval>,
    /// Return data since given timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
}

impl OhlcRequest {
    /// Create a new OHLC request for a pair.
    pub fn new(pair: impl Into<String>) -> Self {
        Self {
            pair: pair.into(),
            interval: None,
            since: None,
        }
    }

    /// Set the interval.
    pub fn interval(mut self, interval: OhlcInterval) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Set the since timestamp.
    pub fn since(mut self, since: i64) -> Self {
        self.since = Some(since);
        self
    }
}

/// OHLC data response.
#[derive(Debug, Clone, Deserialize)]
pub struct OhlcResponse {
    /// OHLC data keyed by pair name.
    #[serde(flatten)]
    pub data: HashMap<String, Vec<OhlcEntry>>,
    /// Last timestamp for pagination.
    pub last: i64,
}

/// Single OHLC entry.
/// Format: [time, open, high, low, close, vwap, volume, count]
#[derive(Debug, Clone)]
pub struct OhlcEntry {
    /// Unix timestamp.
    pub time: i64,
    /// Open price.
    pub open: Decimal,
    /// High price.
    pub high: Decimal,
    /// Low price.
    pub low: Decimal,
    /// Close price.
    pub close: Decimal,
    /// Volume weighted average price.
    pub vwap: Decimal,
    /// Volume.
    pub volume: Decimal,
    /// Number of trades.
    pub count: u64,
}

impl<'de> Deserialize<'de> for OhlcEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: (
            i64,
            Decimal,
            Decimal,
            Decimal,
            Decimal,
            Decimal,
            Decimal,
            u64,
        ) = Deserialize::deserialize(deserializer)?;
        Ok(OhlcEntry {
            time: arr.0,
            open: arr.1,
            high: arr.2,
            low: arr.3,
            close: arr.4,
            vwap: arr.5,
            volume: arr.6,
            count: arr.7,
        })
    }
}

/// Request parameters for order book.
#[derive(Debug, Clone, Serialize)]
pub struct OrderBookRequest {
    /// Asset pair.
    pub pair: String,
    /// Maximum number of asks/bids (default: 100, max: 500).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u16>,
}

impl OrderBookRequest {
    /// Create a new order book request.
    pub fn new(pair: impl Into<String>) -> Self {
        Self {
            pair: pair.into(),
            count: None,
        }
    }

    /// Set the depth count.
    pub fn count(mut self, count: u16) -> Self {
        self.count = Some(count.min(500));
        self
    }
}

/// Order book data.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderBook {
    /// Ask side entries.
    pub asks: Vec<OrderBookEntry>,
    /// Bid side entries.
    pub bids: Vec<OrderBookEntry>,
}

/// Single order book entry.
/// Format: [price, volume, timestamp]
#[derive(Debug, Clone)]
pub struct OrderBookEntry {
    /// Price level.
    pub price: Decimal,
    /// Volume at price level.
    pub volume: Decimal,
    /// Timestamp.
    pub timestamp: i64,
}

impl<'de> Deserialize<'de> for OrderBookEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: (Decimal, Decimal, i64) = Deserialize::deserialize(deserializer)?;
        Ok(OrderBookEntry {
            price: arr.0,
            volume: arr.1,
            timestamp: arr.2,
        })
    }
}

/// Request parameters for recent trades.
#[derive(Debug, Clone, Serialize)]
pub struct RecentTradesRequest {
    /// Asset pair.
    pub pair: String,
    /// Return data since given trade ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Maximum number of trades (default: 1000).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u16>,
}

impl RecentTradesRequest {
    /// Create a new recent trades request.
    pub fn new(pair: impl Into<String>) -> Self {
        Self {
            pair: pair.into(),
            since: None,
            count: None,
        }
    }

    /// Set the since trade ID.
    pub fn since(mut self, since: impl Into<String>) -> Self {
        self.since = Some(since.into());
        self
    }

    /// Set the count limit.
    pub fn count(mut self, count: u16) -> Self {
        self.count = Some(count);
        self
    }
}

/// Recent trades response.
#[derive(Debug, Clone)]
pub struct RecentTradesResponse {
    /// Trades keyed by pair name.
    pub trades: HashMap<String, Vec<TradeEntry>>,
    /// Last trade ID for pagination.
    pub last: String,
}

impl<'de> Deserialize<'de> for RecentTradesResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let mut map: HashMap<String, serde_json::Value> = HashMap::deserialize(deserializer)?;
        let last_value = map
            .remove("last")
            .ok_or_else(|| serde::de::Error::missing_field("last"))?;
        let last = String::deserialize(last_value).map_err(serde::de::Error::custom)?;

        let mut trades = HashMap::with_capacity(map.len());
        for (pair, value) in map {
            let entries: Vec<TradeEntry> =
                serde_json::from_value(value).map_err(serde::de::Error::custom)?;
            trades.insert(pair, entries);
        }

        Ok(Self { trades, last })
    }
}

/// Single trade entry.
/// Format: [price, volume, time, buy/sell, market/limit, misc, trade_id]
#[derive(Debug, Clone)]
pub struct TradeEntry {
    /// Trade price.
    pub price: Decimal,
    /// Trade volume.
    pub volume: Decimal,
    /// Trade timestamp.
    pub time: f64,
    /// Buy or sell.
    pub side: String,
    /// Market or limit.
    pub order_type: String,
    /// Miscellaneous.
    pub misc: String,
    /// Trade ID.
    pub trade_id: i64,
}

impl<'de> Deserialize<'de> for TradeEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: (Decimal, Decimal, f64, String, String, String, i64) =
            Deserialize::deserialize(deserializer)?;
        Ok(TradeEntry {
            price: arr.0,
            volume: arr.1,
            time: arr.2,
            side: arr.3,
            order_type: arr.4,
            misc: arr.5,
            trade_id: arr.6,
        })
    }
}

/// Request parameters for recent spreads.
#[derive(Debug, Clone, Serialize)]
pub struct RecentSpreadsRequest {
    /// Asset pair.
    pub pair: String,
    /// Return data since given timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<i64>,
}

impl RecentSpreadsRequest {
    /// Create a new recent spreads request.
    pub fn new(pair: impl Into<String>) -> Self {
        Self {
            pair: pair.into(),
            since: None,
        }
    }

    /// Set the since timestamp.
    pub fn since(mut self, since: i64) -> Self {
        self.since = Some(since);
        self
    }
}

/// Recent spreads response.
#[derive(Debug, Clone, Deserialize)]
pub struct RecentSpreadsResponse {
    /// Spreads keyed by pair name.
    #[serde(flatten)]
    pub spreads: HashMap<String, Vec<SpreadEntry>>,
    /// Last timestamp for pagination.
    pub last: i64,
}

/// Single spread entry.
/// Format: [time, bid, ask]
#[derive(Debug, Clone)]
pub struct SpreadEntry {
    /// Timestamp.
    pub time: i64,
    /// Best bid price.
    pub bid: Decimal,
    /// Best ask price.
    pub ask: Decimal,
}

impl<'de> Deserialize<'de> for SpreadEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let arr: (i64, Decimal, Decimal) = Deserialize::deserialize(deserializer)?;
        Ok(SpreadEntry {
            time: arr.0,
            bid: arr.1,
            ask: arr.2,
        })
    }
}

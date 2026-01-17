//! Futures-specific domain types.
//!
//! This module contains types specific to Futures trading that differ from
//! or extend the Spot API types.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::common::BuySell;


// Contract Types


/// Type of futures contract.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContractType {
    /// Perpetual contract (no expiry)
    #[default]
    Perpetual,
    /// Fixed maturity contract (has expiry date)
    #[serde(alias = "futures_inverse", alias = "futures_vanilla")]
    FixedMaturity,
    /// Index (not tradeable)
    #[serde(alias = "spot index")]
    Index,
}

/// Futures order type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FuturesOrderType {
    /// Limit order
    #[serde(alias = "lmt")]
    #[default]
    Limit,
    /// Market order
    #[serde(alias = "mkt")]
    Market,
    /// Stop order (stop-loss)
    #[serde(alias = "stp")]
    Stop,
    /// Take profit order
    TakeProfit,
    /// Immediate or cancel
    #[serde(alias = "ioc")]
    ImmediateOrCancel,
    /// Post-only (maker only)
    PostOnly,
}

/// Order status for futures orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FuturesOrderStatus {
    /// Order is in the book
    #[serde(alias = "untouched")]
    #[default]
    Open,
    /// Order is partially filled
    #[serde(alias = "partiallyFilled")]
    PartiallyFilled,
    /// Order is fully filled
    Filled,
    /// Order was cancelled
    Cancelled,
}

/// Fill type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FillType {
    /// Maker (provided liquidity)
    Maker,
    /// Taker (removed liquidity)
    Taker,
    /// Liquidation
    Liquidation,
    /// Assignment (options)
    Assignee,
    /// Assigned from (options)
    Assignor,
}


// Account Types


/// Futures account type.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountType {
    /// Cash account (no leverage)
    #[serde(alias = "cashAccount")]
    #[default]
    Cash,
    /// Margin account (single currency collateral)
    #[serde(alias = "marginAccount")]
    Margin,
    /// Multi-collateral margin account
    #[serde(alias = "multiCollateralMarginAccount")]
    MultiCollateral,
    /// Flex futures (cross-margined)
    FlexFutures,
}


// Position and Order Structs


/// A futures position.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesPosition {
    /// The futures symbol (e.g., "PI_XBTUSD")
    pub symbol: String,
    /// Position side
    pub side: BuySell,
    /// Position size (positive)
    pub size: Decimal,
    /// Average entry price
    #[serde(alias = "price")]
    pub entry_price: Decimal,
    /// Current mark price
    #[serde(default)]
    pub mark_price: Option<Decimal>,
    /// Liquidation price (None for cash accounts)
    #[serde(default)]
    pub liquidation_threshold: Option<Decimal>,
    /// Unrealized PnL
    #[serde(default)]
    pub unrealized_pnl: Option<Decimal>,
    /// Unrealized funding (perpetuals only)
    #[serde(default, alias = "unrealizedFunding")]
    pub unrealized_funding: Option<Decimal>,
    /// Initial margin requirement
    #[serde(default)]
    pub initial_margin: Option<Decimal>,
    /// Maintenance margin requirement
    #[serde(default)]
    pub maintenance_margin: Option<Decimal>,
    /// Effective leverage
    #[serde(default)]
    pub effective_leverage: Option<Decimal>,
    /// Return on equity percentage
    #[serde(default)]
    pub return_on_equity: Option<Decimal>,
    /// PnL currency for multi-collateral
    #[serde(default)]
    pub pnl_currency: Option<String>,
    /// Maximum fixed leverage for isolated positions
    #[serde(default, alias = "maxFixedLeverage")]
    pub max_fixed_leverage: Option<Decimal>,
    /// Fill time (deprecated but still returned)
    #[serde(default, alias = "fillTime")]
    pub fill_time: Option<String>,
}

/// A futures order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesOrder {
    /// Order ID
    #[serde(alias = "order_id")]
    pub order_id: String,
    /// Client order ID (if provided)
    #[serde(default, alias = "cliOrdId")]
    pub cli_ord_id: Option<String>,
    /// Futures symbol
    pub symbol: String,
    /// Order side
    pub side: BuySell,
    /// Order type
    #[serde(alias = "orderType")]
    pub order_type: FuturesOrderType,
    /// Order status
    pub status: FuturesOrderStatus,
    /// Total order size
    #[serde(alias = "quantity", alias = "qty")]
    pub size: Decimal,
    /// Filled size
    #[serde(default, alias = "filledSize")]
    pub filled_size: Decimal,
    /// Unfilled (remaining) size
    #[serde(default, alias = "unfilledSize")]
    pub unfilled_size: Decimal,
    /// Limit price (for limit orders)
    #[serde(default, alias = "limitPrice")]
    pub limit_price: Option<Decimal>,
    /// Stop price (for stop orders)
    #[serde(default, alias = "stopPrice")]
    pub stop_price: Option<Decimal>,
    /// Whether this is a reduce-only order
    #[serde(default, alias = "reduceOnly")]
    pub reduce_only: bool,
    /// Time the order was received
    #[serde(default, alias = "receivedTime")]
    pub received_time: Option<String>,
    /// Last update time
    #[serde(default, alias = "lastUpdateTime")]
    pub last_update_time: Option<String>,
}

/// A fill (trade execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesFill {
    /// Fill ID
    #[serde(alias = "fill_id")]
    pub fill_id: String,
    /// Order ID this fill belongs to
    #[serde(alias = "order_id")]
    pub order_id: String,
    /// Client order ID
    #[serde(default, alias = "cliOrdId")]
    pub cli_ord_id: Option<String>,
    /// Futures symbol
    pub symbol: String,
    /// Fill side
    pub side: BuySell,
    /// Fill size
    pub size: Decimal,
    /// Fill price
    pub price: Decimal,
    /// Fill type (maker/taker/liquidation)
    #[serde(alias = "fillType")]
    pub fill_type: FillType,
    /// Fill timestamp
    #[serde(alias = "fillTime")]
    pub fill_time: String,
}


// Account Information


/// Futures account summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesAccount {
    /// Account type
    #[serde(alias = "type")]
    pub account_type: AccountType,
    /// Base currency for this account
    #[serde(default)]
    pub currency: Option<String>,
    /// Balances by currency
    #[serde(default)]
    pub balances: Option<std::collections::HashMap<String, Decimal>>,
    /// Margin requirements
    #[serde(default)]
    pub margin_requirements: Option<MarginRequirements>,
    /// Trigger estimates (liquidation prices)
    #[serde(default)]
    pub trigger_estimates: Option<TriggerEstimates>,
    /// Auxiliary account info
    #[serde(default)]
    pub auxiliary: Option<AuxiliaryInfo>,
}

/// Margin requirements for an account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRequirements {
    /// Initial margin
    pub im: Decimal,
    /// Maintenance margin
    pub mm: Decimal,
    /// Liquidation threshold
    pub lt: Decimal,
    /// Termination threshold
    pub tt: Decimal,
}

/// Trigger price estimates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerEstimates {
    /// Initial margin trigger
    pub im: Decimal,
    /// Maintenance margin trigger
    pub mm: Decimal,
    /// Liquidation trigger
    pub lt: Decimal,
    /// Termination trigger
    pub tt: Decimal,
}

/// Auxiliary account information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuxiliaryInfo {
    /// Available funds
    pub af: Decimal,
    /// Funding component
    #[serde(default)]
    pub funding: Option<Decimal>,
    /// Profit/Loss
    pub pnl: Decimal,
    /// Portfolio value
    pub pv: Decimal,
    /// USD equivalent
    #[serde(default)]
    pub usd: Option<Decimal>,
}

/// Multi-collateral (Flex) account info.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexAccountInfo {
    /// Balances by currency
    pub currencies: std::collections::HashMap<String, FlexCurrencyBalance>,
    /// Total balance value in USD
    pub balance_value: Decimal,
    /// Total portfolio value
    pub portfolio_value: Decimal,
    /// Total collateral value
    pub collateral_value: Decimal,
    /// Initial margin requirement
    pub initial_margin: Decimal,
    /// Maintenance margin requirement
    pub maintenance_margin: Decimal,
    /// Total PnL
    pub pnl: Decimal,
    /// Unrealized funding
    #[serde(default)]
    pub unrealized_funding: Option<Decimal>,
    /// Available margin
    pub available_margin: Decimal,
    /// Margin equity
    pub margin_equity: Decimal,
}

/// Balance info for a single currency in flex account.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlexCurrencyBalance {
    /// Quantity held
    pub quantity: Decimal,
    /// Value in USD
    pub value: Decimal,
    /// Collateral value (after haircut)
    #[serde(alias = "collateral_value")]
    pub collateral_value: Decimal,
    /// Available for withdrawal
    pub available: Decimal,
    /// Haircut percentage
    #[serde(default)]
    pub haircut: Option<Decimal>,
    /// Conversion spread
    #[serde(default)]
    pub conversion_spread: Option<Decimal>,
}


// Instrument Information


/// A futures instrument (contract).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesInstrument {
    /// Symbol (e.g., "PI_XBTUSD")
    pub symbol: String,
    /// Trading pair (e.g., "XBT:USD")
    #[serde(default)]
    pub pair: Option<String>,
    /// Contract type
    #[serde(default, alias = "type")]
    pub contract_type: Option<ContractType>,
    /// Whether the instrument is tradeable
    #[serde(default)]
    pub tradeable: Option<bool>,
    /// Tick size (minimum price increment)
    #[serde(default, alias = "tickSize")]
    pub tick_size: Option<Decimal>,
    /// Contract size (value per unit)
    #[serde(default, alias = "contractSize")]
    pub contract_size: Option<Decimal>,
    /// Maximum leverage
    #[serde(default)]
    pub leverage: Option<String>,
    /// Margin type
    #[serde(default, alias = "marginLevels")]
    pub margin_levels: Option<Vec<MarginLevel>>,
    /// Maturity time (for fixed maturity contracts)
    #[serde(default, alias = "lastTradingTime")]
    pub maturity_time: Option<String>,
    /// Opening time
    #[serde(default, alias = "openingDate")]
    pub opening_date: Option<String>,
    /// Category tag (perpetual, month, quarter)
    #[serde(default)]
    pub tag: Option<String>,
    /// Post-only mode
    #[serde(default, alias = "postOnly")]
    pub post_only: Option<bool>,
}

/// Margin level tier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarginLevel {
    /// Number of contracts
    pub contracts: Decimal,
    /// Initial margin percentage
    #[serde(alias = "initialMargin")]
    pub initial_margin: Decimal,
    /// Maintenance margin percentage
    #[serde(alias = "maintenanceMargin")]
    pub maintenance_margin: Decimal,
}


// Ticker Data


/// Futures ticker data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesTicker {
    /// Symbol
    pub symbol: String,
    /// Trading pair
    #[serde(default)]
    pub pair: Option<String>,
    /// Last trade price
    pub last: Decimal,
    /// Best bid price
    #[serde(default)]
    pub bid: Option<Decimal>,
    /// Best bid size
    #[serde(default, alias = "bidSize")]
    pub bid_size: Option<Decimal>,
    /// Best ask price
    #[serde(default)]
    pub ask: Option<Decimal>,
    /// Best ask size
    #[serde(default, alias = "askSize")]
    pub ask_size: Option<Decimal>,
    /// 24h volume
    #[serde(default)]
    pub volume: Option<Decimal>,
    /// 24h volume in quote currency
    #[serde(default, alias = "volumeQuote")]
    pub volume_quote: Option<Decimal>,
    /// Open interest
    #[serde(default, alias = "openInterest")]
    pub open_interest: Option<Decimal>,
    /// 24h open price
    #[serde(default)]
    pub open: Option<Decimal>,
    /// 24h high price
    #[serde(default)]
    pub high: Option<Decimal>,
    /// 24h low price
    #[serde(default)]
    pub low: Option<Decimal>,
    /// 24h change percentage
    #[serde(default)]
    pub change: Option<Decimal>,
    /// Mark price
    #[serde(default, alias = "markPrice")]
    pub mark_price: Option<Decimal>,
    /// Index price
    #[serde(default, alias = "index")]
    pub index_price: Option<Decimal>,
    /// Current funding rate (perpetuals)
    #[serde(default, alias = "fundingRate")]
    pub funding_rate: Option<Decimal>,
    /// Predicted next funding rate
    #[serde(default, alias = "fundingRatePrediction")]
    pub funding_rate_prediction: Option<Decimal>,
    /// Time until next funding
    #[serde(default, alias = "nextFundingRateTime")]
    pub next_funding_rate_time: Option<i64>,
    /// Days to maturity
    #[serde(default)]
    pub dtm: Option<i32>,
    /// Maturity time
    #[serde(default, alias = "maturityTime")]
    pub maturity_time: Option<i64>,
    /// Contract tag
    #[serde(default)]
    pub tag: Option<String>,
    /// Market suspended
    #[serde(default)]
    pub suspended: Option<bool>,
    /// Post-only mode
    #[serde(default, alias = "postOnly")]
    pub post_only: Option<bool>,
    /// Timestamp
    #[serde(default)]
    pub time: Option<i64>,
}


// Order Book


/// Futures order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesOrderBook {
    /// Symbol
    pub symbol: String,
    /// Bids (price, size)
    pub bids: Vec<BookLevel>,
    /// Asks (price, size)
    pub asks: Vec<BookLevel>,
    /// Server time
    #[serde(default, alias = "serverTime")]
    pub server_time: Option<String>,
}

/// A single level in the order book.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    /// Price level
    pub price: Decimal,
    /// Size at this price
    #[serde(alias = "qty", alias = "quantity")]
    pub size: Decimal,
}


// Trade History


/// A public trade.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesTrade {
    /// Trade ID
    #[serde(alias = "uid")]
    pub trade_id: String,
    /// Trade price
    pub price: Decimal,
    /// Trade size
    #[serde(alias = "qty", alias = "quantity")]
    pub size: Decimal,
    /// Trade side
    pub side: BuySell,
    /// Trade timestamp
    #[serde(alias = "time")]
    pub timestamp: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_position() {
        let json = r#"{
            "symbol": "PI_XBTUSD",
            "side": "buy",
            "size": "1000",
            "price": "50000.0",
            "unrealizedFunding": "0.001"
        }"#;

        let pos: FuturesPosition = serde_json::from_str(json).unwrap();
        assert_eq!(pos.symbol, "PI_XBTUSD");
        assert_eq!(pos.side, BuySell::Buy);
        assert_eq!(pos.size, Decimal::from(1000));
    }

    #[test]
    fn test_deserialize_order() {
        let json = r#"{
            "order_id": "abc123",
            "symbol": "PI_XBTUSD",
            "side": "sell",
            "orderType": "lmt",
            "status": "open",
            "quantity": "500",
            "filledSize": "0",
            "unfilledSize": "500",
            "limitPrice": "55000.0",
            "reduceOnly": true
        }"#;

        let order: FuturesOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.order_id, "abc123");
        assert!(order.reduce_only);
        assert_eq!(order.limit_price, Some(Decimal::from(55000)));
    }

    #[test]
    fn test_deserialize_fill() {
        let json = r#"{
            "fill_id": "fill123",
            "order_id": "order456",
            "symbol": "PI_ETHUSD",
            "side": "buy",
            "size": "10",
            "price": "3500.5",
            "fillType": "taker",
            "fillTime": "2024-01-15T10:30:00Z"
        }"#;

        let fill: FuturesFill = serde_json::from_str(json).unwrap();
        assert_eq!(fill.fill_type, FillType::Taker);
    }

    #[test]
    fn test_deserialize_ticker() {
        let json = r#"{
            "symbol": "PI_XBTUSD",
            "last": "50000.0",
            "bid": "49999.5",
            "ask": "50000.5",
            "fundingRate": "0.0001",
            "openInterest": "1000000"
        }"#;

        let ticker: FuturesTicker = serde_json::from_str(json).unwrap();
        assert_eq!(ticker.symbol, "PI_XBTUSD");
        assert!(ticker.funding_rate.is_some());
    }

    #[test]
    fn test_contract_type_serde() {
        assert_eq!(
            serde_json::from_str::<ContractType>(r#""perpetual""#).unwrap(),
            ContractType::Perpetual
        );
    }

    #[test]
    fn test_order_type_serde() {
        // Test alias
        assert_eq!(
            serde_json::from_str::<FuturesOrderType>(r#""lmt""#).unwrap(),
            FuturesOrderType::Limit
        );
        // Test snake_case
        assert_eq!(
            serde_json::from_str::<FuturesOrderType>(r#""take_profit""#).unwrap(),
            FuturesOrderType::TakeProfit
        );
    }
}

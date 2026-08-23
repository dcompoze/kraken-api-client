//! Request and response types for Futures REST API.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::futures::types::*;
use crate::types::common::BuySell;

// Response Wrappers

/// Response for tickers endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TickersResponse {
    /// Result status
    pub result: String,
    /// List of tickers
    pub tickers: Vec<FuturesTicker>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Response for order book endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderBookResponse {
    /// Result status
    pub result: String,
    /// Order book data
    #[serde(rename = "orderBook")]
    pub order_book: FuturesOrderBook,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Response for trade history endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeHistoryResponse {
    /// Result status
    pub result: String,
    /// Trade history
    pub history: Vec<FuturesTrade>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Response for instruments endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentsResponse {
    /// Result status
    pub result: String,
    /// List of instruments
    pub instruments: Vec<FuturesInstrument>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Response for accounts endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountsResponse {
    /// Result status
    pub result: String,
    /// Account information by account type
    pub accounts: HashMap<String, FuturesAccount>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Response for open positions endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenPositionsResponse {
    /// Result status
    pub result: String,
    /// List of open positions
    #[serde(rename = "openPositions")]
    pub open_positions: Vec<FuturesPosition>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Response for open orders endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenOrdersResponse {
    /// Result status
    pub result: String,
    /// List of open orders
    #[serde(alias = "openOrders", alias = "orders")]
    pub open_orders: Vec<FuturesOrder>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Request for fills endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct FillsRequest {
    /// Symbol filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Get fills after this time
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "lastFillTime")]
    pub last_fill_time: Option<String>,
}

/// Response for fills endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FillsResponse {
    /// Result status
    pub result: String,
    /// List of fills
    pub fills: Vec<FuturesFill>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

// Trading Request/Response Types

/// Request to send a new order.
#[derive(Debug, Clone, Serialize)]
pub struct SendOrderRequest {
    /// The order type (lmt, mkt, stp, take_profit, ioc)
    #[serde(rename = "orderType")]
    pub order_type: FuturesOrderType,
    /// The symbol (e.g., "PI_XBTUSD")
    pub symbol: String,
    /// Order side (buy or sell)
    pub side: BuySell,
    /// Order size (number of contracts)
    pub size: Decimal,
    /// Limit price (required for limit orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "limitPrice")]
    pub limit_price: Option<Decimal>,
    /// Stop price (required for stop orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopPrice")]
    pub stop_price: Option<Decimal>,
    /// Trigger signal for stop orders (mark or last)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "triggerSignal")]
    pub trigger_signal: Option<String>,
    /// Reduce-only order
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "reduceOnly")]
    pub reduce_only: Option<bool>,
    /// Client order ID
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cliOrdId")]
    pub cli_ord_id: Option<String>,
}

impl SendOrderRequest {
    /// Create a new limit order request.
    pub fn limit(symbol: impl Into<String>, side: BuySell, size: Decimal, price: Decimal) -> Self {
        Self {
            order_type: FuturesOrderType::Limit,
            symbol: symbol.into(),
            side,
            size,
            limit_price: Some(price),
            stop_price: None,
            trigger_signal: None,
            reduce_only: None,
            cli_ord_id: None,
        }
    }

    /// Create a new market order request.
    pub fn market(symbol: impl Into<String>, side: BuySell, size: Decimal) -> Self {
        Self {
            order_type: FuturesOrderType::Market,
            symbol: symbol.into(),
            side,
            size,
            limit_price: None,
            stop_price: None,
            trigger_signal: None,
            reduce_only: None,
            cli_ord_id: None,
        }
    }

    /// Create a new stop order request.
    pub fn stop(
        symbol: impl Into<String>,
        side: BuySell,
        size: Decimal,
        stop_price: Decimal,
    ) -> Self {
        Self {
            order_type: FuturesOrderType::Stop,
            symbol: symbol.into(),
            side,
            size,
            limit_price: None,
            stop_price: Some(stop_price),
            trigger_signal: None,
            reduce_only: None,
            cli_ord_id: None,
        }
    }

    /// Set the reduce-only flag.
    pub fn reduce_only(mut self, reduce_only: bool) -> Self {
        self.reduce_only = Some(reduce_only);
        self
    }

    /// Set a client order ID.
    pub fn cli_ord_id(mut self, id: impl Into<String>) -> Self {
        self.cli_ord_id = Some(id.into());
        self
    }

    /// Set the trigger signal for stop orders.
    pub fn trigger_signal(mut self, signal: impl Into<String>) -> Self {
        self.trigger_signal = Some(signal.into());
        self
    }
}

/// Response for send order endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct SendOrderResponse {
    /// Result status
    pub result: String,
    /// The status of the send (e.g., "placed")
    #[serde(rename = "sendStatus")]
    pub send_status: SendStatus,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Status of an order placement.
#[derive(Debug, Clone, Deserialize)]
pub struct SendStatus {
    /// Order ID
    #[serde(rename = "order_id")]
    pub order_id: String,
    /// Status message
    pub status: String,
    /// Received time
    #[serde(rename = "receivedTime")]
    pub received_time: Option<String>,
    /// Client order ID
    #[serde(rename = "cliOrdId")]
    pub cli_ord_id: Option<String>,
}

/// Request to edit an existing order.
#[derive(Debug, Clone, Serialize)]
pub struct EditOrderRequest {
    /// Order ID to edit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "orderId")]
    pub order_id: Option<String>,
    /// Client order ID to edit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cliOrdId")]
    pub cli_ord_id: Option<String>,
    /// New size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<Decimal>,
    /// New limit price
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "limitPrice")]
    pub limit_price: Option<Decimal>,
    /// New stop price
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopPrice")]
    pub stop_price: Option<Decimal>,
}

impl EditOrderRequest {
    /// Create an edit request by order ID.
    pub fn by_order_id(order_id: impl Into<String>) -> Self {
        Self {
            order_id: Some(order_id.into()),
            cli_ord_id: None,
            size: None,
            limit_price: None,
            stop_price: None,
        }
    }

    /// Create an edit request by client order ID.
    pub fn by_cli_ord_id(cli_ord_id: impl Into<String>) -> Self {
        Self {
            order_id: None,
            cli_ord_id: Some(cli_ord_id.into()),
            size: None,
            limit_price: None,
            stop_price: None,
        }
    }

    /// Set new size.
    pub fn size(mut self, size: Decimal) -> Self {
        self.size = Some(size);
        self
    }

    /// Set new limit price.
    pub fn limit_price(mut self, price: Decimal) -> Self {
        self.limit_price = Some(price);
        self
    }

    /// Set new stop price.
    pub fn stop_price(mut self, price: Decimal) -> Self {
        self.stop_price = Some(price);
        self
    }
}

/// Response for edit order endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct EditOrderResponse {
    /// Result status
    pub result: String,
    /// Edit status
    #[serde(rename = "editStatus")]
    pub edit_status: EditStatus,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Status of an order edit.
#[derive(Debug, Clone, Deserialize)]
pub struct EditStatus {
    /// Order ID
    #[serde(rename = "orderId")]
    pub order_id: String,
    /// Status message
    pub status: String,
    /// Received time
    #[serde(rename = "receivedTime")]
    pub received_time: Option<String>,
}

/// Response for cancel order endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    /// Result status
    pub result: String,
    /// Cancel status
    #[serde(rename = "cancelStatus")]
    pub cancel_status: CancelStatus,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Status of an order cancellation.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelStatus {
    /// Order ID
    #[serde(rename = "order_id")]
    pub order_id: Option<String>,
    /// Client order ID
    #[serde(rename = "cliOrdId")]
    pub cli_ord_id: Option<String>,
    /// Status message
    pub status: String,
    /// Received time
    #[serde(rename = "receivedTime")]
    pub received_time: Option<String>,
}

/// Response for cancel all orders endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelAllOrdersResponse {
    /// Result status
    pub result: String,
    /// List of cancelled orders
    #[serde(rename = "cancelStatus")]
    pub cancel_status: CancelAllStatus,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Status of cancel all operation.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelAllStatus {
    /// Number of orders cancelled
    #[serde(rename = "cancelledOrders")]
    pub cancelled_orders: Option<Vec<CancelledOrder>>,
    /// Status message
    pub status: Option<String>,
    /// Received time
    #[serde(rename = "receivedTime")]
    pub received_time: Option<String>,
}

/// Info about a cancelled order.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelledOrder {
    /// Order ID
    #[serde(rename = "order_id")]
    pub order_id: String,
}

/// Response for cancel all orders after (dead man's switch).
#[derive(Debug, Clone, Deserialize)]
pub struct CancelAllOrdersAfterResponse {
    /// Result status
    pub result: String,
    /// The status of the request
    pub status: String,
    /// Current time
    #[serde(rename = "currentTime")]
    pub current_time: Option<String>,
    /// Trigger time (when orders will be cancelled)
    #[serde(rename = "triggerTime")]
    pub trigger_time: Option<String>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

// Batch Order Types

/// Request for batch order operations.
#[derive(Debug, Clone, Serialize)]
pub struct BatchOrderRequest {
    /// The batch elements
    #[serde(rename = "batchOrder")]
    pub batch_order: Vec<BatchElement>,
}

impl BatchOrderRequest {
    /// Create a new batch request.
    pub fn new() -> Self {
        Self {
            batch_order: Vec::new(),
        }
    }

    /// Add a place order element.
    pub fn place(mut self, order: SendOrderRequest) -> Self {
        self.batch_order
            .push(BatchElement::Place(PlaceBatchElement {
                order_type: order.order_type,
                symbol: order.symbol,
                side: order.side,
                size: order.size,
                limit_price: order.limit_price,
                stop_price: order.stop_price,
                reduce_only: order.reduce_only,
                cli_ord_id: order.cli_ord_id,
            }));
        self
    }

    /// Add a cancel order element.
    pub fn cancel(mut self, order_id: impl Into<String>) -> Self {
        self.batch_order
            .push(BatchElement::Cancel(CancelBatchElement {
                order_id: Some(order_id.into()),
                cli_ord_id: None,
            }));
        self
    }

    /// Add a cancel by client order ID element.
    pub fn cancel_by_cli_ord_id(mut self, cli_ord_id: impl Into<String>) -> Self {
        self.batch_order
            .push(BatchElement::Cancel(CancelBatchElement {
                order_id: None,
                cli_ord_id: Some(cli_ord_id.into()),
            }));
        self
    }
}

impl Default for BatchOrderRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// A single element in a batch order request.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "order", rename_all = "lowercase")]
pub enum BatchElement {
    /// Place a new order
    Place(PlaceBatchElement),
    /// Cancel an existing order
    Cancel(CancelBatchElement),
}

/// Element for placing an order in a batch.
#[derive(Debug, Clone, Serialize)]
pub struct PlaceBatchElement {
    #[serde(rename = "orderType")]
    pub order_type: FuturesOrderType,
    pub symbol: String,
    pub side: BuySell,
    pub size: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "limitPrice")]
    pub limit_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "stopPrice")]
    pub stop_price: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "reduceOnly")]
    pub reduce_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cliOrdId")]
    pub cli_ord_id: Option<String>,
}

/// Element for cancelling an order in a batch.
#[derive(Debug, Clone, Serialize)]
pub struct CancelBatchElement {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "order_id")]
    pub order_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cliOrdId")]
    pub cli_ord_id: Option<String>,
}

/// Response for batch order endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchOrderResponse {
    /// Result status
    pub result: String,
    /// Batch status
    #[serde(rename = "batchStatus")]
    pub batch_status: Vec<BatchElementStatus>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Status of a single element in a batch.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchElementStatus {
    /// Order ID (for place operations)
    #[serde(rename = "order_id")]
    pub order_id: Option<String>,
    /// Status message
    pub status: String,
    /// Error message (if failed)
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
}

// Generic Response Types

/// Generic response containing only the result status and server time.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesResultResponse {
    /// Result status
    pub result: String,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

// Fee Schedule Types

/// Response for fee schedules endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeSchedulesResponse {
    /// Result status
    pub result: String,
    /// List of fee schedules
    #[serde(rename = "feeSchedules")]
    pub fee_schedules: Vec<FeeSchedule>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// A fee schedule.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeSchedule {
    /// Schedule UID
    pub uid: String,
    /// Schedule name
    pub name: String,
    /// Fee tiers
    pub tiers: Vec<FeeTier>,
}

/// A single tier in a fee schedule.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeTier {
    /// Maker fee in percent
    #[serde(rename = "makerFee")]
    pub maker_fee: f64,
    /// Taker fee in percent
    #[serde(rename = "takerFee")]
    pub taker_fee: f64,
    /// USD volume required for this tier
    #[serde(rename = "usdVolume")]
    pub usd_volume: f64,
}

/// Response for fee schedule volumes endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeScheduleVolumesResponse {
    /// Result status
    pub result: String,
    /// Volumes keyed by fee schedule UID
    #[serde(rename = "volumesByFeeSchedule")]
    pub volumes_by_fee_schedule: HashMap<String, f64>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

// Funding Rate Types

/// Response for historical funding rates endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoricalFundingRatesResponse {
    /// Result status
    #[serde(default)]
    pub result: Option<String>,
    /// List of funding rates
    pub rates: Vec<FundingRateEntry>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// A single historical funding rate.
#[derive(Debug, Clone, Deserialize)]
pub struct FundingRateEntry {
    /// Funding rate timestamp
    pub timestamp: String,
    /// Absolute funding rate
    #[serde(rename = "fundingRate")]
    pub funding_rate: f64,
    /// Funding rate relative to the price
    #[serde(rename = "relativeFundingRate")]
    pub relative_funding_rate: Option<f64>,
}

// Chart Types

/// Tick type for OHLC chart data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickType {
    /// Spot price data
    Spot,
    /// Mark price data
    Mark,
    /// Trade price data
    Trade,
}

impl TickType {
    /// Return the path segment used by the charts API.
    pub fn as_str(&self) -> &'static str {
        match self {
            TickType::Spot => "spot",
            TickType::Mark => "mark",
            TickType::Trade => "trade",
        }
    }
}

impl std::fmt::Display for TickType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Response for OHLC chart endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OhlcResponse {
    /// List of candles
    pub candles: Vec<Candle>,
    /// Whether more candles are available in the requested range
    #[serde(default)]
    pub more_candles: Option<bool>,
}

/// A single OHLC candle.
#[derive(Debug, Clone, Deserialize)]
pub struct Candle {
    /// Candle timestamp in milliseconds
    pub time: i64,
    /// Open price
    pub open: Decimal,
    /// High price
    pub high: Decimal,
    /// Low price
    pub low: Decimal,
    /// Close price
    pub close: Decimal,
    /// Volume
    pub volume: Decimal,
}

// Transfer and Withdrawal Types

/// Request to transfer funds between margin accounts.
#[derive(Debug, Clone, Serialize)]
pub struct TransferRequest {
    /// Account to withdraw from
    #[serde(rename = "fromAccount")]
    pub from_account: String,
    /// Account to deposit to
    #[serde(rename = "toAccount")]
    pub to_account: String,
    /// Currency or asset to transfer
    pub unit: String,
    /// Amount to transfer
    pub amount: Decimal,
}

/// Request to transfer funds between the main account and a subaccount.
#[derive(Debug, Clone, Serialize)]
pub struct SubAccountTransferRequest {
    /// Account to withdraw from
    #[serde(rename = "fromAccount")]
    pub from_account: String,
    /// User to transfer from
    #[serde(rename = "fromUser")]
    pub from_user: String,
    /// Account to deposit to
    #[serde(rename = "toAccount")]
    pub to_account: String,
    /// User to transfer to
    #[serde(rename = "toUser")]
    pub to_user: String,
    /// Asset to transfer
    pub unit: String,
    /// Amount to transfer
    pub amount: Decimal,
}

/// Request to withdraw funds from the futures wallet to the spot wallet.
#[derive(Debug, Clone, Serialize)]
pub struct WithdrawalRequest {
    /// Amount to withdraw
    pub amount: Decimal,
    /// Asset or currency to withdraw
    pub currency: String,
    /// Wallet to withdraw from (defaults to cash)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sourceWallet")]
    pub source_wallet: Option<String>,
}

// Account Log Types

/// Request parameters for the account log endpoint.
#[derive(Debug, Clone, Default, Serialize)]
pub struct AccountLogRequest {
    /// Return results before this timestamp or date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Maximum number of results (max 500)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    /// First entry id to start with
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Filter by info string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub info: Option<String>,
    /// First entry to begin with by item
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
    /// Sort order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    /// Last entry id
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

/// Response for the account log endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountLogResponse {
    /// Account UID
    #[serde(default, rename = "accountUid")]
    pub account_uid: Option<String>,
    /// Account log entries
    pub logs: Vec<AccountLogEntry>,
}

/// A single account log entry.
#[derive(Debug, Clone, Deserialize)]
pub struct AccountLogEntry {
    /// Entry id
    pub id: i64,
    /// Entry date
    pub date: String,
    /// Entry description
    pub info: String,
    /// Asset
    #[serde(default)]
    pub asset: Option<String>,
    /// Contract
    #[serde(default)]
    pub contract: Option<String>,
    /// Booking UID
    #[serde(default)]
    pub booking_uid: Option<String>,
    /// Collateral
    #[serde(default)]
    pub collateral: Option<String>,
    /// Execution id
    #[serde(default)]
    pub execution: Option<String>,
    /// Fee
    #[serde(default)]
    pub fee: Option<f64>,
    /// Funding rate
    #[serde(default)]
    pub funding_rate: Option<f64>,
    /// Margin account
    #[serde(default)]
    pub margin_account: Option<String>,
    /// Mark price
    #[serde(default)]
    pub mark_price: Option<f64>,
    /// New average entry price
    #[serde(default)]
    pub new_average_entry_price: Option<f64>,
    /// New balance
    #[serde(default)]
    pub new_balance: Option<f64>,
    /// Old average entry price
    #[serde(default)]
    pub old_average_entry_price: Option<f64>,
    /// Old balance
    #[serde(default)]
    pub old_balance: Option<f64>,
    /// Realized funding
    #[serde(default)]
    pub realized_funding: Option<f64>,
    /// Realized PnL
    #[serde(default)]
    pub realized_pnl: Option<f64>,
    /// Trade price
    #[serde(default)]
    pub trade_price: Option<f64>,
}

// Notification Types

/// Response for the notifications endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationsResponse {
    /// Result status
    pub result: String,
    /// List of notifications
    pub notifications: Vec<FuturesNotification>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// A single notification.
#[derive(Debug, Clone, Deserialize)]
pub struct FuturesNotification {
    /// Notification type
    #[serde(rename = "type")]
    pub notification_type: String,
    /// Priority
    #[serde(default)]
    pub priority: Option<String>,
    /// Notification text
    #[serde(default)]
    pub note: Option<String>,
    /// Time when the notification becomes effective
    #[serde(default, rename = "effectiveTime")]
    pub effective_time: Option<String>,
}

// Unwind Queue Types

/// Response for the unwind queue endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct UnwindQueueResponse {
    /// Result status
    pub result: String,
    /// Unwind queue entries
    pub queue: Vec<UnwindQueueEntry>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Unwind queue position for a single symbol.
#[derive(Debug, Clone, Deserialize)]
pub struct UnwindQueueEntry {
    /// Futures symbol
    pub symbol: String,
    /// Percentile of the open interest in the unwind queue
    pub percentile: f64,
}

// Preference Types

/// Response for the leverage preferences endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct LeveragePreferencesResponse {
    /// Result status
    pub result: String,
    /// Leverage preferences per symbol
    #[serde(rename = "leveragePreferences")]
    pub leverage_preferences: Vec<LeveragePreference>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Leverage preference for a single symbol.
#[derive(Debug, Clone, Deserialize)]
pub struct LeveragePreference {
    /// Futures symbol
    pub symbol: String,
    /// Maximum leverage
    #[serde(default, rename = "maxLeverage")]
    pub max_leverage: Option<f64>,
}

/// Response for the PnL preferences endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct PnlPreferencesResponse {
    /// Result status
    pub result: String,
    /// PnL preferences per symbol
    pub preferences: Vec<PnlPreference>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// PnL currency preference for a single symbol.
#[derive(Debug, Clone, Deserialize)]
pub struct PnlPreference {
    /// Futures symbol
    pub symbol: String,
    /// Currency in which profits and losses are realized
    #[serde(rename = "pnlCurrency")]
    pub pnl_currency: String,
}

// Order Status Types

/// Response for the orders status endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OrdersStatusResponse {
    /// Result status
    pub result: String,
    /// Status of the requested orders
    pub orders: Vec<OrderStatusEntry>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Status of a single order.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderStatusEntry {
    /// Order status (e.g. ENTERED_BOOK)
    #[serde(default)]
    pub status: Option<String>,
    /// Order details
    #[serde(default)]
    pub order: Option<OrderStatusDetails>,
    /// Reason for the last update
    #[serde(default, rename = "updateReason")]
    pub update_reason: Option<String>,
    /// Error message (if the order could not be found)
    #[serde(default)]
    pub error: Option<String>,
}

/// Order details returned by the orders status endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderStatusDetails {
    /// Order ID
    #[serde(default, alias = "orderId", alias = "order_id")]
    pub order_id: Option<String>,
    /// Client order ID
    #[serde(default, rename = "cliOrdId")]
    pub cli_ord_id: Option<String>,
    /// Order type
    #[serde(default, rename = "type")]
    pub order_type: Option<String>,
    /// Futures symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Order side
    #[serde(default)]
    pub side: Option<BuySell>,
    /// Order quantity
    #[serde(default)]
    pub quantity: Option<f64>,
    /// Filled quantity
    #[serde(default)]
    pub filled: Option<f64>,
    /// Limit price
    #[serde(default, rename = "limitPrice")]
    pub limit_price: Option<f64>,
    /// Stop price
    #[serde(default, rename = "stopPrice")]
    pub stop_price: Option<f64>,
    /// Reduce-only flag
    #[serde(default, rename = "reduceOnly")]
    pub reduce_only: Option<bool>,
    /// Order timestamp
    #[serde(default)]
    pub timestamp: Option<String>,
    /// Last update timestamp
    #[serde(default, rename = "lastUpdateTimestamp")]
    pub last_update_timestamp: Option<String>,
}

/// Status information for a single instrument.
#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentStatus {
    /// Futures symbol
    pub tradeable: String,
    /// Whether the market price is dislocated
    #[serde(rename = "experiencingDislocation")]
    pub experiencing_dislocation: bool,
    /// Dislocation direction, `ABOVE_UPPER_BOUND` or `BELOW_LOWER_BOUND`
    #[serde(default, rename = "priceDislocationDirection")]
    pub price_dislocation_direction: Option<String>,
    /// Whether the market is experiencing extreme volatility
    #[serde(rename = "experiencingExtremeVolatility")]
    pub experiencing_extreme_volatility: bool,
    /// Initial margin multiplier applied during extreme volatility
    #[serde(default, rename = "extremeVolatilityInitialMarginMultiplier")]
    pub extreme_volatility_initial_margin_multiplier: Option<f64>,
}

/// Response for the instrument status list endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct InstrumentsStatusResponse {
    /// Result status
    pub result: String,
    /// Status of each instrument
    #[serde(rename = "instrumentStatus")]
    pub instrument_status: Vec<InstrumentStatus>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// A holding account balance of a sub-account.
#[derive(Debug, Clone, Deserialize)]
pub struct SubaccountHolding {
    /// Currency code
    pub currency: String,
    /// Balance amount
    pub amount: f64,
}

/// A futures margin account of a sub-account.
#[derive(Debug, Clone, Deserialize)]
pub struct SubaccountFuturesAccount {
    /// Account name
    pub name: String,
    /// Available margin
    #[serde(rename = "availableMargin")]
    pub available_margin: f64,
}

/// A sub-account of the master account.
#[derive(Debug, Clone, Deserialize)]
pub struct Subaccount {
    /// Sub-account UID
    #[serde(rename = "accountUid")]
    pub account_uid: String,
    /// Sub-account email
    #[serde(default)]
    pub email: Option<String>,
    /// Sub-account full name
    #[serde(default, rename = "fullName")]
    pub full_name: Option<String>,
    /// Holding account balances
    #[serde(default, rename = "holdingAccounts")]
    pub holding_accounts: Vec<SubaccountHolding>,
    /// Futures margin accounts
    #[serde(default, rename = "futuresAccounts")]
    pub futures_accounts: Vec<SubaccountFuturesAccount>,
    /// Multi-collateral flex account details
    #[serde(default, rename = "flexAccount")]
    pub flex_account: Option<serde_json::Value>,
}

/// Response for the sub-accounts endpoint.
#[derive(Debug, Clone, Deserialize)]
pub struct SubaccountsResponse {
    /// Result status
    pub result: String,
    /// Master account UID
    #[serde(rename = "masterAccountUid")]
    pub master_account_uid: String,
    /// Sub-accounts owned by the master account
    #[serde(default)]
    pub subaccounts: Vec<Subaccount>,
    /// Server time
    #[serde(rename = "serverTime")]
    pub server_time: Option<String>,
}

/// Request parameters for the historical event endpoints.
#[derive(Debug, Clone, Default, Serialize)]
pub struct HistoryEventsRequest {
    /// Return events before this timestamp in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<u64>,
    /// Token from a previous response to continue pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation_token: Option<String>,
    /// Return events after this timestamp in milliseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    /// Sort order, `asc` or `desc`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    /// Filter by futures symbol
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradeable: Option<String>,
}

/// A single historical event.
///
/// The `event` payload varies by endpoint and event kind (order placed,
/// execution, trigger, and so on), so it is kept as raw JSON.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEvent {
    /// Event UID
    #[serde(default)]
    pub uid: Option<String>,
    /// Event timestamp in milliseconds
    pub timestamp: i64,
    /// Event payload
    pub event: serde_json::Value,
}

/// Response for the historical event endpoints.
#[derive(Debug, Clone, Deserialize)]
pub struct HistoryEventsResponse {
    /// Account UID
    #[serde(default, rename = "accountUid")]
    pub account_uid: Option<String>,
    /// Events in this page
    #[serde(default)]
    pub elements: Vec<HistoryEvent>,
    /// Number of events in this page
    #[serde(default)]
    pub len: Option<u64>,
    /// Token to request the next page
    #[serde(default, rename = "continuationToken")]
    pub continuation_token: Option<String>,
    /// Server time
    #[serde(default, rename = "serverTime")]
    pub server_time: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_order_request_limit() {
        let request = SendOrderRequest::limit(
            "PI_XBTUSD",
            BuySell::Buy,
            Decimal::from(100),
            Decimal::from(50000),
        )
        .reduce_only(true)
        .cli_ord_id("my-order-1");

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("limitPrice"));
        assert!(json.contains("reduceOnly"));
        assert!(json.contains("cliOrdId"));
    }

    #[test]
    fn test_send_order_request_market() {
        let request = SendOrderRequest::market("PI_ETHUSD", BuySell::Sell, Decimal::from(50));

        let json = serde_json::to_string(&request).unwrap();
        assert!(!json.contains("limitPrice"));
        assert!(json.contains("PI_ETHUSD"));
    }

    #[test]
    fn test_edit_order_request() {
        let request = EditOrderRequest::by_order_id("abc123")
            .size(Decimal::from(200))
            .limit_price(Decimal::from(51000));

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("orderId"));
        assert!(json.contains("size"));
        assert!(json.contains("limitPrice"));
    }

    #[test]
    fn test_batch_order_request() {
        let batch = BatchOrderRequest::new()
            .place(SendOrderRequest::limit(
                "PI_XBTUSD",
                BuySell::Buy,
                Decimal::from(100),
                Decimal::from(50000),
            ))
            .cancel("order-to-cancel");

        assert_eq!(batch.batch_order.len(), 2);
    }

    #[test]
    fn test_deserialize_instruments_status_response() {
        let json = r#"{
            "result": "success",
            "instrumentStatus": [{
                "tradeable": "PF_BTCUSD",
                "experiencingDislocation": false,
                "priceDislocationDirection": null,
                "experiencingExtremeVolatility": false,
                "extremeVolatilityInitialMarginMultiplier": 1
            }],
            "serverTime": "2024-01-15T10:00:00Z"
        }"#;

        let response: InstrumentsStatusResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.instrument_status.len(), 1);
        assert_eq!(response.instrument_status[0].tradeable, "PF_BTCUSD");
        assert!(!response.instrument_status[0].experiencing_dislocation);
    }

    #[test]
    fn test_deserialize_subaccounts_response() {
        let json = r#"{
            "result": "success",
            "serverTime": "2024-01-15T10:00:00Z",
            "masterAccountUid": "f7d5571c-6d10-4cf1-944a-048d25682ed0",
            "subaccounts": [{
                "accountUid": "aa2f70eb-d3e6-4d0b-9a4b-1a3d5e2f7a10",
                "email": "sub@example.com",
                "fullName": null,
                "holdingAccounts": [{"currency": "usd", "amount": 12.5}],
                "futuresAccounts": [{"name": "f-xbt:usd", "availableMargin": 1.25}]
            }]
        }"#;

        let response: SubaccountsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.subaccounts.len(), 1);
        assert_eq!(response.subaccounts[0].holding_accounts[0].currency, "usd");
        assert_eq!(response.subaccounts[0].futures_accounts[0].name, "f-xbt:usd");
    }

    #[test]
    fn test_deserialize_history_events_response() {
        let json = r#"{
            "accountUid": "f7d5571c-6d10-4cf1-944a-048d25682ed0",
            "continuationToken": "alp81a",
            "elements": [{
                "uid": "b0a4b8e1-4e0b-4a52-9c37-6b4d1f7a9d2e",
                "timestamp": 1605126171852,
                "event": {"OrderPlaced": {"reason": "new_user_order"}}
            }],
            "len": 1,
            "serverTime": "2024-01-15T10:00:00Z"
        }"#;

        let response: HistoryEventsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.elements.len(), 1);
        assert_eq!(response.elements[0].timestamp, 1605126171852);
        assert_eq!(response.continuation_token.as_deref(), Some("alp81a"));
        assert!(response.elements[0].event.get("OrderPlaced").is_some());
    }

    #[test]
    fn test_serialize_history_events_request() {
        let request = HistoryEventsRequest {
            since: Some(1668989233),
            sort: Some("asc".to_string()),
            tradeable: Some("PF_SOLUSD".to_string()),
            ..Default::default()
        };

        let query = serde_urlencoded::to_string(&request).unwrap();
        assert_eq!(query, "since=1668989233&sort=asc&tradeable=PF_SOLUSD");
    }

    #[test]
    fn test_deserialize_send_order_response() {
        let json = r#"{
            "result": "success",
            "sendStatus": {
                "order_id": "abc123",
                "status": "placed",
                "receivedTime": "2024-01-15T10:00:00Z"
            },
            "serverTime": "2024-01-15T10:00:00Z"
        }"#;

        let response: SendOrderResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.result, "success");
        assert_eq!(response.send_status.order_id, "abc123");
    }
}

//! Types for private REST API endpoints.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::types::serde_helpers::{empty_string_as_none, maybe_decimal};
use crate::types::{BuySell, LedgerType, OrderStatus, OrderType};

/// Extended balance information.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtendedBalances {
    /// Balances keyed by asset.
    #[serde(flatten)]
    pub balances: HashMap<String, ExtendedBalance>,
}

/// Extended balance for a single asset.
#[derive(Debug, Clone, Deserialize)]
pub struct ExtendedBalance {
    /// Available balance.
    pub balance: Decimal,
    /// Amount on hold.
    #[serde(default)]
    pub hold_trade: Option<Decimal>,
}

/// Request for trade balance.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TradeBalanceRequest {
    /// Base asset for calculations (default: ZUSD).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
}

/// Trade balance response.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeBalance {
    /// Equivalent balance (combined balance of all currencies).
    #[serde(rename = "eb")]
    pub equivalent_balance: Decimal,
    /// Trade balance (combined balance of all equity currencies).
    #[serde(rename = "tb")]
    pub trade_balance: Decimal,
    /// Margin amount of open positions.
    #[serde(rename = "m", default)]
    pub margin: Decimal,
    /// Unrealized net profit/loss of open positions.
    #[serde(rename = "n", default)]
    pub unrealized_pnl: Decimal,
    /// Cost basis of open positions.
    #[serde(rename = "c", default)]
    pub cost_basis: Decimal,
    /// Current floating valuation of open positions.
    #[serde(rename = "v", default)]
    pub floating_valuation: Decimal,
    /// Equity = trade balance + unrealized net profit/loss.
    #[serde(rename = "e")]
    pub equity: Decimal,
    /// Free margin = equity - initial margin.
    #[serde(rename = "mf")]
    pub free_margin: Decimal,
    /// Margin level = (equity / initial margin) * 100.
    #[serde(rename = "ml", default)]
    pub margin_level: Option<Decimal>,
    /// Unexecuted value.
    #[serde(rename = "uv", default)]
    pub unexecuted_value: Option<Decimal>,
}

/// Request for open orders.
#[derive(Debug, Clone, Default, Serialize)]
pub struct OpenOrdersRequest {
    /// Include trades in output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<bool>,
    /// Restrict to given user reference ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userref: Option<i64>,
}

/// Open orders response.
#[derive(Debug, Clone, Deserialize)]
pub struct OpenOrders {
    /// Open orders keyed by order ID.
    pub open: HashMap<String, Order>,
}

/// Request for closed orders.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ClosedOrdersRequest {
    /// Include trades in output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<bool>,
    /// Restrict to given user reference ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userref: Option<i64>,
    /// Start timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    /// End timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    /// Result offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ofs: Option<u32>,
    /// Which time to use (open, close, both).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closetime: Option<String>,
}

/// Closed orders response.
#[derive(Debug, Clone, Deserialize)]
pub struct ClosedOrders {
    /// Closed orders keyed by order ID.
    pub closed: HashMap<String, Order>,
    /// Total count of orders.
    pub count: u32,
}

/// Request to query specific orders.
#[derive(Debug, Clone, Serialize)]
pub struct QueryOrdersRequest {
    /// Comma-separated list of order IDs.
    pub txid: String,
    /// Include trades in output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<bool>,
    /// Restrict to given user reference ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userref: Option<i64>,
}

impl QueryOrdersRequest {
    /// Create a new query for order IDs.
    pub fn new(txids: impl Into<String>) -> Self {
        Self {
            txid: txids.into(),
            trades: None,
            userref: None,
        }
    }
}

/// Order details.
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    /// Referral order transaction ID.
    #[serde(default)]
    pub refid: Option<String>,
    /// User reference ID.
    #[serde(default)]
    pub userref: Option<i64>,
    /// Status of order.
    pub status: OrderStatus,
    /// Open timestamp.
    pub opentm: f64,
    /// Start timestamp.
    #[serde(default)]
    pub starttm: Option<f64>,
    /// Expiration timestamp.
    #[serde(default)]
    pub expiretm: Option<f64>,
    /// Close timestamp.
    #[serde(default)]
    pub closetm: Option<f64>,
    /// Order description.
    pub descr: OrderDescription,
    /// Volume of order.
    pub vol: Decimal,
    /// Volume executed.
    pub vol_exec: Decimal,
    /// Total cost.
    pub cost: Decimal,
    /// Total fee.
    pub fee: Decimal,
    /// Average price.
    pub price: Decimal,
    /// Stop price.
    #[serde(default)]
    pub stopprice: Option<Decimal>,
    /// Limit price.
    #[serde(default)]
    pub limitprice: Option<Decimal>,
    /// Triggered limit price.
    #[serde(default)]
    pub trigger: Option<String>,
    /// Miscellaneous info.
    #[serde(default)]
    pub misc: String,
    /// Order flags.
    #[serde(default)]
    pub oflags: String,
    /// List of trade IDs.
    #[serde(default)]
    pub trades: Vec<String>,
    /// Reason for closure.
    #[serde(default)]
    pub reason: Option<String>,
}

/// Order description.
#[derive(Debug, Clone, Deserialize)]
pub struct OrderDescription {
    /// Asset pair.
    pub pair: String,
    /// Type of order (buy/sell).
    #[serde(rename = "type")]
    pub side: BuySell,
    /// Order type.
    pub ordertype: OrderType,
    /// Primary price.
    pub price: Decimal,
    /// Secondary price.
    pub price2: Decimal,
    /// Leverage amount.
    pub leverage: String,
    /// Order description.
    pub order: String,
    /// Conditional close order description.
    #[serde(default)]
    pub close: Option<String>,
}

/// Request for trades history.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TradesHistoryRequest {
    /// Type of trade.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub trade_type: Option<String>,
    /// Include trades in output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<bool>,
    /// Start timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    /// End timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    /// Result offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ofs: Option<u32>,
}

/// Trades history response.
#[derive(Debug, Clone, Deserialize)]
pub struct TradesHistory {
    /// Trades keyed by trade ID.
    pub trades: HashMap<String, Trade>,
    /// Total count of trades.
    pub count: u32,
}

/// Trade details.
#[derive(Debug, Clone, Deserialize)]
pub struct Trade {
    /// Order responsible for trade.
    pub ordertxid: String,
    /// Position ID.
    #[serde(default)]
    pub postxid: Option<String>,
    /// Asset pair.
    pub pair: String,
    /// Timestamp.
    pub time: f64,
    /// Type (buy/sell).
    #[serde(rename = "type")]
    pub side: BuySell,
    /// Order type.
    pub ordertype: OrderType,
    /// Price.
    pub price: Decimal,
    /// Cost.
    pub cost: Decimal,
    /// Fee.
    pub fee: Decimal,
    /// Volume.
    pub vol: Decimal,
    /// Initial margin.
    #[serde(default)]
    pub margin: Option<Decimal>,
    /// Miscellaneous info.
    #[serde(default)]
    pub misc: String,
}

/// Request for open positions.
#[derive(Debug, Clone, Default, Serialize)]
pub struct OpenPositionsRequest {
    /// Comma-separated list of transaction IDs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub txid: Option<String>,
    /// Include profit/loss calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docalcs: Option<bool>,
}

/// Position details.
#[derive(Debug, Clone, Deserialize)]
pub struct Position {
    /// Order ID.
    pub ordertxid: String,
    /// Position status.
    pub posstatus: String,
    /// Asset pair.
    pub pair: String,
    /// Open timestamp.
    pub time: f64,
    /// Type (buy/sell).
    #[serde(rename = "type")]
    pub side: BuySell,
    /// Order type.
    pub ordertype: OrderType,
    /// Opening cost.
    pub cost: Decimal,
    /// Opening fee.
    pub fee: Decimal,
    /// Position volume.
    pub vol: Decimal,
    /// Volume closed.
    pub vol_closed: Decimal,
    /// Initial margin.
    pub margin: Decimal,
    /// Current value.
    #[serde(default)]
    pub value: Option<Decimal>,
    /// Unrealized profit/loss.
    #[serde(default)]
    pub net: Option<Decimal>,
    /// Terms.
    #[serde(default)]
    pub terms: Option<String>,
    /// Roll over cost.
    #[serde(default)]
    pub rollovertm: Option<String>,
    /// Miscellaneous info.
    #[serde(default)]
    pub misc: String,
    /// Order flags.
    #[serde(default)]
    pub oflags: String,
}

/// Request for ledger entries.
#[derive(Debug, Clone, Default, Serialize)]
pub struct LedgersRequest {
    /// Comma-separated list of assets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Asset class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aclass: Option<String>,
    /// Type of ledger entry.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ledger_type: Option<LedgerType>,
    /// Start timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<i64>,
    /// End timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
    /// Result offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ofs: Option<u32>,
    /// Include trades.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub without_count: Option<bool>,
}

/// Ledgers info response.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgersInfo {
    /// Ledger entries keyed by ledger ID.
    pub ledger: HashMap<String, LedgerEntry>,
    /// Total count of entries.
    #[serde(default)]
    pub count: Option<u32>,
}

/// Ledger entry details.
#[derive(Debug, Clone, Deserialize)]
pub struct LedgerEntry {
    /// Reference ID.
    pub refid: String,
    /// Timestamp.
    pub time: f64,
    /// Type of ledger entry.
    #[serde(rename = "type")]
    pub ledger_type: LedgerType,
    /// Sub-type.
    #[serde(default)]
    pub subtype: Option<String>,
    /// Asset class.
    pub aclass: String,
    /// Asset.
    pub asset: String,
    /// Amount.
    pub amount: Decimal,
    /// Fee.
    pub fee: Decimal,
    /// Balance after.
    pub balance: Decimal,
}

/// Request for trade volume.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TradeVolumeRequest {
    /// Comma-separated list of pairs for fee info.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pair: Option<String>,
}

/// Trade volume response.
#[derive(Debug, Clone, Deserialize)]
pub struct TradeVolume {
    /// Currency for volume.
    pub currency: String,
    /// Current 30-day volume.
    pub volume: Decimal,
    /// Fee info by pair.
    #[serde(default)]
    pub fees: Option<HashMap<String, FeeInfo>>,
    /// Maker fee info by pair.
    #[serde(default)]
    pub fees_maker: Option<HashMap<String, FeeInfo>>,
}

/// Fee information.
#[derive(Debug, Clone, Deserialize)]
pub struct FeeInfo {
    /// Current fee.
    pub fee: Decimal,
    /// Minimum fee.
    #[serde(default)]
    pub minfee: Option<Decimal>,
    /// Maximum fee.
    #[serde(default)]
    pub maxfee: Option<Decimal>,
    /// Next fee tier volume.
    #[serde(default)]
    pub nextfee: Option<Decimal>,
    /// Next tier volume.
    #[serde(default)]
    pub nextvolume: Option<Decimal>,
    /// Tier volume.
    #[serde(default)]
    pub tiervolume: Option<Decimal>,
}

/// Request to add an order.
#[derive(Debug, Clone, Serialize)]
pub struct AddOrderRequest {
    /// Asset pair.
    pub pair: String,
    /// Order side (buy/sell).
    #[serde(rename = "type")]
    pub side: BuySell,
    /// Order type.
    pub ordertype: OrderType,
    /// Order volume.
    pub volume: Decimal,
    /// Display volume for iceberg orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub displayvol: Option<Decimal>,
    /// Price (limit price for limit orders, trigger price for stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Decimal>,
    /// Secondary price (limit price for stop-limit orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price2: Option<Decimal>,
    /// Price type for triggered orders.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    /// Leverage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leverage: Option<String>,
    /// Reduce only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Self trade prevention.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stptype: Option<String>,
    /// Order flags (comma-separated).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oflags: Option<String>,
    /// Time in force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeinforce: Option<String>,
    /// Scheduled start time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub starttm: Option<String>,
    /// Expiration time.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiretm: Option<String>,
    /// User reference ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub userref: Option<i64>,
    /// Validate only (don't submit).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<bool>,
    /// Close order type.
    #[serde(rename = "close[ordertype]", skip_serializing_if = "Option::is_none")]
    pub close_ordertype: Option<OrderType>,
    /// Close order price.
    #[serde(rename = "close[price]", skip_serializing_if = "Option::is_none")]
    pub close_price: Option<Decimal>,
    /// Close order secondary price.
    #[serde(rename = "close[price2]", skip_serializing_if = "Option::is_none")]
    pub close_price2: Option<Decimal>,
}

impl AddOrderRequest {
    /// Create a new order request.
    pub fn new(
        pair: impl Into<String>,
        side: BuySell,
        ordertype: OrderType,
        volume: Decimal,
    ) -> Self {
        Self {
            pair: pair.into(),
            side,
            ordertype,
            volume,
            displayvol: None,
            price: None,
            price2: None,
            trigger: None,
            leverage: None,
            reduce_only: None,
            stptype: None,
            oflags: None,
            timeinforce: None,
            starttm: None,
            expiretm: None,
            userref: None,
            validate: None,
            close_ordertype: None,
            close_price: None,
            close_price2: None,
        }
    }

    /// Set the price.
    pub fn price(mut self, price: Decimal) -> Self {
        self.price = Some(price);
        self
    }

    /// Set the secondary price.
    pub fn price2(mut self, price2: Decimal) -> Self {
        self.price2 = Some(price2);
        self
    }

    /// Set leverage.
    pub fn leverage(mut self, leverage: impl Into<String>) -> Self {
        self.leverage = Some(leverage.into());
        self
    }

    /// Set as validate only.
    pub fn validate(mut self, validate: bool) -> Self {
        self.validate = Some(validate);
        self
    }

    /// Set user reference ID.
    pub fn userref(mut self, userref: i64) -> Self {
        self.userref = Some(userref);
        self
    }

    /// Set order flags.
    pub fn oflags(mut self, flags: impl Into<String>) -> Self {
        self.oflags = Some(flags.into());
        self
    }

    /// Set as post-only order.
    pub fn post_only(mut self) -> Self {
        self.oflags = Some("post".to_string());
        self
    }

    /// Set time in force.
    pub fn time_in_force(mut self, tif: impl Into<String>) -> Self {
        self.timeinforce = Some(tif.into());
        self
    }
}

/// Add order response.
#[derive(Debug, Clone, Deserialize)]
pub struct AddOrderResponse {
    /// Order description.
    pub descr: AddOrderDescription,
    /// Transaction IDs (if order was submitted).
    #[serde(default)]
    pub txid: Option<Vec<String>>,
}

/// Add order description.
#[derive(Debug, Clone, Deserialize)]
pub struct AddOrderDescription {
    /// Order description.
    pub order: String,
    /// Close order description.
    #[serde(default)]
    pub close: Option<String>,
}

/// Request to cancel an order.
#[derive(Debug, Clone, Serialize)]
pub struct CancelOrderRequest {
    /// Transaction ID or user reference ID.
    pub txid: String,
}

impl CancelOrderRequest {
    /// Create a new cancel order request.
    pub fn new(txid: impl Into<String>) -> Self {
        Self { txid: txid.into() }
    }
}

/// Cancel order response.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResponse {
    /// Number of orders cancelled.
    pub count: u32,
    /// Orders that are pending cancellation.
    #[serde(default)]
    pub pending: Option<bool>,
}

/// WebSocket token response.
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketToken {
    /// The authentication token.
    pub token: String,
    /// Token expiry time in seconds.
    pub expires: u32,
}

// Funding Endpoints

/// Request for available deposit methods.
#[derive(Debug, Clone, Serialize)]
pub struct DepositMethodsRequest {
    /// Asset (e.g., "XBT", "ETH").
    pub asset: String,
    /// Asset class (default: "currency").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aclass: Option<String>,
}

impl DepositMethodsRequest {
    /// Create a new deposit methods request.
    pub fn new(asset: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            aclass: None,
        }
    }
}

/// Request for deposit addresses.
#[derive(Debug, Clone, Serialize)]
pub struct DepositAddressesRequest {
    /// Asset (e.g., "XBT", "ETH").
    pub asset: String,
    /// Deposit method.
    pub method: String,
    /// Generate a new address.
    #[serde(rename = "new", skip_serializing_if = "Option::is_none")]
    pub new_address: Option<bool>,
    /// Amount for Lightning Network invoices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amount: Option<Decimal>,
}

impl DepositAddressesRequest {
    /// Create a new deposit addresses request.
    pub fn new(asset: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            method: method.into(),
            new_address: None,
            amount: None,
        }
    }

    /// Request a new address.
    pub fn new_address(mut self, new_address: bool) -> Self {
        self.new_address = Some(new_address);
        self
    }

    /// Set the amount (Lightning Network only).
    pub fn amount(mut self, amount: Decimal) -> Self {
        self.amount = Some(amount);
        self
    }
}

/// Cursor for paginated deposit/withdrawal status requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Cursor {
    /// String cursor.
    String(String),
    /// Boolean cursor (true to request cursor-based pagination).
    Bool(bool),
}

/// Request for deposit or withdrawal status.
#[derive(Debug, Clone, Default, Serialize)]
pub struct TransferStatusRequest {
    /// Asset (e.g., "XBT", "ETH").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Asset class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aclass: Option<String>,
    /// Method name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Start time (unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// End time (unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<Cursor>,
    /// Maximum number of records to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

/// Request for deposit status.
pub type DepositStatusRequest = TransferStatusRequest;

/// Request for withdrawal status.
pub type WithdrawStatusRequest = TransferStatusRequest;

/// Request for available withdrawal methods.
#[derive(Debug, Clone, Default, Serialize)]
pub struct WithdrawMethodsRequest {
    /// Asset (e.g., "XBT", "ETH").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Asset class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aclass: Option<String>,
    /// Network name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
}

/// Request for withdrawal addresses.
#[derive(Debug, Clone, Default, Serialize)]
pub struct WithdrawAddressesRequest {
    /// Asset (e.g., "XBT", "ETH").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Asset class.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aclass: Option<String>,
    /// Method name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Address key name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// Only return verified addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
}

/// Request for withdrawal information (limits, fees, etc.).
#[derive(Debug, Clone, Serialize)]
pub struct WithdrawInfoRequest {
    /// Asset to withdraw.
    pub asset: String,
    /// Withdrawal key.
    pub key: String,
    /// Amount to withdraw.
    pub amount: Decimal,
}

impl WithdrawInfoRequest {
    /// Create a new withdrawal info request.
    pub fn new(asset: impl Into<String>, key: impl Into<String>, amount: Decimal) -> Self {
        Self {
            asset: asset.into(),
            key: key.into(),
            amount,
        }
    }
}

/// Request to withdraw funds.
#[derive(Debug, Clone, Serialize)]
pub struct WithdrawRequest {
    /// Asset to withdraw.
    pub asset: String,
    /// Withdrawal key.
    pub key: String,
    /// Amount to withdraw.
    pub amount: Decimal,
    /// Override default withdrawal address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Maximum fee to pay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee: Option<Decimal>,
}

impl WithdrawRequest {
    /// Create a new withdrawal request.
    pub fn new(asset: impl Into<String>, key: impl Into<String>, amount: Decimal) -> Self {
        Self {
            asset: asset.into(),
            key: key.into(),
            amount,
            address: None,
            max_fee: None,
        }
    }

    /// Override the withdrawal address.
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Set the maximum fee to pay.
    pub fn max_fee(mut self, max_fee: Decimal) -> Self {
        self.max_fee = Some(max_fee);
        self
    }
}

/// Request to cancel a withdrawal.
#[derive(Debug, Clone, Serialize)]
pub struct WithdrawCancelRequest {
    /// Asset to cancel withdrawal for.
    pub asset: String,
    /// Withdrawal reference ID.
    #[serde(rename = "refid")]
    pub ref_id: String,
}

impl WithdrawCancelRequest {
    /// Create a new withdrawal cancel request.
    pub fn new(asset: impl Into<String>, ref_id: impl Into<String>) -> Self {
        Self {
            asset: asset.into(),
            ref_id: ref_id.into(),
        }
    }
}

/// Request to transfer between wallets (e.g., Spot to Futures).
#[derive(Debug, Clone, Serialize)]
pub struct WalletTransferRequest {
    /// Asset to transfer.
    pub asset: String,
    /// Source wallet.
    pub from: String,
    /// Destination wallet.
    pub to: String,
    /// Amount to transfer.
    pub amount: Decimal,
}

impl WalletTransferRequest {
    /// Create a new wallet transfer request.
    pub fn new(
        asset: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        amount: Decimal,
    ) -> Self {
        Self {
            asset: asset.into(),
            from: from.into(),
            to: to.into(),
            amount,
        }
    }
}

/// Deposit method details.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DepositMethod {
    /// Method name.
    pub method: String,
    /// Deposit limit (None if no limit).
    #[serde(deserialize_with = "maybe_decimal::deserialize", default)]
    pub limit: Option<Decimal>,
    /// Fee charged for deposits.
    #[serde(default)]
    pub fee: Option<Decimal>,
    /// Address setup fee.
    #[serde(default)]
    pub address_setup_fee: Option<Decimal>,
    /// Whether a new address can be generated.
    #[serde(default)]
    pub gen_address: Option<bool>,
    /// Minimum deposit.
    pub minimum: Decimal,
}

/// Withdrawal method details.
#[derive(Debug, Clone, Deserialize)]
pub struct WithdrawMethod {
    /// Asset name.
    pub asset: String,
    /// Method name.
    pub method: String,
    /// Network name.
    #[serde(default)]
    pub network: Option<String>,
    /// Minimum withdrawal amount.
    pub minimum: Decimal,
}

/// Deposit address information.
#[derive(Debug, Clone, Deserialize)]
pub struct DepositAddress {
    /// Deposit address.
    pub address: String,
    /// Expiry time.
    #[serde(rename = "expiretm")]
    pub expire_time: String,
    /// True if newly generated.
    #[serde(default)]
    pub new: Option<bool>,
    /// Memo for the address (if required).
    #[serde(deserialize_with = "empty_string_as_none::deserialize", default)]
    pub memo: Option<String>,
    /// Tag for the address (if required).
    #[serde(deserialize_with = "empty_string_as_none::deserialize", default)]
    pub tag: Option<String>,
}

/// Withdrawal address information.
#[derive(Debug, Clone, Deserialize)]
pub struct WithdrawalAddress {
    /// Withdrawal address.
    pub address: String,
    /// Asset name.
    pub asset: String,
    /// Method name.
    pub method: String,
    /// Address key.
    pub key: String,
    /// Memo for the address (if required).
    #[serde(deserialize_with = "empty_string_as_none::deserialize", default)]
    pub memo: Option<String>,
    /// Whether the address is verified.
    pub verified: bool,
}

/// Additional status properties about a deposit or withdrawal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StatusProp {
    /// Cancellation is pending.
    CancelPending,
    /// Cancellation was successful.
    Canceled,
    /// Cancellation denied.
    CancelDenied,
    /// Returned to sender.
    Return,
    /// On hold.
    #[serde(rename = "onhold")]
    OnHold,
}

/// Status of a transfer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum TransferStatus {
    /// Initial status.
    #[serde(alias = "initial")]
    Initial,
    /// Pending status.
    #[serde(alias = "pending")]
    Pending,
    /// Settled status.
    #[serde(alias = "settled")]
    Settled,
    /// Success status.
    #[serde(alias = "success")]
    Success,
    /// Failure status.
    #[serde(alias = "failure")]
    Failure,
}

/// Deposit or withdrawal record.
#[derive(Debug, Clone, Deserialize)]
pub struct DepositWithdrawal {
    /// Method name.
    pub method: String,
    /// Asset class.
    #[serde(rename = "aclass")]
    pub asset_class: String,
    /// Asset name.
    pub asset: String,
    /// Reference ID.
    #[serde(rename = "refid")]
    pub ref_id: String,
    /// Transaction ID.
    #[serde(rename = "txid")]
    pub tx_id: String,
    /// Additional info.
    pub info: String,
    /// Amount.
    pub amount: Decimal,
    /// Fee.
    pub fee: Decimal,
    /// Time (unix timestamp).
    pub time: i64,
    /// Status.
    pub status: TransferStatus,
    /// Status properties.
    #[serde(rename = "status-prop", default)]
    pub status_prop: Option<StatusProp>,
    /// Originators (if any).
    #[serde(default, rename = "originators", alias = "orginators")]
    pub originators: Option<Vec<String>>,
}

/// Response for deposit or withdrawal status endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum DepositWithdrawStatusResponse {
    /// Simple list response.
    List(Vec<DepositWithdrawal>),
    /// Deposit response with cursor.
    DepositCursor {
        /// Deposit entries.
        deposit: Vec<DepositWithdrawal>,
        /// Cursor for pagination.
        cursor: Cursor,
    },
    /// Withdrawal response with cursor.
    WithdrawCursor {
        /// Withdrawal entries.
        withdraw: Vec<DepositWithdrawal>,
        /// Cursor for pagination.
        cursor: Cursor,
    },
}

impl DepositWithdrawStatusResponse {
    /// Return the list of entries regardless of response shape.
    pub fn entries(&self) -> &[DepositWithdrawal] {
        match self {
            Self::List(items) => items,
            Self::DepositCursor { deposit, .. } => deposit,
            Self::WithdrawCursor { withdraw, .. } => withdraw,
        }
    }

    /// Return the cursor if present.
    pub fn cursor(&self) -> Option<&Cursor> {
        match self {
            Self::DepositCursor { cursor, .. } | Self::WithdrawCursor { cursor, .. } => {
                Some(cursor)
            }
            Self::List(_) => None,
        }
    }
}

/// Withdrawal info response.
#[derive(Debug, Clone, Deserialize)]
pub struct WithdrawInfo {
    /// Method name.
    pub method: String,
    /// Withdraw limit.
    #[serde(deserialize_with = "maybe_decimal::deserialize", default)]
    pub limit: Option<Decimal>,
    /// Fee.
    pub fee: Decimal,
    /// Amount.
    pub amount: Decimal,
}

/// Confirmation response containing a ref id.
#[derive(Debug, Clone, Deserialize)]
pub struct ConfirmationRefId {
    /// Reference ID.
    #[serde(rename = "refid")]
    pub ref_id: String,
}

// Earn Endpoints

/// Request to allocate funds to an earn strategy.
#[derive(Debug, Clone, Serialize)]
pub struct EarnAllocateRequest {
    /// Amount to allocate.
    pub amount: Decimal,
    /// Strategy ID.
    #[serde(rename = "strategy_id")]
    pub strategy_id: String,
}

impl EarnAllocateRequest {
    /// Create a new earn allocation request.
    pub fn new(amount: Decimal, strategy_id: impl Into<String>) -> Self {
        Self {
            amount,
            strategy_id: strategy_id.into(),
        }
    }
}

/// Request for earn allocation status.
#[derive(Debug, Clone, Serialize)]
pub struct EarnAllocationStatusRequest {
    /// Strategy ID.
    #[serde(rename = "strategy_id")]
    pub strategy_id: String,
}

impl EarnAllocationStatusRequest {
    /// Create a new earn allocation status request.
    pub fn new(strategy_id: impl Into<String>) -> Self {
        Self {
            strategy_id: strategy_id.into(),
        }
    }
}

/// Request to list earn strategies.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EarnStrategiesRequest {
    /// Sort ascending.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    /// Filter by asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset: Option<String>,
    /// Pagination cursor.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Result limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u16>,
    /// Filter by lock type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock_type: Option<LockType>,
}

/// Request to list earn allocations.
#[derive(Debug, Clone, Default, Serialize)]
pub struct EarnAllocationsRequest {
    /// Sort ascending.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascending: Option<bool>,
    /// Convert amounts to this asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub converted_asset: Option<String>,
    /// Hide zero allocations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hide_zero_allocations: Option<bool>,
}

/// Allocation status response.
#[derive(Debug, Clone, Deserialize)]
pub struct AllocationStatus {
    /// Whether the allocation is pending.
    pub pending: bool,
}

/// Wrapper type for earn fees.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(untagged)]
pub enum EarnFee {
    /// Decimal fee.
    Decimal(Decimal),
    /// Integer fee.
    Integer(i64),
    /// Floating fee.
    Float(f64),
}

/// Source of yield for a given earn strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum YieldSourceType {
    /// Staking rewards.
    Staking,
    /// Off-chain rewards.
    OffChain,
    /// Opt-in rewards.
    OptInRewards,
}

/// Type of compounding for a given earn strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AutoCompoundType {
    /// Compounding enabled.
    Enabled,
    /// Compounding disabled.
    Disabled,
    /// Optional compounding.
    Optional,
}

/// Type of asset lock-up for a given earn strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LockType {
    /// Flexible lock-up.
    Flex,
    /// Bonded lock-up.
    Bonded,
    /// Timed lock-up.
    Timed,
    /// Instant lock-up.
    Instant,
}

/// Wrapper type for the origin of rewards from a strategy.
#[derive(Debug, Clone, Deserialize)]
pub struct YieldSource {
    /// Yield source type.
    #[serde(rename = "type")]
    pub yield_type: YieldSourceType,
}

/// Wrapper type for compounding nature of a strategy.
#[derive(Debug, Clone, Deserialize)]
pub struct AutoCompound {
    /// Auto-compound type.
    #[serde(rename = "type")]
    pub auto_compound_type: AutoCompoundType,
    /// Whether this is the default option.
    #[serde(default)]
    pub default: Option<bool>,
}

/// Bracketed estimate for a strategy's APR.
#[derive(Debug, Clone, Deserialize)]
pub struct AprEstimate {
    /// Low APR estimate.
    pub low: Decimal,
    /// High APR estimate.
    pub high: Decimal,
}

/// Details of how funds are locked by an earn strategy.
#[derive(Debug, Clone, Deserialize)]
pub struct LockTypeDetail {
    /// Lock type.
    #[serde(rename = "type")]
    pub lock_type: LockType,
    /// Bonding details (if applicable).
    #[serde(flatten)]
    pub bonding: Option<BondingDetail>,
}

/// Details of an earn strategy's commitments and rewards.
#[derive(Debug, Clone, Deserialize)]
pub struct BondingDetail {
    /// Payout frequency.
    pub payout_frequency: Option<i64>,
    /// Bonding period.
    pub bonding_period: Option<i64>,
    /// Whether bonding period is variable.
    pub bonding_period_variable: Option<bool>,
    /// Whether bonding rewards are paid.
    pub bonding_rewards: Option<bool>,
    /// Exit queue period.
    pub exit_queue_period: Option<i64>,
    /// Unbonding period.
    pub unbonding_period: Option<i64>,
    /// Whether unbonding period is variable.
    pub unbonding_period_variable: Option<bool>,
    /// Whether unbonding rewards are paid.
    pub unbonding_rewards: Option<bool>,
}

/// Paginated response for earn strategies.
#[derive(Debug, Clone, Deserialize)]
pub struct EarnStrategies {
    /// Strategy list.
    pub items: Vec<EarnStrategy>,
    /// Cursor for next page.
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Earn strategy details.
#[derive(Debug, Clone, Deserialize)]
pub struct EarnStrategy {
    /// Allocation fee.
    pub allocation_fee: EarnFee,
    /// Allocation restriction info.
    pub allocation_restriction_info: Vec<String>,
    /// APR estimate.
    #[serde(default)]
    pub apr_estimate: Option<AprEstimate>,
    /// Asset name.
    pub asset: String,
    /// Auto-compound settings.
    pub auto_compound: AutoCompound,
    /// Whether allocation is allowed.
    pub can_allocate: bool,
    /// Whether deallocation is allowed.
    pub can_deallocate: bool,
    /// Deallocation fee.
    pub deallocation_fee: EarnFee,
    /// Strategy ID.
    pub id: String,
    /// Lock type details.
    pub lock_type: LockTypeDetail,
    /// User cap.
    #[serde(default)]
    pub user_cap: Option<Decimal>,
    /// User minimum allocation.
    #[serde(default)]
    pub user_min_allocation: Option<Decimal>,
    /// Yield source.
    pub yield_source: YieldSource,
}

/// Response for earn allocations.
#[derive(Debug, Clone, Deserialize)]
pub struct EarnAllocations {
    /// Converted asset.
    pub converted_asset: String,
    /// Allocation list.
    pub items: Vec<EarnAllocation>,
    /// Total allocated amount.
    pub total_allocated: Decimal,
    /// Total rewarded amount.
    pub total_rewarded: Decimal,
}

/// Earn allocation details.
#[derive(Debug, Clone, Deserialize)]
pub struct EarnAllocation {
    /// Allocation amounts.
    pub amount_allocated: AmountAllocated,
    /// Native asset.
    pub native_asset: String,
    /// Payout details.
    #[serde(default)]
    pub payout: Option<Payout>,
    /// Strategy ID.
    pub strategy_id: String,
    /// Total rewarded amount.
    pub total_rewarded: EarnAmount,
}

/// Amounts allocated to a strategy.
#[derive(Debug, Clone, Deserialize)]
pub struct AmountAllocated {
    /// Bonding allocations.
    #[serde(default)]
    pub bonding: Option<AllocationState>,
    /// Exit queue allocations.
    #[serde(default)]
    pub exit_queue: Option<AllocationState>,
    /// Pending allocations.
    #[serde(default)]
    pub pending: Option<EarnAmount>,
    /// Total allocation.
    pub total: EarnAmount,
    /// Unbonding allocations.
    #[serde(default)]
    pub unbonding: Option<AllocationState>,
}

/// State of a single allocation to a strategy.
#[derive(Debug, Clone, Deserialize)]
pub struct AllocationState {
    /// Allocation count.
    pub allocation_count: i64,
    /// Individual allocations.
    pub allocations: Vec<Allocation>,
    /// Converted amount.
    pub converted: Decimal,
    /// Native amount.
    pub native: Decimal,
}

/// Description of assets allocated to a strategy.
#[derive(Debug, Clone, Deserialize)]
pub struct Allocation {
    /// Creation timestamp.
    pub created_at: String,
    /// Expiration timestamp.
    pub expires: String,
    /// Converted amount.
    pub converted: Decimal,
    /// Native amount.
    pub native: Decimal,
}

/// Payout information for an allocation.
#[derive(Debug, Clone, Deserialize)]
pub struct Payout {
    /// Period end.
    pub period_end: String,
    /// Period start.
    pub period_start: String,
    /// Accumulated reward.
    pub accumulated_reward: EarnAmount,
    /// Estimated reward.
    pub estimated_reward: EarnAmount,
}

/// Amount earned by an allocation in converted and native assets.
#[derive(Debug, Clone, Copy, Deserialize)]
pub struct EarnAmount {
    /// Converted amount.
    pub converted: Decimal,
    /// Native amount.
    pub native: Decimal,
}

//! User data WebSocket messages (executions, balances).

use rust_decimal::Decimal;
use serde::Deserialize;

/// Executions channel message.
#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionsMessage {
    /// Channel name.
    pub channel: String,
    /// Message type ("snapshot" or "update").
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Execution data.
    pub data: Vec<ExecutionData>,
    /// Sequence number.
    #[serde(default)]
    pub sequence: Option<u64>,
}

/// Single execution data.
#[derive(Debug, Clone, Deserialize)]
pub struct ExecutionData {
    /// Execution ID.
    #[serde(default)]
    pub exec_id: Option<String>,
    /// Order ID.
    pub order_id: String,
    /// Client order ID.
    #[serde(default)]
    pub cl_ord_id: Option<String>,
    /// Symbol.
    pub symbol: String,
    /// Side (buy/sell).
    pub side: String,
    /// Order type.
    pub order_type: String,
    /// Order status.
    pub order_status: String,
    /// Limit price.
    #[serde(default)]
    pub limit_price: Option<Decimal>,
    /// Order quantity.
    #[serde(default)]
    pub order_qty: Option<Decimal>,
    /// Filled quantity.
    #[serde(default)]
    pub filled_qty: Option<Decimal>,
    /// Remaining quantity.
    #[serde(default)]
    pub leaves_qty: Option<Decimal>,
    /// Cumulative cost.
    #[serde(default)]
    pub cum_cost: Option<Decimal>,
    /// Cumulative fee.
    #[serde(default)]
    pub cum_fee: Option<Decimal>,
    /// Average price.
    #[serde(default)]
    pub avg_price: Option<Decimal>,
    /// Fee currency.
    #[serde(default)]
    pub fee_ccy: Option<String>,
    /// Fee preference.
    #[serde(default)]
    pub fee_preference: Option<String>,
    /// Time in force.
    #[serde(default)]
    pub time_in_force: Option<String>,
    /// Execution type.
    #[serde(default)]
    pub exec_type: Option<String>,
    /// Last quantity (for fill reports).
    #[serde(default)]
    pub last_qty: Option<Decimal>,
    /// Last price (for fill reports).
    #[serde(default)]
    pub last_price: Option<Decimal>,
    /// Liquidity indicator.
    #[serde(default)]
    pub liquidity_ind: Option<String>,
    /// Trade ID.
    #[serde(default)]
    pub trade_id: Option<i64>,
    /// Post-only flag.
    #[serde(default)]
    pub post_only: Option<bool>,
    /// Reduce-only flag.
    #[serde(default)]
    pub reduce_only: Option<bool>,
    /// Timestamp.
    #[serde(default)]
    pub timestamp: Option<String>,
}

impl ExecutionData {
    /// Check if this is a fill execution.
    pub fn is_fill(&self) -> bool {
        self.exec_type.as_deref() == Some("trade")
    }

    /// Check if the order is fully filled.
    pub fn is_filled(&self) -> bool {
        self.order_status == "filled"
    }

    /// Check if the order is cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.order_status == "canceled"
    }

    /// Check if the order is open.
    pub fn is_open(&self) -> bool {
        self.order_status == "open" || self.order_status == "new"
    }
}

/// Balances channel message.
#[derive(Debug, Clone, Deserialize)]
pub struct BalancesMessage {
    /// Channel name.
    pub channel: String,
    /// Message type ("snapshot" or "update").
    #[serde(rename = "type")]
    pub msg_type: String,
    /// Balance data.
    pub data: Vec<BalanceData>,
    /// Sequence number.
    #[serde(default)]
    pub sequence: Option<u64>,
}

/// Single balance data.
#[derive(Debug, Clone, Deserialize)]
pub struct BalanceData {
    /// Asset.
    pub asset: String,
    /// Available balance (free to trade).
    pub balance: Decimal,
    /// Amount on hold (in open orders).
    #[serde(default)]
    pub hold_trade: Option<Decimal>,
}

impl BalanceData {
    /// Get the total balance (available + on hold).
    pub fn total(&self) -> Decimal {
        self.balance + self.hold_trade.unwrap_or_default()
    }

    /// Get the available balance.
    pub fn available(&self) -> Decimal {
        self.balance
    }

    /// Get the amount on hold.
    pub fn on_hold(&self) -> Decimal {
        self.hold_trade.unwrap_or_default()
    }
}

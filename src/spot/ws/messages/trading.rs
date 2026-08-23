//! Trading WebSocket messages (order operations).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::types::{BuySell, OrderType, TimeInForce};

/// Add order request parameters.
#[derive(Debug, Clone, Serialize)]
pub struct AddOrderParams {
    /// Order type.
    pub order_type: OrderType,
    /// Buy or sell.
    pub side: BuySell,
    /// Trading pair symbol.
    pub symbol: String,
    /// Order quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_qty: Option<Decimal>,
    /// Limit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    /// Time in force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// Trigger price (for stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<Decimal>,
    /// Authentication token.
    pub token: String,
    /// Client order ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
    /// Post-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Reduce-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Display quantity (for iceberg orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_qty: Option<Decimal>,
    /// Fee preference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_preference: Option<String>,
    /// Validate only (don't submit).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<bool>,
}

impl AddOrderParams {
    /// Create a new add order request.
    pub fn new(
        order_type: OrderType,
        side: BuySell,
        symbol: impl Into<String>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            order_type,
            side,
            symbol: symbol.into(),
            order_qty: None,
            limit_price: None,
            time_in_force: None,
            trigger_price: None,
            token: token.into(),
            cl_ord_id: None,
            post_only: None,
            reduce_only: None,
            display_qty: None,
            fee_preference: None,
            validate: None,
        }
    }

    /// Set order quantity.
    pub fn order_qty(mut self, qty: Decimal) -> Self {
        self.order_qty = Some(qty);
        self
    }

    /// Set limit price.
    pub fn limit_price(mut self, price: Decimal) -> Self {
        self.limit_price = Some(price);
        self
    }

    /// Set time in force.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set as post-only.
    pub fn post_only(mut self, post_only: bool) -> Self {
        self.post_only = Some(post_only);
        self
    }

    /// Set client order ID.
    pub fn cl_ord_id(mut self, id: impl Into<String>) -> Self {
        self.cl_ord_id = Some(id.into());
        self
    }

    /// Set validate only.
    pub fn validate(mut self, validate: bool) -> Self {
        self.validate = Some(validate);
        self
    }
}

/// Add order response.
#[derive(Debug, Clone, Deserialize)]
pub struct AddOrderResult {
    /// Order ID.
    pub order_id: String,
    /// Client order ID (if provided).
    #[serde(default)]
    pub cl_ord_id: Option<String>,
    /// Order status.
    #[serde(default)]
    pub order_status: Option<String>,
    /// Symbol.
    #[serde(default)]
    pub symbol: Option<String>,
    /// Execution reports.
    #[serde(default)]
    pub exec_reports: Option<Vec<ExecReport>>,
}

/// Execution report.
#[derive(Debug, Clone, Deserialize)]
pub struct ExecReport {
    /// Execution ID.
    pub exec_id: String,
    /// Order ID.
    pub order_id: String,
    /// Execution type.
    pub exec_type: String,
    /// Order status.
    pub order_status: String,
    /// Symbol.
    pub symbol: String,
    /// Side.
    pub side: String,
    /// Last quantity.
    #[serde(default)]
    pub last_qty: Option<Decimal>,
    /// Last price.
    #[serde(default)]
    pub last_price: Option<Decimal>,
}

/// Cancel order request parameters.
#[derive(Debug, Clone, Serialize)]
pub struct CancelOrderParams {
    /// Order ID(s) to cancel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<Vec<String>>,
    /// Client order ID(s) to cancel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<Vec<String>>,
    /// Authentication token.
    pub token: String,
}

impl CancelOrderParams {
    /// Create a cancel request by order ID.
    pub fn by_order_id(order_ids: Vec<String>, token: impl Into<String>) -> Self {
        Self {
            order_id: Some(order_ids),
            cl_ord_id: None,
            token: token.into(),
        }
    }

    /// Create a cancel request by client order ID.
    pub fn by_cl_ord_id(cl_ord_ids: Vec<String>, token: impl Into<String>) -> Self {
        Self {
            order_id: None,
            cl_ord_id: Some(cl_ord_ids),
            token: token.into(),
        }
    }
}

/// Cancel order response.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelOrderResult {
    /// Order ID.
    #[serde(default)]
    pub order_id: Option<String>,
    /// Client order ID.
    #[serde(default)]
    pub cl_ord_id: Option<String>,
}

/// Cancel all orders request parameters.
#[derive(Debug, Clone, Serialize)]
pub struct CancelAllParams {
    /// Authentication token.
    pub token: String,
}

impl CancelAllParams {
    /// Create a cancel all request.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
        }
    }
}

/// Cancel all orders response.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelAllResult {
    /// Number of orders cancelled.
    pub count: u32,
}

/// Edit order request parameters.
#[derive(Debug, Clone, Serialize)]
pub struct EditOrderParams {
    /// Order ID to edit.
    pub order_id: String,
    /// New quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_qty: Option<Decimal>,
    /// New limit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    /// New display quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_qty: Option<Decimal>,
    /// New trigger price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<Decimal>,
    /// Post-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Authentication token.
    pub token: String,
}

impl EditOrderParams {
    /// Create an edit order request.
    pub fn new(order_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            order_id: order_id.into(),
            order_qty: None,
            limit_price: None,
            display_qty: None,
            trigger_price: None,
            post_only: None,
            token: token.into(),
        }
    }

    /// Set new quantity.
    pub fn order_qty(mut self, qty: Decimal) -> Self {
        self.order_qty = Some(qty);
        self
    }

    /// Set new limit price.
    pub fn limit_price(mut self, price: Decimal) -> Self {
        self.limit_price = Some(price);
        self
    }
}

/// Edit order response.
#[derive(Debug, Clone, Deserialize)]
pub struct EditOrderResult {
    /// Order ID.
    pub order_id: String,
    /// Original order ID (if replaced).
    #[serde(default)]
    pub original_order_id: Option<String>,
}

/// Amend order request parameters.
///
/// Amending keeps the order ID and queue priority, unlike editing.
#[derive(Debug, Clone, Serialize)]
pub struct AmendOrderParams {
    /// Order ID to amend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_id: Option<String>,
    /// Client order ID to amend.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
    /// New order quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_qty: Option<Decimal>,
    /// New display quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_qty: Option<Decimal>,
    /// New limit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    /// New trigger price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<Decimal>,
    /// Post-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Deadline in RFC 3339 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// Authentication token.
    pub token: String,
}

impl AmendOrderParams {
    /// Create an amend request by order ID.
    pub fn by_order_id(order_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            order_id: Some(order_id.into()),
            cl_ord_id: None,
            order_qty: None,
            display_qty: None,
            limit_price: None,
            trigger_price: None,
            post_only: None,
            deadline: None,
            token: token.into(),
        }
    }

    /// Create an amend request by client order ID.
    pub fn by_cl_ord_id(cl_ord_id: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            order_id: None,
            cl_ord_id: Some(cl_ord_id.into()),
            order_qty: None,
            display_qty: None,
            limit_price: None,
            trigger_price: None,
            post_only: None,
            deadline: None,
            token: token.into(),
        }
    }

    /// Set new order quantity.
    pub fn order_qty(mut self, qty: Decimal) -> Self {
        self.order_qty = Some(qty);
        self
    }

    /// Set new limit price.
    pub fn limit_price(mut self, price: Decimal) -> Self {
        self.limit_price = Some(price);
        self
    }

    /// Set new trigger price.
    pub fn trigger_price(mut self, price: Decimal) -> Self {
        self.trigger_price = Some(price);
        self
    }

    /// Set post-only flag.
    pub fn post_only(mut self, post_only: bool) -> Self {
        self.post_only = Some(post_only);
        self
    }
}

/// Amend order response.
#[derive(Debug, Clone, Deserialize)]
pub struct AmendOrderResult {
    /// Amend ID.
    pub amend_id: String,
    /// Order ID.
    #[serde(default)]
    pub order_id: Option<String>,
    /// Client order ID.
    #[serde(default)]
    pub cl_ord_id: Option<String>,
    /// Warnings.
    #[serde(default)]
    pub warnings: Option<Vec<String>>,
}

/// A single order within a batch add request.
#[derive(Debug, Clone, Serialize)]
pub struct BatchAddOrder {
    /// Order type.
    pub order_type: OrderType,
    /// Buy or sell.
    pub side: BuySell,
    /// Order quantity.
    pub order_qty: Decimal,
    /// Limit price.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<Decimal>,
    /// Time in force.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,
    /// Trigger price (for stop orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<Decimal>,
    /// Client order ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cl_ord_id: Option<String>,
    /// User reference ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_userref: Option<i64>,
    /// Post-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    /// Reduce-only flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
    /// Display quantity (for iceberg orders).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_qty: Option<Decimal>,
    /// Fee preference ("base" or "quote").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_preference: Option<String>,
}

impl BatchAddOrder {
    /// Create a new batch order entry.
    pub fn new(order_type: OrderType, side: BuySell, order_qty: Decimal) -> Self {
        Self {
            order_type,
            side,
            order_qty,
            limit_price: None,
            time_in_force: None,
            trigger_price: None,
            cl_ord_id: None,
            order_userref: None,
            post_only: None,
            reduce_only: None,
            display_qty: None,
            fee_preference: None,
        }
    }

    /// Set limit price.
    pub fn limit_price(mut self, price: Decimal) -> Self {
        self.limit_price = Some(price);
        self
    }
}

/// Batch add request parameters.
#[derive(Debug, Clone, Serialize)]
pub struct BatchAddParams {
    /// Orders to place.
    pub orders: Vec<BatchAddOrder>,
    /// Trading pair symbol for all orders.
    pub symbol: String,
    /// Deadline in RFC 3339 format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// Validate only (don't submit).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validate: Option<bool>,
    /// Authentication token.
    pub token: String,
}

impl BatchAddParams {
    /// Create a new batch add request.
    pub fn new(
        symbol: impl Into<String>,
        orders: Vec<BatchAddOrder>,
        token: impl Into<String>,
    ) -> Self {
        Self {
            orders,
            symbol: symbol.into(),
            deadline: None,
            validate: None,
            token: token.into(),
        }
    }

    /// Set validate only.
    pub fn validate(mut self, validate: bool) -> Self {
        self.validate = Some(validate);
        self
    }
}

/// Batch cancel request parameters.
#[derive(Debug, Clone, Serialize)]
pub struct BatchCancelParams {
    /// Order IDs to cancel.
    pub orders: Vec<String>,
    /// Client order IDs to cancel.
    #[serde(rename = "cl_ord_id", skip_serializing_if = "Option::is_none")]
    pub cl_ord_ids: Option<Vec<String>>,
    /// Authentication token.
    pub token: String,
}

impl BatchCancelParams {
    /// Create a batch cancel request by order IDs.
    pub fn by_order_ids(orders: Vec<String>, token: impl Into<String>) -> Self {
        Self {
            orders,
            cl_ord_ids: None,
            token: token.into(),
        }
    }
}

/// Batch cancel response.
#[derive(Debug, Clone, Deserialize)]
pub struct BatchCancelResult {
    /// Number of orders cancelled.
    pub orders_cancelled: i64,
    /// Client order IDs that were cancelled.
    #[serde(default)]
    pub cl_ord_id: Option<Vec<String>>,
}

/// Cancel all orders after request parameters (dead man's switch).
#[derive(Debug, Clone, Serialize)]
pub struct CancelAllOrdersAfterParams {
    /// Timeout in seconds, 0 disables the timer.
    pub timeout: i64,
    /// Authentication token.
    pub token: String,
}

impl CancelAllOrdersAfterParams {
    /// Create a new request with the given timeout in seconds.
    pub fn new(timeout: i64, token: impl Into<String>) -> Self {
        Self {
            timeout,
            token: token.into(),
        }
    }
}

/// Cancel all orders after response.
#[derive(Debug, Clone, Deserialize)]
pub struct CancelAllOrdersAfterResult {
    /// Current server time.
    #[serde(rename = "currentTime")]
    pub current_time: String,
    /// Time at which orders will be cancelled.
    #[serde(rename = "triggerTime", default)]
    pub trigger_time: Option<String>,
    /// Warnings.
    #[serde(default)]
    pub warnings: Option<Vec<String>>,
}

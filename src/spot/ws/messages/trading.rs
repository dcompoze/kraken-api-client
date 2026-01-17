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

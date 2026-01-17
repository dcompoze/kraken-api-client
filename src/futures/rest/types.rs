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
        self.batch_order.push(BatchElement::Place(PlaceBatchElement {
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
        self.batch_order.push(BatchElement::Cancel(CancelBatchElement {
            order_id: Some(order_id.into()),
            cli_ord_id: None,
        }));
        self
    }

    /// Add a cancel by client order ID element.
    pub fn cancel_by_cli_ord_id(mut self, cli_ord_id: impl Into<String>) -> Self {
        self.batch_order.push(BatchElement::Cancel(CancelBatchElement {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_send_order_request_limit() {
        let request = SendOrderRequest::limit("PI_XBTUSD", BuySell::Buy, Decimal::from(100), Decimal::from(50000))
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
            .place(SendOrderRequest::limit("PI_XBTUSD", BuySell::Buy, Decimal::from(100), Decimal::from(50000)))
            .cancel("order-to-cancel");

        assert_eq!(batch.batch_order.len(), 2);
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

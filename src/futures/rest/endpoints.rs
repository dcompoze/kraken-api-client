//! Futures REST API endpoint constants.

/// Base URL for Kraken Futures production API.
pub const FUTURES_BASE_URL: &str = "https://futures.kraken.com/derivatives";

/// Base URL for Kraken Futures demo/testnet API.
pub const FUTURES_DEMO_URL: &str = "https://demo-futures.kraken.com/derivatives";

/// Public endpoints (no authentication required).
pub mod public {
    /// Get all tickers.
    pub const TICKERS: &str = "/api/v3/tickers";

    /// Get order book for a symbol.
    pub const ORDERBOOK: &str = "/api/v3/orderbook";

    /// Get recent trade history.
    pub const HISTORY: &str = "/api/v3/history";

    /// Get available instruments.
    pub const INSTRUMENTS: &str = "/api/v3/instruments";
}

/// Private endpoints (authentication required).
pub mod private {
    /// Get account information.
    pub const ACCOUNTS: &str = "/api/v3/accounts";

    /// Get open positions.
    pub const OPEN_POSITIONS: &str = "/api/v3/openpositions";

    /// Get open orders.
    pub const OPEN_ORDERS: &str = "/api/v3/openorders";

    /// Get fills (trade history).
    pub const FILLS: &str = "/api/v3/fills";

    /// Send a new order.
    pub const SEND_ORDER: &str = "/api/v3/sendorder";

    /// Edit an existing order.
    pub const EDIT_ORDER: &str = "/api/v3/editorder";

    /// Cancel an order.
    pub const CANCEL_ORDER: &str = "/api/v3/cancelorder";

    /// Cancel all orders.
    pub const CANCEL_ALL_ORDERS: &str = "/api/v3/cancelallorders";

    /// Cancel all orders after timeout (dead man's switch).
    pub const CANCEL_ALL_ORDERS_AFTER: &str = "/api/v3/cancelallordersafter";

    /// Batch order operations.
    pub const BATCH_ORDER: &str = "/api/v3/batchorder";
}

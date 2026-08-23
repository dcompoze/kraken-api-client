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

    /// Get fee schedules.
    pub const FEE_SCHEDULES: &str = "/api/v3/feeschedules";

    /// Get historical funding rates.
    pub const HISTORICAL_FUNDING_RATES: &str = "/api/v4/historicalfundingrates";

    /// Base path for OHLC chart data.
    /// This path is served from the domain root, not under `/derivatives`.
    pub const CHARTS: &str = "/api/charts/v1";
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

    /// Get the status of orders by order ID or client order ID.
    pub const ORDERS_STATUS: &str = "/api/v3/orders/status";

    /// Transfer funds between margin accounts.
    pub const TRANSFER: &str = "/api/v3/transfer";

    /// Transfer funds between the main account and a subaccount.
    pub const TRANSFER_SUBACCOUNT: &str = "/api/v3/transfer/subaccount";

    /// Withdraw funds from the futures wallet to the spot wallet.
    pub const WITHDRAWAL: &str = "/api/v3/withdrawal";

    /// Get account log entries.
    /// This path is served from the domain root, not under `/derivatives`.
    pub const ACCOUNT_LOG: &str = "/api/history/v2/account-log";

    /// Get the latest notifications.
    pub const NOTIFICATIONS: &str = "/api/v3/notifications";

    /// Get personal volumes per fee schedule.
    pub const FEE_SCHEDULE_VOLUMES: &str = "/api/v3/feeschedules/volumes";

    /// Get the percentile of open interest in the unwind queue.
    pub const UNWIND_QUEUE: &str = "/api/v3/unwindqueue";

    /// Get or set leverage preferences.
    pub const LEVERAGE_PREFERENCES: &str = "/api/v3/leveragepreferences";

    /// Get or set PnL currency preferences.
    pub const PNL_PREFERENCES: &str = "/api/v3/pnlpreferences";
}

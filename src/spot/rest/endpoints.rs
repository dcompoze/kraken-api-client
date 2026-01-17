//! Kraken REST API endpoint constants.

/// Base URL for the Kraken REST API.
pub const KRAKEN_BASE_URL: &str = "https://api.kraken.com";

/// Public endpoints (no authentication required).
pub mod public {
    /// Get server time.
    pub const TIME: &str = "/0/public/Time";
    /// Get system status.
    pub const SYSTEM_STATUS: &str = "/0/public/SystemStatus";
    /// Get asset info.
    pub const ASSETS: &str = "/0/public/Assets";
    /// Get tradable asset pairs.
    pub const ASSET_PAIRS: &str = "/0/public/AssetPairs";
    /// Get ticker information.
    pub const TICKER: &str = "/0/public/Ticker";
    /// Get OHLC data.
    pub const OHLC: &str = "/0/public/OHLC";
    /// Get order book.
    pub const DEPTH: &str = "/0/public/Depth";
    /// Get recent trades.
    pub const TRADES: &str = "/0/public/Trades";
    /// Get recent spreads.
    pub const SPREAD: &str = "/0/public/Spread";
}

/// Private endpoints (authentication required).
#[allow(dead_code)]
pub mod private {
    // Account endpoints
    /// Get account balance.
    pub const BALANCE: &str = "/0/private/Balance";
    /// Get extended balance.
    pub const BALANCE_EX: &str = "/0/private/BalanceEx";
    /// Get trade balance.
    pub const TRADE_BALANCE: &str = "/0/private/TradeBalance";
    /// Get open orders.
    pub const OPEN_ORDERS: &str = "/0/private/OpenOrders";
    /// Get closed orders.
    pub const CLOSED_ORDERS: &str = "/0/private/ClosedOrders";
    /// Query orders info.
    pub const QUERY_ORDERS: &str = "/0/private/QueryOrders";
    /// Get order amends.
    pub const ORDER_AMENDS: &str = "/0/private/OrderAmends";
    /// Get trades history.
    pub const TRADES_HISTORY: &str = "/0/private/TradesHistory";
    /// Query trades info.
    pub const QUERY_TRADES: &str = "/0/private/QueryTrades";
    /// Get open positions.
    pub const OPEN_POSITIONS: &str = "/0/private/OpenPositions";
    /// Get ledgers.
    pub const LEDGERS: &str = "/0/private/Ledgers";
    /// Query ledgers.
    pub const QUERY_LEDGERS: &str = "/0/private/QueryLedgers";
    /// Get trade volume.
    pub const TRADE_VOLUME: &str = "/0/private/TradeVolume";

    // Export endpoints
    /// Request export report.
    pub const ADD_EXPORT: &str = "/0/private/AddExport";
    /// Get export status.
    pub const EXPORT_STATUS: &str = "/0/private/ExportStatus";
    /// Retrieve export report.
    pub const RETRIEVE_EXPORT: &str = "/0/private/RetrieveExport";
    /// Remove export report.
    pub const REMOVE_EXPORT: &str = "/0/private/RemoveExport";

    // Trading endpoints
    /// Add order.
    pub const ADD_ORDER: &str = "/0/private/AddOrder";
    /// Add order batch.
    pub const ADD_ORDER_BATCH: &str = "/0/private/AddOrderBatch";
    /// Amend order.
    pub const AMEND_ORDER: &str = "/0/private/AmendOrder";
    /// Edit order.
    pub const EDIT_ORDER: &str = "/0/private/EditOrder";
    /// Cancel order.
    pub const CANCEL_ORDER: &str = "/0/private/CancelOrder";
    /// Cancel all orders.
    pub const CANCEL_ALL: &str = "/0/private/CancelAll";
    /// Cancel all orders after timeout.
    pub const CANCEL_ALL_ORDERS_AFTER: &str = "/0/private/CancelAllOrdersAfter";
    /// Cancel order batch.
    pub const CANCEL_ORDER_BATCH: &str = "/0/private/CancelOrderBatch";

    // Funding endpoints
    /// Get deposit methods.
    pub const DEPOSIT_METHODS: &str = "/0/private/DepositMethods";
    /// Get deposit addresses.
    pub const DEPOSIT_ADDRESSES: &str = "/0/private/DepositAddresses";
    /// Get deposit status.
    pub const DEPOSIT_STATUS: &str = "/0/private/DepositStatus";
    /// Get withdrawal methods.
    pub const WITHDRAW_METHODS: &str = "/0/private/WithdrawMethods";
    /// Get withdrawal addresses.
    pub const WITHDRAW_ADDRESSES: &str = "/0/private/WithdrawAddresses";
    /// Get withdrawal info.
    pub const WITHDRAW_INFO: &str = "/0/private/WithdrawInfo";
    /// Withdraw funds.
    pub const WITHDRAW: &str = "/0/private/Withdraw";
    /// Get withdrawal status.
    pub const WITHDRAW_STATUS: &str = "/0/private/WithdrawStatus";
    /// Cancel withdrawal.
    pub const WITHDRAW_CANCEL: &str = "/0/private/WithdrawCancel";
    /// Wallet transfer.
    pub const WALLET_TRANSFER: &str = "/0/private/WalletTransfer";

    // Sub-account endpoints
    /// Create sub-account.
    pub const CREATE_SUBACCOUNT: &str = "/0/private/CreateSubaccount";
    /// Account transfer.
    pub const ACCOUNT_TRANSFER: &str = "/0/private/AccountTransfer";

    // Earn endpoints
    /// Allocate earn funds.
    pub const EARN_ALLOCATE: &str = "/0/private/Earn/Allocate";
    /// Deallocate earn funds.
    pub const EARN_DEALLOCATE: &str = "/0/private/Earn/Deallocate";
    /// Get earn allocation status.
    pub const EARN_ALLOCATE_STATUS: &str = "/0/private/Earn/AllocateStatus";
    /// Get earn deallocation status.
    pub const EARN_DEALLOCATE_STATUS: &str = "/0/private/Earn/DeallocateStatus";
    /// List earn strategies.
    pub const EARN_STRATEGIES: &str = "/0/private/Earn/Strategies";
    /// List earn allocations.
    pub const EARN_ALLOCATIONS: &str = "/0/private/Earn/Allocations";

    // WebSocket token
    /// Get WebSocket authentication token.
    pub const GET_WEBSOCKETS_TOKEN: &str = "/0/private/GetWebSocketsToken";
}

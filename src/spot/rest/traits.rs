//! Trait definition for the Kraken REST API client.

use std::collections::HashMap;
use std::future::Future;

use rust_decimal::Decimal;

use crate::error::KrakenError;
use crate::spot::rest::private::{
    AccountTransferRequest, AccountTransferResponse, AddExportRequest, AddExportResponse,
    AddOrderBatchRequest, AddOrderBatchResponse, AddOrderRequest, AddOrderResponse,
    AllocationStatus, AmendOrderRequest, AmendOrderResponse, CancelAllOrdersAfterRequest,
    CancelAllOrdersAfterResponse, CancelOrderBatchRequest, CancelOrderRequest,
    CancelOrderResponse, ClosedOrders, ClosedOrdersRequest, ConfirmationRefId,
    CreateSubaccountRequest, DepositAddress, DepositAddressesRequest, DepositMethod,
    DepositMethodsRequest, DepositStatusRequest, DepositWithdrawStatusResponse,
    EarnAllocateRequest, EarnAllocationStatusRequest, EarnAllocations, EarnAllocationsRequest,
    EarnStrategies, EarnStrategiesRequest, EditOrderRequest, EditOrderResponse,
    ExportReportStatus, ExportStatusRequest, ExtendedBalances, LedgerEntry, LedgersInfo,
    LedgersRequest, OpenOrders, OpenOrdersRequest, OpenPositionsRequest, Order, OrderAmends,
    OrderAmendsRequest, Position, QueryLedgersRequest, QueryOrdersRequest, QueryTradesRequest,
    RemoveExportRequest, RemoveExportResponse, RetrieveExportRequest, Trade, TradeBalance,
    TradeBalanceRequest, TradeVolume, TradeVolumeRequest, TradesHistory, TradesHistoryRequest,
    WalletTransferRequest, WebSocketToken, WithdrawAddressesRequest, WithdrawCancelRequest,
    WithdrawInfo, WithdrawInfoRequest, WithdrawMethod, WithdrawMethodsRequest, WithdrawRequest,
    WithdrawStatusRequest, WithdrawalAddress,
};
use crate::spot::rest::public::{
    AssetInfo, AssetInfoRequest, AssetPair, AssetPairsRequest, OhlcRequest, OhlcResponse,
    OrderBook, OrderBookRequest, RecentSpreadsRequest, RecentSpreadsResponse, RecentTradesRequest,
    RecentTradesResponse, ServerTime, SystemStatus, TickerInfo,
};

/// Trait defining all Kraken REST API operations.
///
/// Enables mock implementations and decorators such as a rate limiting wrapper.
pub trait KrakenClient: Send + Sync {
    // Public Endpoints.

    /// Get the server time.
    fn get_server_time(&self) -> impl Future<Output = Result<ServerTime, KrakenError>> + Send;

    /// Get the system status.
    fn get_system_status(&self) -> impl Future<Output = Result<SystemStatus, KrakenError>> + Send;

    /// Get asset information.
    fn get_assets(
        &self,
        request: Option<&AssetInfoRequest>,
    ) -> impl Future<Output = Result<HashMap<String, AssetInfo>, KrakenError>> + Send;

    /// Get tradable asset pairs.
    fn get_asset_pairs(
        &self,
        request: Option<&AssetPairsRequest>,
    ) -> impl Future<Output = Result<HashMap<String, AssetPair>, KrakenError>> + Send;

    /// Get ticker information for one or more pairs.
    fn get_ticker(
        &self,
        pairs: &str,
    ) -> impl Future<Output = Result<HashMap<String, TickerInfo>, KrakenError>> + Send;

    /// Get OHLC (candlestick) data.
    fn get_ohlc(
        &self,
        request: &OhlcRequest,
    ) -> impl Future<Output = Result<OhlcResponse, KrakenError>> + Send;

    /// Get order book for a pair.
    fn get_order_book(
        &self,
        request: &OrderBookRequest,
    ) -> impl Future<Output = Result<HashMap<String, OrderBook>, KrakenError>> + Send;

    /// Get recent trades for a pair.
    fn get_recent_trades(
        &self,
        request: &RecentTradesRequest,
    ) -> impl Future<Output = Result<RecentTradesResponse, KrakenError>> + Send;

    /// Get recent spreads for a pair.
    fn get_recent_spreads(
        &self,
        request: &RecentSpreadsRequest,
    ) -> impl Future<Output = Result<RecentSpreadsResponse, KrakenError>> + Send;

    // Private Endpoints - Account.

    /// Get account balance.
    fn get_account_balance(
        &self,
    ) -> impl Future<Output = Result<HashMap<String, Decimal>, KrakenError>> + Send;

    /// Get extended balance with hold amounts.
    fn get_extended_balance(
        &self,
    ) -> impl Future<Output = Result<ExtendedBalances, KrakenError>> + Send;

    /// Get trade balance (margin account details).
    fn get_trade_balance(
        &self,
        request: Option<&TradeBalanceRequest>,
    ) -> impl Future<Output = Result<TradeBalance, KrakenError>> + Send;

    /// Get open orders.
    fn get_open_orders(
        &self,
        request: Option<&OpenOrdersRequest>,
    ) -> impl Future<Output = Result<OpenOrders, KrakenError>> + Send;

    /// Get closed orders.
    fn get_closed_orders(
        &self,
        request: Option<&ClosedOrdersRequest>,
    ) -> impl Future<Output = Result<ClosedOrders, KrakenError>> + Send;

    /// Query specific orders by ID.
    fn query_orders(
        &self,
        request: &QueryOrdersRequest,
    ) -> impl Future<Output = Result<HashMap<String, Order>, KrakenError>> + Send;

    /// Get trades history.
    fn get_trades_history(
        &self,
        request: Option<&TradesHistoryRequest>,
    ) -> impl Future<Output = Result<TradesHistory, KrakenError>> + Send;

    /// Get open positions.
    fn get_open_positions(
        &self,
        request: Option<&OpenPositionsRequest>,
    ) -> impl Future<Output = Result<HashMap<String, Position>, KrakenError>> + Send;

    /// Get ledger entries.
    fn get_ledgers(
        &self,
        request: Option<&LedgersRequest>,
    ) -> impl Future<Output = Result<LedgersInfo, KrakenError>> + Send;

    /// Get trade volume and fee info.
    fn get_trade_volume(
        &self,
        request: Option<&TradeVolumeRequest>,
    ) -> impl Future<Output = Result<TradeVolume, KrakenError>> + Send;

    /// Query specific trades by transaction ID.
    fn query_trades(
        &self,
        request: &QueryTradesRequest,
    ) -> impl Future<Output = Result<HashMap<String, Trade>, KrakenError>> + Send;

    /// Query specific ledger entries by ID.
    fn query_ledgers(
        &self,
        request: &QueryLedgersRequest,
    ) -> impl Future<Output = Result<HashMap<String, LedgerEntry>, KrakenError>> + Send;

    /// Get the amend history of an order.
    fn get_order_amends(
        &self,
        request: &OrderAmendsRequest,
    ) -> impl Future<Output = Result<OrderAmends, KrakenError>> + Send;

    // Private Endpoints - Export.

    /// Request generation of an export report.
    fn add_export(
        &self,
        request: &AddExportRequest,
    ) -> impl Future<Output = Result<AddExportResponse, KrakenError>> + Send;

    /// Get the status of requested export reports.
    fn get_export_status(
        &self,
        request: &ExportStatusRequest,
    ) -> impl Future<Output = Result<Vec<ExportReportStatus>, KrakenError>> + Send;

    /// Retrieve a generated export report as raw bytes.
    fn retrieve_export(
        &self,
        request: &RetrieveExportRequest,
    ) -> impl Future<Output = Result<Vec<u8>, KrakenError>> + Send;

    /// Cancel or delete an export report.
    fn remove_export(
        &self,
        request: &RemoveExportRequest,
    ) -> impl Future<Output = Result<RemoveExportResponse, KrakenError>> + Send;

    // Private Endpoints - Subaccounts.

    /// Create a trading subaccount.
    fn create_subaccount(
        &self,
        request: &CreateSubaccountRequest,
    ) -> impl Future<Output = Result<bool, KrakenError>> + Send;

    /// Transfer funds between master and subaccounts.
    fn account_transfer(
        &self,
        request: &AccountTransferRequest,
    ) -> impl Future<Output = Result<AccountTransferResponse, KrakenError>> + Send;

    // Private Endpoints - Funding.

    /// Get available deposit methods.
    fn get_deposit_methods(
        &self,
        request: &DepositMethodsRequest,
    ) -> impl Future<Output = Result<Vec<DepositMethod>, KrakenError>> + Send;

    /// Get deposit addresses.
    fn get_deposit_addresses(
        &self,
        request: &DepositAddressesRequest,
    ) -> impl Future<Output = Result<Vec<DepositAddress>, KrakenError>> + Send;

    /// Get deposit status.
    fn get_deposit_status(
        &self,
        request: Option<&DepositStatusRequest>,
    ) -> impl Future<Output = Result<DepositWithdrawStatusResponse, KrakenError>> + Send;

    /// Get available withdrawal methods.
    fn get_withdraw_methods(
        &self,
        request: Option<&WithdrawMethodsRequest>,
    ) -> impl Future<Output = Result<Vec<WithdrawMethod>, KrakenError>> + Send;

    /// Get withdrawal addresses.
    fn get_withdraw_addresses(
        &self,
        request: Option<&WithdrawAddressesRequest>,
    ) -> impl Future<Output = Result<Vec<WithdrawalAddress>, KrakenError>> + Send;

    /// Get withdrawal info.
    fn get_withdraw_info(
        &self,
        request: &WithdrawInfoRequest,
    ) -> impl Future<Output = Result<WithdrawInfo, KrakenError>> + Send;

    /// Withdraw funds.
    fn withdraw_funds(
        &self,
        request: &WithdrawRequest,
    ) -> impl Future<Output = Result<ConfirmationRefId, KrakenError>> + Send;

    /// Get withdrawal status.
    fn get_withdraw_status(
        &self,
        request: Option<&WithdrawStatusRequest>,
    ) -> impl Future<Output = Result<DepositWithdrawStatusResponse, KrakenError>> + Send;

    /// Cancel a withdrawal.
    fn withdraw_cancel(
        &self,
        request: &WithdrawCancelRequest,
    ) -> impl Future<Output = Result<bool, KrakenError>> + Send;

    /// Transfer funds between wallets.
    fn wallet_transfer(
        &self,
        request: &WalletTransferRequest,
    ) -> impl Future<Output = Result<ConfirmationRefId, KrakenError>> + Send;

    // Private Endpoints - Earn.

    /// Allocate funds to an earn strategy.
    fn earn_allocate(
        &self,
        request: &EarnAllocateRequest,
    ) -> impl Future<Output = Result<bool, KrakenError>> + Send;

    /// Deallocate funds from an earn strategy.
    fn earn_deallocate(
        &self,
        request: &EarnAllocateRequest,
    ) -> impl Future<Output = Result<bool, KrakenError>> + Send;

    /// Get earn allocation status.
    fn get_earn_allocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> impl Future<Output = Result<AllocationStatus, KrakenError>> + Send;

    /// Get earn deallocation status.
    fn get_earn_deallocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> impl Future<Output = Result<AllocationStatus, KrakenError>> + Send;

    /// List earn strategies.
    fn list_earn_strategies(
        &self,
        request: Option<&EarnStrategiesRequest>,
    ) -> impl Future<Output = Result<EarnStrategies, KrakenError>> + Send;

    /// List earn allocations.
    fn list_earn_allocations(
        &self,
        request: Option<&EarnAllocationsRequest>,
    ) -> impl Future<Output = Result<EarnAllocations, KrakenError>> + Send;

    // Private Endpoints - Trading.

    /// Add a new order.
    fn add_order(
        &self,
        request: &AddOrderRequest,
    ) -> impl Future<Output = Result<AddOrderResponse, KrakenError>> + Send;

    /// Place multiple orders in a single batch.
    fn add_order_batch(
        &self,
        request: &AddOrderBatchRequest,
    ) -> impl Future<Output = Result<AddOrderBatchResponse, KrakenError>> + Send;

    /// Amend an existing order in place.
    fn amend_order(
        &self,
        request: &AmendOrderRequest,
    ) -> impl Future<Output = Result<AmendOrderResponse, KrakenError>> + Send;

    /// Edit an existing order.
    fn edit_order(
        &self,
        request: &EditOrderRequest,
    ) -> impl Future<Output = Result<EditOrderResponse, KrakenError>> + Send;

    /// Cancel an order.
    fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> impl Future<Output = Result<CancelOrderResponse, KrakenError>> + Send;

    /// Cancel all open orders.
    fn cancel_all_orders(
        &self,
    ) -> impl Future<Output = Result<CancelOrderResponse, KrakenError>> + Send;

    /// Cancel all orders after a timeout (dead man's switch).
    fn cancel_all_orders_after(
        &self,
        request: &CancelAllOrdersAfterRequest,
    ) -> impl Future<Output = Result<CancelAllOrdersAfterResponse, KrakenError>> + Send;

    /// Cancel multiple orders in a single batch.
    fn cancel_order_batch(
        &self,
        request: &CancelOrderBatchRequest,
    ) -> impl Future<Output = Result<CancelOrderResponse, KrakenError>> + Send;

    // Private Endpoints - WebSocket.

    /// Get a WebSocket authentication token.
    fn get_websocket_token(
        &self,
    ) -> impl Future<Output = Result<WebSocketToken, KrakenError>> + Send;
}

/// Object-safe version of [`KrakenClient`] for use as `Box<dyn KrakenClientExt>`.
#[allow(async_fn_in_trait)]
pub trait KrakenClientExt: Send + Sync {
    // Public Endpoints.

    async fn get_server_time(&self) -> Result<ServerTime, KrakenError>;
    async fn get_system_status(&self) -> Result<SystemStatus, KrakenError>;
    async fn get_assets(
        &self,
        request: Option<&AssetInfoRequest>,
    ) -> Result<HashMap<String, AssetInfo>, KrakenError>;
    async fn get_asset_pairs(
        &self,
        request: Option<&AssetPairsRequest>,
    ) -> Result<HashMap<String, AssetPair>, KrakenError>;
    async fn get_ticker(&self, pairs: &str) -> Result<HashMap<String, TickerInfo>, KrakenError>;
    async fn get_ohlc(&self, request: &OhlcRequest) -> Result<OhlcResponse, KrakenError>;
    async fn get_order_book(
        &self,
        request: &OrderBookRequest,
    ) -> Result<HashMap<String, OrderBook>, KrakenError>;
    async fn get_recent_trades(
        &self,
        request: &RecentTradesRequest,
    ) -> Result<RecentTradesResponse, KrakenError>;
    async fn get_recent_spreads(
        &self,
        request: &RecentSpreadsRequest,
    ) -> Result<RecentSpreadsResponse, KrakenError>;

    // Private Endpoints - Account.

    async fn get_account_balance(&self) -> Result<HashMap<String, Decimal>, KrakenError>;
    async fn get_extended_balance(&self) -> Result<ExtendedBalances, KrakenError>;
    async fn get_trade_balance(
        &self,
        request: Option<&TradeBalanceRequest>,
    ) -> Result<TradeBalance, KrakenError>;
    async fn get_open_orders(
        &self,
        request: Option<&OpenOrdersRequest>,
    ) -> Result<OpenOrders, KrakenError>;
    async fn get_closed_orders(
        &self,
        request: Option<&ClosedOrdersRequest>,
    ) -> Result<ClosedOrders, KrakenError>;
    async fn query_orders(
        &self,
        request: &QueryOrdersRequest,
    ) -> Result<HashMap<String, Order>, KrakenError>;
    async fn get_trades_history(
        &self,
        request: Option<&TradesHistoryRequest>,
    ) -> Result<TradesHistory, KrakenError>;
    async fn get_open_positions(
        &self,
        request: Option<&OpenPositionsRequest>,
    ) -> Result<HashMap<String, Position>, KrakenError>;
    async fn get_ledgers(
        &self,
        request: Option<&LedgersRequest>,
    ) -> Result<LedgersInfo, KrakenError>;
    async fn get_trade_volume(
        &self,
        request: Option<&TradeVolumeRequest>,
    ) -> Result<TradeVolume, KrakenError>;
    async fn query_trades(
        &self,
        request: &QueryTradesRequest,
    ) -> Result<HashMap<String, Trade>, KrakenError>;
    async fn query_ledgers(
        &self,
        request: &QueryLedgersRequest,
    ) -> Result<HashMap<String, LedgerEntry>, KrakenError>;
    async fn get_order_amends(
        &self,
        request: &OrderAmendsRequest,
    ) -> Result<OrderAmends, KrakenError>;

    // Private Endpoints - Export.

    async fn add_export(&self, request: &AddExportRequest)
    -> Result<AddExportResponse, KrakenError>;
    async fn get_export_status(
        &self,
        request: &ExportStatusRequest,
    ) -> Result<Vec<ExportReportStatus>, KrakenError>;
    async fn retrieve_export(
        &self,
        request: &RetrieveExportRequest,
    ) -> Result<Vec<u8>, KrakenError>;
    async fn remove_export(
        &self,
        request: &RemoveExportRequest,
    ) -> Result<RemoveExportResponse, KrakenError>;

    // Private Endpoints - Subaccounts.

    async fn create_subaccount(
        &self,
        request: &CreateSubaccountRequest,
    ) -> Result<bool, KrakenError>;
    async fn account_transfer(
        &self,
        request: &AccountTransferRequest,
    ) -> Result<AccountTransferResponse, KrakenError>;

    // Private Endpoints - Funding.

    async fn get_deposit_methods(
        &self,
        request: &DepositMethodsRequest,
    ) -> Result<Vec<DepositMethod>, KrakenError>;
    async fn get_deposit_addresses(
        &self,
        request: &DepositAddressesRequest,
    ) -> Result<Vec<DepositAddress>, KrakenError>;
    async fn get_deposit_status(
        &self,
        request: Option<&DepositStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError>;
    async fn get_withdraw_methods(
        &self,
        request: Option<&WithdrawMethodsRequest>,
    ) -> Result<Vec<WithdrawMethod>, KrakenError>;
    async fn get_withdraw_addresses(
        &self,
        request: Option<&WithdrawAddressesRequest>,
    ) -> Result<Vec<WithdrawalAddress>, KrakenError>;
    async fn get_withdraw_info(
        &self,
        request: &WithdrawInfoRequest,
    ) -> Result<WithdrawInfo, KrakenError>;
    async fn withdraw_funds(
        &self,
        request: &WithdrawRequest,
    ) -> Result<ConfirmationRefId, KrakenError>;
    async fn get_withdraw_status(
        &self,
        request: Option<&WithdrawStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError>;
    async fn withdraw_cancel(&self, request: &WithdrawCancelRequest) -> Result<bool, KrakenError>;
    async fn wallet_transfer(
        &self,
        request: &WalletTransferRequest,
    ) -> Result<ConfirmationRefId, KrakenError>;

    // Private Endpoints - Earn.

    async fn earn_allocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError>;
    async fn earn_deallocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError>;
    async fn get_earn_allocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError>;
    async fn get_earn_deallocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError>;
    async fn list_earn_strategies(
        &self,
        request: Option<&EarnStrategiesRequest>,
    ) -> Result<EarnStrategies, KrakenError>;
    async fn list_earn_allocations(
        &self,
        request: Option<&EarnAllocationsRequest>,
    ) -> Result<EarnAllocations, KrakenError>;

    // Private Endpoints - Trading.

    async fn add_order(&self, request: &AddOrderRequest) -> Result<AddOrderResponse, KrakenError>;
    async fn add_order_batch(
        &self,
        request: &AddOrderBatchRequest,
    ) -> Result<AddOrderBatchResponse, KrakenError>;
    async fn amend_order(
        &self,
        request: &AmendOrderRequest,
    ) -> Result<AmendOrderResponse, KrakenError>;
    async fn edit_order(
        &self,
        request: &EditOrderRequest,
    ) -> Result<EditOrderResponse, KrakenError>;
    async fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<CancelOrderResponse, KrakenError>;
    async fn cancel_all_orders(&self) -> Result<CancelOrderResponse, KrakenError>;
    async fn cancel_all_orders_after(
        &self,
        request: &CancelAllOrdersAfterRequest,
    ) -> Result<CancelAllOrdersAfterResponse, KrakenError>;
    async fn cancel_order_batch(
        &self,
        request: &CancelOrderBatchRequest,
    ) -> Result<CancelOrderResponse, KrakenError>;

    // Private Endpoints - WebSocket.

    async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError>;
}

impl<T: KrakenClient> KrakenClientExt for T {
    async fn get_server_time(&self) -> Result<ServerTime, KrakenError> {
        KrakenClient::get_server_time(self).await
    }

    async fn get_system_status(&self) -> Result<SystemStatus, KrakenError> {
        KrakenClient::get_system_status(self).await
    }

    async fn get_assets(
        &self,
        request: Option<&AssetInfoRequest>,
    ) -> Result<HashMap<String, AssetInfo>, KrakenError> {
        KrakenClient::get_assets(self, request).await
    }

    async fn get_asset_pairs(
        &self,
        request: Option<&AssetPairsRequest>,
    ) -> Result<HashMap<String, AssetPair>, KrakenError> {
        KrakenClient::get_asset_pairs(self, request).await
    }

    async fn get_ticker(&self, pairs: &str) -> Result<HashMap<String, TickerInfo>, KrakenError> {
        KrakenClient::get_ticker(self, pairs).await
    }

    async fn get_ohlc(&self, request: &OhlcRequest) -> Result<OhlcResponse, KrakenError> {
        KrakenClient::get_ohlc(self, request).await
    }

    async fn get_order_book(
        &self,
        request: &OrderBookRequest,
    ) -> Result<HashMap<String, OrderBook>, KrakenError> {
        KrakenClient::get_order_book(self, request).await
    }

    async fn get_recent_trades(
        &self,
        request: &RecentTradesRequest,
    ) -> Result<RecentTradesResponse, KrakenError> {
        KrakenClient::get_recent_trades(self, request).await
    }

    async fn get_recent_spreads(
        &self,
        request: &RecentSpreadsRequest,
    ) -> Result<RecentSpreadsResponse, KrakenError> {
        KrakenClient::get_recent_spreads(self, request).await
    }

    async fn get_account_balance(&self) -> Result<HashMap<String, Decimal>, KrakenError> {
        KrakenClient::get_account_balance(self).await
    }

    async fn get_extended_balance(&self) -> Result<ExtendedBalances, KrakenError> {
        KrakenClient::get_extended_balance(self).await
    }

    async fn get_trade_balance(
        &self,
        request: Option<&TradeBalanceRequest>,
    ) -> Result<TradeBalance, KrakenError> {
        KrakenClient::get_trade_balance(self, request).await
    }

    async fn get_open_orders(
        &self,
        request: Option<&OpenOrdersRequest>,
    ) -> Result<OpenOrders, KrakenError> {
        KrakenClient::get_open_orders(self, request).await
    }

    async fn get_closed_orders(
        &self,
        request: Option<&ClosedOrdersRequest>,
    ) -> Result<ClosedOrders, KrakenError> {
        KrakenClient::get_closed_orders(self, request).await
    }

    async fn query_orders(
        &self,
        request: &QueryOrdersRequest,
    ) -> Result<HashMap<String, Order>, KrakenError> {
        KrakenClient::query_orders(self, request).await
    }

    async fn get_trades_history(
        &self,
        request: Option<&TradesHistoryRequest>,
    ) -> Result<TradesHistory, KrakenError> {
        KrakenClient::get_trades_history(self, request).await
    }

    async fn get_open_positions(
        &self,
        request: Option<&OpenPositionsRequest>,
    ) -> Result<HashMap<String, Position>, KrakenError> {
        KrakenClient::get_open_positions(self, request).await
    }

    async fn get_ledgers(
        &self,
        request: Option<&LedgersRequest>,
    ) -> Result<LedgersInfo, KrakenError> {
        KrakenClient::get_ledgers(self, request).await
    }

    async fn get_trade_volume(
        &self,
        request: Option<&TradeVolumeRequest>,
    ) -> Result<TradeVolume, KrakenError> {
        KrakenClient::get_trade_volume(self, request).await
    }

    async fn query_trades(
        &self,
        request: &QueryTradesRequest,
    ) -> Result<HashMap<String, Trade>, KrakenError> {
        KrakenClient::query_trades(self, request).await
    }

    async fn query_ledgers(
        &self,
        request: &QueryLedgersRequest,
    ) -> Result<HashMap<String, LedgerEntry>, KrakenError> {
        KrakenClient::query_ledgers(self, request).await
    }

    async fn get_order_amends(
        &self,
        request: &OrderAmendsRequest,
    ) -> Result<OrderAmends, KrakenError> {
        KrakenClient::get_order_amends(self, request).await
    }

    async fn add_export(
        &self,
        request: &AddExportRequest,
    ) -> Result<AddExportResponse, KrakenError> {
        KrakenClient::add_export(self, request).await
    }

    async fn get_export_status(
        &self,
        request: &ExportStatusRequest,
    ) -> Result<Vec<ExportReportStatus>, KrakenError> {
        KrakenClient::get_export_status(self, request).await
    }

    async fn retrieve_export(
        &self,
        request: &RetrieveExportRequest,
    ) -> Result<Vec<u8>, KrakenError> {
        KrakenClient::retrieve_export(self, request).await
    }

    async fn remove_export(
        &self,
        request: &RemoveExportRequest,
    ) -> Result<RemoveExportResponse, KrakenError> {
        KrakenClient::remove_export(self, request).await
    }

    async fn create_subaccount(
        &self,
        request: &CreateSubaccountRequest,
    ) -> Result<bool, KrakenError> {
        KrakenClient::create_subaccount(self, request).await
    }

    async fn account_transfer(
        &self,
        request: &AccountTransferRequest,
    ) -> Result<AccountTransferResponse, KrakenError> {
        KrakenClient::account_transfer(self, request).await
    }

    async fn get_deposit_methods(
        &self,
        request: &DepositMethodsRequest,
    ) -> Result<Vec<DepositMethod>, KrakenError> {
        KrakenClient::get_deposit_methods(self, request).await
    }

    async fn get_deposit_addresses(
        &self,
        request: &DepositAddressesRequest,
    ) -> Result<Vec<DepositAddress>, KrakenError> {
        KrakenClient::get_deposit_addresses(self, request).await
    }

    async fn get_deposit_status(
        &self,
        request: Option<&DepositStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        KrakenClient::get_deposit_status(self, request).await
    }

    async fn get_withdraw_methods(
        &self,
        request: Option<&WithdrawMethodsRequest>,
    ) -> Result<Vec<WithdrawMethod>, KrakenError> {
        KrakenClient::get_withdraw_methods(self, request).await
    }

    async fn get_withdraw_addresses(
        &self,
        request: Option<&WithdrawAddressesRequest>,
    ) -> Result<Vec<WithdrawalAddress>, KrakenError> {
        KrakenClient::get_withdraw_addresses(self, request).await
    }

    async fn get_withdraw_info(
        &self,
        request: &WithdrawInfoRequest,
    ) -> Result<WithdrawInfo, KrakenError> {
        KrakenClient::get_withdraw_info(self, request).await
    }

    async fn withdraw_funds(
        &self,
        request: &WithdrawRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        KrakenClient::withdraw_funds(self, request).await
    }

    async fn get_withdraw_status(
        &self,
        request: Option<&WithdrawStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        KrakenClient::get_withdraw_status(self, request).await
    }

    async fn withdraw_cancel(&self, request: &WithdrawCancelRequest) -> Result<bool, KrakenError> {
        KrakenClient::withdraw_cancel(self, request).await
    }

    async fn wallet_transfer(
        &self,
        request: &WalletTransferRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        KrakenClient::wallet_transfer(self, request).await
    }

    async fn earn_allocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError> {
        KrakenClient::earn_allocate(self, request).await
    }

    async fn earn_deallocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError> {
        KrakenClient::earn_deallocate(self, request).await
    }

    async fn get_earn_allocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        KrakenClient::get_earn_allocation_status(self, request).await
    }

    async fn get_earn_deallocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        KrakenClient::get_earn_deallocation_status(self, request).await
    }

    async fn list_earn_strategies(
        &self,
        request: Option<&EarnStrategiesRequest>,
    ) -> Result<EarnStrategies, KrakenError> {
        KrakenClient::list_earn_strategies(self, request).await
    }

    async fn list_earn_allocations(
        &self,
        request: Option<&EarnAllocationsRequest>,
    ) -> Result<EarnAllocations, KrakenError> {
        KrakenClient::list_earn_allocations(self, request).await
    }

    async fn add_order(&self, request: &AddOrderRequest) -> Result<AddOrderResponse, KrakenError> {
        KrakenClient::add_order(self, request).await
    }

    async fn add_order_batch(
        &self,
        request: &AddOrderBatchRequest,
    ) -> Result<AddOrderBatchResponse, KrakenError> {
        KrakenClient::add_order_batch(self, request).await
    }

    async fn amend_order(
        &self,
        request: &AmendOrderRequest,
    ) -> Result<AmendOrderResponse, KrakenError> {
        KrakenClient::amend_order(self, request).await
    }

    async fn edit_order(
        &self,
        request: &EditOrderRequest,
    ) -> Result<EditOrderResponse, KrakenError> {
        KrakenClient::edit_order(self, request).await
    }

    async fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        KrakenClient::cancel_order(self, request).await
    }

    async fn cancel_all_orders(&self) -> Result<CancelOrderResponse, KrakenError> {
        KrakenClient::cancel_all_orders(self).await
    }

    async fn cancel_all_orders_after(
        &self,
        request: &CancelAllOrdersAfterRequest,
    ) -> Result<CancelAllOrdersAfterResponse, KrakenError> {
        KrakenClient::cancel_all_orders_after(self, request).await
    }

    async fn cancel_order_batch(
        &self,
        request: &CancelOrderBatchRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        KrakenClient::cancel_order_batch(self, request).await
    }

    async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError> {
        KrakenClient::get_websocket_token(self).await
    }
}

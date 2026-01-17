//! Trait definition for the Kraken REST API client.
//!
//! This module provides the `KrakenClient` trait which abstracts all REST API operations.
//! This enables:
//! - Mock implementations for testing
//! - Decorator pattern (e.g., rate limiting wrapper)
//! - Alternative implementations
//!
//! # Example
//!
//! ```rust,ignore
//! use kraken_api_client::spot::rest::{KrakenClient, SpotRestClient};
//!
//! async fn check_balance<C: KrakenClient>(client: &C) -> Result<(), kraken_api_client::KrakenError> {
//!     let time = client.get_server_time().await?;
//!     println!("Server time: {}", time.unixtime);
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;
use std::future::Future;

use rust_decimal::Decimal;

use crate::error::KrakenError;
use crate::spot::rest::private::{
    AddOrderRequest, AddOrderResponse, AllocationStatus, CancelOrderRequest, CancelOrderResponse,
    ClosedOrders, ClosedOrdersRequest, ConfirmationRefId, DepositAddress, DepositAddressesRequest,
    DepositMethod, DepositMethodsRequest, DepositStatusRequest, DepositWithdrawStatusResponse,
    EarnAllocateRequest, EarnAllocationStatusRequest, EarnAllocations, EarnAllocationsRequest,
    EarnStrategies, EarnStrategiesRequest, ExtendedBalances, LedgersInfo, LedgersRequest,
    OpenOrders, OpenOrdersRequest, OpenPositionsRequest, Order, Position, QueryOrdersRequest,
    TradeBalance, TradeBalanceRequest, TradeVolume, TradeVolumeRequest, TradesHistory,
    TradesHistoryRequest, WalletTransferRequest, WebSocketToken, WithdrawAddressesRequest,
    WithdrawCancelRequest, WithdrawInfo, WithdrawInfoRequest, WithdrawMethod,
    WithdrawMethodsRequest, WithdrawRequest, WithdrawStatusRequest, WithdrawalAddress,
};
use crate::spot::rest::public::{
    AssetInfo, AssetInfoRequest, AssetPair, AssetPairsRequest, OhlcRequest, OhlcResponse,
    OrderBook, OrderBookRequest, RecentSpreadsRequest, RecentSpreadsResponse, RecentTradesRequest,
    RecentTradesResponse, ServerTime, SystemStatus, TickerInfo,
};

/// Trait defining all Kraken REST API operations.
///
/// This trait enables dependency injection and allows for:
/// - Testing with mock implementations
/// - Wrapping with decorators (e.g., rate limiting)
/// - Alternative implementations
///
/// All methods are async and return `Result<T, KrakenError>`.
pub trait KrakenClient: Send + Sync {
    // ========== Public Endpoints ==========

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

    // ========== Private Endpoints - Account ==========

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

    // ========== Private Endpoints - Funding ==========

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

    // ========== Private Endpoints - Earn ==========

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

    // ========== Private Endpoints - Trading ==========

    /// Add a new order.
    fn add_order(
        &self,
        request: &AddOrderRequest,
    ) -> impl Future<Output = Result<AddOrderResponse, KrakenError>> + Send;

    /// Cancel an order.
    fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> impl Future<Output = Result<CancelOrderResponse, KrakenError>> + Send;

    /// Cancel all open orders.
    fn cancel_all_orders(
        &self,
    ) -> impl Future<Output = Result<CancelOrderResponse, KrakenError>> + Send;

    // ========== Private Endpoints - WebSocket ==========

    /// Get a WebSocket authentication token.
    fn get_websocket_token(
        &self,
    ) -> impl Future<Output = Result<WebSocketToken, KrakenError>> + Send;
}

/// Extension trait for boxed trait objects.
///
/// This allows using `KrakenClient` as a trait object via `Box<dyn KrakenClientExt>`.
#[allow(async_fn_in_trait)]
pub trait KrakenClientExt: Send + Sync {
    // ========== Public Endpoints ==========

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

    // ========== Private Endpoints - Account ==========

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

    // ========== Private Endpoints - Funding ==========

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

    // ========== Private Endpoints - Earn ==========

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

    // ========== Private Endpoints - Trading ==========

    async fn add_order(&self, request: &AddOrderRequest) -> Result<AddOrderResponse, KrakenError>;
    async fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<CancelOrderResponse, KrakenError>;
    async fn cancel_all_orders(&self) -> Result<CancelOrderResponse, KrakenError>;

    // ========== Private Endpoints - WebSocket ==========

    async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError>;
}

// Blanket implementation for types that implement KrakenClient
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

    async fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        KrakenClient::cancel_order(self, request).await
    }

    async fn cancel_all_orders(&self) -> Result<CancelOrderResponse, KrakenError> {
        KrakenClient::cancel_all_orders(self).await
    }

    async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError> {
        KrakenClient::get_websocket_token(self).await
    }
}

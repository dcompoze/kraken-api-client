//! Rate-limited REST client wrapper.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use rust_decimal::Decimal;
use tokio::sync::Mutex;

use crate::error::KrakenError;
use crate::rate_limit::{
    KeyedRateLimiter, OrderTrackingInfo, RateLimitConfig, SlidingWindow, TradingRateLimiter,
};
use crate::spot::rest::private::{
    AccountTransferRequest, AccountTransferResponse, AddExportRequest, AddExportResponse,
    AddOrderBatchRequest, AddOrderBatchResponse, AddOrderRequest, AddOrderResponse,
    AllocationStatus, AmendOrderRequest, AmendOrderResponse, BatchCancelId,
    CancelAllOrdersAfterRequest, CancelAllOrdersAfterResponse, CancelOrderBatchRequest,
    CancelOrderRequest, CancelOrderResponse, ClosedOrders, ClosedOrdersRequest, ConfirmationRefId,
    CreateSubaccountRequest, DepositAddress, DepositAddressesRequest, DepositMethod,
    DepositMethodsRequest, DepositStatusRequest, DepositWithdrawStatusResponse,
    EarnAllocationStatusRequest, EarnAllocateRequest, EarnAllocations, EarnAllocationsRequest,
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
    AssetInfo, AssetInfoRequest, AssetPair, AssetPairsRequest, OhlcRequest, OhlcResponse, OrderBook,
    OrderBookRequest, RecentSpreadsRequest, RecentSpreadsResponse, RecentTradesRequest,
    RecentTradesResponse, ServerTime, SystemStatus, TickerInfo,
};
use crate::spot::rest::KrakenClient;
use crate::types::VerificationTier;

/// A rate-limited wrapper around any [`KrakenClient`] implementation.
///
/// Handles public endpoint limits (sliding window), private endpoint limits (token bucket, tier-based), and trading limits with order lifetime penalties.
pub struct RateLimitedClient<C> {
    inner: C,
    config: RateLimitConfig,
    /// Public endpoint rate limiter (sliding window)
    public_limiter: Arc<Mutex<SlidingWindow>>,
    /// Private endpoint rate limiter (token bucket)
    private_limiter: Arc<Mutex<PrivateRateLimiter>>,
    /// Trading rate limiter with order penalties
    trading_limiter: Arc<Mutex<TradingRateLimiter>>,
    /// Per-pair rate limiter for order book requests
    orderbook_limiter: Arc<Mutex<KeyedRateLimiter<String>>>,
}

impl<C> RateLimitedClient<C> {
    /// Create a new rate-limited client wrapper.
    pub fn new(inner: C, config: RateLimitConfig) -> Self {
        let (max_counter, decay_rate) = config.tier.rate_limit_params();

        Self {
            inner,
            config: config.clone(),
            // Public: 1 request per second per endpoint.
            public_limiter: Arc::new(Mutex::new(SlidingWindow::new(
                Duration::from_secs(1),
                1,
            ))),
            private_limiter: Arc::new(Mutex::new(PrivateRateLimiter::new(
                max_counter,
                decay_rate,
            ))),
            trading_limiter: Arc::new(Mutex::new(TradingRateLimiter::new(
                max_counter,
                decay_rate,
            ))),
            // Order book: 1 request per second per pair.
            orderbook_limiter: Arc::new(Mutex::new(KeyedRateLimiter::new(
                Duration::from_secs(1),
                1,
            ))),
        }
    }

    /// Create a new rate-limited client with a specific verification tier.
    pub fn with_tier(inner: C, tier: VerificationTier) -> Self {
        Self::new(
            inner,
            RateLimitConfig {
                tier,
                enabled: true,
            },
        )
    }

    /// Get a reference to the inner client.
    pub fn inner(&self) -> &C {
        &self.inner
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }

    /// Enable or disable rate limiting.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.config.enabled = enabled;
    }

    /// Wait for the public rate limiter.
    async fn wait_public(&self) -> Result<(), KrakenError> {
        if !self.config.enabled {
            return Ok(());
        }

        loop {
            let mut limiter = self.public_limiter.lock().await;
            match limiter.try_acquire() {
                Ok(()) => return Ok(()),
                Err(wait_time) => {
                    drop(limiter);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }

    /// Wait for the private rate limiter.
    async fn wait_private(&self) -> Result<(), KrakenError> {
        if !self.config.enabled {
            return Ok(());
        }

        loop {
            let mut limiter = self.private_limiter.lock().await;
            match limiter.try_acquire() {
                Ok(()) => return Ok(()),
                Err(wait_time) => {
                    drop(limiter);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }

    /// Wait for the order book rate limiter (per-pair).
    async fn wait_orderbook(&self, pair: &str) -> Result<(), KrakenError> {
        if !self.config.enabled {
            return Ok(());
        }

        loop {
            let mut limiter = self.orderbook_limiter.lock().await;
            match limiter.try_acquire(pair.to_string()) {
                Ok(()) => return Ok(()),
                Err(wait_time) => {
                    drop(limiter);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }

    /// Wait for the trading rate limiter (order placement).
    async fn wait_trading_order(
        &self,
        order_id: &str,
        pair: &str,
    ) -> Result<(), KrakenError> {
        if !self.config.enabled {
            return Ok(());
        }

        loop {
            let mut limiter = self.trading_limiter.lock().await;
            let info = OrderTrackingInfo::new(pair);
            match limiter.try_place_order(order_id, info) {
                Ok(()) => return Ok(()),
                Err(wait_time) => {
                    drop(limiter);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }

    /// Wait for the trading rate limiter (order cancellation).
    async fn wait_trading_cancel(&self, order_id: &str) -> Result<(), KrakenError> {
        if !self.config.enabled {
            return Ok(());
        }

        loop {
            let mut limiter = self.trading_limiter.lock().await;
            match limiter.try_cancel_order(order_id) {
                Ok(_penalty) => return Ok(()),
                Err(wait_time) => {
                    drop(limiter);
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }
}

impl<C: std::fmt::Debug> std::fmt::Debug for RateLimitedClient<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimitedClient")
            .field("inner", &self.inner)
            .field("config", &self.config)
            .finish()
    }
}

impl<C: Clone> Clone for RateLimitedClient<C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            config: self.config.clone(),
            public_limiter: self.public_limiter.clone(),
            private_limiter: self.private_limiter.clone(),
            trading_limiter: self.trading_limiter.clone(),
            orderbook_limiter: self.orderbook_limiter.clone(),
        }
    }
}

/// Private endpoint rate limiter using token bucket algorithm.
#[derive(Debug)]
struct PrivateRateLimiter {
    /// Current counter (scaled 100x for precision)
    counter: i64,
    /// Maximum counter (scaled 100x)
    max_counter: i64,
    /// Decay rate per second (scaled 100x)
    decay_rate: i64,
    /// Last update timestamp
    last_update: std::time::Instant,
}

impl PrivateRateLimiter {
    fn new(max_counter: u32, decay_rate_per_sec: f64) -> Self {
        Self {
            counter: 0,
            max_counter: (max_counter as i64) * 100,
            decay_rate: (decay_rate_per_sec * 100.0) as i64,
            last_update: std::time::Instant::now(),
        }
    }

    fn update(&mut self) {
        let elapsed = self.last_update.elapsed();
        let elapsed_secs = elapsed.as_secs_f64();
        let decay = (elapsed_secs * self.decay_rate as f64) as i64;
        self.counter = (self.counter - decay).max(0);
        self.last_update = std::time::Instant::now();
    }

    fn try_acquire(&mut self) -> Result<(), Duration> {
        self.update();

        // Most private endpoints cost 1 point.
        let cost = 100;

        if self.counter + cost <= self.max_counter {
            self.counter += cost;
            Ok(())
        } else {
            let excess = self.counter + cost - self.max_counter;
            let wait_secs = excess as f64 / self.decay_rate as f64;
            Err(Duration::from_secs_f64(wait_secs))
        }
    }
}

impl<C: KrakenClient> KrakenClient for RateLimitedClient<C> {
    async fn get_server_time(&self) -> Result<ServerTime, KrakenError> {
        self.wait_public().await?;
        self.inner.get_server_time().await
    }

    async fn get_system_status(&self) -> Result<SystemStatus, KrakenError> {
        self.wait_public().await?;
        self.inner.get_system_status().await
    }

    async fn get_assets(
        &self,
        request: Option<&AssetInfoRequest>,
    ) -> Result<HashMap<String, AssetInfo>, KrakenError> {
        self.wait_public().await?;
        self.inner.get_assets(request).await
    }

    async fn get_asset_pairs(
        &self,
        request: Option<&AssetPairsRequest>,
    ) -> Result<HashMap<String, AssetPair>, KrakenError> {
        self.wait_public().await?;
        self.inner.get_asset_pairs(request).await
    }

    async fn get_ticker(&self, pairs: &str) -> Result<HashMap<String, TickerInfo>, KrakenError> {
        self.wait_public().await?;
        self.inner.get_ticker(pairs).await
    }

    async fn get_ohlc(&self, request: &OhlcRequest) -> Result<OhlcResponse, KrakenError> {
        self.wait_public().await?;
        self.inner.get_ohlc(request).await
    }

    async fn get_order_book(
        &self,
        request: &OrderBookRequest,
    ) -> Result<HashMap<String, OrderBook>, KrakenError> {
        self.wait_orderbook(&request.pair).await?;
        self.inner.get_order_book(request).await
    }

    async fn get_recent_trades(
        &self,
        request: &RecentTradesRequest,
    ) -> Result<RecentTradesResponse, KrakenError> {
        self.wait_public().await?;
        self.inner.get_recent_trades(request).await
    }

    async fn get_recent_spreads(
        &self,
        request: &RecentSpreadsRequest,
    ) -> Result<RecentSpreadsResponse, KrakenError> {
        self.wait_public().await?;
        self.inner.get_recent_spreads(request).await
    }

    async fn get_account_balance(&self) -> Result<HashMap<String, Decimal>, KrakenError> {
        self.wait_private().await?;
        self.inner.get_account_balance().await
    }

    async fn get_extended_balance(&self) -> Result<ExtendedBalances, KrakenError> {
        self.wait_private().await?;
        self.inner.get_extended_balance().await
    }

    async fn get_trade_balance(
        &self,
        request: Option<&TradeBalanceRequest>,
    ) -> Result<TradeBalance, KrakenError> {
        self.wait_private().await?;
        self.inner.get_trade_balance(request).await
    }

    async fn get_open_orders(
        &self,
        request: Option<&OpenOrdersRequest>,
    ) -> Result<OpenOrders, KrakenError> {
        self.wait_private().await?;
        self.inner.get_open_orders(request).await
    }

    async fn get_closed_orders(
        &self,
        request: Option<&ClosedOrdersRequest>,
    ) -> Result<ClosedOrders, KrakenError> {
        self.wait_private().await?;
        self.inner.get_closed_orders(request).await
    }

    async fn query_orders(
        &self,
        request: &QueryOrdersRequest,
    ) -> Result<HashMap<String, Order>, KrakenError> {
        self.wait_private().await?;
        self.inner.query_orders(request).await
    }

    async fn get_trades_history(
        &self,
        request: Option<&TradesHistoryRequest>,
    ) -> Result<TradesHistory, KrakenError> {
        self.wait_private().await?;
        self.inner.get_trades_history(request).await
    }

    async fn get_open_positions(
        &self,
        request: Option<&OpenPositionsRequest>,
    ) -> Result<HashMap<String, Position>, KrakenError> {
        self.wait_private().await?;
        self.inner.get_open_positions(request).await
    }

    async fn get_ledgers(
        &self,
        request: Option<&LedgersRequest>,
    ) -> Result<LedgersInfo, KrakenError> {
        self.wait_private().await?;
        self.inner.get_ledgers(request).await
    }

    async fn get_trade_volume(
        &self,
        request: Option<&TradeVolumeRequest>,
    ) -> Result<TradeVolume, KrakenError> {
        self.wait_private().await?;
        self.inner.get_trade_volume(request).await
    }

    async fn get_deposit_methods(
        &self,
        request: &DepositMethodsRequest,
    ) -> Result<Vec<DepositMethod>, KrakenError> {
        self.wait_private().await?;
        self.inner.get_deposit_methods(request).await
    }

    async fn get_deposit_addresses(
        &self,
        request: &DepositAddressesRequest,
    ) -> Result<Vec<DepositAddress>, KrakenError> {
        self.wait_private().await?;
        self.inner.get_deposit_addresses(request).await
    }

    async fn get_deposit_status(
        &self,
        request: Option<&DepositStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        self.wait_private().await?;
        self.inner.get_deposit_status(request).await
    }

    async fn get_withdraw_methods(
        &self,
        request: Option<&WithdrawMethodsRequest>,
    ) -> Result<Vec<WithdrawMethod>, KrakenError> {
        self.wait_private().await?;
        self.inner.get_withdraw_methods(request).await
    }

    async fn get_withdraw_addresses(
        &self,
        request: Option<&WithdrawAddressesRequest>,
    ) -> Result<Vec<WithdrawalAddress>, KrakenError> {
        self.wait_private().await?;
        self.inner.get_withdraw_addresses(request).await
    }

    async fn get_withdraw_info(
        &self,
        request: &WithdrawInfoRequest,
    ) -> Result<WithdrawInfo, KrakenError> {
        self.wait_private().await?;
        self.inner.get_withdraw_info(request).await
    }

    async fn withdraw_funds(
        &self,
        request: &WithdrawRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        self.wait_private().await?;
        self.inner.withdraw_funds(request).await
    }

    async fn get_withdraw_status(
        &self,
        request: Option<&WithdrawStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        self.wait_private().await?;
        self.inner.get_withdraw_status(request).await
    }

    async fn withdraw_cancel(&self, request: &WithdrawCancelRequest) -> Result<bool, KrakenError> {
        self.wait_private().await?;
        self.inner.withdraw_cancel(request).await
    }

    async fn wallet_transfer(
        &self,
        request: &WalletTransferRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        self.wait_private().await?;
        self.inner.wallet_transfer(request).await
    }

    async fn earn_allocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError> {
        self.wait_private().await?;
        self.inner.earn_allocate(request).await
    }

    async fn earn_deallocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError> {
        self.wait_private().await?;
        self.inner.earn_deallocate(request).await
    }

    async fn query_trades(
        &self,
        request: &QueryTradesRequest,
    ) -> Result<HashMap<String, Trade>, KrakenError> {
        self.wait_private().await?;
        self.inner.query_trades(request).await
    }

    async fn query_ledgers(
        &self,
        request: &QueryLedgersRequest,
    ) -> Result<HashMap<String, LedgerEntry>, KrakenError> {
        self.wait_private().await?;
        self.inner.query_ledgers(request).await
    }

    async fn get_order_amends(
        &self,
        request: &OrderAmendsRequest,
    ) -> Result<OrderAmends, KrakenError> {
        self.wait_private().await?;
        self.inner.get_order_amends(request).await
    }

    async fn add_export(
        &self,
        request: &AddExportRequest,
    ) -> Result<AddExportResponse, KrakenError> {
        self.wait_private().await?;
        self.inner.add_export(request).await
    }

    async fn get_export_status(
        &self,
        request: &ExportStatusRequest,
    ) -> Result<Vec<ExportReportStatus>, KrakenError> {
        self.wait_private().await?;
        self.inner.get_export_status(request).await
    }

    async fn retrieve_export(
        &self,
        request: &RetrieveExportRequest,
    ) -> Result<Vec<u8>, KrakenError> {
        self.wait_private().await?;
        self.inner.retrieve_export(request).await
    }

    async fn remove_export(
        &self,
        request: &RemoveExportRequest,
    ) -> Result<RemoveExportResponse, KrakenError> {
        self.wait_private().await?;
        self.inner.remove_export(request).await
    }

    async fn create_subaccount(
        &self,
        request: &CreateSubaccountRequest,
    ) -> Result<bool, KrakenError> {
        self.wait_private().await?;
        self.inner.create_subaccount(request).await
    }

    async fn account_transfer(
        &self,
        request: &AccountTransferRequest,
    ) -> Result<AccountTransferResponse, KrakenError> {
        self.wait_private().await?;
        self.inner.account_transfer(request).await
    }

    async fn get_earn_allocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        self.wait_private().await?;
        self.inner.get_earn_allocation_status(request).await
    }

    async fn get_earn_deallocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        self.wait_private().await?;
        self.inner.get_earn_deallocation_status(request).await
    }

    async fn list_earn_strategies(
        &self,
        request: Option<&EarnStrategiesRequest>,
    ) -> Result<EarnStrategies, KrakenError> {
        self.wait_private().await?;
        self.inner.list_earn_strategies(request).await
    }

    async fn list_earn_allocations(
        &self,
        request: Option<&EarnAllocationsRequest>,
    ) -> Result<EarnAllocations, KrakenError> {
        self.wait_private().await?;
        self.inner.list_earn_allocations(request).await
    }

    async fn add_order(&self, request: &AddOrderRequest) -> Result<AddOrderResponse, KrakenError> {
        // The real order ID only arrives in the response, so charge the limiter under a temporary ID.
        let temp_id = format!("pending_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos());

        self.wait_trading_order(&temp_id, &request.pair).await?;
        let result = self.inner.add_order(request).await?;

        // Replace the temporary entry with the real order ID.
        let mut limiter = self.trading_limiter.lock().await;
        limiter.order_cancelled(&temp_id);
        if let Some(order_id) = result.txid.as_ref().and_then(|ids| ids.first()) {
            limiter.track_order(order_id.to_string(), OrderTrackingInfo::new(&request.pair));
        }

        Ok(result)
    }

    async fn add_order_batch(
        &self,
        request: &AddOrderBatchRequest,
    ) -> Result<AddOrderBatchResponse, KrakenError> {
        // Batched orders count against the trading counter like single orders,
        // so charge capacity for each order before sending the request.
        let base = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut temp_ids = Vec::with_capacity(request.orders.len());
        for index in 0..request.orders.len() {
            let temp_id = format!("pending_{}_{}", base, index);
            self.wait_trading_order(&temp_id, &request.pair).await?;
            temp_ids.push(temp_id);
        }

        let result = self.inner.add_order_batch(request).await?;

        // Replace the temporary entries with the real order IDs.
        let mut limiter = self.trading_limiter.lock().await;
        for temp_id in &temp_ids {
            limiter.order_cancelled(temp_id);
        }
        for order in &result.orders {
            if let Some(txid) = &order.txid {
                limiter.track_order(txid.to_string(), OrderTrackingInfo::new(&request.pair));
            }
        }

        Ok(result)
    }

    async fn amend_order(
        &self,
        request: &AmendOrderRequest,
    ) -> Result<AmendOrderResponse, KrakenError> {
        // Amends count against the trading counter with the age based penalty,
        // the same schedule that applies to edits.
        // Amends by client order ID are not tracked, so they take the worst case.
        let id = request.txid.as_deref().or(request.cl_ord_id.as_deref());
        if let Some(id) = id {
            self.wait_trading_cancel(id).await?;
        }
        let result = self.inner.amend_order(request).await?;

        // The order stays open under the same transaction ID, so track it again.
        if let Some(txid) = &request.txid {
            let mut limiter = self.trading_limiter.lock().await;
            limiter.track_order(txid.to_string(), OrderTrackingInfo::new(""));
        }

        Ok(result)
    }

    async fn edit_order(
        &self,
        request: &EditOrderRequest,
    ) -> Result<EditOrderResponse, KrakenError> {
        // Editing cancels the original order, so the age-based penalty applies.
        self.wait_trading_cancel(&request.txid).await?;
        let result = self.inner.edit_order(request).await?;

        if let Some(txid) = &result.txid {
            let mut limiter = self.trading_limiter.lock().await;
            limiter.track_order(txid.to_string(), OrderTrackingInfo::new(&request.pair));
        }

        Ok(result)
    }

    async fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        self.wait_trading_cancel(&request.txid).await?;
        self.inner.cancel_order(request).await
    }

    async fn cancel_all_orders(&self) -> Result<CancelOrderResponse, KrakenError> {
        // Cancel all does not track individual orders.
        self.wait_private().await?;
        self.inner.cancel_all_orders().await
    }

    async fn cancel_all_orders_after(
        &self,
        request: &CancelAllOrdersAfterRequest,
    ) -> Result<CancelAllOrdersAfterResponse, KrakenError> {
        self.wait_private().await?;
        self.inner.cancel_all_orders_after(request).await
    }

    async fn cancel_order_batch(
        &self,
        request: &CancelOrderBatchRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        // Apply the age-based penalty for every order in the batch.
        // Orders identified by user reference or client order ID are not
        // tracked by transaction ID, so they take the worst case penalty.
        for order in &request.orders {
            match order {
                BatchCancelId::Txid(txid) => self.wait_trading_cancel(txid).await?,
                BatchCancelId::Userref(userref) => {
                    self.wait_trading_cancel(&userref.to_string()).await?
                }
            }
        }
        if let Some(cl_ord_ids) = &request.cl_ord_ids {
            for id in cl_ord_ids {
                self.wait_trading_cancel(id).await?;
            }
        }
        self.inner.cancel_order_batch(request).await
    }

    async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError> {
        self.wait_private().await?;
        self.inner.get_websocket_token().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_rate_limiter_allows_initial_requests() {
        let mut limiter = PrivateRateLimiter::new(20, 1.0);

        for _ in 0..15 {
            assert!(limiter.try_acquire().is_ok());
        }
    }

    #[test]
    fn test_private_rate_limiter_blocks_when_full() {
        let mut limiter = PrivateRateLimiter::new(20, 1.0);

        for _ in 0..20 {
            limiter.try_acquire().ok();
        }

        assert!(limiter.try_acquire().is_err());
    }

    #[test]
    fn test_private_rate_limiter_decay() {
        let mut limiter = PrivateRateLimiter::new(20, 100.0);

        for _ in 0..10 {
            limiter.try_acquire().ok();
        }

        std::thread::sleep(Duration::from_millis(150));

        limiter.update();
        assert!(limiter.counter < 1000);
    }
}

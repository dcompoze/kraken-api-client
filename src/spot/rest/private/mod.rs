//! Private REST API endpoints (authentication required).
//!
//! These endpoints require API credentials to be configured on the client.

mod types;

pub use types::*;

use crate::error::KrakenError;
use crate::spot::rest::SpotRestClient;
use crate::spot::rest::endpoints::private;

impl SpotRestClient {
    /// Get account balance.
    ///
    /// Returns the balances of all assets in the account.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use kraken_api_client::spot::rest::SpotRestClient;
    /// use kraken_api_client::auth::StaticCredentials;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let credentials = Arc::new(StaticCredentials::new("key", "secret"));
    ///     let client = SpotRestClient::builder().credentials(credentials).build();
    ///
    ///     let balances = client.get_account_balance().await?;
    ///     for (asset, balance) in balances {
    ///         println!("{}: {}", asset, balance);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_account_balance(
        &self,
    ) -> Result<std::collections::HashMap<String, rust_decimal::Decimal>, KrakenError> {
        #[derive(serde::Serialize)]
        struct Empty {}
        self.private_post(private::BALANCE, &Empty {}).await
    }

    /// Get extended balance with hold amounts.
    pub async fn get_extended_balance(&self) -> Result<ExtendedBalances, KrakenError> {
        #[derive(serde::Serialize)]
        struct Empty {}
        self.private_post(private::BALANCE_EX, &Empty {}).await
    }

    /// Get trade balance.
    ///
    /// Returns margin account details including equity, margin, and P&L.
    pub async fn get_trade_balance(
        &self,
        request: Option<&TradeBalanceRequest>,
    ) -> Result<TradeBalance, KrakenError> {
        match request {
            Some(req) => self.private_post(private::TRADE_BALANCE, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::TRADE_BALANCE, &Empty {}).await
            }
        }
    }

    /// Get open orders.
    pub async fn get_open_orders(
        &self,
        request: Option<&OpenOrdersRequest>,
    ) -> Result<OpenOrders, KrakenError> {
        match request {
            Some(req) => self.private_post(private::OPEN_ORDERS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::OPEN_ORDERS, &Empty {}).await
            }
        }
    }

    /// Get closed orders.
    pub async fn get_closed_orders(
        &self,
        request: Option<&ClosedOrdersRequest>,
    ) -> Result<ClosedOrders, KrakenError> {
        match request {
            Some(req) => self.private_post(private::CLOSED_ORDERS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::CLOSED_ORDERS, &Empty {}).await
            }
        }
    }

    /// Query specific orders by ID.
    pub async fn query_orders(
        &self,
        request: &QueryOrdersRequest,
    ) -> Result<std::collections::HashMap<String, Order>, KrakenError> {
        self.private_post(private::QUERY_ORDERS, request).await
    }

    /// Get trades history.
    pub async fn get_trades_history(
        &self,
        request: Option<&TradesHistoryRequest>,
    ) -> Result<TradesHistory, KrakenError> {
        match request {
            Some(req) => self.private_post(private::TRADES_HISTORY, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::TRADES_HISTORY, &Empty {}).await
            }
        }
    }

    /// Get open positions.
    pub async fn get_open_positions(
        &self,
        request: Option<&OpenPositionsRequest>,
    ) -> Result<std::collections::HashMap<String, Position>, KrakenError> {
        match request {
            Some(req) => self.private_post(private::OPEN_POSITIONS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::OPEN_POSITIONS, &Empty {}).await
            }
        }
    }

    /// Get ledger entries.
    pub async fn get_ledgers(
        &self,
        request: Option<&LedgersRequest>,
    ) -> Result<LedgersInfo, KrakenError> {
        match request {
            Some(req) => self.private_post(private::LEDGERS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::LEDGERS, &Empty {}).await
            }
        }
    }

    /// Get trade volume and fee info.
    pub async fn get_trade_volume(
        &self,
        request: Option<&TradeVolumeRequest>,
    ) -> Result<TradeVolume, KrakenError> {
        match request {
            Some(req) => self.private_post(private::TRADE_VOLUME, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::TRADE_VOLUME, &Empty {}).await
            }
        }
    }

    // ========== Funding Endpoints ==========

    /// Get available deposit methods for an asset.
    pub async fn get_deposit_methods(
        &self,
        request: &DepositMethodsRequest,
    ) -> Result<Vec<DepositMethod>, KrakenError> {
        self.private_post(private::DEPOSIT_METHODS, request).await
    }

    /// Get deposit addresses for an asset and method.
    pub async fn get_deposit_addresses(
        &self,
        request: &DepositAddressesRequest,
    ) -> Result<Vec<DepositAddress>, KrakenError> {
        self.private_post(private::DEPOSIT_ADDRESSES, request).await
    }

    /// Get deposit status.
    pub async fn get_deposit_status(
        &self,
        request: Option<&DepositStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        match request {
            Some(req) => self.private_post(private::DEPOSIT_STATUS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::DEPOSIT_STATUS, &Empty {}).await
            }
        }
    }

    /// Get available withdrawal methods.
    pub async fn get_withdraw_methods(
        &self,
        request: Option<&WithdrawMethodsRequest>,
    ) -> Result<Vec<WithdrawMethod>, KrakenError> {
        match request {
            Some(req) => self.private_post(private::WITHDRAW_METHODS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::WITHDRAW_METHODS, &Empty {})
                    .await
            }
        }
    }

    /// Get withdrawal addresses.
    pub async fn get_withdraw_addresses(
        &self,
        request: Option<&WithdrawAddressesRequest>,
    ) -> Result<Vec<WithdrawalAddress>, KrakenError> {
        match request {
            Some(req) => self.private_post(private::WITHDRAW_ADDRESSES, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::WITHDRAW_ADDRESSES, &Empty {})
                    .await
            }
        }
    }

    /// Get withdrawal info (limits and fees).
    pub async fn get_withdraw_info(
        &self,
        request: &WithdrawInfoRequest,
    ) -> Result<WithdrawInfo, KrakenError> {
        self.private_post(private::WITHDRAW_INFO, request).await
    }

    /// Withdraw funds.
    pub async fn withdraw_funds(
        &self,
        request: &WithdrawRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        self.private_post(private::WITHDRAW, request).await
    }

    /// Get withdrawal status.
    pub async fn get_withdraw_status(
        &self,
        request: Option<&WithdrawStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        match request {
            Some(req) => self.private_post(private::WITHDRAW_STATUS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::WITHDRAW_STATUS, &Empty {}).await
            }
        }
    }

    /// Cancel a withdrawal.
    pub async fn withdraw_cancel(
        &self,
        request: &WithdrawCancelRequest,
    ) -> Result<bool, KrakenError> {
        self.private_post(private::WITHDRAW_CANCEL, request).await
    }

    /// Transfer funds between wallets (e.g., Spot to Futures).
    pub async fn wallet_transfer(
        &self,
        request: &WalletTransferRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        self.private_post(private::WALLET_TRANSFER, request).await
    }

    // ========== Earn Endpoints ==========

    /// Allocate funds to an earn strategy.
    pub async fn earn_allocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError> {
        self.private_post(private::EARN_ALLOCATE, request).await
    }

    /// Deallocate funds from an earn strategy.
    pub async fn earn_deallocate(
        &self,
        request: &EarnAllocateRequest,
    ) -> Result<bool, KrakenError> {
        self.private_post(private::EARN_DEALLOCATE, request).await
    }

    /// Get earn allocation status.
    pub async fn get_earn_allocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        self.private_post(private::EARN_ALLOCATE_STATUS, request)
            .await
    }

    /// Get earn deallocation status.
    pub async fn get_earn_deallocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        self.private_post(private::EARN_DEALLOCATE_STATUS, request)
            .await
    }

    /// List earn strategies.
    pub async fn list_earn_strategies(
        &self,
        request: Option<&EarnStrategiesRequest>,
    ) -> Result<EarnStrategies, KrakenError> {
        match request {
            Some(req) => self.private_post(private::EARN_STRATEGIES, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::EARN_STRATEGIES, &Empty {}).await
            }
        }
    }

    /// List earn allocations.
    pub async fn list_earn_allocations(
        &self,
        request: Option<&EarnAllocationsRequest>,
    ) -> Result<EarnAllocations, KrakenError> {
        match request {
            Some(req) => self.private_post(private::EARN_ALLOCATIONS, req).await,
            None => {
                #[derive(serde::Serialize)]
                struct Empty {}
                self.private_post(private::EARN_ALLOCATIONS, &Empty {})
                    .await
            }
        }
    }

    /// Add a new order.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use kraken_api_client::spot::rest::{SpotRestClient, private::AddOrderRequest};
    /// use kraken_api_client::{BuySell, OrderType};
    /// use kraken_api_client::auth::StaticCredentials;
    /// use rust_decimal::Decimal;
    /// use std::str::FromStr;
    /// use std::sync::Arc;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let credentials = Arc::new(StaticCredentials::new("key", "secret"));
    ///     let client = SpotRestClient::builder().credentials(credentials).build();
    ///
    ///     let request = AddOrderRequest::new(
    ///         "XBTUSD",
    ///         BuySell::Buy,
    ///         OrderType::Limit,
    ///         Decimal::from_str("0.001")?,
    ///     )
    ///     .price(Decimal::from_str("50000")?)
    ///     .validate(true); // Validate only, don't actually place
    ///
    ///     let result = client.add_order(&request).await?;
    ///     println!("Order result: {:?}", result);
    ///     Ok(())
    /// }
    /// ```
    pub async fn add_order(
        &self,
        request: &AddOrderRequest,
    ) -> Result<AddOrderResponse, KrakenError> {
        self.private_post(private::ADD_ORDER, request).await
    }

    /// Cancel an order.
    pub async fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        self.private_post(private::CANCEL_ORDER, request).await
    }

    /// Cancel all open orders.
    pub async fn cancel_all_orders(&self) -> Result<CancelOrderResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Empty {}
        self.private_post(private::CANCEL_ALL, &Empty {}).await
    }

    /// Get a WebSocket authentication token.
    ///
    /// The token is valid for 15 minutes and is used to authenticate
    /// WebSocket connections to private channels.
    pub async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError> {
        #[derive(serde::Serialize)]
        struct Empty {}
        self.private_post(private::GET_WEBSOCKETS_TOKEN, &Empty {})
            .await
    }
}

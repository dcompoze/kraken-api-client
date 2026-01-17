//! Kraken Spot REST API client implementation.

use std::collections::HashMap;
use std::sync::Arc;

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;
use rust_decimal::Decimal;

use crate::auth::{CredentialsProvider, IncreasingNonce, NonceProvider, sign_request};
use crate::error::{ApiError, KrakenError};
use crate::spot::rest::endpoints::KRAKEN_BASE_URL;
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
use crate::spot::rest::traits::KrakenClient;

/// The Kraken Spot REST API client.
///
/// This client provides access to all Kraken Spot trading REST endpoints.
/// It handles authentication, rate limiting, and automatic retries.
///
/// # Example
///
/// ```rust,no_run
/// use kraken_api_client::spot::rest::SpotRestClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client for public endpoints only
///     let client = SpotRestClient::new();
///
///     // Get server time
///     let time = client.get_server_time().await?;
///     println!("Server time: {:?}", time);
///
///     Ok(())
/// }
/// ```
///
/// For private endpoints, provide credentials:
///
/// ```rust,no_run
/// use kraken_api_client::spot::rest::SpotRestClient;
/// use kraken_api_client::auth::StaticCredentials;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let credentials = Arc::new(StaticCredentials::new("api_key", "api_secret"));
///     let client = SpotRestClient::builder()
///         .credentials(credentials)
///         .build();
///
///     let balance = client.get_account_balance().await?;
///     println!("Balance: {:?}", balance);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct SpotRestClient {
    http_client: ClientWithMiddleware,
    base_url: String,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    nonce_provider: Arc<dyn NonceProvider>,
}

impl SpotRestClient {
    /// Create a new client with default settings.
    ///
    /// This client can only access public endpoints.
    /// Use [`SpotRestClient::builder()`] to configure credentials for private endpoints.
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Create a new client builder.
    pub fn builder() -> SpotRestClientBuilder {
        SpotRestClientBuilder::new()
    }

    /// Make a public GET request.
    pub(crate) async fn public_get<T>(&self, endpoint: &str) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self.http_client.get(&url).send().await?;
        self.parse_response(response).await
    }

    /// Make a public GET request with query parameters.
    pub(crate) async fn public_get_with_params<T, Q>(
        &self,
        endpoint: &str,
        params: &Q,
    ) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
        Q: serde::Serialize + ?Sized,
    {
        let query_string = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;
        let url = if query_string.is_empty() {
            format!("{}{}", self.base_url, endpoint)
        } else {
            format!("{}{}?{}", self.base_url, endpoint, query_string)
        };
        let response = self.http_client.get(&url).send().await?;
        self.parse_response(response).await
    }

    /// Make an authenticated POST request.
    pub(crate) async fn private_post<T, P>(
        &self,
        endpoint: &str,
        params: &P,
    ) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
        P: serde::Serialize,
    {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(KrakenError::MissingCredentials)?;

        let nonce = self.nonce_provider.next_nonce();
        let creds = credentials.get_credentials();

        // Build the POST body with nonce.
        let mut form_data = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        if form_data.is_empty() {
            form_data = format!("nonce={}", nonce);
        } else {
            form_data = format!("nonce={}&{}", nonce, form_data);
        }

        // Sign the request.
        let signature = sign_request(creds, endpoint, nonce, &form_data)?;

        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .http_client
            .post(&url)
            .header("API-Key", &creds.api_key)
            .header("API-Sign", signature)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(form_data)
            .send()
            .await?;

        self.parse_response(response).await
    }

    /// Parse a response from the Kraken API.
    async fn parse_response<T>(&self, response: reqwest::Response) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;

        // Kraken always returns 200 even for errors, so parse the JSON response.
        let parsed: KrakenResponse<T> = serde_json::from_str(&body).map_err(|e| {
            KrakenError::InvalidResponse(format!("Failed to parse response: {}. Body: {}", e, body))
        })?;

        // Check for API errors.
        if !parsed.error.is_empty() {
            if let Some(api_error) = ApiError::from_error_array(&parsed.error) {
                if api_error.is_rate_limit() {
                    return Err(KrakenError::RateLimitExceeded {
                        retry_after_ms: None,
                    });
                }
                return Err(KrakenError::Api(api_error));
            }
        }

        // Return the result.
        parsed.result.ok_or_else(|| {
            if !status.is_success() {
                KrakenError::InvalidResponse(format!("HTTP {}: {}", status, body))
            } else {
                KrakenError::InvalidResponse("Response missing 'result' field".to_string())
            }
        })
    }
}

impl Default for SpotRestClient {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SpotRestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpotRestClient")
            .field("base_url", &self.base_url)
            .field("has_credentials", &self.credentials.is_some())
            .finish()
    }
}

/// Builder for [`SpotRestClient`].
pub struct SpotRestClientBuilder {
    base_url: String,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    nonce_provider: Option<Arc<dyn NonceProvider>>,
    user_agent: Option<String>,
    max_retries: u32,
}

impl SpotRestClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            base_url: KRAKEN_BASE_URL.to_string(),
            credentials: None,
            nonce_provider: None,
            user_agent: None,
            max_retries: 3,
        }
    }

    /// Set the base URL (useful for testing with a mock server).
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set the credentials provider for authenticated requests.
    pub fn credentials(mut self, credentials: Arc<dyn CredentialsProvider>) -> Self {
        self.credentials = Some(credentials);
        self
    }

    /// Set a custom nonce provider.
    pub fn nonce_provider(mut self, provider: Arc<dyn NonceProvider>) -> Self {
        self.nonce_provider = Some(provider);
        self
    }

    /// Set a custom user agent.
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set the maximum number of retries for transient failures.
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Build the client.
    pub fn build(self) -> SpotRestClient {
        // Build default headers.
        let mut headers = HeaderMap::new();
        let user_agent = self
            .user_agent
            .unwrap_or_else(|| format!("kraken-api-client/{}", env!("CARGO_PKG_VERSION")));
        let header_value = HeaderValue::from_str(&user_agent)
            .unwrap_or_else(|_| HeaderValue::from_static("kraken-api-client"));
        headers.insert(USER_AGENT, header_value);

        // Build the HTTP client with middleware.
        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.max_retries);

        let client = ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        let nonce_provider = self
            .nonce_provider
            .unwrap_or_else(|| Arc::new(IncreasingNonce::new()));

        SpotRestClient {
            http_client: client,
            base_url: self.base_url,
            credentials: self.credentials,
            nonce_provider,
        }
    }
}

impl Default for SpotRestClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal response wrapper for Kraken API responses.
#[derive(Debug, serde::Deserialize)]
struct KrakenResponse<T> {
    error: Vec<String>,
    result: Option<T>,
}

// KrakenClient trait implementation.

impl KrakenClient for SpotRestClient {
    // ========== Public Endpoints ==========

    async fn get_server_time(&self) -> Result<ServerTime, KrakenError> {
        SpotRestClient::get_server_time(self).await
    }

    async fn get_system_status(&self) -> Result<SystemStatus, KrakenError> {
        SpotRestClient::get_system_status(self).await
    }

    async fn get_assets(
        &self,
        request: Option<&AssetInfoRequest>,
    ) -> Result<HashMap<String, AssetInfo>, KrakenError> {
        SpotRestClient::get_assets(self, request).await
    }

    async fn get_asset_pairs(
        &self,
        request: Option<&AssetPairsRequest>,
    ) -> Result<HashMap<String, AssetPair>, KrakenError> {
        SpotRestClient::get_asset_pairs(self, request).await
    }

    async fn get_ticker(&self, pairs: &str) -> Result<HashMap<String, TickerInfo>, KrakenError> {
        SpotRestClient::get_ticker(self, pairs).await
    }

    async fn get_ohlc(&self, request: &OhlcRequest) -> Result<OhlcResponse, KrakenError> {
        SpotRestClient::get_ohlc(self, request).await
    }

    async fn get_order_book(
        &self,
        request: &OrderBookRequest,
    ) -> Result<HashMap<String, OrderBook>, KrakenError> {
        SpotRestClient::get_order_book(self, request).await
    }

    async fn get_recent_trades(
        &self,
        request: &RecentTradesRequest,
    ) -> Result<RecentTradesResponse, KrakenError> {
        SpotRestClient::get_recent_trades(self, request).await
    }

    async fn get_recent_spreads(
        &self,
        request: &RecentSpreadsRequest,
    ) -> Result<RecentSpreadsResponse, KrakenError> {
        SpotRestClient::get_recent_spreads(self, request).await
    }

    // ========== Private Endpoints - Account ==========

    async fn get_account_balance(&self) -> Result<HashMap<String, Decimal>, KrakenError> {
        SpotRestClient::get_account_balance(self).await
    }

    async fn get_extended_balance(&self) -> Result<ExtendedBalances, KrakenError> {
        SpotRestClient::get_extended_balance(self).await
    }

    async fn get_trade_balance(
        &self,
        request: Option<&TradeBalanceRequest>,
    ) -> Result<TradeBalance, KrakenError> {
        SpotRestClient::get_trade_balance(self, request).await
    }

    async fn get_open_orders(
        &self,
        request: Option<&OpenOrdersRequest>,
    ) -> Result<OpenOrders, KrakenError> {
        SpotRestClient::get_open_orders(self, request).await
    }

    async fn get_closed_orders(
        &self,
        request: Option<&ClosedOrdersRequest>,
    ) -> Result<ClosedOrders, KrakenError> {
        SpotRestClient::get_closed_orders(self, request).await
    }

    async fn query_orders(
        &self,
        request: &QueryOrdersRequest,
    ) -> Result<HashMap<String, Order>, KrakenError> {
        SpotRestClient::query_orders(self, request).await
    }

    async fn get_trades_history(
        &self,
        request: Option<&TradesHistoryRequest>,
    ) -> Result<TradesHistory, KrakenError> {
        SpotRestClient::get_trades_history(self, request).await
    }

    async fn get_open_positions(
        &self,
        request: Option<&OpenPositionsRequest>,
    ) -> Result<HashMap<String, Position>, KrakenError> {
        SpotRestClient::get_open_positions(self, request).await
    }

    async fn get_ledgers(
        &self,
        request: Option<&LedgersRequest>,
    ) -> Result<LedgersInfo, KrakenError> {
        SpotRestClient::get_ledgers(self, request).await
    }

    async fn get_trade_volume(
        &self,
        request: Option<&TradeVolumeRequest>,
    ) -> Result<TradeVolume, KrakenError> {
        SpotRestClient::get_trade_volume(self, request).await
    }

    async fn get_deposit_methods(
        &self,
        request: &DepositMethodsRequest,
    ) -> Result<Vec<DepositMethod>, KrakenError> {
        SpotRestClient::get_deposit_methods(self, request).await
    }

    async fn get_deposit_addresses(
        &self,
        request: &DepositAddressesRequest,
    ) -> Result<Vec<DepositAddress>, KrakenError> {
        SpotRestClient::get_deposit_addresses(self, request).await
    }

    async fn get_deposit_status(
        &self,
        request: Option<&DepositStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        SpotRestClient::get_deposit_status(self, request).await
    }

    async fn get_withdraw_methods(
        &self,
        request: Option<&WithdrawMethodsRequest>,
    ) -> Result<Vec<WithdrawMethod>, KrakenError> {
        SpotRestClient::get_withdraw_methods(self, request).await
    }

    async fn get_withdraw_addresses(
        &self,
        request: Option<&WithdrawAddressesRequest>,
    ) -> Result<Vec<WithdrawalAddress>, KrakenError> {
        SpotRestClient::get_withdraw_addresses(self, request).await
    }

    async fn get_withdraw_info(
        &self,
        request: &WithdrawInfoRequest,
    ) -> Result<WithdrawInfo, KrakenError> {
        SpotRestClient::get_withdraw_info(self, request).await
    }

    async fn withdraw_funds(
        &self,
        request: &WithdrawRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        SpotRestClient::withdraw_funds(self, request).await
    }

    async fn get_withdraw_status(
        &self,
        request: Option<&WithdrawStatusRequest>,
    ) -> Result<DepositWithdrawStatusResponse, KrakenError> {
        SpotRestClient::get_withdraw_status(self, request).await
    }

    async fn withdraw_cancel(&self, request: &WithdrawCancelRequest) -> Result<bool, KrakenError> {
        SpotRestClient::withdraw_cancel(self, request).await
    }

    async fn wallet_transfer(
        &self,
        request: &WalletTransferRequest,
    ) -> Result<ConfirmationRefId, KrakenError> {
        SpotRestClient::wallet_transfer(self, request).await
    }

    async fn earn_allocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError> {
        SpotRestClient::earn_allocate(self, request).await
    }

    async fn earn_deallocate(&self, request: &EarnAllocateRequest) -> Result<bool, KrakenError> {
        SpotRestClient::earn_deallocate(self, request).await
    }

    async fn get_earn_allocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        SpotRestClient::get_earn_allocation_status(self, request).await
    }

    async fn get_earn_deallocation_status(
        &self,
        request: &EarnAllocationStatusRequest,
    ) -> Result<AllocationStatus, KrakenError> {
        SpotRestClient::get_earn_deallocation_status(self, request).await
    }

    async fn list_earn_strategies(
        &self,
        request: Option<&EarnStrategiesRequest>,
    ) -> Result<EarnStrategies, KrakenError> {
        SpotRestClient::list_earn_strategies(self, request).await
    }

    async fn list_earn_allocations(
        &self,
        request: Option<&EarnAllocationsRequest>,
    ) -> Result<EarnAllocations, KrakenError> {
        SpotRestClient::list_earn_allocations(self, request).await
    }

    // ========== Private Endpoints - Trading ==========

    async fn add_order(&self, request: &AddOrderRequest) -> Result<AddOrderResponse, KrakenError> {
        SpotRestClient::add_order(self, request).await
    }

    async fn cancel_order(
        &self,
        request: &CancelOrderRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        SpotRestClient::cancel_order(self, request).await
    }

    async fn cancel_all_orders(&self) -> Result<CancelOrderResponse, KrakenError> {
        SpotRestClient::cancel_all_orders(self).await
    }

    // ========== Private Endpoints - WebSocket ==========

    async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError> {
        SpotRestClient::get_websocket_token(self).await
    }
}

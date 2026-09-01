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
use crate::spot::rest::traits::KrakenClient;

/// The Kraken Spot REST API client.
///
/// Handles request signing and automatic retries.
#[derive(Clone)]
pub struct SpotRestClient {
    http_client: ClientWithMiddleware,
    base_url: String,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    nonce_provider: Arc<dyn NonceProvider>,
}

impl SpotRestClient {
    /// Create a new client without credentials.
    ///
    /// Such a client can only access public endpoints.
    pub fn new() -> Result<Self, KrakenError> {
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

        let mut form_data = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        if form_data.is_empty() {
            form_data = format!("nonce={}", nonce);
        } else {
            form_data = format!("nonce={}&{}", nonce, form_data);
        }

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

    /// Make an authenticated POST request with a JSON body.
    ///
    /// Some endpoints such as `AddOrderBatch` and `AmendOrder` require a JSON
    /// request body instead of form encoding.
    pub(crate) async fn private_post_json<T, P>(
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

        // The nonce must be part of the signed JSON body.
        let mut value = serde_json::to_value(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;
        match value.as_object_mut() {
            Some(object) => {
                object.insert("nonce".to_string(), serde_json::Value::from(nonce));
            }
            None => {
                return Err(KrakenError::InvalidResponse(
                    "JSON request body must be an object".to_string(),
                ));
            }
        }
        let body = serde_json::to_string(&value)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        let signature = sign_request(creds, endpoint, nonce, &body)?;

        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .http_client
            .post(&url)
            .header("API-Key", &creds.api_key)
            .header("API-Sign", signature)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?;

        self.parse_response(response).await
    }

    /// Make an authenticated POST request that returns raw bytes.
    ///
    /// Used by `RetrieveExport` which returns binary report data.
    /// If the response is a Kraken JSON error, it is parsed and returned as an error.
    pub(crate) async fn private_post_binary<P>(
        &self,
        endpoint: &str,
        params: &P,
    ) -> Result<Vec<u8>, KrakenError>
    where
        P: serde::Serialize,
    {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(KrakenError::MissingCredentials)?;

        let nonce = self.nonce_provider.next_nonce();
        let creds = credentials.get_credentials();

        let mut form_data = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        if form_data.is_empty() {
            form_data = format!("nonce={}", nonce);
        } else {
            form_data = format!("nonce={}&{}", nonce, form_data);
        }

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

        let status = response.status();
        let bytes = response.bytes().await?.to_vec();

        // An error response comes back as the usual JSON envelope.
        if let Ok(parsed) = serde_json::from_slice::<KrakenResponse<serde_json::Value>>(&bytes) {
            if !parsed.error.is_empty() {
                if let Some(api_error) = ApiError::from_error_array(&parsed.error) {
                    if api_error.is_rate_limit() {
                        return Err(KrakenError::RateLimitExceeded {
                            retry_after_ms: None,
                        });
                    }
                    return Err(KrakenError::Api(api_error));
                }
                return Err(KrakenError::InvalidResponse(format!(
                    "API error: {}",
                    parsed.error.join(", ")
                )));
            }
        }

        // Report data is not JSON, so a failed HTTP status is the only signal
        // that the body is not a report.
        if !status.is_success() {
            return Err(KrakenError::InvalidResponse(format!(
                "HTTP {}: {}",
                status,
                String::from_utf8_lossy(&bytes)
            )));
        }

        Ok(bytes)
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

        parsed.result.ok_or_else(|| {
            if !status.is_success() {
                KrakenError::InvalidResponse(format!("HTTP {}: {}", status, body))
            } else {
                KrakenError::InvalidResponse("Response missing 'result' field".to_string())
            }
        })
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
    allow_insecure_transport: bool,
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
            allow_insecure_transport: false,
        }
    }

    /// Set the base URL.
    ///
    /// Authenticated clients require HTTPS unless
    /// [`Self::danger_allow_insecure_transport`] is set for a local test server.
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

    /// Allow authenticated requests over plaintext HTTP.
    ///
    /// This option can expose API credentials. Use it only with a local test server.
    pub fn danger_allow_insecure_transport(mut self) -> Self {
        self.allow_insecure_transport = true;
        self
    }

    /// Build the client.
    pub fn build(self) -> Result<SpotRestClient, KrakenError> {
        url::Url::parse(&self.base_url)?;
        if self.credentials.is_some() {
            crate::tls::require_secure_url(
                &self.base_url,
                "https",
                self.allow_insecure_transport,
            )?;
        }

        let mut headers = HeaderMap::new();
        let user_agent = self
            .user_agent
            .unwrap_or_else(|| format!("kraken-api-client/{}", env!("CARGO_PKG_VERSION")));
        let header_value = HeaderValue::from_str(&user_agent)
            .unwrap_or_else(|_| HeaderValue::from_static("kraken-api-client"));
        headers.insert(USER_AGENT, header_value);

        let reqwest_builder = reqwest::Client::builder()
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none());

        #[cfg(feature = "rustls-tls")]
        let reqwest_builder = reqwest_builder.tls_backend_rustls();
        #[cfg(feature = "native-tls")]
        let reqwest_builder = reqwest_builder.tls_backend_native();

        let reqwest_client = reqwest_builder.build()?;

        let retry_policy = ExponentialBackoff::builder().build_with_max_retries(self.max_retries);

        let client = ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::default())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();

        let nonce_provider = self
            .nonce_provider
            .unwrap_or_else(|| Arc::new(IncreasingNonce::new()));

        Ok(SpotRestClient {
            http_client: client,
            base_url: self.base_url,
            credentials: self.credentials,
            nonce_provider,
        })
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

impl KrakenClient for SpotRestClient {
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

    async fn query_trades(
        &self,
        request: &QueryTradesRequest,
    ) -> Result<HashMap<String, Trade>, KrakenError> {
        SpotRestClient::query_trades(self, request).await
    }

    async fn query_ledgers(
        &self,
        request: &QueryLedgersRequest,
    ) -> Result<HashMap<String, LedgerEntry>, KrakenError> {
        SpotRestClient::query_ledgers(self, request).await
    }

    async fn get_order_amends(
        &self,
        request: &OrderAmendsRequest,
    ) -> Result<OrderAmends, KrakenError> {
        SpotRestClient::get_order_amends(self, request).await
    }

    async fn add_export(
        &self,
        request: &AddExportRequest,
    ) -> Result<AddExportResponse, KrakenError> {
        SpotRestClient::add_export(self, request).await
    }

    async fn get_export_status(
        &self,
        request: &ExportStatusRequest,
    ) -> Result<Vec<ExportReportStatus>, KrakenError> {
        SpotRestClient::get_export_status(self, request).await
    }

    async fn retrieve_export(
        &self,
        request: &RetrieveExportRequest,
    ) -> Result<Vec<u8>, KrakenError> {
        SpotRestClient::retrieve_export(self, request).await
    }

    async fn remove_export(
        &self,
        request: &RemoveExportRequest,
    ) -> Result<RemoveExportResponse, KrakenError> {
        SpotRestClient::remove_export(self, request).await
    }

    async fn create_subaccount(
        &self,
        request: &CreateSubaccountRequest,
    ) -> Result<bool, KrakenError> {
        SpotRestClient::create_subaccount(self, request).await
    }

    async fn account_transfer(
        &self,
        request: &AccountTransferRequest,
    ) -> Result<AccountTransferResponse, KrakenError> {
        SpotRestClient::account_transfer(self, request).await
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

    async fn add_order(&self, request: &AddOrderRequest) -> Result<AddOrderResponse, KrakenError> {
        SpotRestClient::add_order(self, request).await
    }

    async fn add_order_batch(
        &self,
        request: &AddOrderBatchRequest,
    ) -> Result<AddOrderBatchResponse, KrakenError> {
        SpotRestClient::add_order_batch(self, request).await
    }

    async fn amend_order(
        &self,
        request: &AmendOrderRequest,
    ) -> Result<AmendOrderResponse, KrakenError> {
        SpotRestClient::amend_order(self, request).await
    }

    async fn edit_order(
        &self,
        request: &EditOrderRequest,
    ) -> Result<EditOrderResponse, KrakenError> {
        SpotRestClient::edit_order(self, request).await
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

    async fn cancel_all_orders_after(
        &self,
        request: &CancelAllOrdersAfterRequest,
    ) -> Result<CancelAllOrdersAfterResponse, KrakenError> {
        SpotRestClient::cancel_all_orders_after(self, request).await
    }

    async fn cancel_order_batch(
        &self,
        request: &CancelOrderBatchRequest,
    ) -> Result<CancelOrderResponse, KrakenError> {
        SpotRestClient::cancel_order_batch(self, request).await
    }

    async fn get_websocket_token(&self) -> Result<WebSocketToken, KrakenError> {
        SpotRestClient::get_websocket_token(self).await
    }
}

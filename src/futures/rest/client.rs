//! Kraken Futures REST API client implementation.

use std::sync::Arc;

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use reqwest_tracing::TracingMiddleware;

use crate::auth::{CredentialsProvider, IncreasingNonce, NonceProvider};
use crate::error::KrakenError;
use crate::futures::auth::sign_futures_request;
use crate::futures::rest::endpoints::{FUTURES_BASE_URL, private, public};
use crate::futures::rest::types::*;
use crate::futures::types::*;

/// The Kraken Futures REST API client.
///
/// Handles authentication, rate limiting, and automatic retries.
/// Private endpoints require credentials configured via [`FuturesRestClient::builder()`].
#[derive(Clone)]
pub struct FuturesRestClient {
    http_client: ClientWithMiddleware,
    base_url: String,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    nonce_provider: Arc<dyn NonceProvider>,
}

impl FuturesRestClient {
    /// Create a new client that can only access public endpoints.
    pub fn new() -> Result<Self, KrakenError> {
        Self::builder().build()
    }

    /// Create a new client builder.
    pub fn builder() -> FuturesRestClientBuilder {
        FuturesRestClientBuilder::new()
    }

    // HTTP request methods.

    /// Build the full URL for an endpoint.
    ///
    /// Chart and history endpoints are served from the domain root instead of
    /// the `/derivatives` prefix, so the prefix is stripped from the base URL
    /// for those paths.
    fn endpoint_url(&self, endpoint: &str) -> String {
        // Tolerate a trailing slash on custom base URLs.
        let base = self.base_url.trim_end_matches('/');
        let base = if endpoint.starts_with("/api/charts") || endpoint.starts_with("/api/history") {
            base.strip_suffix("/derivatives").unwrap_or(base)
        } else {
            base
        };
        format!("{}{}", base, endpoint)
    }

    /// Make a public GET request.
    pub(crate) async fn public_get<T>(&self, endpoint: &str) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = self.endpoint_url(endpoint);
        let response = self.http_client.get(&url).send().await?;
        self.parse_futures_response(response).await
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
            self.endpoint_url(endpoint)
        } else {
            format!("{}?{}", self.endpoint_url(endpoint), query_string)
        };
        let response = self.http_client.get(&url).send().await?;
        self.parse_futures_response(response).await
    }

    /// Make an authenticated GET request.
    pub(crate) async fn private_get<T>(&self, endpoint: &str) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
    {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(KrakenError::MissingCredentials)?;

        let nonce = self.nonce_provider.next_nonce();
        let creds = credentials.get_credentials();

        let signature = sign_futures_request(creds, endpoint, nonce, "")?;

        let url = self.endpoint_url(endpoint);
        let response = self
            .http_client
            .get(&url)
            .header("APIKey", &creds.api_key)
            .header("Authent", signature)
            .header("Nonce", nonce.to_string())
            .send()
            .await?;

        self.parse_futures_response(response).await
    }

    /// Make an authenticated GET request with query parameters.
    ///
    /// The query string takes the place of the POST body in the signature.
    pub(crate) async fn private_get_with_params<T, Q>(
        &self,
        endpoint: &str,
        params: &Q,
    ) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
        Q: serde::Serialize + ?Sized,
    {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(KrakenError::MissingCredentials)?;

        let nonce = self.nonce_provider.next_nonce();
        let creds = credentials.get_credentials();

        let query_string = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        let signature = sign_futures_request(creds, endpoint, nonce, &query_string)?;

        let url = if query_string.is_empty() {
            self.endpoint_url(endpoint)
        } else {
            format!("{}?{}", self.endpoint_url(endpoint), query_string)
        };
        let response = self
            .http_client
            .get(&url)
            .header("APIKey", &creds.api_key)
            .header("Authent", signature)
            .header("Nonce", nonce.to_string())
            .send()
            .await?;

        self.parse_futures_response(response).await
    }

    /// Make an authenticated GET request that returns the raw response body.
    ///
    /// A JSON error body is still detected and returned as an error.
    pub(crate) async fn private_get_bytes(&self, endpoint: &str) -> Result<Vec<u8>, KrakenError> {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(KrakenError::MissingCredentials)?;

        let nonce = self.nonce_provider.next_nonce();
        let creds = credentials.get_credentials();

        let signature = sign_futures_request(creds, endpoint, nonce, "")?;

        let url = self.endpoint_url(endpoint);
        let response = self
            .http_client
            .get(&url)
            .header("APIKey", &creds.api_key)
            .header("Authent", signature)
            .header("Nonce", nonce.to_string())
            .send()
            .await?;

        let status = response.status();
        let bytes = response.bytes().await?.to_vec();

        if let Ok(error_response) = serde_json::from_slice::<FuturesErrorResponse>(&bytes) {
            if error_response.result == "error" {
                return Err(KrakenError::Api(crate::error::ApiError::new(
                    "EFutures",
                    error_response
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string()),
                )));
            }
        }

        if !status.is_success() {
            return Err(KrakenError::InvalidResponse(format!(
                "HTTP {}: {}",
                status,
                String::from_utf8_lossy(&bytes)
            )));
        }

        Ok(bytes)
    }

    /// Make an authenticated PUT request.
    ///
    /// The Futures API expects PUT parameters in the query string.
    /// The encoded parameters take the place of the POST body in the signature.
    pub(crate) async fn private_put<T, P>(
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

        let query_string = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        let signature = sign_futures_request(creds, endpoint, nonce, &query_string)?;

        let url = if query_string.is_empty() {
            self.endpoint_url(endpoint)
        } else {
            format!("{}?{}", self.endpoint_url(endpoint), query_string)
        };
        let response = self
            .http_client
            .put(&url)
            .header("APIKey", &creds.api_key)
            .header("Authent", signature)
            .header("Nonce", nonce.to_string())
            .send()
            .await?;

        self.parse_futures_response(response).await
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

        let form_data = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        let signature = sign_futures_request(creds, endpoint, nonce, &form_data)?;

        let url = self.endpoint_url(endpoint);
        let response = self
            .http_client
            .post(&url)
            .header("APIKey", &creds.api_key)
            .header("Authent", signature)
            .header("Nonce", nonce.to_string())
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(form_data)
            .send()
            .await?;

        self.parse_futures_response(response).await
    }

    /// Parse a response from the Kraken Futures API.
    ///
    /// Futures API has a different response format than Spot:
    /// - Success: `{ "result": "success", ... }`
    /// - Error: `{ "result": "error", "error": "...", "serverTime": "..." }`
    async fn parse_futures_response<T>(&self, response: reqwest::Response) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let body = response.text().await?;

        if let Ok(error_response) = serde_json::from_str::<FuturesErrorResponse>(&body) {
            if error_response.result == "error" {
                return Err(KrakenError::Api(crate::error::ApiError::new(
                    "EFutures",
                    error_response
                        .error
                        .unwrap_or_else(|| "Unknown error".to_string()),
                )));
            }
        }

        serde_json::from_str::<T>(&body).map_err(|e| {
            if !status.is_success() {
                KrakenError::InvalidResponse(format!("HTTP {}: {}", status, body))
            } else {
                KrakenError::InvalidResponse(format!(
                    "Failed to parse response: {}. Body: {}",
                    e, body
                ))
            }
        })
    }

    // Public endpoints.

    /// Get ticker data for all available futures contracts.
    pub async fn get_tickers(&self) -> Result<Vec<FuturesTicker>, KrakenError> {
        let response: TickersResponse = self.public_get(public::TICKERS).await?;
        Ok(response.tickers)
    }

    /// Get ticker data for a specific symbol, or `None` if not found.
    pub async fn get_ticker(&self, symbol: &str) -> Result<Option<FuturesTicker>, KrakenError> {
        let tickers = self.get_tickers().await?;
        Ok(tickers.into_iter().find(|t| t.symbol == symbol))
    }

    /// Get the order book for a symbol.
    pub async fn get_orderbook(&self, symbol: &str) -> Result<FuturesOrderBook, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            symbol: &'a str,
        }
        let response: OrderBookResponse = self
            .public_get_with_params(public::ORDERBOOK, &Params { symbol })
            .await?;
        Ok(response.order_book)
    }

    /// Get recent trade history for a symbol.
    ///
    /// `last_time` limits the result to trades before that timestamp.
    pub async fn get_trade_history(
        &self,
        symbol: &str,
        last_time: Option<&str>,
    ) -> Result<Vec<FuturesTrade>, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            symbol: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "lastTime")]
            last_time: Option<&'a str>,
        }
        let response: TradeHistoryResponse = self
            .public_get_with_params(public::HISTORY, &Params { symbol, last_time })
            .await?;
        Ok(response.history)
    }

    /// Get information about all available futures contracts.
    pub async fn get_instruments(&self) -> Result<Vec<FuturesInstrument>, KrakenError> {
        let response: InstrumentsResponse = self.public_get(public::INSTRUMENTS).await?;
        Ok(response.instruments)
    }

    /// Get dislocation and volatility status for all instruments.
    pub async fn get_instruments_status(&self) -> Result<Vec<InstrumentStatus>, KrakenError> {
        let response: InstrumentsStatusResponse =
            self.public_get(public::INSTRUMENTS_STATUS).await?;
        Ok(response.instrument_status)
    }

    /// Get dislocation and volatility status for a single instrument.
    pub async fn get_instrument_status(
        &self,
        symbol: &str,
    ) -> Result<InstrumentStatus, KrakenError> {
        let endpoint = format!("/api/v3/instruments/{}/status", symbol);
        self.public_get(&endpoint).await
    }

    /// Get all fee schedules with their tiers.
    pub async fn get_fee_schedules(&self) -> Result<Vec<FeeSchedule>, KrakenError> {
        let response: FeeSchedulesResponse = self.public_get(public::FEE_SCHEDULES).await?;
        Ok(response.fee_schedules)
    }

    /// Get historical funding rates for a symbol.
    pub async fn get_historical_funding_rates(
        &self,
        symbol: &str,
    ) -> Result<Vec<FundingRateEntry>, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            symbol: &'a str,
        }
        let response: HistoricalFundingRatesResponse = self
            .public_get_with_params(public::HISTORICAL_FUNDING_RATES, &Params { symbol })
            .await?;
        Ok(response.rates)
    }

    /// Get OHLC candles for a symbol.
    ///
    /// `resolution` is a candle interval such as "1m", "1h", or "1d".
    /// `from` and `to` are epoch seconds, with `to` inclusive.
    pub async fn get_ohlc(
        &self,
        tick_type: TickType,
        symbol: &str,
        resolution: &str,
        from: Option<u64>,
        to: Option<u64>,
    ) -> Result<OhlcResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params {
            #[serde(skip_serializing_if = "Option::is_none")]
            from: Option<u64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            to: Option<u64>,
        }
        let endpoint = format!(
            "{}/{}/{}/{}",
            public::CHARTS,
            tick_type.as_str(),
            symbol,
            resolution
        );
        self.public_get_with_params(&endpoint, &Params { from, to })
            .await
    }

    // Private endpoints: account.

    /// Get balances, margin info, and PnL for all accounts.
    pub async fn get_accounts(&self) -> Result<AccountsResponse, KrakenError> {
        self.private_get(private::ACCOUNTS).await
    }

    /// Get all open futures positions.
    pub async fn get_open_positions(&self) -> Result<Vec<FuturesPosition>, KrakenError> {
        let response: OpenPositionsResponse = self.private_get(private::OPEN_POSITIONS).await?;
        Ok(response.open_positions)
    }

    /// Get all open (unfilled) orders.
    pub async fn get_open_orders(&self) -> Result<Vec<FuturesOrder>, KrakenError> {
        let response: OpenOrdersResponse = self.private_get(private::OPEN_ORDERS).await?;
        Ok(response.open_orders)
    }

    /// Get fills (trade history) for all futures contracts or a specific symbol.
    pub async fn get_fills(
        &self,
        request: Option<&FillsRequest>,
    ) -> Result<Vec<FuturesFill>, KrakenError> {
        let response: FillsResponse = match request {
            Some(req) => self.public_get_with_params(private::FILLS, req).await?,
            None => self.private_get(private::FILLS).await?,
        };
        Ok(response.fills)
    }

    // Private endpoints: trading.

    /// Send a new order.
    pub async fn send_order(
        &self,
        request: &SendOrderRequest,
    ) -> Result<SendOrderResponse, KrakenError> {
        self.private_post(private::SEND_ORDER, request).await
    }

    /// Edit an existing order.
    pub async fn edit_order(
        &self,
        request: &EditOrderRequest,
    ) -> Result<EditOrderResponse, KrakenError> {
        self.private_post(private::EDIT_ORDER, request).await
    }

    /// Cancel an order by order ID.
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            order_id: &'a str,
        }
        self.private_post(private::CANCEL_ORDER, &Params { order_id })
            .await
    }

    /// Cancel an order by client order ID.
    pub async fn cancel_order_by_cli_ord_id(
        &self,
        cli_ord_id: &str,
    ) -> Result<CancelOrderResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            #[serde(rename = "cliOrdId")]
            cli_ord_id: &'a str,
        }
        self.private_post(private::CANCEL_ORDER, &Params { cli_ord_id })
            .await
    }

    /// Cancel all open orders for the account.
    pub async fn cancel_all_orders(&self) -> Result<CancelAllOrdersResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Empty {}
        self.private_post(private::CANCEL_ALL_ORDERS, &Empty {})
            .await
    }

    /// Cancel all open orders for a specific symbol.
    pub async fn cancel_all_orders_for_symbol(
        &self,
        symbol: &str,
    ) -> Result<CancelAllOrdersResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            symbol: &'a str,
        }
        self.private_post(private::CANCEL_ALL_ORDERS, &Params { symbol })
            .await
    }

    /// Set the dead man's switch to cancel all orders after a timeout.
    ///
    /// A timeout of 0 disables the switch.
    pub async fn cancel_all_orders_after(
        &self,
        timeout_seconds: u32,
    ) -> Result<CancelAllOrdersAfterResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params {
            timeout: u32,
        }
        self.private_post(
            private::CANCEL_ALL_ORDERS_AFTER,
            &Params {
                timeout: timeout_seconds,
            },
        )
        .await
    }

    /// Place, edit, and cancel multiple orders in a single request.
    pub async fn batch_order(
        &self,
        request: &BatchOrderRequest,
    ) -> Result<BatchOrderResponse, KrakenError> {
        self.private_post(private::BATCH_ORDER, request).await
    }

    /// Get the status of orders by order ID.
    pub async fn get_orders_status(
        &self,
        order_ids: &[&str],
    ) -> Result<OrdersStatusResponse, KrakenError> {
        let params: Vec<(&str, &str)> = order_ids.iter().map(|id| ("orderIds", *id)).collect();
        self.private_post(private::ORDERS_STATUS, &params).await
    }

    /// Get the status of orders by client order ID.
    pub async fn get_orders_status_by_cli_ord_ids(
        &self,
        cli_ord_ids: &[&str],
    ) -> Result<OrdersStatusResponse, KrakenError> {
        let params: Vec<(&str, &str)> = cli_ord_ids.iter().map(|id| ("cliOrdIds", *id)).collect();
        self.private_post(private::ORDERS_STATUS, &params).await
    }

    // Private endpoints: transfers and withdrawals.

    /// Transfer funds between margin accounts.
    pub async fn transfer(
        &self,
        request: &TransferRequest,
    ) -> Result<FuturesResultResponse, KrakenError> {
        self.private_post(private::TRANSFER, request).await
    }

    /// Transfer funds between the main account and a subaccount.
    pub async fn sub_account_transfer(
        &self,
        request: &SubAccountTransferRequest,
    ) -> Result<FuturesResultResponse, KrakenError> {
        self.private_post(private::TRANSFER_SUBACCOUNT, request)
            .await
    }

    /// Withdraw funds from the futures wallet to the spot wallet.
    ///
    /// This endpoint is not available in the demo environment.
    pub async fn withdrawal_to_spot_wallet(
        &self,
        request: &WithdrawalRequest,
    ) -> Result<FuturesResultResponse, KrakenError> {
        self.private_post(private::WITHDRAWAL, request).await
    }

    // Private endpoints: history and settings.

    /// Get account log entries.
    ///
    /// This endpoint is not available in the demo environment.
    pub async fn get_account_log(
        &self,
        request: Option<&AccountLogRequest>,
    ) -> Result<AccountLogResponse, KrakenError> {
        match request {
            Some(req) => {
                self.private_get_with_params(private::ACCOUNT_LOG, req)
                    .await
            }
            None => self.private_get(private::ACCOUNT_LOG).await,
        }
    }

    /// Get the account log as raw CSV bytes.
    ///
    /// This endpoint is not available in the demo environment.
    pub async fn get_account_log_csv(&self) -> Result<Vec<u8>, KrakenError> {
        self.private_get_bytes(private::ACCOUNT_LOG_CSV).await
    }

    /// Get historical order events.
    pub async fn get_order_events(
        &self,
        request: Option<&HistoryEventsRequest>,
    ) -> Result<HistoryEventsResponse, KrakenError> {
        self.get_history_events(private::HISTORY_ORDERS, request)
            .await
    }

    /// Get historical execution events.
    pub async fn get_execution_events(
        &self,
        request: Option<&HistoryEventsRequest>,
    ) -> Result<HistoryEventsResponse, KrakenError> {
        self.get_history_events(private::HISTORY_EXECUTIONS, request)
            .await
    }

    /// Get historical trigger events.
    pub async fn get_trigger_events(
        &self,
        request: Option<&HistoryEventsRequest>,
    ) -> Result<HistoryEventsResponse, KrakenError> {
        self.get_history_events(private::HISTORY_TRIGGERS, request)
            .await
    }

    async fn get_history_events(
        &self,
        endpoint: &str,
        request: Option<&HistoryEventsRequest>,
    ) -> Result<HistoryEventsResponse, KrakenError> {
        match request {
            Some(req) => self.private_get_with_params(endpoint, req).await,
            None => self.private_get(endpoint).await,
        }
    }

    /// List sub-accounts of the master account.
    pub async fn get_subaccounts(&self) -> Result<SubaccountsResponse, KrakenError> {
        self.private_get(private::SUBACCOUNTS).await
    }

    /// Get the latest notifications.
    pub async fn get_notifications(&self) -> Result<Vec<FuturesNotification>, KrakenError> {
        let response: NotificationsResponse = self.private_get(private::NOTIFICATIONS).await?;
        Ok(response.notifications)
    }

    /// Get personal 30 day USD volumes keyed by fee schedule UID.
    pub async fn get_fee_schedule_volumes(
        &self,
    ) -> Result<FeeScheduleVolumesResponse, KrakenError> {
        self.private_get(private::FEE_SCHEDULE_VOLUMES).await
    }

    /// Get the percentile of open interest in the unwind queue.
    pub async fn get_unwind_queue(&self) -> Result<Vec<UnwindQueueEntry>, KrakenError> {
        let response: UnwindQueueResponse = self.private_get(private::UNWIND_QUEUE).await?;
        Ok(response.queue)
    }

    /// Get the maximum leverage preference per symbol.
    pub async fn get_leverage_preferences(&self) -> Result<Vec<LeveragePreference>, KrakenError> {
        let response: LeveragePreferencesResponse =
            self.private_get(private::LEVERAGE_PREFERENCES).await?;
        Ok(response.leverage_preferences)
    }

    /// Set the leverage preference for a symbol.
    ///
    /// A `max_leverage` of `None` resets the preference to the default.
    pub async fn set_leverage_preference(
        &self,
        symbol: &str,
        max_leverage: Option<rust_decimal::Decimal>,
    ) -> Result<FuturesResultResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            symbol: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(rename = "maxLeverage")]
            max_leverage: Option<rust_decimal::Decimal>,
        }
        self.private_put(
            private::LEVERAGE_PREFERENCES,
            &Params {
                symbol,
                max_leverage,
            },
        )
        .await
    }

    /// Get PnL currency preferences.
    pub async fn get_pnl_preferences(&self) -> Result<Vec<PnlPreference>, KrakenError> {
        let response: PnlPreferencesResponse = self.private_get(private::PNL_PREFERENCES).await?;
        Ok(response.preferences)
    }

    /// Set the currency in which profits and losses are realized for a symbol.
    pub async fn set_pnl_preference(
        &self,
        symbol: &str,
        pnl_preference: &str,
    ) -> Result<FuturesResultResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            symbol: &'a str,
            #[serde(rename = "pnlPreference")]
            pnl_preference: &'a str,
        }
        self.private_put(
            private::PNL_PREFERENCES,
            &Params {
                symbol,
                pnl_preference,
            },
        )
        .await
    }
}

impl std::fmt::Debug for FuturesRestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuturesRestClient")
            .field("base_url", &self.base_url)
            .field("has_credentials", &self.credentials.is_some())
            .finish()
    }
}

/// Builder for [`FuturesRestClient`].
pub struct FuturesRestClientBuilder {
    base_url: String,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    nonce_provider: Option<Arc<dyn NonceProvider>>,
    user_agent: Option<String>,
    max_retries: u32,
    allow_insecure_transport: bool,
}

impl FuturesRestClientBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            base_url: FUTURES_BASE_URL.to_string(),
            credentials: None,
            nonce_provider: None,
            user_agent: None,
            max_retries: 3,
            allow_insecure_transport: false,
        }
    }

    /// Set the base URL.
    ///
    /// Chart and history endpoints are routed to the domain root by stripping
    /// a `/derivatives` suffix from this URL.
    /// A base URL without that suffix serves all endpoints from one root.
    /// Authenticated clients require HTTPS unless
    /// [`Self::danger_allow_insecure_transport`] is set for a local test server.
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Use the demo/testnet environment.
    pub fn use_demo(mut self) -> Self {
        self.base_url = crate::futures::rest::endpoints::FUTURES_DEMO_URL.to_string();
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
    pub fn build(self) -> Result<FuturesRestClient, KrakenError> {
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

        Ok(FuturesRestClient {
            http_client: client,
            base_url: self.base_url,
            credentials: self.credentials,
            nonce_provider,
        })
    }
}

impl Default for FuturesRestClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Internal error response from Futures API.
#[derive(Debug, serde::Deserialize)]
struct FuturesErrorResponse {
    result: String,
    error: Option<String>,
    #[serde(rename = "serverTime")]
    #[allow(dead_code)]
    server_time: Option<String>,
}

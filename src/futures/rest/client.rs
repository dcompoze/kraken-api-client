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
/// This client provides access to all Kraken Futures trading REST endpoints.
/// It handles authentication, rate limiting, and automatic retries.
///
/// # Example
///
/// ```rust,no_run
/// use kraken_api_client::futures::rest::FuturesRestClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Create a client for public endpoints only
///     let client = FuturesRestClient::new();
///
///     // Get all tickers
///     let tickers = client.get_tickers().await?;
///     for ticker in &tickers {
///         println!("{}: {}", ticker.symbol, ticker.last);
///     }
///
///     Ok(())
/// }
/// ```
///
/// For private endpoints, provide credentials:
///
/// ```rust,no_run
/// use kraken_api_client::futures::rest::FuturesRestClient;
/// use kraken_api_client::auth::StaticCredentials;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let credentials = Arc::new(StaticCredentials::new("api_key", "api_secret"));
///     let client = FuturesRestClient::builder()
///         .credentials(credentials)
///         .build();
///
///     let accounts = client.get_accounts().await?;
///     println!("Accounts: {:?}", accounts);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct FuturesRestClient {
    http_client: ClientWithMiddleware,
    base_url: String,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    nonce_provider: Arc<dyn NonceProvider>,
}

impl FuturesRestClient {
    /// Create a new client with default settings.
    ///
    /// This client can only access public endpoints.
    /// Use [`FuturesRestClient::builder()`] to configure credentials for private endpoints.
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Create a new client builder.
    pub fn builder() -> FuturesRestClientBuilder {
        FuturesRestClientBuilder::new()
    }

    // HTTP request methods.

    /// Make a public GET request.
    pub(crate) async fn public_get<T>(&self, endpoint: &str) -> Result<T, KrakenError>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.base_url, endpoint);
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
            format!("{}{}", self.base_url, endpoint)
        } else {
            format!("{}{}?{}", self.base_url, endpoint, query_string)
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

        // Sign the request (empty post_data for GET).
        let signature = sign_futures_request(creds, endpoint, nonce, "")?;

        let url = format!("{}{}", self.base_url, endpoint);
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

        // Build the POST body.
        let form_data = serde_urlencoded::to_string(params)
            .map_err(|e| KrakenError::InvalidResponse(e.to_string()))?;

        // Sign the request using the Futures algorithm.
        let signature = sign_futures_request(creds, endpoint, nonce, &form_data)?;

        let url = format!("{}{}", self.base_url, endpoint);
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

        // First check if it is an error response.
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

        // Try to parse as success response.
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

    /// Get all tickers.
    ///
    /// Returns ticker data for all available futures contracts.
    pub async fn get_tickers(&self) -> Result<Vec<FuturesTicker>, KrakenError> {
        let response: TickersResponse = self.public_get(public::TICKERS).await?;
        Ok(response.tickers)
    }

    /// Get ticker for a specific symbol.
    ///
    /// Returns ticker data for the specified symbol, or None if not found.
    pub async fn get_ticker(&self, symbol: &str) -> Result<Option<FuturesTicker>, KrakenError> {
        let tickers = self.get_tickers().await?;
        Ok(tickers.into_iter().find(|t| t.symbol == symbol))
    }

    /// Get order book for a symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The futures symbol (e.g., "PI_XBTUSD")
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
    /// # Arguments
    ///
    /// * `symbol` - The futures symbol (e.g., "PI_XBTUSD")
    /// * `last_time` - Optional timestamp to get trades before
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

    /// Get available instruments.
    ///
    /// Returns information about all tradeable futures contracts.
    pub async fn get_instruments(&self) -> Result<Vec<FuturesInstrument>, KrakenError> {
        let response: InstrumentsResponse = self.public_get(public::INSTRUMENTS).await?;
        Ok(response.instruments)
    }

    // Private endpoints: account.

    /// Get account information.
    ///
    /// Returns balances, margin info, and PnL for all accounts.
    pub async fn get_accounts(&self) -> Result<AccountsResponse, KrakenError> {
        self.private_get(private::ACCOUNTS).await
    }

    /// Get open positions.
    ///
    /// Returns all open futures positions.
    pub async fn get_open_positions(&self) -> Result<Vec<FuturesPosition>, KrakenError> {
        let response: OpenPositionsResponse = self.private_get(private::OPEN_POSITIONS).await?;
        Ok(response.open_positions)
    }

    /// Get open orders.
    ///
    /// Returns all open (unfilled) orders.
    pub async fn get_open_orders(&self) -> Result<Vec<FuturesOrder>, KrakenError> {
        let response: OpenOrdersResponse = self.private_get(private::OPEN_ORDERS).await?;
        Ok(response.open_orders)
    }

    /// Get fills (trade history).
    ///
    /// Returns fills for all futures contracts or a specific symbol.
    ///
    /// # Arguments
    ///
    /// * `request` - Optional request parameters
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
    ///
    /// # Arguments
    ///
    /// * `request` - Order parameters
    pub async fn send_order(
        &self,
        request: &SendOrderRequest,
    ) -> Result<SendOrderResponse, KrakenError> {
        self.private_post(private::SEND_ORDER, request).await
    }

    /// Edit an existing order.
    ///
    /// # Arguments
    ///
    /// * `request` - Edit parameters
    pub async fn edit_order(
        &self,
        request: &EditOrderRequest,
    ) -> Result<EditOrderResponse, KrakenError> {
        self.private_post(private::EDIT_ORDER, request).await
    }

    /// Cancel an order.
    ///
    /// # Arguments
    ///
    /// * `order_id` - The order ID to cancel
    pub async fn cancel_order(&self, order_id: &str) -> Result<CancelOrderResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            order_id: &'a str,
        }
        self.private_post(private::CANCEL_ORDER, &Params { order_id })
            .await
    }

    /// Cancel an order by client order ID.
    ///
    /// # Arguments
    ///
    /// * `cli_ord_id` - The client order ID to cancel
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

    /// Cancel all orders.
    ///
    /// Cancels all open orders for the account.
    pub async fn cancel_all_orders(&self) -> Result<CancelAllOrdersResponse, KrakenError> {
        #[derive(serde::Serialize)]
        struct Empty {}
        self.private_post(private::CANCEL_ALL_ORDERS, &Empty {})
            .await
    }

    /// Cancel all orders for a specific symbol.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The futures symbol to cancel orders for
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

    /// Set dead man's switch (cancel all orders after timeout).
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - Timeout in seconds (0 to disable)
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

    /// Send batch orders.
    ///
    /// Allows placing, editing, and cancelling multiple orders in a single request.
    ///
    /// # Arguments
    ///
    /// * `request` - Batch order request
    pub async fn batch_order(
        &self,
        request: &BatchOrderRequest,
    ) -> Result<BatchOrderResponse, KrakenError> {
        self.private_post(private::BATCH_ORDER, request).await
    }
}

impl Default for FuturesRestClient {
    fn default() -> Self {
        Self::new()
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
        }
    }

    /// Set the base URL (useful for testing with a mock server).
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

    /// Build the client.
    pub fn build(self) -> FuturesRestClient {
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

        FuturesRestClient {
            http_client: client,
            base_url: self.base_url,
            credentials: self.credentials,
            nonce_provider,
        }
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

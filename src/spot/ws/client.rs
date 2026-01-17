//! WebSocket client implementation.

use std::time::Duration;

use crate::error::KrakenError;
use crate::spot::ws::stream::KrakenStream;

/// WebSocket endpoint URLs.
pub mod endpoints {
    /// Public WebSocket endpoint.
    pub const WS_PUBLIC: &str = "wss://ws.kraken.com/v2";
    /// Private (authenticated) WebSocket endpoint.
    pub const WS_AUTH: &str = "wss://ws-auth.kraken.com/v2";
}

/// Configuration for WebSocket connections.
#[derive(Debug, Clone)]
pub struct WsConfig {
    /// Initial backoff duration for reconnection.
    pub initial_backoff: Duration,
    /// Maximum backoff duration for reconnection.
    pub max_backoff: Duration,
    /// Maximum number of reconnection attempts (None = infinite).
    pub max_reconnect_attempts: Option<u32>,
    /// Ping interval for connection health checks.
    pub ping_interval: Duration,
    /// Pong timeout - disconnect if no pong received.
    pub pong_timeout: Duration,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            max_reconnect_attempts: None, // Infinite
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
        }
    }
}

impl WsConfig {
    /// Create a new configuration builder.
    pub fn builder() -> WsConfigBuilder {
        WsConfigBuilder::new()
    }
}

/// Builder for [`WsConfig`].
#[derive(Debug, Clone, Default)]
pub struct WsConfigBuilder {
    config: WsConfig,
}

impl WsConfigBuilder {
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            config: WsConfig::default(),
        }
    }

    /// Set the reconnection backoff parameters.
    pub fn reconnect_backoff(mut self, initial: Duration, max: Duration) -> Self {
        self.config.initial_backoff = initial;
        self.config.max_backoff = max;
        self
    }

    /// Set maximum reconnection attempts.
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.config.max_reconnect_attempts = Some(attempts);
        self
    }

    /// Set ping interval.
    pub fn ping_interval(mut self, interval: Duration) -> Self {
        self.config.ping_interval = interval;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> WsConfig {
        self.config
    }
}

/// Kraken Spot WebSocket client.
///
/// Provides methods to connect to public and private WebSocket channels
/// with automatic reconnection and subscription restoration.
#[derive(Debug, Clone)]
pub struct SpotWsClient {
    /// Public WebSocket URL.
    public_url: String,
    /// Private WebSocket URL.
    auth_url: String,
    /// Connection configuration.
    config: WsConfig,
}

impl SpotWsClient {
    /// Create a new WebSocket client with default settings.
    pub fn new() -> Self {
        Self::with_config(WsConfig::default())
    }

    /// Create a new WebSocket client with custom configuration.
    pub fn with_config(config: WsConfig) -> Self {
        Self {
            public_url: endpoints::WS_PUBLIC.to_string(),
            auth_url: endpoints::WS_AUTH.to_string(),
            config,
        }
    }

    /// Create a client with custom URLs (useful for testing).
    pub fn with_urls(public_url: impl Into<String>, auth_url: impl Into<String>) -> Self {
        Self {
            public_url: public_url.into(),
            auth_url: auth_url.into(),
            config: WsConfig::default(),
        }
    }

    /// Get the public WebSocket URL.
    pub fn public_url(&self) -> &str {
        &self.public_url
    }

    /// Get the private WebSocket URL.
    pub fn auth_url(&self) -> &str {
        &self.auth_url
    }

    /// Get the configuration.
    pub fn config(&self) -> &WsConfig {
        &self.config
    }

    /// Connect to the public WebSocket endpoint.
    ///
    /// Returns a stream of market data messages.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::spot::ws::SpotWsClient;
    /// use kraken_api_client::spot::ws::messages::{SubscribeParams, channels};
    /// use futures_util::StreamExt;
    ///
    /// let client = SpotWsClient::new();
    /// let mut stream = client.connect_public().await?;
    ///
    /// // Subscribe to ticker updates
    /// stream.subscribe(SubscribeParams::public(channels::TICKER, vec!["BTC/USD".into()])).await?;
    ///
    /// while let Some(msg) = stream.next().await {
    ///     println!("Message: {:?}", msg);
    /// }
    /// ```
    pub async fn connect_public(&self) -> Result<KrakenStream, KrakenError> {
        KrakenStream::connect_public(&self.public_url, self.config.clone()).await
    }

    /// Connect to the public WebSocket endpoint with custom configuration.
    pub async fn connect_public_with_config(
        &self,
        config: WsConfig,
    ) -> Result<KrakenStream, KrakenError> {
        KrakenStream::connect_public(&self.public_url, config).await
    }

    /// Connect to the private (authenticated) WebSocket endpoint.
    ///
    /// Requires a valid WebSocket token obtained from the REST API.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::spot::ws::SpotWsClient;
    /// use kraken_api_client::spot::rest::SpotRestClient;
    /// use kraken_api_client::auth::StaticCredentials;
    /// use kraken_api_client::spot::ws::messages::{SubscribeParams, channels};
    /// use futures_util::StreamExt;
    /// use std::sync::Arc;
    ///
    /// // First, get a WebSocket token from the REST API
    /// let credentials = Arc::new(StaticCredentials::new("api_key", "api_secret"));
    /// let rest_client = SpotRestClient::builder().credentials(credentials).build();
    /// let token_response = rest_client.get_websocket_token().await?;
    ///
    /// // Then connect to the private WebSocket
    /// let ws_client = SpotWsClient::new();
    /// let mut stream = ws_client.connect_private(token_response.token).await?;
    ///
    /// // Subscribe to execution updates
    /// stream.subscribe(SubscribeParams::private(channels::EXECUTIONS, &token_response.token)).await?;
    ///
    /// while let Some(msg) = stream.next().await {
    ///     println!("Message: {:?}", msg);
    /// }
    /// ```
    pub async fn connect_private(&self, token: impl Into<String>) -> Result<KrakenStream, KrakenError> {
        KrakenStream::connect_private(&self.auth_url, self.config.clone(), token.into()).await
    }

    /// Connect to the private WebSocket endpoint with custom configuration.
    pub async fn connect_private_with_config(
        &self,
        token: impl Into<String>,
        config: WsConfig,
    ) -> Result<KrakenStream, KrakenError> {
        KrakenStream::connect_private(&self.auth_url, config, token.into()).await
    }
}

impl Default for SpotWsClient {
    fn default() -> Self {
        Self::new()
    }
}

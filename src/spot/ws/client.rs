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
    /// Whether private connections may use plaintext WebSocket transport.
    pub danger_allow_insecure_transport: bool,
}

impl Default for WsConfig {
    fn default() -> Self {
        Self {
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            max_reconnect_attempts: None,
            ping_interval: Duration::from_secs(30),
            pong_timeout: Duration::from_secs(10),
            danger_allow_insecure_transport: false,
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

    /// Allow private connections over plaintext WebSocket transport.
    ///
    /// This option can expose authentication tokens. Use it only with a local test server.
    pub fn danger_allow_insecure_transport(mut self) -> Self {
        self.config.danger_allow_insecure_transport = true;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> WsConfig {
        self.config
    }
}

/// Kraken Spot WebSocket client.
///
/// Connects to public and private channels with automatic reconnection and subscription restoration.
#[derive(Debug, Clone)]
pub struct SpotWsClient {
    public_url: String,
    auth_url: String,
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

    /// Create a client with custom URLs.
    ///
    /// Private connections require WSS unless the configuration uses the explicit
    /// insecure transport option for a local test server.
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
    /// Requires a WebSocket token obtained via the `GetWebSocketsToken` REST endpoint.
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

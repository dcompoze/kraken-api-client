//! Futures WebSocket client implementation.

use std::sync::Arc;
use std::time::Duration;

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

use crate::auth::{Credentials, CredentialsProvider};
use crate::error::KrakenError;
use crate::futures::ws::endpoints;
use crate::futures::ws::stream::FuturesStream;

type HmacSha512 = Hmac<Sha512>;

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
            max_reconnect_attempts: None, // Infinite reconnect attempts.
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

    /// Set pong timeout.
    pub fn pong_timeout(mut self, timeout: Duration) -> Self {
        self.config.pong_timeout = timeout;
        self
    }

    /// Build the configuration.
    pub fn build(self) -> WsConfig {
        self.config
    }
}

/// Kraken Futures WebSocket client.
///
/// Provides methods to connect to public and private WebSocket feeds
/// with automatic reconnection and subscription restoration.
///
/// # Example
///
/// ```rust,ignore
/// use kraken_api_client::futures::ws::{FuturesWsClient, feeds};
/// use futures_util::StreamExt;
///
/// let client = FuturesWsClient::new();
/// let mut stream = client.connect_public().await?;
///
/// stream.subscribe_public(feeds::TICKER, vec!["PI_XBTUSD"]).await?;
///
/// while let Some(msg) = stream.next().await {
///     println!("{:?}", msg);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct FuturesWsClient {
    /// WebSocket URL.
    url: String,
    /// Connection configuration.
    config: WsConfig,
}

impl FuturesWsClient {
    /// Create a new WebSocket client with default settings.
    pub fn new() -> Self {
        Self::with_config(WsConfig::default())
    }

    /// Create a new WebSocket client with custom configuration.
    pub fn with_config(config: WsConfig) -> Self {
        Self {
            url: endpoints::WS_PUBLIC.to_string(),
            config,
        }
    }

    /// Create a client for the demo/testnet environment.
    pub fn demo() -> Self {
        Self {
            url: endpoints::WS_DEMO.to_string(),
            config: WsConfig::default(),
        }
    }

    /// Create a client with a custom URL (useful for testing).
    pub fn with_url(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            config: WsConfig::default(),
        }
    }

    /// Get the WebSocket URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the configuration.
    pub fn config(&self) -> &WsConfig {
        &self.config
    }

    /// Connect to the public WebSocket endpoint.
    ///
    /// Returns a stream that can subscribe to public feeds (ticker, book, trades).
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::futures::ws::{FuturesWsClient, feeds};
    /// use futures_util::StreamExt;
    ///
    /// let client = FuturesWsClient::new();
    /// let mut stream = client.connect_public().await?;
    ///
    /// stream.subscribe_public(feeds::BOOK, vec!["PI_XBTUSD"]).await?;
    ///
    /// while let Some(msg) = stream.next().await {
    ///     match msg? {
    ///         FuturesWsEvent::Book(book) => println!("Book: {:?}", book),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn connect_public(&self) -> Result<FuturesStream, KrakenError> {
        FuturesStream::connect_public(&self.url, self.config.clone()).await
    }

    /// Connect to the private WebSocket endpoint with authentication.
    ///
    /// This will perform the challenge/response authentication automatically.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::futures::ws::{FuturesWsClient, feeds};
    /// use kraken_api_client::auth::StaticCredentials;
    /// use futures_util::StreamExt;
    /// use std::sync::Arc;
    ///
    /// let credentials = Arc::new(StaticCredentials::new("api_key", "api_secret"));
    /// let client = FuturesWsClient::new();
    /// let mut stream = client.connect_private(credentials).await?;
    ///
    /// stream.subscribe_private(feeds::OPEN_ORDERS).await?;
    /// stream.subscribe_private(feeds::FILLS).await?;
    ///
    /// while let Some(msg) = stream.next().await {
    ///     match msg? {
    ///         FuturesWsEvent::Order(order) => println!("Order: {:?}", order),
    ///         FuturesWsEvent::Fill(fill) => println!("Fill: {:?}", fill),
    ///         _ => {}
    ///     }
    /// }
    /// ```
    pub async fn connect_private(
        &self,
        credentials: Arc<dyn CredentialsProvider>,
    ) -> Result<FuturesStream, KrakenError> {
        FuturesStream::connect_private(&self.url, self.config.clone(), credentials).await
    }

    /// Connect to the public WebSocket endpoint with custom configuration.
    pub async fn connect_public_with_config(
        &self,
        config: WsConfig,
    ) -> Result<FuturesStream, KrakenError> {
        FuturesStream::connect_public(&self.url, config).await
    }

    /// Connect to the private WebSocket endpoint with custom configuration.
    pub async fn connect_private_with_config(
        &self,
        credentials: Arc<dyn CredentialsProvider>,
        config: WsConfig,
    ) -> Result<FuturesStream, KrakenError> {
        FuturesStream::connect_private(&self.url, config, credentials).await
    }
}

impl Default for FuturesWsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Sign a WebSocket challenge for authentication.
///
/// The Futures WebSocket API uses challenge-based authentication:
/// 1. SHA-256 hash the challenge message
/// 2. HMAC-SHA-512 with the base64-decoded API secret
/// 3. Base64 encode the result
///
/// # Arguments
///
/// * `credentials` - API credentials containing the secret
/// * `challenge` - The challenge string received from the server
///
/// # Returns
///
/// Base64-encoded signed challenge.
pub fn sign_challenge(credentials: &Credentials, challenge: &str) -> Result<String, KrakenError> {
    // Decode the API secret from base64.
    let secret_decoded = BASE64
        .decode(credentials.expose_secret())
        .map_err(|_| KrakenError::Auth("API secret must be valid base64.".to_string()))?;

    // SHA-256 hash the challenge.
    let sha256_hash = Sha256::digest(challenge.as_bytes());

    // HMAC-SHA-512 with the decoded secret.
    let mut hmac = HmacSha512::new_from_slice(&secret_decoded)
        .map_err(|e| KrakenError::Auth(format!("Invalid HMAC key: {e}")))?;
    hmac.update(&sha256_hash);
    let hmac_result = hmac.finalize().into_bytes();

    // Base64 encode the result.
    Ok(BASE64.encode(hmac_result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_challenge() {
        // Test that challenge signing produces correct format
        let secret = BASE64.encode("test_secret_key");
        let credentials = Credentials::new("api_key", secret);

        let signed = sign_challenge(&credentials, "123e4567-e89b-12d3-a456-426614174000").unwrap();

        // Should be valid base64
        assert!(BASE64.decode(&signed).is_ok());
        // HMAC-SHA512 produces 64 bytes, base64 encoded = 88 chars
        assert_eq!(signed.len(), 88);
    }

    #[test]
    fn test_sign_challenge_consistency() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_challenge(&credentials, "test-challenge").unwrap();
        let sig2 = sign_challenge(&credentials, "test-challenge").unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_sign_challenge_different_challenges() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_challenge(&credentials, "challenge-1").unwrap();
        let sig2 = sign_challenge(&credentials, "challenge-2").unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_config_builder() {
        let config = WsConfig::builder()
            .reconnect_backoff(Duration::from_secs(2), Duration::from_secs(120))
            .max_reconnect_attempts(5)
            .ping_interval(Duration::from_secs(15))
            .pong_timeout(Duration::from_secs(5))
            .build();

        assert_eq!(config.initial_backoff, Duration::from_secs(2));
        assert_eq!(config.max_backoff, Duration::from_secs(120));
        assert_eq!(config.max_reconnect_attempts, Some(5));
        assert_eq!(config.ping_interval, Duration::from_secs(15));
        assert_eq!(config.pong_timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_client_urls() {
        let client = FuturesWsClient::new();
        assert_eq!(client.url(), "wss://futures.kraken.com/ws/v1");

        let demo = FuturesWsClient::demo();
        assert_eq!(demo.url(), "wss://demo-futures.kraken.com/ws/v1");

        let custom = FuturesWsClient::with_url("wss://custom.example.com");
        assert_eq!(custom.url(), "wss://custom.example.com");
    }
}

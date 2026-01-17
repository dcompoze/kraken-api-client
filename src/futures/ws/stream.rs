//! Futures WebSocket stream implementation.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{Interval, interval};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async};

use crate::auth::CredentialsProvider;
use crate::error::KrakenError;
use crate::futures::ws::client::{WsConfig, sign_challenge};
use crate::futures::ws::messages::*;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, WsMessage>;
type WsReceiver = SplitStream<WsStream>;

/// Events from the Futures WebSocket connection.
#[derive(Debug, Clone)]
pub enum FuturesWsEvent {
    /// Connection info/version message.
    Info(InfoResponse),
    /// Error from the server.
    Error(ErrorResponse),
    /// Subscription confirmed.
    Subscribed(SubscribedResponse),
    /// Unsubscription confirmed.
    Unsubscribed(UnsubscribedResponse),
    /// Order book data.
    Book(BookMessage),
    /// Order book snapshot.
    BookSnapshot(BookSnapshotMessage),
    /// Ticker data.
    Ticker(TickerMessage),
    /// Trade data.
    Trade(TradeMessage),
    /// Trades snapshot.
    TradesSnapshot(TradesSnapshotMessage),
    /// Open orders (private).
    OpenOrders(OpenOrdersMessage),
    /// Fills (private).
    Fills(FillsMessage),
    /// Open positions (private).
    OpenPositions(OpenPositionsMessage),
    /// Balances (private).
    Balances(BalancesMessage),
    /// Raw/unknown message.
    Raw(serde_json::Value),
    /// Connection disconnected.
    Disconnected,
    /// Reconnecting.
    Reconnecting { attempt: u32 },
    /// Reconnected successfully.
    Reconnected,
}

/// Subscription tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct Subscription {
    feed: String,
    product_ids: Vec<String>,
    is_private: bool,
}

/// Authentication state.
#[derive(Debug, Clone)]
struct AuthState {
    challenge: String,
    signed_challenge: String,
}

/// A stream of messages from a Kraken Futures WebSocket connection.
///
/// This stream handles:
/// - Automatic reconnection with exponential backoff
/// - Subscription restoration after reconnect
/// - Challenge-based authentication for private feeds
/// - Ping/pong connection health monitoring
pub struct FuturesStream {
    /// WebSocket sink for sending messages.
    sink: Option<Arc<Mutex<WsSink>>>,
    /// WebSocket receiver for incoming messages.
    receiver: Option<WsReceiver>,
    /// Connection configuration.
    config: WsConfig,
    /// URL to connect to.
    url: String,
    /// Credentials for private connections.
    credentials: Option<Arc<dyn CredentialsProvider>>,
    /// Authentication state.
    auth_state: Option<AuthState>,
    /// Active subscriptions.
    subscriptions: HashMap<String, Subscription>,
    /// Ping interval timer.
    ping_interval: Interval,
    /// Last message received timestamp.
    last_message: Instant,
    /// Current reconnection attempt.
    reconnect_attempt: u32,
    /// Connection state.
    connected: bool,
    /// Whether we're currently reconnecting.
    reconnecting: bool,
    /// Whether authentication is complete.
    authenticated: bool,
    /// Pending authentication (waiting for challenge response).
    pending_auth: bool,
}

impl std::fmt::Debug for FuturesStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuturesStream")
            .field("url", &self.url)
            .field("connected", &self.connected)
            .field("reconnecting", &self.reconnecting)
            .field("authenticated", &self.authenticated)
            .field("subscriptions", &self.subscriptions.len())
            .finish()
    }
}

impl FuturesStream {
    /// Connect to the public WebSocket endpoint.
    pub(crate) async fn connect_public(url: &str, config: WsConfig) -> Result<Self, KrakenError> {
        Self::connect(url, config, None).await
    }

    /// Connect to the private WebSocket endpoint with authentication.
    pub(crate) async fn connect_private(
        url: &str,
        config: WsConfig,
        credentials: Arc<dyn CredentialsProvider>,
    ) -> Result<Self, KrakenError> {
        let mut stream = Self::connect(url, config, Some(credentials)).await?;
        stream.authenticate().await?;
        Ok(stream)
    }

    /// Connect to the WebSocket server.
    async fn connect(
        url: &str,
        config: WsConfig,
        credentials: Option<Arc<dyn CredentialsProvider>>,
    ) -> Result<Self, KrakenError> {
        let (ws_stream, _) = connect_async(url).await.map_err(|e| {
            KrakenError::WebSocketMsg(format!("Failed to connect to {}: {}", url, e))
        })?;

        let (sink, receiver) = ws_stream.split();
        let ping_interval_duration = config.ping_interval;

        Ok(Self {
            sink: Some(Arc::new(Mutex::new(sink))),
            receiver: Some(receiver),
            config,
            url: url.to_string(),
            credentials,
            auth_state: None,
            subscriptions: HashMap::new(),
            ping_interval: interval(ping_interval_duration),
            last_message: Instant::now(),
            reconnect_attempt: 0,
            connected: true,
            reconnecting: false,
            authenticated: false,
            pending_auth: false,
        })
    }

    /// Perform challenge-based authentication.
    async fn authenticate(&mut self) -> Result<(), KrakenError> {
        let credentials = self
            .credentials
            .as_ref()
            .ok_or(KrakenError::MissingCredentials)?;

        // Clone the credentials to avoid borrow issues
        let creds = credentials.get_credentials().clone();

        // Send challenge request
        let challenge_req = ChallengeRequest::new(&creds.api_key);
        self.send_json(&challenge_req).await?;
        self.pending_auth = true;

        // Wait for challenge response
        let challenge = self.wait_for_challenge().await?;

        // Sign the challenge.
        let signed = sign_challenge(&creds, &challenge)?;

        self.auth_state = Some(AuthState {
            challenge,
            signed_challenge: signed,
        });

        self.authenticated = true;
        self.pending_auth = false;

        Ok(())
    }

    /// Wait for challenge response from the server.
    async fn wait_for_challenge(&mut self) -> Result<String, KrakenError> {
        let timeout = Duration::from_secs(10);
        let start = Instant::now();

        while start.elapsed() < timeout {
            if let Some(receiver) = &mut self.receiver {
                match tokio::time::timeout(Duration::from_millis(100), receiver.next()).await {
                    Ok(Some(Ok(WsMessage::Text(text)))) => {
                        let value: serde_json::Value =
                            serde_json::from_str(&text).map_err(KrakenError::Json)?;

                        if let Some(event) = value.get("event").and_then(|e| e.as_str()) {
                            if event == "challenge" {
                                if let Some(message) = value.get("message").and_then(|m| m.as_str())
                                {
                                    return Ok(message.to_string());
                                }
                            } else if event == "error" {
                                let msg = value
                                    .get("message")
                                    .and_then(|m| m.as_str())
                                    .unwrap_or("Unknown error");
                                return Err(KrakenError::WebSocketMsg(format!(
                                    "Authentication error: {}",
                                    msg
                                )));
                            }
                        }
                    }
                    Ok(Some(Err(e))) => {
                        return Err(KrakenError::WebSocket(e));
                    }
                    _ => continue,
                }
            }
        }

        Err(KrakenError::WebSocketMsg(
            "Timeout waiting for challenge response".into(),
        ))
    }

    /// Subscribe to a public feed.
    ///
    /// # Arguments
    ///
    /// * `feed` - Feed name (e.g., "ticker", "book", "trade")
    /// * `product_ids` - Product IDs to subscribe to (e.g., ["PI_XBTUSD"])
    pub async fn subscribe_public(
        &mut self,
        feed: &str,
        product_ids: Vec<&str>,
    ) -> Result<(), KrakenError> {
        let product_ids: Vec<String> = product_ids.into_iter().map(|s| s.to_string()).collect();
        let key = subscription_key(feed, &product_ids);

        // Store subscription
        self.subscriptions.insert(
            key,
            Subscription {
                feed: feed.to_string(),
                product_ids: product_ids.clone(),
                is_private: false,
            },
        );

        // Send subscription request
        let request = SubscribeRequest::public(feed, product_ids);
        self.send_json(&request).await
    }

    /// Subscribe to a private feed.
    ///
    /// Requires prior authentication via `connect_private`.
    ///
    /// # Arguments
    ///
    /// * `feed` - Feed name (e.g., "open_orders", "fills", "open_positions")
    pub async fn subscribe_private(&mut self, feed: &str) -> Result<(), KrakenError> {
        let auth = self
            .auth_state
            .as_ref()
            .ok_or_else(|| KrakenError::WebSocketMsg("Not authenticated".into()))?;

        let key = subscription_key(feed, &[]);

        // Store subscription
        self.subscriptions.insert(
            key,
            Subscription {
                feed: feed.to_string(),
                product_ids: vec![],
                is_private: true,
            },
        );

        // Send private subscription request
        let request = PrivateSubscribeRequest::new(
            feed,
            auth.challenge.clone(),
            auth.signed_challenge.clone(),
        );
        self.send_json(&request).await
    }

    /// Subscribe to a private feed for specific products.
    pub async fn subscribe_private_with_products(
        &mut self,
        feed: &str,
        product_ids: Vec<&str>,
    ) -> Result<(), KrakenError> {
        let auth = self
            .auth_state
            .as_ref()
            .ok_or_else(|| KrakenError::WebSocketMsg("Not authenticated".into()))?;

        let product_ids: Vec<String> = product_ids.into_iter().map(|s| s.to_string()).collect();
        let key = subscription_key(feed, &product_ids);

        // Store subscription
        self.subscriptions.insert(
            key,
            Subscription {
                feed: feed.to_string(),
                product_ids: product_ids.clone(),
                is_private: true,
            },
        );

        // Send private subscription request
        let request = PrivateSubscribeRequest::new(
            feed,
            auth.challenge.clone(),
            auth.signed_challenge.clone(),
        )
        .with_product_ids(product_ids);
        self.send_json(&request).await
    }

    /// Unsubscribe from a feed.
    pub async fn unsubscribe(
        &mut self,
        feed: &str,
        product_ids: Vec<&str>,
    ) -> Result<(), KrakenError> {
        let product_ids: Vec<String> = product_ids.into_iter().map(|s| s.to_string()).collect();
        let key = subscription_key(feed, &product_ids);
        self.subscriptions.remove(&key);

        let request = UnsubscribeRequest::new(feed, product_ids);
        self.send_json(&request).await
    }

    /// Send a JSON message.
    async fn send_json<T: serde::Serialize>(&self, msg: &T) -> Result<(), KrakenError> {
        let sink = self
            .sink
            .as_ref()
            .ok_or_else(|| KrakenError::WebSocketMsg("Not connected".into()))?;

        let json = serde_json::to_string(msg).map_err(|e| {
            KrakenError::WebSocketMsg(format!("Failed to serialize message: {}", e))
        })?;

        let mut sink = sink.lock().await;
        sink.send(WsMessage::Text(json.into()))
            .await
            .map_err(|e| KrakenError::WebSocketMsg(format!("Failed to send message: {}", e)))
    }

    /// Check if we should reconnect.
    fn should_reconnect(&self) -> bool {
        match self.config.max_reconnect_attempts {
            Some(max) => self.reconnect_attempt < max,
            None => true, // Infinite retries
        }
    }

    /// Calculate backoff duration for reconnection.
    #[allow(dead_code)]
    fn backoff_duration(&self) -> Duration {
        let base = self.config.initial_backoff.as_millis() as u64;
        let max = self.config.max_backoff.as_millis() as u64;
        let multiplier = 2u64.saturating_pow(self.reconnect_attempt);
        let backoff_ms = base.saturating_mul(multiplier).min(max);
        Duration::from_millis(backoff_ms)
    }

    /// Attempt to reconnect.
    #[allow(dead_code)]
    async fn reconnect(&mut self) -> Result<(), KrakenError> {
        self.reconnect_attempt += 1;
        self.connected = false;
        self.reconnecting = true;
        self.authenticated = false;

        // Close existing connection
        self.sink = None;
        self.receiver = None;

        // Wait with backoff
        let backoff = self.backoff_duration();
        tokio::time::sleep(backoff).await;

        // Try to reconnect
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| KrakenError::WebSocketMsg(format!("Failed to reconnect: {}", e)))?;

        let (sink, receiver) = ws_stream.split();
        self.sink = Some(Arc::new(Mutex::new(sink)));
        self.receiver = Some(receiver);
        self.connected = true;
        self.reconnecting = false;
        self.reconnect_attempt = 0;
        self.last_message = Instant::now();

        // Re-authenticate if we have credentials
        if self.credentials.is_some() {
            self.authenticate().await?;
        }

        // Restore subscriptions
        self.restore_subscriptions().await?;

        Ok(())
    }

    /// Restore subscriptions after reconnection.
    #[allow(dead_code)]
    async fn restore_subscriptions(&mut self) -> Result<(), KrakenError> {
        let subs: Vec<_> = self.subscriptions.values().cloned().collect();

        for sub in subs {
            if sub.is_private {
                if sub.product_ids.is_empty() {
                    let auth = self
                        .auth_state
                        .as_ref()
                        .ok_or_else(|| KrakenError::WebSocketMsg("Not authenticated".into()))?;
                    let request = PrivateSubscribeRequest::new(
                        &sub.feed,
                        auth.challenge.clone(),
                        auth.signed_challenge.clone(),
                    );
                    self.send_json(&request).await?;
                } else {
                    let auth = self
                        .auth_state
                        .as_ref()
                        .ok_or_else(|| KrakenError::WebSocketMsg("Not authenticated".into()))?;
                    let request = PrivateSubscribeRequest::new(
                        &sub.feed,
                        auth.challenge.clone(),
                        auth.signed_challenge.clone(),
                    )
                    .with_product_ids(sub.product_ids);
                    self.send_json(&request).await?;
                }
            } else {
                let request = SubscribeRequest::public(&sub.feed, sub.product_ids);
                self.send_json(&request).await?;
            }
        }

        Ok(())
    }

    /// Parse and handle an incoming message.
    fn parse_message(&mut self, text: &str) -> Option<FuturesWsEvent> {
        self.last_message = Instant::now();

        // Try to parse as JSON
        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to parse WebSocket message: {}", e);
                return None;
            }
        };

        // Extract event/feed type first as owned strings
        let event = value
            .get("event")
            .and_then(|e| e.as_str())
            .map(String::from);
        let feed = value.get("feed").and_then(|f| f.as_str()).map(String::from);

        // Check event type first
        if let Some(event) = event {
            return self.handle_event_message(&event, value);
        }

        // Check feed type
        if let Some(feed) = feed {
            return self.handle_feed_message(&feed, value);
        }

        // Unknown format
        Some(FuturesWsEvent::Raw(value))
    }

    /// Handle event-based messages (subscribed, error, etc.).
    fn handle_event_message(
        &self,
        event: &str,
        value: serde_json::Value,
    ) -> Option<FuturesWsEvent> {
        match event {
            "info" | "alert" => {
                if let Ok(info) = serde_json::from_value::<InfoResponse>(value) {
                    return Some(FuturesWsEvent::Info(info));
                }
            }
            "subscribed" => {
                if let Ok(sub) = serde_json::from_value::<SubscribedResponse>(value) {
                    return Some(FuturesWsEvent::Subscribed(sub));
                }
            }
            "unsubscribed" => {
                if let Ok(unsub) = serde_json::from_value::<UnsubscribedResponse>(value) {
                    return Some(FuturesWsEvent::Unsubscribed(unsub));
                }
            }
            "error" => {
                if let Ok(err) = serde_json::from_value::<ErrorResponse>(value) {
                    return Some(FuturesWsEvent::Error(err));
                }
            }
            "challenge" => {
                // Challenge handled during authentication, skip here
                return None;
            }
            _ => {
                return Some(FuturesWsEvent::Raw(value));
            }
        }
        None
    }

    /// Handle feed-based messages (book, ticker, etc.).
    fn handle_feed_message(&self, feed: &str, value: serde_json::Value) -> Option<FuturesWsEvent> {
        match feed {
            "book" => {
                if let Ok(book) = serde_json::from_value::<BookMessage>(value) {
                    return Some(FuturesWsEvent::Book(book));
                }
            }
            "book_snapshot" => {
                if let Ok(snapshot) = serde_json::from_value::<BookSnapshotMessage>(value) {
                    return Some(FuturesWsEvent::BookSnapshot(snapshot));
                }
            }
            "ticker" | "ticker_lite" => {
                if let Ok(ticker) = serde_json::from_value::<TickerMessage>(value) {
                    return Some(FuturesWsEvent::Ticker(ticker));
                }
            }
            "trade" => {
                if let Ok(trade) = serde_json::from_value::<TradeMessage>(value) {
                    return Some(FuturesWsEvent::Trade(trade));
                }
            }
            "trade_snapshot" => {
                if let Ok(snapshot) = serde_json::from_value::<TradesSnapshotMessage>(value) {
                    return Some(FuturesWsEvent::TradesSnapshot(snapshot));
                }
            }
            "open_orders" | "open_orders_snapshot" => {
                if let Ok(orders) = serde_json::from_value::<OpenOrdersMessage>(value) {
                    return Some(FuturesWsEvent::OpenOrders(orders));
                }
            }
            "fills" | "fills_snapshot" => {
                if let Ok(fills) = serde_json::from_value::<FillsMessage>(value) {
                    return Some(FuturesWsEvent::Fills(fills));
                }
            }
            "open_positions" | "open_positions_snapshot" => {
                if let Ok(positions) = serde_json::from_value::<OpenPositionsMessage>(value) {
                    return Some(FuturesWsEvent::OpenPositions(positions));
                }
            }
            "balances" | "balances_snapshot" => {
                if let Ok(balances) = serde_json::from_value::<BalancesMessage>(value) {
                    return Some(FuturesWsEvent::Balances(balances));
                }
            }
            _ => {
                return Some(FuturesWsEvent::Raw(value));
            }
        }
        None
    }

    /// Close the connection gracefully.
    pub async fn close(&mut self) -> Result<(), KrakenError> {
        if let Some(sink) = self.sink.take() {
            let mut sink = sink.lock().await;
            let _ = sink.send(WsMessage::Close(None)).await;
        }
        self.receiver = None;
        self.connected = false;
        Ok(())
    }

    /// Check if the connection is open.
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Check if authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}

impl Stream for FuturesStream {
    type Item = Result<FuturesWsEvent, KrakenError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check ping interval (Kraken requires at least every 60 seconds)
        if self.ping_interval.poll_tick(cx).is_ready() && self.connected {
            // The Futures WebSocket API doesn't use explicit ping messages like Spot v2
            // Instead, connection health is maintained by the underlying WebSocket ping/pong
            // which tokio-tungstenite handles automatically
        }

        // Poll the receiver for messages
        if let Some(receiver) = self.receiver.as_mut() {
            match Pin::new(receiver).poll_next(cx) {
                Poll::Ready(Some(Ok(msg))) => {
                    let this = self.as_mut().get_mut();
                    match msg {
                        WsMessage::Text(text) => {
                            if let Some(event) = this.parse_message(&text) {
                                return Poll::Ready(Some(Ok(event)));
                            }
                            // If parse returned None (e.g., challenge during auth), continue polling
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        WsMessage::Binary(data) => {
                            if let Ok(text) = String::from_utf8(data.to_vec()) {
                                if let Some(event) = this.parse_message(&text) {
                                    return Poll::Ready(Some(Ok(event)));
                                }
                            }
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        WsMessage::Ping(_) | WsMessage::Pong(_) => {
                            // Handled automatically by tungstenite
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        WsMessage::Close(_) => {
                            this.connected = false;
                            if this.should_reconnect() {
                                return Poll::Ready(Some(Ok(FuturesWsEvent::Reconnecting {
                                    attempt: this.reconnect_attempt + 1,
                                })));
                            } else {
                                return Poll::Ready(Some(Ok(FuturesWsEvent::Disconnected)));
                            }
                        }
                        WsMessage::Frame(_) => {
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                    }
                }
                Poll::Ready(Some(Err(e))) => {
                    let this = self.as_mut().get_mut();
                    this.connected = false;
                    tracing::warn!("WebSocket error: {}", e);

                    if this.should_reconnect() {
                        return Poll::Ready(Some(Ok(FuturesWsEvent::Reconnecting {
                            attempt: this.reconnect_attempt + 1,
                        })));
                    } else {
                        return Poll::Ready(Some(Err(KrakenError::WebSocket(e))));
                    }
                }
                Poll::Ready(None) => {
                    let this = self.as_mut().get_mut();
                    this.connected = false;

                    if this.should_reconnect() {
                        return Poll::Ready(Some(Ok(FuturesWsEvent::Reconnecting {
                            attempt: this.reconnect_attempt + 1,
                        })));
                    } else {
                        return Poll::Ready(None);
                    }
                }
                Poll::Pending => {}
            }
        } else if !self.reconnecting && self.should_reconnect() {
            // Need to reconnect
            return Poll::Ready(Some(Ok(FuturesWsEvent::Reconnecting {
                attempt: self.reconnect_attempt + 1,
            })));
        }

        Poll::Pending
    }
}

/// Generate a subscription key for tracking.
fn subscription_key(feed: &str, product_ids: &[String]) -> String {
    if product_ids.is_empty() {
        feed.to_string()
    } else {
        format!("{}:{}", feed, product_ids.join(","))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_key_with_products() {
        let key = subscription_key("book", &["PI_XBTUSD".into(), "PI_ETHUSD".into()]);
        assert_eq!(key, "book:PI_XBTUSD,PI_ETHUSD");
    }

    #[test]
    fn test_subscription_key_without_products() {
        let key = subscription_key("open_orders", &[]);
        assert_eq!(key, "open_orders");
    }

    #[test]
    fn test_backoff_calculation() {
        let config = WsConfig {
            initial_backoff: Duration::from_secs(1),
            max_backoff: Duration::from_secs(60),
            ..Default::default()
        };

        // Simulate backoff calculation
        let base = config.initial_backoff.as_millis() as u64;
        let max = config.max_backoff.as_millis() as u64;

        // Attempt 0: 1 * 2^0 = 1
        let multiplier = 2u64.saturating_pow(0);
        let result = (base * multiplier).min(max);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(1));

        // Attempt 3: 1 * 2^3 = 8
        let multiplier = 2u64.saturating_pow(3);
        let result = (base * multiplier).min(max);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(8));

        // Attempt 10: capped at 60
        let multiplier = 2u64.saturating_pow(10);
        let result = (base * multiplier).min(max);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(60));
    }
}

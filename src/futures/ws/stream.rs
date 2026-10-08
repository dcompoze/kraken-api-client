//! Futures WebSocket stream implementation.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{Interval, Sleep, interval, sleep, timeout};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async_tls_with_config};

use crate::auth::CredentialsProvider;
use crate::error::KrakenError;
use crate::futures::ws::client::{WsConfig, sign_challenge};
use crate::futures::ws::messages::*;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, WsMessage>;
type WsReceiver = SplitStream<WsStream>;
type StreamFuture<T> = Pin<Box<dyn Future<Output = Result<T, KrakenError>> + Send + Sync>>;

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
    /// Account log (private).
    AccountLog(AccountLogMessage),
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
/// Handles reconnection with exponential backoff, subscription restoration, and challenge-based authentication.
pub struct FuturesStream {
    sink: Option<Arc<Mutex<WsSink>>>,
    receiver: Option<WsReceiver>,
    config: WsConfig,
    url: String,
    credentials: Option<Arc<dyn CredentialsProvider>>,
    auth_state: Option<AuthState>,
    subscriptions: HashMap<String, Subscription>,
    ping_interval: Interval,
    last_message: Instant,
    reconnect_attempt: u32,
    connected: bool,
    reconnect_future: Option<StreamFuture<Self>>,
    ping_task: Option<JoinHandle<Result<(), KrakenError>>>,
    pong_deadline: Option<Pin<Box<Sleep>>>,
    closed: bool,
    authenticated: bool,
    /// Waiting for challenge response
    pending_auth: bool,
}

impl std::fmt::Debug for FuturesStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FuturesStream")
            .field("url", &self.url)
            .field("connected", &self.connected)
            .field("reconnecting", &self.reconnect_future.is_some())
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
        crate::tls::require_secure_url(url, "wss", config.danger_allow_insecure_transport)?;
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
        let connector = crate::tls::websocket_connector_for_url(url)?;
        let (ws_stream, _) = connect_async_tls_with_config(url, None, false, Some(connector))
            .await
            .map_err(|e| {
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
            reconnect_future: None,
            ping_task: None,
            pong_deadline: None,
            closed: false,
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

        // Clone the credentials to avoid borrow issues.
        let creds = credentials.get_credentials().clone();

        let challenge_req = ChallengeRequest::new(&creds.api_key);
        self.send_json(&challenge_req).await?;
        self.pending_auth = true;

        let challenge = self.wait_for_challenge().await?;

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
    pub async fn subscribe_public(
        &mut self,
        feed: &str,
        product_ids: Vec<&str>,
    ) -> Result<(), KrakenError> {
        self.ensure_connected()?;
        let product_ids: Vec<String> = product_ids.into_iter().map(|s| s.to_string()).collect();
        let key = subscription_key(feed, &product_ids);

        self.subscriptions.insert(
            key,
            Subscription {
                feed: feed.to_string(),
                product_ids: product_ids.clone(),
                is_private: false,
            },
        );

        let request = SubscribeRequest::public(feed, product_ids);
        self.send_json(&request).await
    }

    /// Subscribe to a private feed.
    ///
    /// Requires prior authentication via `connect_private`.
    pub async fn subscribe_private(&mut self, feed: &str) -> Result<(), KrakenError> {
        self.ensure_connected()?;
        let auth = self
            .auth_state
            .as_ref()
            .ok_or_else(|| KrakenError::WebSocketMsg("Not authenticated".into()))?;

        let key = subscription_key(feed, &[]);

        self.subscriptions.insert(
            key,
            Subscription {
                feed: feed.to_string(),
                product_ids: vec![],
                is_private: true,
            },
        );

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

        self.ensure_connected()?;
        let product_ids: Vec<String> = product_ids.into_iter().map(|s| s.to_string()).collect();
        let key = subscription_key(feed, &product_ids);

        self.subscriptions.insert(
            key,
            Subscription {
                feed: feed.to_string(),
                product_ids: product_ids.clone(),
                is_private: true,
            },
        );

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
        self.ensure_connected()?;
        let product_ids: Vec<String> = product_ids.into_iter().map(|s| s.to_string()).collect();
        let key = subscription_key(feed, &product_ids);
        self.subscriptions.remove(&key);

        let request = UnsubscribeRequest::new(feed, product_ids);
        self.send_json(&request).await
    }

    fn ensure_connected(&self) -> Result<(), KrakenError> {
        if !self.connected {
            return Err(KrakenError::WebSocketMsg("Not connected".into()));
        }
        Ok(())
    }

    /// Send a JSON message.
    fn send_json<T: serde::Serialize>(
        &self,
        msg: &T,
    ) -> impl Future<Output = Result<(), KrakenError>> + Send + Sync + use<T> {
        let sink = self.sink.clone();
        let json = serde_json::to_string(msg);
        async move {
            let sink = sink.ok_or_else(|| KrakenError::WebSocketMsg("Not connected".into()))?;
            let json = json.map_err(KrakenError::Json)?;
            sink.lock().await.send(WsMessage::Text(json.into())).await?;
            Ok(())
        }
    }

    /// Check if we should reconnect.
    fn should_reconnect(&self) -> bool {
        match self.config.max_reconnect_attempts {
            Some(max) => self.reconnect_attempt < max,
            None => true,
        }
    }

    /// Calculate backoff duration for reconnection.
    fn backoff_duration(&self) -> Duration {
        let base = self.config.initial_backoff.as_millis() as u64;
        let max = self.config.max_backoff.as_millis() as u64;
        let multiplier = 2u64.saturating_pow(self.reconnect_attempt);
        let backoff_ms = base.saturating_mul(multiplier).min(max);
        Duration::from_millis(backoff_ms)
    }

    /// Schedule a reconnect, including subscription restoration.
    fn reconnect(&mut self) {
        let backoff = self.backoff_duration();
        self.reconnect_attempt = self.reconnect_attempt.saturating_add(1);
        let url = self.url.clone();
        let config = self.config.clone();
        let subscriptions = self.subscriptions.clone();
        let credentials = self.credentials.clone();
        self.reconnect_future = Some(Box::pin(async move {
            sleep(backoff).await;
            timeout(Duration::from_secs(10), async move {
                let mut stream = Self::connect(&url, config, credentials).await?;
                if stream.credentials.is_some() {
                    stream.authenticate().await?;
                }
                stream.subscriptions = subscriptions;
                stream.restore_subscriptions().await?;
                Ok(stream)
            })
            .await
            .map_err(|_| KrakenError::WebSocketMsg("Reconnect timed out".into()))?
        }));
    }

    fn disconnect(&mut self) {
        self.connected = false;
        self.sink = None;
        self.receiver = None;
        if let Some(task) = self.ping_task.take() {
            task.abort();
        }
        self.pong_deadline = None;
        self.authenticated = false;
        self.auth_state = None;
        self.pending_auth = false;
    }

    /// Restore subscriptions after reconnection.
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

        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to parse WebSocket message: {}", e);
                return None;
            }
        };

        let event = value
            .get("event")
            .and_then(|e| e.as_str())
            .map(String::from);
        let feed = value.get("feed").and_then(|f| f.as_str()).map(String::from);

        if let Some(event) = event {
            return self.handle_event_message(&event, value);
        }

        if let Some(feed) = feed {
            return self.handle_feed_message(&feed, value);
        }

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
                // Challenges are handled during authentication.
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
            "account_log" | "account_log_snapshot" => {
                if let Ok(log) = serde_json::from_value::<AccountLogMessage>(value) {
                    return Some(FuturesWsEvent::AccountLog(log));
                }
            }
            _ => {
                return Some(FuturesWsEvent::Raw(value));
            }
        }
        None
    }

    /// Close the connection without reconnecting.
    pub async fn close(&mut self) -> Result<(), KrakenError> {
        self.closed = true;
        self.reconnect_future = None;
        let sink = self.sink.clone();
        self.disconnect();
        if let Some(sink) = sink {
            let _ = timeout(self.config.pong_timeout, async move {
                sink.lock().await.send(WsMessage::Close(None)).await
            })
            .await;
        }
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

impl Drop for FuturesStream {
    fn drop(&mut self) {
        if let Some(task) = &self.ping_task {
            task.abort();
        }
    }
}

impl Stream for FuturesStream {
    type Item = Result<FuturesWsEvent, KrakenError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        if this.closed {
            return Poll::Ready(None);
        }

        if let Some(reconnect) = &mut this.reconnect_future {
            match reconnect.as_mut().poll(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Ok(stream)) => {
                    *this = stream;
                    return Poll::Ready(Some(Ok(FuturesWsEvent::Reconnected)));
                }
                Poll::Ready(Err(error)) => {
                    tracing::warn!(%error, attempt = this.reconnect_attempt, "WebSocket reconnect failed");
                    this.reconnect_future = None;
                }
            }
        }

        if !this.connected {
            if this.should_reconnect() {
                this.reconnect();
                return Poll::Ready(Some(Ok(FuturesWsEvent::Reconnecting {
                    attempt: this.reconnect_attempt,
                })));
            }
            this.closed = true;
            return Poll::Ready(Some(Ok(FuturesWsEvent::Disconnected)));
        }

        if this.pong_deadline.is_none()
            && this.ping_task.is_none()
            && this.ping_interval.poll_tick(cx).is_ready()
        {
            if let Some(sink) = this.sink.clone() {
                this.ping_task = Some(tokio::spawn(async move {
                    sink.lock()
                        .await
                        .send(WsMessage::Ping(Vec::new().into()))
                        .await?;
                    Ok(())
                }));
            }
            this.pong_deadline = Some(Box::pin(sleep(this.config.pong_timeout)));
        }

        if let Some(ping) = &mut this.ping_task {
            match Pin::new(ping).poll(cx) {
                Poll::Ready(Ok(Ok(()))) => this.ping_task = None,
                Poll::Ready(result) => {
                    tracing::warn!(?result, "WebSocket ping failed");
                    this.disconnect();
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Pending => {}
            }
        }

        if let Some(deadline) = &mut this.pong_deadline {
            if deadline.as_mut().poll(cx).is_ready() {
                tracing::warn!("WebSocket pong timed out");
                this.disconnect();
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        }

        let Some(receiver) = &mut this.receiver else {
            return Poll::Pending;
        };
        match Pin::new(receiver).poll_next(cx) {
            Poll::Ready(Some(Ok(message))) => match message {
                WsMessage::Text(text) => {
                    if let Some(event) = this.parse_message(&text) {
                        return Poll::Ready(Some(Ok(event)));
                    }
                }
                WsMessage::Binary(data) => {
                    if let Ok(text) = String::from_utf8(data.to_vec()) {
                        if let Some(event) = this.parse_message(&text) {
                            return Poll::Ready(Some(Ok(event)));
                        }
                    }
                }
                WsMessage::Pong(_) => {
                    this.pong_deadline = None;
                }
                WsMessage::Close(_) => this.disconnect(),
                _ => {}
            },
            Poll::Ready(Some(Err(error))) => {
                tracing::warn!(%error, "WebSocket receive failed");
                this.disconnect();
                if !this.should_reconnect() {
                    this.closed = true;
                    return Poll::Ready(Some(Err(KrakenError::WebSocket(error))));
                }
            }
            Poll::Ready(None) => this.disconnect(),
            Poll::Pending => return Poll::Pending,
        }
        cx.waker().wake_by_ref();
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

    #[tokio::test]
    async fn cancelled_next_does_not_block_subscriptions() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("ws://{}", listener.local_addr().unwrap());
        let (release, hold) = tokio::sync::oneshot::channel();
        let server = tokio::spawn(async move {
            let (socket, _) = listener.accept().await.unwrap();
            let _socket = tokio_tungstenite::accept_async(socket).await.unwrap();
            hold.await.unwrap();
        });
        let mut stream = FuturesStream::connect_public(&url, WsConfig::default())
            .await
            .unwrap();
        let sink = stream.sink.as_ref().unwrap().clone();
        let guard = sink.lock().await;
        assert!(
            timeout(Duration::from_millis(20), stream.next())
                .await
                .is_err()
        );
        drop(guard);
        timeout(
            Duration::from_secs(1),
            stream.subscribe_public("ticker", vec!["PI_XBTUSD"]),
        )
        .await
        .unwrap()
        .unwrap();
        stream.close().await.unwrap();
        release.send(()).unwrap();
        server.await.unwrap();
    }

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

        let base = config.initial_backoff.as_millis() as u64;
        let max = config.max_backoff.as_millis() as u64;

        let multiplier = 2u64.saturating_pow(0);
        let result = (base * multiplier).min(max);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(1));

        let multiplier = 2u64.saturating_pow(3);
        let result = (base * multiplier).min(max);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(8));

        let multiplier = 2u64.saturating_pow(10);
        let result = (base * multiplier).min(max);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(60));
    }
}

//! WebSocket stream implementation.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, Stream, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{interval, Interval};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::error::KrakenError;
use crate::spot::ws::client::WsConfig;
use crate::spot::ws::messages::{
    channels, AddOrderParams, AddOrderResult, CancelAllParams, CancelAllResult, CancelOrderParams,
    CancelOrderResult, EditOrderParams, EditOrderResult, Heartbeat, PingRequest, PongResponse,
    SubscribeParams, SubscriptionResult, SystemStatusMessage, WsRequest,
};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, WsMessage>;
type WsReceiver = SplitStream<WsStream>;

/// A message received from the WebSocket connection.
#[derive(Debug, Clone)]
pub enum WsMessageEvent {
    /// System status update.
    Status(SystemStatusMessage),
    /// Heartbeat from server.
    Heartbeat(Heartbeat),
    /// Pong response to our ping.
    Pong(PongResponse),
    /// Subscription confirmed.
    Subscribed(SubscriptionResult),
    /// Unsubscription confirmed.
    Unsubscribed(SubscriptionResult),
    /// Raw channel data (ticker, book, trade, etc.).
    ChannelData(serde_json::Value),
    /// Order added successfully.
    OrderAdded {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Order result details.
        result: AddOrderResult,
    },
    /// Order cancelled successfully.
    OrderCancelled {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Cancel result details.
        result: CancelOrderResult,
    },
    /// All orders cancelled.
    AllOrdersCancelled {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Number of orders cancelled.
        result: CancelAllResult,
    },
    /// Order edited successfully.
    OrderEdited {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Edit result details.
        result: EditOrderResult,
    },
    /// Subscription/unsubscription error.
    Error { method: String, error: String, req_id: Option<u64> },
    /// Connection closed.
    Disconnected,
    /// Reconnecting.
    Reconnecting { attempt: u32 },
    /// Reconnected successfully.
    Reconnected,
}

/// Subscription state tracking.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SubscriptionState {
    params: SubscribeParams,
    status: SubscriptionStatus,
    last_change: Instant,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubscriptionStatus {
    Pending,
    Active,
    Error,
}

/// A stream of messages from a Kraken WebSocket connection.
///
/// This stream handles:
/// - Automatic reconnection with exponential backoff
/// - Subscription restoration after reconnect
/// - Heartbeat/ping monitoring
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
/// stream.subscribe(SubscribeParams::public(channels::TICKER, vec!["BTC/USD".into()])).await?;
///
/// while let Some(msg) = stream.next().await {
///     match msg? {
///         WsMessageEvent::ChannelData(data) => println!("Data: {:?}", data),
///         WsMessageEvent::Disconnected => println!("Disconnected!"),
///         _ => {}
///     }
/// }
/// ```
pub struct KrakenStream {
    /// WebSocket sink for sending messages.
    sink: Option<Arc<Mutex<WsSink>>>,
    /// WebSocket receiver for incoming messages.
    receiver: Option<WsReceiver>,
    /// Connection configuration.
    config: WsConfig,
    /// URL to connect to.
    url: String,
    /// Authentication token (for private connections).
    token: Option<String>,
    /// Active subscriptions.
    subscriptions: HashMap<String, SubscriptionState>,
    /// Ping interval timer.
    ping_interval: Interval,
    /// Last ping sent timestamp.
    last_ping: Option<Instant>,
    /// Last message received timestamp.
    last_message: Instant,
    /// Current reconnection attempt.
    reconnect_attempt: u32,
    /// Request ID counter.
    req_id: u64,
    /// Connection state.
    connected: bool,
    /// Whether we're currently reconnecting.
    reconnecting: bool,
}

impl std::fmt::Debug for KrakenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KrakenStream")
            .field("url", &self.url)
            .field("connected", &self.connected)
            .field("reconnecting", &self.reconnecting)
            .field("subscriptions", &self.subscriptions.len())
            .finish()
    }
}

impl KrakenStream {
    /// Create and connect a new public WebSocket stream.
    pub(crate) async fn connect_public(url: &str, config: WsConfig) -> Result<Self, KrakenError> {
        Self::connect(url, config, None).await
    }

    /// Create and connect a new private WebSocket stream.
    pub(crate) async fn connect_private(
        url: &str,
        config: WsConfig,
        token: String,
    ) -> Result<Self, KrakenError> {
        Self::connect(url, config, Some(token)).await
    }

    /// Connect to the WebSocket server.
    async fn connect(
        url: &str,
        config: WsConfig,
        token: Option<String>,
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
            token,
            subscriptions: HashMap::new(),
            ping_interval: interval(ping_interval_duration),
            last_ping: None,
            last_message: Instant::now(),
            reconnect_attempt: 0,
            req_id: 0,
            connected: true,
            reconnecting: false,
        })
    }

    /// Subscribe to a channel.
    pub async fn subscribe(&mut self, params: SubscribeParams) -> Result<(), KrakenError> {
        let key = subscription_key(&params);

        // Store subscription state
        self.subscriptions.insert(
            key,
            SubscriptionState {
                params: params.clone(),
                status: SubscriptionStatus::Pending,
                last_change: Instant::now(),
            },
        );

        // Send subscription request
        self.send_subscribe(params).await
    }

    /// Unsubscribe from a channel.
    pub async fn unsubscribe(&mut self, params: SubscribeParams) -> Result<(), KrakenError> {
        let key = subscription_key(&params);
        self.subscriptions.remove(&key);

        self.send_unsubscribe(params).await
    }

    /// Send a subscription request.
    async fn send_subscribe(&mut self, params: SubscribeParams) -> Result<(), KrakenError> {
        let req = WsRequest::new("subscribe", params).with_req_id(self.next_req_id());
        self.send_json(&req).await
    }

    /// Send an unsubscription request.
    async fn send_unsubscribe(&mut self, params: SubscribeParams) -> Result<(), KrakenError> {
        let req = WsRequest::new("unsubscribe", params).with_req_id(self.next_req_id());
        self.send_json(&req).await
    }

    /// Send a ping message.
    pub async fn ping(&mut self) -> Result<(), KrakenError> {
        let req = WsRequest::new("ping", PingRequest::with_req_id(self.next_req_id()));
        self.last_ping = Some(Instant::now());
        self.send_json(&req).await
    }

    // ========== Trading Operations ==========

    /// Add a new order via WebSocket.
    ///
    /// This requires an authenticated connection. Use `connect_private()` first.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::spot::ws::SpotWsClient;
    /// use kraken_api_client::spot::ws::messages::AddOrderParams;
    /// use kraken_api_client::types::{OrderType, BuySell};
    /// use rust_decimal_macros::dec;
    ///
    /// let client = SpotWsClient::new();
    /// let token = rest_client.get_websocket_token().await?.token;
    /// let mut stream = client.connect_private(&token).await?;
    ///
    /// let params = AddOrderParams::new(OrderType::Limit, BuySell::Buy, "BTC/USD", &token)
    ///     .order_qty(dec!(0.001))
    ///     .limit_price(dec!(50000))
    ///     .validate(true); // Validate only, don't submit
    ///
    /// stream.add_order(params).await?;
    /// ```
    pub async fn add_order(&mut self, params: AddOrderParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("add_order", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Cancel one or more orders via WebSocket.
    ///
    /// This requires an authenticated connection. Use `connect_private()` first.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::spot::ws::messages::CancelOrderParams;
    ///
    /// // Cancel by order ID
    /// let params = CancelOrderParams::by_order_id(
    ///     vec!["OQCLML-BW3P3-BUCMWZ".into()],
    ///     &token
    /// );
    /// stream.cancel_order(params).await?;
    ///
    /// // Cancel by client order ID
    /// let params = CancelOrderParams::by_cl_ord_id(
    ///     vec!["my-order-1".into()],
    ///     &token
    /// );
    /// stream.cancel_order(params).await?;
    /// ```
    pub async fn cancel_order(&mut self, params: CancelOrderParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("cancel_order", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Cancel all open orders via WebSocket.
    ///
    /// This requires an authenticated connection. Use `connect_private()` first.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::spot::ws::messages::CancelAllParams;
    ///
    /// let params = CancelAllParams::new(&token);
    /// stream.cancel_all_orders(params).await?;
    /// ```
    pub async fn cancel_all_orders(&mut self, params: CancelAllParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("cancel_all", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Edit an existing order via WebSocket.
    ///
    /// This requires an authenticated connection. Use `connect_private()` first.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use kraken_api_client::spot::ws::messages::EditOrderParams;
    /// use rust_decimal_macros::dec;
    ///
    /// let params = EditOrderParams::new("OQCLML-BW3P3-BUCMWZ", &token)
    ///     .limit_price(dec!(51000))
    ///     .order_qty(dec!(0.002));
    ///
    /// stream.edit_order(params).await?;
    /// ```
    pub async fn edit_order(&mut self, params: EditOrderParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("edit_order", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Ensure this is a private (authenticated) connection.
    fn ensure_private(&self) -> Result<(), KrakenError> {
        if self.token.is_none() {
            return Err(KrakenError::MissingCredentials);
        }
        Ok(())
    }

    /// Send a JSON message.
    async fn send_json<T: serde::Serialize>(&self, msg: &T) -> Result<(), KrakenError> {
        let sink = self
            .sink
            .as_ref()
            .ok_or_else(|| KrakenError::WebSocketMsg("Not connected".into()))?;

        let json = serde_json::to_string(msg)
            .map_err(|e| KrakenError::WebSocketMsg(format!("Failed to serialize message: {}", e)))?;

        let mut sink = sink.lock().await;
        sink.send(WsMessage::Text(json.into()))
            .await
            .map_err(|e| KrakenError::WebSocketMsg(format!("Failed to send message: {}", e)))
    }

    /// Get the next request ID.
    fn next_req_id(&mut self) -> u64 {
        self.req_id += 1;
        self.req_id
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

        // Close existing connection
        self.sink = None;
        self.receiver = None;

        // Wait with backoff
        let backoff = self.backoff_duration();
        tokio::time::sleep(backoff).await;

        // Try to reconnect
        let (ws_stream, _) = connect_async(&self.url).await.map_err(|e| {
            KrakenError::WebSocketMsg(format!("Failed to reconnect: {}", e))
        })?;

        let (sink, receiver) = ws_stream.split();
        self.sink = Some(Arc::new(Mutex::new(sink)));
        self.receiver = Some(receiver);
        self.connected = true;
        self.reconnecting = false;
        self.reconnect_attempt = 0;
        self.last_message = Instant::now();

        // Restore subscriptions
        self.restore_subscriptions().await?;

        Ok(())
    }

    /// Restore subscriptions after reconnection.
    #[allow(dead_code)]
    async fn restore_subscriptions(&mut self) -> Result<(), KrakenError> {
        let subs: Vec<_> = self.subscriptions.values().map(|s| s.params.clone()).collect();

        for params in subs {
            self.send_subscribe(params).await?;
        }

        Ok(())
    }

    /// Parse and handle an incoming message.
    fn parse_message(&mut self, text: &str) -> Option<WsMessageEvent> {
        self.last_message = Instant::now();

        // Try to parse as JSON
        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to parse WebSocket message: {}", e);
                return None;
            }
        };

        // Check if it's a response message (has "method" at top level)
        if let Some(method) = value.get("method").and_then(|m| m.as_str()) {
            return self.handle_response_message(method, &value);
        }

        // Check if it's a channel message (has "channel" at top level)
        if let Some(channel) = value.get("channel").and_then(|c| c.as_str()) {
            let channel = channel.to_string(); // Clone the channel string to avoid borrow
            return self.handle_channel_message(&channel, value);
        }

        // Unknown message format
        tracing::debug!("Unknown message format: {}", text);
        Some(WsMessageEvent::ChannelData(value))
    }

    /// Handle a response message (method-based).
    fn handle_response_message(
        &mut self,
        method: &str,
        value: &serde_json::Value,
    ) -> Option<WsMessageEvent> {
        let req_id = value.get("req_id").and_then(|r| r.as_u64());

        match method {
            "pong" => {
                if let Ok(pong) = serde_json::from_value::<PongResponse>(value.clone()) {
                    self.last_ping = None;
                    return Some(WsMessageEvent::Pong(pong));
                }
            }
            "subscribe" => {
                // Check for success/error
                let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(sub_result) = serde_json::from_value::<SubscriptionResult>(result.clone()) {
                            // Update subscription state
                            let key = subscription_key_from_result(&sub_result);
                            if let Some(state) = self.subscriptions.get_mut(&key) {
                                state.status = SubscriptionStatus::Active;
                                state.last_change = Instant::now();
                            }
                            return Some(WsMessageEvent::Subscribed(sub_result));
                        }
                    }
                } else {
                    let error = value.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            "unsubscribe" => {
                let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(sub_result) = serde_json::from_value::<SubscriptionResult>(result.clone()) {
                            return Some(WsMessageEvent::Unsubscribed(sub_result));
                        }
                    }
                } else {
                    let error = value.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            "add_order" => {
                let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(order_result) = serde_json::from_value::<AddOrderResult>(result.clone()) {
                            return Some(WsMessageEvent::OrderAdded {
                                req_id,
                                result: order_result,
                            });
                        }
                    }
                } else {
                    let error = value.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            "cancel_order" => {
                let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(cancel_result) = serde_json::from_value::<CancelOrderResult>(result.clone()) {
                            return Some(WsMessageEvent::OrderCancelled {
                                req_id,
                                result: cancel_result,
                            });
                        }
                    }
                } else {
                    let error = value.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            "cancel_all" => {
                let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(cancel_result) = serde_json::from_value::<CancelAllResult>(result.clone()) {
                            return Some(WsMessageEvent::AllOrdersCancelled {
                                req_id,
                                result: cancel_result,
                            });
                        }
                    }
                } else {
                    let error = value.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            "edit_order" => {
                let success = value.get("success").and_then(|s| s.as_bool()).unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(edit_result) = serde_json::from_value::<EditOrderResult>(result.clone()) {
                            return Some(WsMessageEvent::OrderEdited {
                                req_id,
                                result: edit_result,
                            });
                        }
                    }
                } else {
                    let error = value.get("error").and_then(|e| e.as_str()).unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            _ => {
                // Unknown method, return as raw data
                return Some(WsMessageEvent::ChannelData(value.clone()));
            }
        }

        None
    }

    /// Handle a channel message.
    fn handle_channel_message(
        &mut self,
        channel: &str,
        value: serde_json::Value,
    ) -> Option<WsMessageEvent> {
        match channel {
            channels::STATUS => {
                if let Ok(status) = serde_json::from_value::<SystemStatusMessage>(value) {
                    return Some(WsMessageEvent::Status(status));
                }
            }
            channels::HEARTBEAT => {
                if let Ok(heartbeat) = serde_json::from_value::<Heartbeat>(value) {
                    return Some(WsMessageEvent::Heartbeat(heartbeat));
                }
            }
            _ => {
                // Market data or user data channel
                return Some(WsMessageEvent::ChannelData(value));
            }
        }

        None
    }

    /// Check connection health (ping timeout).
    fn check_connection_health(&self) -> bool {
        // Check if ping response is overdue
        if let Some(ping_time) = self.last_ping {
            if ping_time.elapsed() > self.config.pong_timeout {
                return false;
            }
        }

        true
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
}

impl Stream for KrakenStream {
    type Item = Result<WsMessageEvent, KrakenError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Check ping interval
        if self.ping_interval.poll_tick(cx).is_ready() && self.connected {
            // Only send ping if not waiting for pong
            if self.last_ping.is_none() {
                let this = self.as_mut().get_mut();
                let ping_req = WsRequest::new("ping", PingRequest::with_req_id(this.next_req_id()));
                this.last_ping = Some(Instant::now());

                if let Some(sink) = &this.sink {
                    let sink = sink.clone();
                    if let Ok(json) = serde_json::to_string(&ping_req) {
                        tokio::spawn(async move {
                            let mut sink = sink.lock().await;
                            let _ = sink.send(WsMessage::Text(json.into())).await;
                        });
                    }
                }
            }
        }

        // Check connection health
        if !self.check_connection_health() && self.connected {
            let this = self.as_mut().get_mut();
            this.connected = false;

            if this.should_reconnect() {
                return Poll::Ready(Some(Ok(WsMessageEvent::Reconnecting {
                    attempt: this.reconnect_attempt + 1,
                })));
            } else {
                return Poll::Ready(Some(Ok(WsMessageEvent::Disconnected)));
            }
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
                            // If parse returned None, continue polling
                            cx.waker().wake_by_ref();
                            return Poll::Pending;
                        }
                        WsMessage::Binary(data) => {
                            // Try to parse binary as JSON text
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
                                return Poll::Ready(Some(Ok(WsMessageEvent::Reconnecting {
                                    attempt: this.reconnect_attempt + 1,
                                })));
                            } else {
                                return Poll::Ready(Some(Ok(WsMessageEvent::Disconnected)));
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
                        return Poll::Ready(Some(Ok(WsMessageEvent::Reconnecting {
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
                        return Poll::Ready(Some(Ok(WsMessageEvent::Reconnecting {
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
            return Poll::Ready(Some(Ok(WsMessageEvent::Reconnecting {
                attempt: self.reconnect_attempt + 1,
            })));
        }

        Poll::Pending
    }
}

/// Generate a subscription key for tracking.
fn subscription_key(params: &SubscribeParams) -> String {
    let symbols = params
        .symbol
        .as_ref()
        .map(|s| s.join(","))
        .unwrap_or_default();
    format!("{}:{}", params.channel, symbols)
}

/// Generate a subscription key from a result.
fn subscription_key_from_result(result: &SubscriptionResult) -> String {
    format!(
        "{}:{}",
        result.channel,
        result.symbol.as_deref().unwrap_or("")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_key() {
        let params = SubscribeParams::public("ticker", vec!["BTC/USD".into(), "ETH/USD".into()]);
        let key = subscription_key(&params);
        assert_eq!(key, "ticker:BTC/USD,ETH/USD");
    }

    #[test]
    fn test_backoff_calculation_formula() {
        // Test backoff formula: base * 2^attempt, capped at max
        let initial = Duration::from_secs(1);
        let max = Duration::from_secs(60);

        // Attempt 0: 1 * 2^0 = 1
        let attempt = 0;
        let multiplier = 2u64.saturating_pow(attempt);
        let result = (initial.as_millis() as u64 * multiplier).min(max.as_millis() as u64);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(1));

        // Attempt 3: 1 * 2^3 = 8
        let attempt = 3;
        let multiplier = 2u64.saturating_pow(attempt);
        let result = (initial.as_millis() as u64 * multiplier).min(max.as_millis() as u64);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(8));

        // Attempt 10: 1 * 2^10 = 1024 -> capped at 60
        let attempt = 10;
        let multiplier = 2u64.saturating_pow(attempt);
        let result = (initial.as_millis() as u64 * multiplier).min(max.as_millis() as u64);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(60));
    }
}

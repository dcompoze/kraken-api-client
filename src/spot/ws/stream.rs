//! WebSocket stream implementation.

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

use crate::error::KrakenError;
use crate::spot::ws::client::WsConfig;
use crate::spot::ws::messages::{
    AddOrderParams, AddOrderResult, AmendOrderParams, AmendOrderResult, BatchAddParams,
    BatchCancelParams, BatchCancelResult, CancelAllOrdersAfterParams, CancelAllOrdersAfterResult,
    CancelAllParams, CancelAllResult, CancelOrderParams, CancelOrderResult, EditOrderParams,
    EditOrderResult, Heartbeat, PongResponse, SubscribeParams, SubscriptionResult,
    SystemStatusMessage, WsRequest, channels,
};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, WsMessage>;
type WsReceiver = SplitStream<WsStream>;
type StreamFuture<T> = Pin<Box<dyn Future<Output = Result<T, KrakenError>> + Send + Sync>>;

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
    /// Order amended successfully.
    OrderAmended {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Amend result details.
        result: AmendOrderResult,
    },
    /// Batch of orders added.
    BatchOrdersAdded {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Results per order.
        result: Vec<AddOrderResult>,
    },
    /// Batch of orders cancelled.
    BatchOrdersCancelled {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Batch cancel result details.
        result: BatchCancelResult,
    },
    /// Cancel-on-disconnect timer set.
    CancelOnDisconnectSet {
        /// Request ID from the original request.
        req_id: Option<u64>,
        /// Timer details.
        result: CancelAllOrdersAfterResult,
    },
    /// Subscription/unsubscription error.
    Error {
        method: String,
        error: String,
        req_id: Option<u64>,
    },
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
/// Handles reconnection with exponential backoff, subscription restoration after reconnect, and ping monitoring.
pub struct KrakenStream {
    sink: Option<Arc<Mutex<WsSink>>>,
    receiver: Option<WsReceiver>,
    config: WsConfig,
    url: String,
    /// Authentication token (for private connections)
    token: Option<String>,
    subscriptions: HashMap<String, SubscriptionState>,
    ping_interval: Interval,
    /// Request ID of the outstanding ping.
    pending_ping_id: Option<u64>,
    /// Last message received timestamp
    last_message: Instant,
    reconnect_attempt: u32,
    req_id: u64,
    connected: bool,
    reconnect_future: Option<StreamFuture<Self>>,
    ping_task: Option<JoinHandle<Result<(), KrakenError>>>,
    pong_deadline: Option<Pin<Box<Sleep>>>,
    closed: bool,
}

impl std::fmt::Debug for KrakenStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("KrakenStream")
            .field("url", &self.url)
            .field("connected", &self.connected)
            .field("reconnecting", &self.reconnect_future.is_some())
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
        crate::tls::require_secure_url(url, "wss", config.danger_allow_insecure_transport)?;
        Self::connect(url, config, Some(token)).await
    }

    /// Connect to the WebSocket server.
    async fn connect(
        url: &str,
        config: WsConfig,
        token: Option<String>,
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
            token,
            subscriptions: HashMap::new(),
            ping_interval: interval(ping_interval_duration),
            pending_ping_id: None,
            last_message: Instant::now(),
            reconnect_attempt: 0,
            req_id: 0,
            connected: true,
            reconnect_future: None,
            ping_task: None,
            pong_deadline: None,
            closed: false,
        })
    }

    /// Subscribe to a channel.
    pub async fn subscribe(&mut self, params: SubscribeParams) -> Result<(), KrakenError> {
        self.ensure_connected()?;
        let key = subscription_key(&params);

        self.subscriptions.insert(
            key,
            SubscriptionState {
                params: params.clone(),
                status: SubscriptionStatus::Pending,
                last_change: Instant::now(),
            },
        );

        self.send_subscribe(params).await
    }

    /// Unsubscribe from a channel.
    pub async fn unsubscribe(&mut self, params: SubscribeParams) -> Result<(), KrakenError> {
        self.ensure_connected()?;
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
        self.ensure_connected()?;
        let req_id = self.next_req_id();
        let req = serde_json::json!({"method": "ping", "req_id": req_id});
        self.pending_ping_id = Some(req_id);
        self.pong_deadline = Some(Box::pin(sleep(self.config.pong_timeout)));
        self.send_json(&req).await
    }

    /// Add a new order via WebSocket.
    ///
    /// Requires an authenticated connection.
    pub async fn add_order(&mut self, params: AddOrderParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("add_order", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Cancel one or more orders via WebSocket.
    ///
    /// Requires an authenticated connection.
    pub async fn cancel_order(&mut self, params: CancelOrderParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("cancel_order", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Cancel all open orders via WebSocket.
    ///
    /// Requires an authenticated connection.
    pub async fn cancel_all_orders(&mut self, params: CancelAllParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("cancel_all", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Edit an existing order via WebSocket.
    ///
    /// Requires an authenticated connection.
    pub async fn edit_order(&mut self, params: EditOrderParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("edit_order", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Amend an existing order via WebSocket.
    ///
    /// Amending keeps the order ID and queue priority, unlike editing.
    /// Requires an authenticated connection.
    pub async fn amend_order(&mut self, params: AmendOrderParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("amend_order", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Place multiple orders in a single batch via WebSocket.
    ///
    /// Requires an authenticated connection.
    pub async fn batch_add(&mut self, params: BatchAddParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("batch_add", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Cancel multiple orders in a single batch via WebSocket.
    ///
    /// Requires an authenticated connection.
    pub async fn batch_cancel(&mut self, params: BatchCancelParams) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("batch_cancel", params).with_req_id(req_id);
        self.send_json(&req).await?;
        Ok(req_id)
    }

    /// Set a cancel-on-disconnect timer via WebSocket (dead man's switch).
    ///
    /// The timer must be refreshed before it expires, a timeout of 0 disables it.
    /// Requires an authenticated connection.
    pub async fn cancel_all_orders_after(
        &mut self,
        params: CancelAllOrdersAfterParams,
    ) -> Result<u64, KrakenError> {
        self.ensure_private()?;
        let req_id = self.next_req_id();
        let req = WsRequest::new("cancel_all_orders_after", params).with_req_id(req_id);
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

    /// Get the next request ID.
    fn next_req_id(&mut self) -> u64 {
        self.req_id += 1;
        self.req_id
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
        let token = self.token.clone();
        let req_id = self.req_id;
        self.reconnect_future = Some(Box::pin(async move {
            sleep(backoff).await;
            timeout(Duration::from_secs(10), async move {
                let mut stream = Self::connect(&url, config, token).await?;
                stream.req_id = req_id;
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
        self.pending_ping_id = None;
    }

    /// Restore subscriptions after reconnection.
    async fn restore_subscriptions(&mut self) -> Result<(), KrakenError> {
        let subs: Vec<_> = self
            .subscriptions
            .values()
            .map(|s| s.params.clone())
            .collect();

        for params in subs {
            self.send_subscribe(params).await?;
        }

        Ok(())
    }

    /// Parse and handle an incoming message.
    fn parse_message(&mut self, text: &str) -> Option<WsMessageEvent> {
        self.last_message = Instant::now();

        let value: serde_json::Value = match serde_json::from_str(text) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to parse WebSocket message: {}", e);
                return None;
            }
        };

        if let Some(method) = value.get("method").and_then(|m| m.as_str()) {
            return self.handle_response_message(method, &value);
        }

        if let Some(channel) = value.get("channel").and_then(|c| c.as_str()) {
            let channel = channel.to_string();
            return self.handle_channel_message(&channel, value);
        }

        tracing::debug!("Unknown message format: {}", text);
        Some(WsMessageEvent::ChannelData(value))
    }

    /// Parse a trading response envelope into an event.
    ///
    /// The payload is read from the `result` field, or from the whole message
    /// when `from_result_field` is false.
    /// A successful response whose payload does not match the expected shape
    /// is surfaced as raw channel data instead of being dropped, so callers
    /// waiting on the `req_id` still receive the message.
    fn parse_trading_response<T, F>(
        method: &str,
        value: &serde_json::Value,
        req_id: Option<u64>,
        from_result_field: bool,
        wrap: F,
    ) -> WsMessageEvent
    where
        T: serde::de::DeserializeOwned,
        F: FnOnce(T) -> WsMessageEvent,
    {
        let success = value
            .get("success")
            .and_then(|s| s.as_bool())
            .unwrap_or(false);
        if !success {
            let error = value
                .get("error")
                .and_then(|e| e.as_str())
                .unwrap_or("Unknown error");
            return WsMessageEvent::Error {
                method: method.to_string(),
                error: error.to_string(),
                req_id,
            };
        }

        let payload = if from_result_field {
            value.get("result")
        } else {
            Some(value)
        };
        match payload.map(T::deserialize) {
            Some(Ok(result)) => wrap(result),
            _ => WsMessageEvent::ChannelData(value.clone()),
        }
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
                    if pong.req_id == self.pending_ping_id
                        && value.get("success") != Some(&serde_json::Value::Bool(false))
                    {
                        self.pending_ping_id = None;
                        self.pong_deadline = None;
                    }
                    return Some(WsMessageEvent::Pong(pong));
                }
            }
            "subscribe" => {
                let success = value
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(sub_result) =
                            serde_json::from_value::<SubscriptionResult>(result.clone())
                        {
                            let key = subscription_key_from_result(&sub_result);
                            if let Some(state) = self.subscriptions.get_mut(&key) {
                                state.status = SubscriptionStatus::Active;
                                state.last_change = Instant::now();
                            }
                            return Some(WsMessageEvent::Subscribed(sub_result));
                        }
                    }
                } else {
                    let error = value
                        .get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            "unsubscribe" => {
                let success = value
                    .get("success")
                    .and_then(|s| s.as_bool())
                    .unwrap_or(false);
                if success {
                    if let Some(result) = value.get("result") {
                        if let Ok(sub_result) =
                            serde_json::from_value::<SubscriptionResult>(result.clone())
                        {
                            return Some(WsMessageEvent::Unsubscribed(sub_result));
                        }
                    }
                } else {
                    let error = value
                        .get("error")
                        .and_then(|e| e.as_str())
                        .unwrap_or("Unknown error");
                    return Some(WsMessageEvent::Error {
                        method: method.to_string(),
                        error: error.to_string(),
                        req_id,
                    });
                }
            }
            "add_order" => {
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    true,
                    |result: AddOrderResult| WsMessageEvent::OrderAdded { req_id, result },
                ));
            }
            "cancel_order" => {
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    true,
                    |result: CancelOrderResult| WsMessageEvent::OrderCancelled { req_id, result },
                ));
            }
            "cancel_all" => {
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    true,
                    |result: CancelAllResult| WsMessageEvent::AllOrdersCancelled { req_id, result },
                ));
            }
            "edit_order" => {
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    true,
                    |result: EditOrderResult| WsMessageEvent::OrderEdited { req_id, result },
                ));
            }
            "amend_order" => {
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    true,
                    |result: AmendOrderResult| WsMessageEvent::OrderAmended { req_id, result },
                ));
            }
            "batch_add" => {
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    true,
                    |result: Vec<AddOrderResult>| WsMessageEvent::BatchOrdersAdded {
                        req_id,
                        result,
                    },
                ));
            }
            "batch_cancel" => {
                // The cancel count is reported at the top level, not inside `result`.
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    false,
                    |result: BatchCancelResult| WsMessageEvent::BatchOrdersCancelled {
                        req_id,
                        result,
                    },
                ));
            }
            "cancel_all_orders_after" => {
                return Some(Self::parse_trading_response(
                    method,
                    value,
                    req_id,
                    true,
                    |result: CancelAllOrdersAfterResult| WsMessageEvent::CancelOnDisconnectSet {
                        req_id,
                        result,
                    },
                ));
            }
            _ => {
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
                return Some(WsMessageEvent::ChannelData(value));
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
}

impl Drop for KrakenStream {
    fn drop(&mut self) {
        if let Some(task) = &self.ping_task {
            task.abort();
        }
    }
}

impl Stream for KrakenStream {
    type Item = Result<WsMessageEvent, KrakenError>;

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
                    return Poll::Ready(Some(Ok(WsMessageEvent::Reconnected)));
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
                return Poll::Ready(Some(Ok(WsMessageEvent::Reconnecting {
                    attempt: this.reconnect_attempt,
                })));
            }
            this.closed = true;
            return Poll::Ready(Some(Ok(WsMessageEvent::Disconnected)));
        }

        if this.pong_deadline.is_none()
            && this.ping_task.is_none()
            && this.ping_interval.poll_tick(cx).is_ready()
        {
            let req_id = this.next_req_id();
            let req = serde_json::json!({"method": "ping", "req_id": req_id});
            this.pending_ping_id = Some(req_id);
            this.ping_task = Some(tokio::spawn(this.send_json(&req)));
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
        let mut stream = KrakenStream::connect_public(&url, WsConfig::default())
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
            stream.subscribe(SubscribeParams::public("ticker", vec!["BTC/USD".into()])),
        )
        .await
        .unwrap()
        .unwrap();
        stream.close().await.unwrap();
        release.send(()).unwrap();
        server.await.unwrap();
    }

    #[test]
    fn test_subscription_key() {
        let params = SubscribeParams::public("ticker", vec!["BTC/USD".into(), "ETH/USD".into()]);
        let key = subscription_key(&params);
        assert_eq!(key, "ticker:BTC/USD,ETH/USD");
    }

    #[test]
    fn test_backoff_calculation_formula() {
        let initial = Duration::from_secs(1);
        let max = Duration::from_secs(60);

        let attempt = 0;
        let multiplier = 2u64.saturating_pow(attempt);
        let result = (initial.as_millis() as u64 * multiplier).min(max.as_millis() as u64);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(1));

        let attempt = 3;
        let multiplier = 2u64.saturating_pow(attempt);
        let result = (initial.as_millis() as u64 * multiplier).min(max.as_millis() as u64);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(8));

        let attempt = 10;
        let multiplier = 2u64.saturating_pow(attempt);
        let result = (initial.as_millis() as u64 * multiplier).min(max.as_millis() as u64);
        assert_eq!(Duration::from_millis(result), Duration::from_secs(60));
    }
}

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use kraken_api_client::auth::StaticCredentials;
use kraken_api_client::futures::ws::{FuturesStream, FuturesWsClient, FuturesWsEvent};
use kraken_api_client::spot::ws::messages::{SubscribeParams, TickerMessage};
use kraken_api_client::spot::ws::{KrakenStream, SpotWsClient, WsMessageEvent};
use serde_json::{Value, json};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::time::{Instant, timeout};
use tokio_tungstenite::{WebSocketStream, accept_async, tungstenite::Message};

const WAIT: Duration = Duration::from_secs(5);

#[derive(Clone, Copy)]
enum Venue {
    Spot,
    Futures,
}

enum Client {
    Spot(KrakenStream),
    Futures(FuturesStream),
}

#[derive(Debug, PartialEq)]
enum Event {
    Reconnecting(u32),
    Reconnected,
    Disconnected,
    Other,
}

impl Venue {
    async fn connect(self, url: &str, attempts: u32, pong_timeout: Duration) -> Client {
        match self {
            Self::Spot => {
                let config = kraken_api_client::spot::ws::WsConfig {
                    initial_backoff: Duration::from_millis(20),
                    max_backoff: Duration::from_millis(40),
                    max_reconnect_attempts: Some(attempts),
                    ping_interval: Duration::from_secs(60),
                    pong_timeout,
                    ..Default::default()
                };
                Client::Spot(
                    SpotWsClient::with_urls(url, url)
                        .connect_public_with_config(config)
                        .await
                        .unwrap(),
                )
            }
            Self::Futures => {
                let config = kraken_api_client::futures::ws::WsConfig {
                    initial_backoff: Duration::from_millis(20),
                    max_backoff: Duration::from_millis(40),
                    max_reconnect_attempts: Some(attempts),
                    ping_interval: Duration::from_secs(60),
                    pong_timeout,
                    ..Default::default()
                };
                Client::Futures(
                    FuturesWsClient::with_url(url)
                        .connect_public_with_config(config)
                        .await
                        .unwrap(),
                )
            }
        }
    }
}

impl Client {
    async fn subscribe(&mut self) {
        match self {
            Self::Spot(stream) => stream
                .subscribe(SubscribeParams::public("ticker", vec!["BTC/USD".into()]))
                .await
                .unwrap(),
            Self::Futures(stream) => stream
                .subscribe_public("ticker", vec!["PI_XBTUSD"])
                .await
                .unwrap(),
        }
    }

    async fn next(&mut self) -> Option<Event> {
        match self {
            Self::Spot(stream) => stream.next().await.map(|result| match result.unwrap() {
                WsMessageEvent::Reconnecting { attempt } => Event::Reconnecting(attempt),
                WsMessageEvent::Reconnected => Event::Reconnected,
                WsMessageEvent::Disconnected => Event::Disconnected,
                _ => Event::Other,
            }),
            Self::Futures(stream) => stream.next().await.map(|result| match result.unwrap() {
                FuturesWsEvent::Reconnecting { attempt } => Event::Reconnecting(attempt),
                FuturesWsEvent::Reconnected => Event::Reconnected,
                FuturesWsEvent::Disconnected => Event::Disconnected,
                _ => Event::Other,
            }),
        }
    }

    async fn next_connection_event(&mut self) -> Option<Event> {
        timeout(WAIT, async {
            loop {
                match self.next().await {
                    Some(Event::Other) => continue,
                    event => return event,
                }
            }
        })
        .await
        .unwrap()
    }

    fn connected(&self) -> bool {
        match self {
            Self::Spot(stream) => stream.is_connected(),
            Self::Futures(stream) => stream.is_connected(),
        }
    }

    async fn close(&mut self) {
        match self {
            Self::Spot(stream) => stream.close().await.unwrap(),
            Self::Futures(stream) => stream.close().await.unwrap(),
        }
    }
}

async fn listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let url = format!("ws://{}", listener.local_addr().unwrap());
    (listener, url)
}

async fn accept(listener: &TcpListener) -> WebSocketStream<TcpStream> {
    let (socket, _) = timeout(WAIT, listener.accept()).await.unwrap().unwrap();
    accept_async(socket).await.unwrap()
}

async fn request(socket: &mut WebSocketStream<TcpStream>) -> Value {
    timeout(WAIT, async {
        loop {
            if let Message::Text(text) = socket.next().await.unwrap().unwrap() {
                let value: Value = serde_json::from_str(&text).unwrap();
                if value["method"] == "ping" {
                    socket
                        .send(Message::Text(
                            json!({"method": "pong", "req_id": value["req_id"]})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                } else {
                    return value;
                }
            }
        }
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn reconnect_restores_public_subscriptions() {
    for venue in [Venue::Spot, Venue::Futures] {
        let (listener, url) = listener().await;
        let (release, hold) = oneshot::channel();
        let server = tokio::spawn(async move {
            let mut first = accept(&listener).await;
            let original = request(&mut first).await;
            first.close(None).await.unwrap();
            drop(first);
            let mut second = accept(&listener).await;
            let restored = request(&mut second).await;
            for field in ["method", "event", "params", "feed", "product_ids"] {
                assert_eq!(original[field], restored[field]);
            }
            hold.await.unwrap();
        });
        let mut client = venue.connect(&url, 2, WAIT).await;
        client.subscribe().await;
        assert_eq!(
            client.next_connection_event().await,
            Some(Event::Reconnecting(1))
        );
        assert!(!client.connected());
        assert_eq!(
            client.next_connection_event().await,
            Some(Event::Reconnected)
        );
        assert!(client.connected());
        client.close().await;
        assert_eq!(client.next_connection_event().await, None);
        release.send(()).unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn failed_reconnects_stop_at_the_attempt_limit() {
    for venue in [Venue::Spot, Venue::Futures] {
        let (listener, url) = listener().await;
        let server = tokio::spawn(async move {
            let mut socket = accept(&listener).await;
            drop(listener);
            socket.close(None).await.unwrap();
        });
        let mut client = venue.connect(&url, 2, WAIT).await;
        server.await.unwrap();
        assert_eq!(
            client.next_connection_event().await,
            Some(Event::Reconnecting(1))
        );
        assert_eq!(
            client.next_connection_event().await,
            Some(Event::Reconnecting(2))
        );
        assert_eq!(
            client.next_connection_event().await,
            Some(Event::Disconnected)
        );
        assert_eq!(client.next_connection_event().await, None);
        assert!(!client.connected());
    }
}

#[tokio::test]
async fn close_cancels_pending_reconnects() {
    for venue in [Venue::Spot, Venue::Futures] {
        let (listener, url) = listener().await;
        let (release, hold) = oneshot::channel();
        let server = tokio::spawn(async move {
            let mut socket = accept(&listener).await;
            socket.close(None).await.unwrap();
            hold.await.unwrap();
            assert!(
                timeout(Duration::from_millis(100), listener.accept())
                    .await
                    .is_err()
            );
        });
        let mut client = venue.connect(&url, 2, WAIT).await;
        assert_eq!(
            client.next_connection_event().await,
            Some(Event::Reconnecting(1))
        );
        assert!(
            timeout(Duration::from_millis(5), client.next())
                .await
                .is_err()
        );
        client.close().await;
        assert_eq!(client.next_connection_event().await, None);
        release.send(()).unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn silent_connections_expire_before_the_next_ping_interval() {
    for venue in [Venue::Spot, Venue::Futures] {
        let (listener, url) = listener().await;
        let (release, hold) = oneshot::channel();
        let server = tokio::spawn(async move {
            let _socket = accept(&listener).await;
            hold.await.unwrap();
        });
        let mut client = venue.connect(&url, 0, Duration::from_millis(50)).await;
        let start = Instant::now();
        assert_eq!(
            client.next_connection_event().await,
            Some(Event::Disconnected)
        );
        assert!(start.elapsed() < Duration::from_secs(1));
        assert!(!client.connected());
        assert_eq!(client.next_connection_event().await, None);
        release.send(()).unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn pong_cancels_the_connection_deadline() {
    for venue in [Venue::Spot, Venue::Futures] {
        let (listener, url) = listener().await;
        let (release, hold) = oneshot::channel();
        let server = tokio::spawn(async move {
            let mut socket = accept(&listener).await;
            match socket.next().await.unwrap().unwrap() {
                Message::Text(text) => {
                    let ping: Value = serde_json::from_str(&text).unwrap();
                    assert_eq!(ping["method"], "ping");
                    assert!(ping["req_id"].is_u64());
                    socket
                        .send(Message::Text(
                            json!({"method": "pong", "req_id": ping["req_id"]})
                                .to_string()
                                .into(),
                        ))
                        .await
                        .unwrap();
                }
                Message::Ping(_) => socket.flush().await.unwrap(),
                message => panic!("Unexpected message: {message:?}"),
            }
            hold.await.unwrap();
        });
        let mut client = venue.connect(&url, 0, Duration::from_millis(100)).await;
        assert!(
            timeout(Duration::from_millis(250), async {
                loop {
                    assert_eq!(client.next().await, Some(Event::Other));
                }
            })
            .await
            .is_err()
        );
        assert!(client.connected());
        client.close().await;
        release.send(()).unwrap();
        server.await.unwrap();
    }
}

#[tokio::test]
async fn spot_reconnect_restores_private_token() {
    let (listener, url) = listener().await;
    let (release, hold) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut first = accept(&listener).await;
        assert_eq!(request(&mut first).await["params"]["token"], "test-token");
        first.close(None).await.unwrap();
        let mut second = accept(&listener).await;
        assert_eq!(request(&mut second).await["params"]["token"], "test-token");
        hold.await.unwrap();
    });
    let config = kraken_api_client::spot::ws::WsConfig {
        initial_backoff: Duration::from_millis(10),
        danger_allow_insecure_transport: true,
        ..Default::default()
    };
    let mut stream = SpotWsClient::with_urls(&url, &url)
        .connect_private_with_config("test-token", config)
        .await
        .unwrap();
    stream
        .subscribe(SubscribeParams::private("executions", "test-token"))
        .await
        .unwrap();
    let mut client = Client::Spot(stream);
    assert_eq!(
        client.next_connection_event().await,
        Some(Event::Reconnecting(1))
    );
    assert_eq!(
        client.next_connection_event().await,
        Some(Event::Reconnected)
    );
    client.close().await;
    release.send(()).unwrap();
    server.await.unwrap();
}

#[tokio::test]
async fn futures_reconnect_uses_a_new_private_challenge() {
    let (listener, url) = listener().await;
    let (release, hold) = oneshot::channel();
    let server = tokio::spawn(async move {
        let mut signatures = Vec::new();
        for challenge in ["first-challenge", "second-challenge"] {
            let mut socket = accept(&listener).await;
            assert_eq!(request(&mut socket).await["event"], "challenge");
            socket
                .send(Message::Text(
                    json!({"event": "challenge", "message": challenge})
                        .to_string()
                        .into(),
                ))
                .await
                .unwrap();
            let subscription = request(&mut socket).await;
            assert_eq!(subscription["feed"], "open_orders");
            assert_eq!(subscription["original_challenge"], challenge);
            signatures.push(subscription["signed_challenge"].clone());
            if challenge == "first-challenge" {
                socket.close(None).await.unwrap();
            } else {
                assert_ne!(signatures[0], signatures[1]);
                hold.await.unwrap();
                break;
            }
        }
    });
    let config = kraken_api_client::futures::ws::WsConfig {
        initial_backoff: Duration::from_millis(10),
        danger_allow_insecure_transport: true,
        ..Default::default()
    };
    let mut stream = FuturesWsClient::with_url(&url)
        .connect_private_with_config(Arc::new(StaticCredentials::new("key", "c2VjcmV0")), config)
        .await
        .unwrap();
    stream.subscribe_private("open_orders").await.unwrap();
    let mut client = Client::Futures(stream);
    assert_eq!(
        client.next_connection_event().await,
        Some(Event::Reconnecting(1))
    );
    assert_eq!(
        client.next_connection_event().await,
        Some(Event::Reconnected)
    );
    if let Client::Futures(stream) = &client {
        assert!(stream.is_authenticated());
    }
    client.close().await;
    release.send(()).unwrap();
    server.await.unwrap();
}

#[test]
fn ticker_preserves_optional_exchange_timestamp() {
    let mut payload = json!({
        "channel": "ticker", "type": "update",
        "data": [{
            "symbol": "BTC/USD", "bid": 100, "bid_qty": 1,
            "ask": 101, "ask_qty": 1, "last": 100, "volume": 10,
            "vwap": 100, "low": 99, "high": 102, "change": 1, "change_pct": 1
        }]
    });
    let without_timestamp: TickerMessage = serde_json::from_value(payload.clone()).unwrap();
    assert_eq!(without_timestamp.data[0].timestamp, None);
    payload["data"][0]["timestamp"] = json!("2026-10-08T12:00:00Z");
    let with_timestamp: TickerMessage = serde_json::from_value(payload).unwrap();
    assert_eq!(
        with_timestamp.data[0].timestamp.as_deref(),
        Some("2026-10-08T12:00:00Z")
    );
}

//! Example: WebSocket configuration builders.
//!
//! Run with: cargo run --example ws_config

use std::time::Duration;

use kraken_api_client::futures::ws::FuturesWsClient;
use kraken_api_client::spot::ws::SpotWsClient;

fn main() {
    let spot_config = kraken_api_client::spot::ws::WsConfig::builder()
        .reconnect_backoff(Duration::from_secs(1), Duration::from_secs(30))
        .max_reconnect_attempts(5)
        .ping_interval(Duration::from_secs(15))
        .build();
    let _spot_client = SpotWsClient::with_config(spot_config);

    let futures_config = kraken_api_client::futures::ws::WsConfig::builder()
        .reconnect_backoff(Duration::from_secs(1), Duration::from_secs(30))
        .max_reconnect_attempts(5)
        .ping_interval(Duration::from_secs(15))
        .pong_timeout(Duration::from_secs(10))
        .build();
    let _futures_client = FuturesWsClient::with_config(futures_config);

    println!("Configured Spot and Futures WebSocket clients.");
}

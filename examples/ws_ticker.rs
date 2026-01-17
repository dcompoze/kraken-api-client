//! Example: Streaming ticker data via WebSocket.
//!
//! This example demonstrates how to use the Kraken WebSocket API to
//! receive real-time ticker updates for trading pairs.
//!
//! Run with: cargo run --example ws_ticker

use futures_util::StreamExt;
use kraken_api_client::spot::ws::messages::{channels, SubscribeParams};
use kraken_api_client::spot::ws::{SpotWsClient, WsMessageEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging (optional)
    tracing_subscriber::fmt::init();

    println!("Connecting to Kraken WebSocket...");

    // Create WebSocket client and connect to public endpoint
    let client = SpotWsClient::new();
    let mut stream = client.connect_public().await?;

    println!("Connected! Subscribing to ticker...");

    // Subscribe to ticker channel for BTC/USD and ETH/USD
    let ticker_params = SubscribeParams::public(
        channels::TICKER,
        vec!["BTC/USD".into(), "ETH/USD".into()],
    );
    stream.subscribe(ticker_params).await?;

    println!("Subscribed! Waiting for ticker updates...\n");
    println!("Press Ctrl+C to exit.\n");

    // Process incoming messages
    let mut message_count = 0;
    while let Some(msg) = stream.next().await {
        match msg {
            Ok(event) => match event {
                WsMessageEvent::Status(status) => {
                    if let Some(data) = status.data.first() {
                        println!("[Status] System: {}", data.system);
                    }
                }
                WsMessageEvent::Heartbeat(_) => {
                    // Heartbeats are normal, just ignore or log at debug level
                }
                WsMessageEvent::Pong(pong) => {
                    println!("[Pong] req_id: {:?}", pong.req_id);
                }
                WsMessageEvent::Subscribed(sub) => {
                    println!(
                        "[Subscribed] channel={}, symbol={:?}",
                        sub.channel,
                        sub.symbol
                    );
                }
                WsMessageEvent::Unsubscribed(sub) => {
                    println!(
                        "[Unsubscribed] channel={}, symbol={:?}",
                        sub.channel,
                        sub.symbol
                    );
                }
                WsMessageEvent::ChannelData(data) => {
                    // Parse ticker data from the channel message
                    let channel = data["channel"].as_str().unwrap_or("");
                    if channel == "ticker" {
                        if let Some(ticker_data) = data["data"].as_array() {
                            for ticker in ticker_data {
                                let symbol = ticker["symbol"].as_str().unwrap_or("?");
                                let bid = ticker["bid"].as_str().unwrap_or("?");
                                let ask = ticker["ask"].as_str().unwrap_or("?");
                                let last = ticker["last"].as_str().unwrap_or("?");
                                let volume = ticker["volume"].as_str().unwrap_or("?");
                                let vwap = ticker["vwap"].as_str().unwrap_or("?");

                                println!(
                                    "[Ticker] {} | Bid: {} | Ask: {} | Last: {} | Vol: {} | VWAP: {}",
                                    symbol, bid, ask, last, volume, vwap
                                );
                            }
                        }
                    } else if !channel.is_empty() {
                        println!("[Channel: {}] {:?}", channel, data);
                    }

                    message_count += 1;
                    if message_count >= 50 {
                        println!("\nReceived 50 messages, closing...");
                        break;
                    }
                }
                WsMessageEvent::Error { method, error, req_id } => {
                    println!("[Error] method={}, error={}, req_id={:?}", method, error, req_id);
                }
                WsMessageEvent::Disconnected => {
                    println!("[Disconnected] Connection closed");
                    break;
                }
                WsMessageEvent::Reconnecting { attempt } => {
                    println!("[Reconnecting] Attempt {}", attempt);
                }
                WsMessageEvent::Reconnected => {
                    println!("[Reconnected] Connection restored");
                }
                _ => {
                    // Handle other events (trading responses, etc.)
                }
            },
            Err(e) => {
                println!("[Error] {:?}", e);
                break;
            }
        }
    }

    // Clean up
    stream.close().await?;
    println!("Connection closed.");

    Ok(())
}

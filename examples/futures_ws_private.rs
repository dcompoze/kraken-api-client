//! Example: Futures WebSocket private feeds.
//!
//! Run with: cargo run --example futures_ws_private

use std::sync::Arc;

use futures_util::StreamExt;
use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::futures::ws::{FuturesWsClient, FuturesWsEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = match EnvCredentials::try_from_env() {
        Some(creds) => Arc::new(creds),
        None => {
            println!("Set KRAKEN_API_KEY and KRAKEN_API_SECRET to run this example.");
            return Ok(());
        }
    };

    let client = FuturesWsClient::new();
    let mut stream = client.connect_private(credentials).await?;

    stream.subscribe_private("open_orders").await?;
    stream.subscribe_private("fills").await?;
    stream.subscribe_private("open_positions").await?;
    stream.subscribe_private("balances").await?;

    let mut seen = 0;
    while let Some(msg) = stream.next().await {
        match msg? {
            FuturesWsEvent::OpenOrders(orders) => {
                let count = orders.orders.as_ref().map(|v| v.len()).unwrap_or(0);
                println!("Open orders update: {}", count);
            }
            FuturesWsEvent::Fills(fills) => {
                let count = fills.fills.as_ref().map(|v| v.len()).unwrap_or(0);
                println!("Fills update: {}", count);
            }
            FuturesWsEvent::OpenPositions(positions) => {
                let count = positions.positions.as_ref().map(|v| v.len()).unwrap_or(0);
                println!("Positions update: {}", count);
            }
            FuturesWsEvent::Balances(balances) => {
                println!(
                    "Balances update: balance={:?} available={:?}",
                    balances.balance, balances.available
                );
            }
            FuturesWsEvent::Subscribed(sub) => {
                println!("Subscribed: {}", sub.feed);
            }
            FuturesWsEvent::Error(err) => {
                println!("Error: {}", err.message);
            }
            FuturesWsEvent::Disconnected => break,
            _ => {}
        }
        seen += 1;
        if seen >= 25 {
            break;
        }
    }

    stream.close().await?;
    Ok(())
}

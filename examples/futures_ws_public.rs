//! Example: Futures WebSocket public feeds.
//!
//! Run with: cargo run --example futures_ws_public

use futures_util::StreamExt;
use kraken_api_client::futures::ws::{FuturesWsClient, FuturesWsEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FuturesWsClient::new();
    let mut stream = client.connect_public().await?;

    stream.subscribe_public("ticker", vec!["PI_XBTUSD"]).await?;
    stream.subscribe_public("book", vec!["PI_XBTUSD"]).await?;
    stream.subscribe_public("trade", vec!["PI_XBTUSD"]).await?;

    let mut seen = 0;
    while let Some(msg) = stream.next().await {
        match msg? {
            FuturesWsEvent::Ticker(ticker) => {
                println!("Ticker {}: last={:?}", ticker.product_id, ticker.last);
            }
            FuturesWsEvent::Book(book) => {
                if let Some(first) = book.bids.first() {
                    println!("Book update: bid={} size={}", first.price, first.qty);
                }
            }
            FuturesWsEvent::Trade(trade) => {
                println!(
                    "Trade {}: {:?} @ {:?}",
                    trade.product_id, trade.qty, trade.price
                );
            }
            FuturesWsEvent::Subscribed(sub) => {
                println!("Subscribed: {} ({:?})", sub.feed, sub.product_ids);
            }
            FuturesWsEvent::Info(info) => {
                println!("Info: {:?}", info.version);
            }
            FuturesWsEvent::Error(err) => {
                println!("Error: {}", err.message);
            }
            FuturesWsEvent::Disconnected => break,
            _ => {}
        }
        seen += 1;
        if seen >= 50 {
            break;
        }
    }

    stream.close().await?;
    Ok(())
}

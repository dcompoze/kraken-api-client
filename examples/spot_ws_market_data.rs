//! Example: Spot WebSocket market data with typed parsing.
//!
//! Run with: cargo run --example spot_ws_market_data

use futures_util::StreamExt;
use kraken_api_client::spot::ws::messages::{
    channels, BookMessage, InstrumentMessage, OhlcMessage, SubscribeParams, TickerMessage,
    TradeMessage,
};
use kraken_api_client::spot::ws::{SpotWsClient, WsMessageEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SpotWsClient::new();
    let mut stream = client.connect_public().await?;

    stream
        .subscribe(SubscribeParams::public(
            channels::TICKER,
            vec!["BTC/USD".into()],
        ))
        .await?;
    stream
        .subscribe(
            SubscribeParams::public(channels::BOOK, vec!["BTC/USD".into()])
                .with_snapshot(true)
                .with_depth(10),
        )
        .await?;
    stream
        .subscribe(SubscribeParams::public(
            channels::TRADE,
            vec!["BTC/USD".into()],
        ))
        .await?;
    stream
        .subscribe(SubscribeParams::public(
            channels::OHLC,
            vec!["BTC/USD".into()],
        ))
        .await?;
    stream
        .subscribe(SubscribeParams::public(channels::INSTRUMENT, vec![]))
        .await?;

    let mut seen = 0;
    while let Some(msg) = stream.next().await {
        let event = msg?;
        match event {
            WsMessageEvent::ChannelData(value) => {
                let channel = value
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                match channel {
                    channels::TICKER => {
                        if let Ok(ticker) = serde_json::from_value::<TickerMessage>(value) {
                            if let Some(first) = ticker.data.first() {
                                println!(
                                    "Ticker {}: bid={}, ask={}, last={}",
                                    first.symbol, first.bid, first.ask, first.last
                                );
                            }
                        }
                    }
                    channels::BOOK => {
                        if let Ok(book) = serde_json::from_value::<BookMessage>(value) {
                            if let Some(first) = book.data.first() {
                                println!(
                                    "Book {}: bids={}, asks={}",
                                    first.symbol,
                                    first.bids.len(),
                                    first.asks.len()
                                );
                            }
                        }
                    }
                    channels::TRADE => {
                        if let Ok(trades) = serde_json::from_value::<TradeMessage>(value) {
                            if let Some(first) = trades.data.first() {
                                println!(
                                    "Trade {}: {} {} @ {}",
                                    first.symbol, first.side, first.qty, first.price
                                );
                            }
                        }
                    }
                    channels::OHLC => {
                        if let Ok(ohlc) = serde_json::from_value::<OhlcMessage>(value) {
                            if let Some(first) = ohlc.data.first() {
                                println!(
                                    "OHLC {}: O={} H={} L={} C={}",
                                    first.symbol, first.open, first.high, first.low, first.close
                                );
                            }
                        }
                    }
                    channels::INSTRUMENT => {
                        if let Ok(instr) = serde_json::from_value::<InstrumentMessage>(value) {
                            println!(
                                "Instrument update: assets={}, pairs={}",
                                instr.data.assets.len(),
                                instr.data.pairs.len()
                            );
                        }
                    }
                    other => {
                        println!("Unhandled channel: {}", other);
                    }
                }
                seen += 1;
                if seen >= 50 {
                    break;
                }
            }
            WsMessageEvent::Status(status) => {
                println!("Status: {:?}", status.data.first());
            }
            WsMessageEvent::Heartbeat(_) => {}
            WsMessageEvent::Disconnected => break,
            _ => {}
        }
    }

    stream.close().await?;
    Ok(())
}

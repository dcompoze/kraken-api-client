//! Example: Fetching public market data from Kraken.
//!
//! This example demonstrates how to use the Kraken REST API to fetch
//! publicly available market data without authentication.
//!
//! Run with: cargo run --example public_data

use kraken_api_client::spot::rest::public::{
    AssetInfoRequest, AssetPairsRequest, OhlcRequest, OrderBookRequest, RecentSpreadsRequest,
    RecentTradesRequest,
};
use kraken_api_client::spot::rest::SpotRestClient;
use kraken_api_client::types::OhlcInterval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client for public endpoints (no credentials needed)
    let client = SpotRestClient::new();

    // Get server time
    println!("=== Server Time ===");
    let time = client.get_server_time().await?;
    println!("Unix time: {}", time.unixtime);
    println!("RFC1123: {}", time.rfc1123);

    // Get system status
    println!("\n=== System Status ===");
    let status = client.get_system_status().await?;
    println!("Status: {}", status.status);
    println!("Timestamp: {}", status.timestamp);

    // Get asset pairs (filtered to a few)
    println!("\n=== Asset Pairs ===");
    let pairs_request = AssetPairsRequest {
        pair: Some("BTC/USD,ETH/USD".into()),
        ..Default::default()
    };
    let pairs = client.get_asset_pairs(Some(&pairs_request)).await?;
    for (name, pair) in pairs.iter().take(3) {
        println!(
            "{}: base={}, quote={}, decimals={}",
            name, pair.base, pair.quote, pair.pair_decimals
        );
    }

    // Get asset info for a few assets
    println!("\n=== Asset Info ===");
    let assets_request = AssetInfoRequest::for_assets("XBT,ETH,USD");
    let assets = client.get_assets(Some(&assets_request)).await?;
    for (name, asset) in assets.iter() {
        println!(
            "{}: class={}, decimals={}, display_decimals={}",
            name, asset.aclass, asset.decimals, asset.display_decimals
        );
    }

    // Get ticker information
    println!("\n=== Ticker (BTC/USD) ===");
    let ticker = client.get_ticker("XBTUSD").await?;
    if let Some((pair, info)) = ticker.iter().next() {
        println!("Pair: {}", pair);
        println!("  Ask: {:?} (best: {})", info.a, info.ask_price());
        println!("  Bid: {:?} (best: {})", info.b, info.bid_price());
        println!("  Last: {:?} (last: {})", info.c, info.last_price());
        println!("  Volume: {:?} (today: {})", info.v, info.volume_today());
        println!("  VWAP: {:?}", info.p);
        println!("  Trades: {:?}", info.t);
        println!("  High: {:?}", info.h);
        println!("  Low: {:?}", info.l);
        println!("  Open: {:?}", info.o);
    }

    // Get OHLC data
    println!("\n=== OHLC (BTC/USD, 1 hour) ===");
    let ohlc_request = OhlcRequest::new("XBTUSD").interval(OhlcInterval::Hour1);
    let ohlc = client.get_ohlc(&ohlc_request).await?;
    println!("Last cursor: {}", ohlc.last);
    // Get the first (and usually only) pair's data
    if let Some((_pair, candles)) = ohlc.data.iter().next() {
        for candle in candles.iter().rev().take(3) {
            println!(
                "  Time: {}, O: {}, H: {}, L: {}, C: {}, Vol: {}",
                candle.time, candle.open, candle.high, candle.low, candle.close, candle.volume
            );
        }
    }

    // Get order book
    println!("\n=== Order Book (BTC/USD, depth=5) ===");
    let book_request = OrderBookRequest {
        pair: "XBTUSD".into(),
        count: Some(5),
    };
    let books = client.get_order_book(&book_request).await?;
    if let Some((pair, book)) = books.iter().next() {
        println!("Pair: {}", pair);
        println!("Asks (lowest first):");
        for ask in book.asks.iter().take(3) {
            println!("  {} @ {} (timestamp: {})", ask.volume, ask.price, ask.timestamp);
        }
        println!("Bids (highest first):");
        for bid in book.bids.iter().take(3) {
            println!("  {} @ {} (timestamp: {})", bid.volume, bid.price, bid.timestamp);
        }
    }

    // Get recent trades
    println!("\n=== Recent Trades (BTC/USD) ===");
    let trades_request = RecentTradesRequest::new("XBTUSD").count(5);
    let trades_response = client.get_recent_trades(&trades_request).await?;
    println!("Last cursor: {}", trades_response.last);
    // Get the first (and usually only) pair's trades
    if let Some((_pair, trades)) = trades_response.trades.iter().next() {
        for trade in trades.iter().take(5) {
            println!(
                "  {} {} @ {} (time: {})",
                trade.side, trade.volume, trade.price, trade.time
            );
        }
    }

    // Get recent spreads
    println!("\n=== Recent Spreads (BTC/USD) ===");
    let spreads_request = RecentSpreadsRequest::new("XBTUSD");
    let spreads = client.get_recent_spreads(&spreads_request).await?;
    println!("Last cursor: {}", spreads.last);
    if let Some((_pair, entries)) = spreads.spreads.iter().next() {
        for entry in entries.iter().take(5) {
            println!("  Bid: {} Ask: {} Time: {}", entry.bid, entry.ask, entry.time);
        }
    }

    println!("\nDone!");
    Ok(())
}

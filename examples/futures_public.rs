//! Example: Fetching Kraken Futures public data.
//!
//! Run with: cargo run --example futures_public

use kraken_api_client::futures::rest::FuturesRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = FuturesRestClient::new();

    println!("=== Futures Tickers (first 5) ===");
    let tickers = client.get_tickers().await?;
    for ticker in tickers.iter().take(5) {
        println!("{}: {}", ticker.symbol, ticker.last);
    }

    println!("\n=== Single Ticker (PI_XBTUSD) ===");
    if let Some(ticker) = client.get_ticker("PI_XBTUSD").await? {
        println!(
            "{}: last={:?}, mark={:?}",
            ticker.symbol, ticker.last, ticker.mark_price
        );
    }

    println!("\n=== Order Book (PI_XBTUSD) ===");
    let book = client.get_orderbook("PI_XBTUSD").await?;
    println!("Bids: {}, Asks: {}", book.bids.len(), book.asks.len());

    println!("\n=== Trade History (PI_XBTUSD) ===");
    let trades = client.get_trade_history("PI_XBTUSD", None).await?;
    println!("Trades: {}", trades.len());

    println!("\n=== Instruments (first 5) ===");
    let instruments = client.get_instruments().await?;
    for instrument in instruments.iter().take(5) {
        println!("{}: {:?}", instrument.symbol, instrument.contract_type);
    }

    Ok(())
}

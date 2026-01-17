//! Example: Spot private account endpoints.
//!
//! Run with: cargo run --example spot_private_account

use std::env;
use std::sync::Arc;

use kraken_api_client::auth::{EnvCredentials, IncreasingNonce};
use kraken_api_client::spot::rest::private::{
    ClosedOrdersRequest, LedgersRequest, OpenOrdersRequest, OpenPositionsRequest, QueryOrdersRequest,
    TradeBalanceRequest, TradeVolumeRequest, TradesHistoryRequest,
};
use kraken_api_client::spot::rest::SpotRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = match EnvCredentials::try_from_env() {
        Some(creds) => Arc::new(creds),
        None => {
            println!("Set KRAKEN_API_KEY and KRAKEN_API_SECRET to run this example.");
            return Ok(());
        }
    };

    let client = SpotRestClient::builder()
        .credentials(credentials)
        .nonce_provider(Arc::new(IncreasingNonce::new()))
        .user_agent("kraken-api-client-examples/spot_private_account")
        .max_retries(2)
        .build();

    println!("=== Account Balance ===");
    let balances = client.get_account_balance().await?;
    println!("Assets: {}", balances.len());

    println!("\n=== Extended Balance ===");
    let extended = client.get_extended_balance().await?;
    println!("Extended assets: {}", extended.balances.len());

    println!("\n=== Trade Balance ===");
    let trade_balance = client
        .get_trade_balance(Some(&TradeBalanceRequest {
            asset: Some("ZUSD".into()),
        }))
        .await?;
    println!("Equity: {}", trade_balance.equity);

    println!("\n=== Open Orders ===");
    let open_orders = client
        .get_open_orders(Some(&OpenOrdersRequest {
            trades: Some(true),
            userref: None,
        }))
        .await?;
    println!("Open orders: {}", open_orders.open.len());

    println!("\n=== Closed Orders ===");
    let closed_orders = client
        .get_closed_orders(Some(&ClosedOrdersRequest {
            trades: Some(false),
            userref: None,
            start: None,
            end: None,
            ofs: Some(0),
            closetime: Some("close".into()),
        }))
        .await?;
    println!("Closed orders: {}", closed_orders.closed.len());

    if let Ok(txids) = env::var("KRAKEN_QUERY_TXIDS") {
        println!("\n=== Query Orders ===");
        let query = QueryOrdersRequest {
            txid: txids,
            trades: Some(true),
            userref: None,
        };
        let orders = client.query_orders(&query).await?;
        println!("Queried orders: {}", orders.len());
    } else {
        println!("\nSet KRAKEN_QUERY_TXIDS to query specific orders.");
    }

    println!("\n=== Trades History ===");
    let trades = client
        .get_trades_history(Some(&TradesHistoryRequest {
            trade_type: None,
            trades: Some(true),
            start: None,
            end: None,
            ofs: Some(0),
        }))
        .await?;
    println!("Trades: {}", trades.trades.len());

    println!("\n=== Open Positions ===");
    let positions = client
        .get_open_positions(Some(&OpenPositionsRequest {
            docalcs: Some(true),
            txid: None,
        }))
        .await?;
    println!("Positions: {}", positions.len());

    println!("\n=== Ledgers ===");
    let ledgers = client
        .get_ledgers(Some(&LedgersRequest {
            asset: None,
            aclass: None,
            ledger_type: None,
            start: None,
            end: None,
            ofs: Some(0),
            without_count: None,
        }))
        .await?;
    println!("Ledgers: {}", ledgers.ledger.len());

    println!("\n=== Trade Volume ===");
    let volume = client
        .get_trade_volume(Some(&TradeVolumeRequest {
            pair: Some("XBTUSD".into()),
        }))
        .await?;
    println!("Volume currency: {}", volume.currency);

    Ok(())
}

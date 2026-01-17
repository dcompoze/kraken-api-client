//! Example: Spot private trading endpoints.
//!
//! Run with: cargo run --example spot_private_trading

use std::env;
use std::str::FromStr;
use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::private::CancelOrderRequest;
use kraken_api_client::spot::rest::SpotRestClient;
use kraken_api_client::{BuySell, OrderType};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = match EnvCredentials::try_from_env() {
        Some(creds) => Arc::new(creds),
        None => {
            println!("Set KRAKEN_API_KEY and KRAKEN_API_SECRET to run this example.");
            return Ok(());
        }
    };

    let client = SpotRestClient::builder().credentials(credentials).build();

    // Validate-only order (won't be submitted).
    let add_request = kraken_api_client::spot::rest::private::AddOrderRequest::new(
        "XBTUSD",
        BuySell::Buy,
        OrderType::Limit,
        Decimal::from_str("0.001")?,
    )
    .price(Decimal::from_str("50000")?)
    .time_in_force("GTC")
    .post_only()
    .validate(true);

    let add_result = client.add_order(&add_request).await?;
    println!("Validated order: {}", add_result.descr.order);

    if let Ok(txid) = env::var("KRAKEN_CANCEL_ORDER_ID") {
        let cancel_request = CancelOrderRequest::new(txid);
        let cancel_result = client.cancel_order(&cancel_request).await?;
        println!("Cancelled orders: {}", cancel_result.count);
    } else {
        println!("Set KRAKEN_CANCEL_ORDER_ID to cancel a specific order.");
    }

    if env::var("KRAKEN_CANCEL_ALL").is_ok() {
        let cancel_all = client.cancel_all_orders().await?;
        println!("Cancel-all result: {}", cancel_all.count);
    } else {
        println!("Set KRAKEN_CANCEL_ALL=1 to cancel all open orders.");
    }

    Ok(())
}

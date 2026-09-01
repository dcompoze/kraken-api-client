//! Batch order placement and amendment in validate mode.
//!
//! Run with: cargo run --example spot_batch_orders

use std::str::FromStr;
use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::SpotRestClient;
use kraken_api_client::spot::rest::private::{
    AddOrderBatchRequest, AmendOrderRequest, BatchOrder, CancelAllOrdersAfterRequest,
};
use kraken_api_client::types::{BuySell, OrderType};
use rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let credentials = Arc::new(EnvCredentials::from_env());
    let client = SpotRestClient::builder().credentials(credentials).build()?;

    // Place two orders in one batch, validate only.
    let orders = vec![
        BatchOrder::new(BuySell::Buy, OrderType::Limit, Decimal::from_str("0.001")?)
            .price(Decimal::from_str("28800.0")?),
        BatchOrder::new(BuySell::Sell, OrderType::Limit, Decimal::from_str("0.001")?)
            .price(Decimal::from_str("70100.0")?),
    ];
    let request = AddOrderBatchRequest::new("XBTUSD", orders).validate(true);

    match client.add_order_batch(&request).await {
        Ok(result) => {
            for order in result.orders {
                println!("Order: {:?} {:?}", order.txid, order.descr);
            }
        }
        Err(e) => println!("Batch validation failed: {}", e),
    }

    // Amend an order in place (requires a real open order ID).
    let amend = AmendOrderRequest::by_txid("OHYO67-6LP66-HMQ437").limit_price("29000.0");
    match client.amend_order(&amend).await {
        Ok(result) => println!("Amended: {}", result.amend_id),
        Err(e) => println!("Amend failed (expected without an open order): {}", e),
    }

    // Set a dead man's switch, then disable it.
    let timer = client
        .cancel_all_orders_after(&CancelAllOrdersAfterRequest::new(60))
        .await?;
    println!("Timer set, triggers at: {}", timer.trigger_time);

    client
        .cancel_all_orders_after(&CancelAllOrdersAfterRequest::disable())
        .await?;
    println!("Timer disabled");

    Ok(())
}

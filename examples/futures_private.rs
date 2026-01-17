//! Example: Futures private endpoints (accounts + trading).
//!
//! Run with: cargo run --example futures_private

use std::env;
use std::sync::Arc;

use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::futures::rest::{
    BatchOrderRequest, EditOrderRequest, FillsRequest, FuturesRestClient, SendOrderRequest,
};
use kraken_api_client::BuySell;
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

    let mut builder = FuturesRestClient::builder().credentials(credentials);
    if env::var("KRAKEN_FUTURES_DEMO").is_ok() {
        builder = builder.use_demo();
    }
    let client = builder.build();

    println!("=== Accounts ===");
    let accounts = client.get_accounts().await?;
    println!("Accounts: {}", accounts.accounts.len());

    println!("\n=== Open Positions ===");
    let positions = client.get_open_positions().await?;
    println!("Positions: {}", positions.len());

    println!("\n=== Open Orders ===");
    let orders = client.get_open_orders().await?;
    println!("Open orders: {}", orders.len());

    println!("\n=== Fills ===");
    let fills = client
        .get_fills(Some(&FillsRequest {
            symbol: Some("PI_XBTUSD".into()),
            last_fill_time: None,
        }))
        .await?;
    println!("Fills: {}", fills.len());

    if env::var("KRAKEN_FUTURES_SEND_ORDER").is_ok() {
        println!("\n=== Send Order (Dangerous) ===");
        let order = SendOrderRequest::limit(
            "PI_XBTUSD",
            BuySell::Buy,
            Decimal::from(10),
            Decimal::from(50000),
        )
        .reduce_only(true)
        .cli_ord_id("example-order-1");
        let response = client.send_order(&order).await?;
        println!("Send order result: {}", response.send_status.status);
    } else {
        println!("Set KRAKEN_FUTURES_SEND_ORDER=1 to submit an order.");
    }

    if let Ok(order_id) = env::var("KRAKEN_FUTURES_EDIT_ORDER_ID") {
        let edit = EditOrderRequest::by_order_id(order_id)
            .size(Decimal::from(20))
            .limit_price(Decimal::from(51000));
        let response = client.edit_order(&edit).await?;
        println!("Edit order result: {}", response.edit_status.status);
    }

    if let Ok(order_id) = env::var("KRAKEN_FUTURES_CANCEL_ORDER_ID") {
        let response = client.cancel_order(&order_id).await?;
        println!("Cancel order result: {}", response.cancel_status.status);
    }

    if let Ok(cli_ord_id) = env::var("KRAKEN_FUTURES_CANCEL_CLI_ORD_ID") {
        let response = client.cancel_order_by_cli_ord_id(&cli_ord_id).await?;
        println!("Cancel by cliOrdId result: {}", response.cancel_status.status);
    }

    if env::var("KRAKEN_FUTURES_CANCEL_ALL").is_ok() {
        let response = client.cancel_all_orders().await?;
        println!("Cancel all result: {}", response.result);
    }

    if let Ok(symbol) = env::var("KRAKEN_FUTURES_CANCEL_ALL_SYMBOL") {
        let response = client.cancel_all_orders_for_symbol(&symbol).await?;
        println!("Cancel all for symbol result: {}", response.result);
    }

    if let Ok(after) = env::var("KRAKEN_FUTURES_CANCEL_ALL_AFTER") {
        let seconds: u32 = after.parse().unwrap_or(10);
        let response = client.cancel_all_orders_after(seconds).await?;
        println!("Cancel all after result: {}", response.result);
    }

    if env::var("KRAKEN_FUTURES_BATCH").is_ok() {
        let batch = BatchOrderRequest::new()
            .place(SendOrderRequest::market(
                "PI_XBTUSD",
                BuySell::Sell,
                Decimal::from(5),
            ))
            .cancel_by_cli_ord_id("example-order-1");
        let response = client.batch_order(&batch).await?;
        println!("Batch order result: {}", response.result);
    }

    // Example of constructing a stop order without submitting it.
    let stop = SendOrderRequest::stop(
        "PI_XBTUSD",
        BuySell::Sell,
        Decimal::from(5),
        Decimal::from(49000),
    )
    .trigger_signal("mark");
    println!("Constructed stop order type: {:?}", stop.order_type);

    Ok(())
}

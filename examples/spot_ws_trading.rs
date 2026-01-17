//! Example: Spot WebSocket private trading and user data.
//!
//! Run with: cargo run --example spot_ws_trading

use std::env;
use std::str::FromStr;
use std::sync::Arc;

use futures_util::StreamExt;
use kraken_api_client::auth::EnvCredentials;
use kraken_api_client::spot::rest::SpotRestClient;
use kraken_api_client::spot::ws::messages::{
    channels, AddOrderParams, BalancesMessage, CancelAllParams, CancelOrderParams,
    EditOrderParams, ExecutionsMessage, SubscribeParams,
};
use kraken_api_client::spot::ws::{SpotWsClient, WsMessageEvent};
use kraken_api_client::types::TimeInForce;
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

    let rest = SpotRestClient::builder().credentials(credentials).build();
    let token = rest.get_websocket_token().await?.token;

    let ws_client = SpotWsClient::new();
    let mut stream = ws_client.connect_private(token.clone()).await?;

    stream
        .subscribe(SubscribeParams::private(channels::EXECUTIONS, token.clone()))
        .await?;
    stream
        .subscribe(SubscribeParams::private(channels::BALANCES, token.clone()))
        .await?;

    // Optional trading commands.
    if env::var("KRAKEN_WS_ADD_ORDER").is_ok() {
        let add_params = AddOrderParams::new(OrderType::Limit, BuySell::Buy, "XBT/USD", token.clone())
            .order_qty(Decimal::from_str("0.001")?)
            .limit_price(Decimal::from_str("50000")?)
            .time_in_force(TimeInForce::GTC)
            .post_only(true)
            .validate(true);
        let req_id = stream.add_order(add_params).await?;
        println!("Add order request id: {}", req_id);
    } else {
        println!("Set KRAKEN_WS_ADD_ORDER=1 to send a validate-only add order.");
    }

    if let Ok(order_id) = env::var("KRAKEN_WS_CANCEL_ORDER_ID") {
        let params = CancelOrderParams::by_order_id(vec![order_id], token.clone());
        let req_id = stream.cancel_order(params).await?;
        println!("Cancel order request id: {}", req_id);
    }

    if let Ok(order_id) = env::var("KRAKEN_WS_EDIT_ORDER_ID") {
        let params = EditOrderParams::new(order_id, token.clone())
            .order_qty(Decimal::from_str("0.002")?)
            .limit_price(Decimal::from_str("51000")?);
        let req_id = stream.edit_order(params).await?;
        println!("Edit order request id: {}", req_id);
    }

    if env::var("KRAKEN_WS_CANCEL_ALL").is_ok() {
        let req_id = stream.cancel_all_orders(CancelAllParams::new(token.clone())).await?;
        println!("Cancel all request id: {}", req_id);
    }

    // Read a handful of events.
    let mut seen = 0;
    while let Some(msg) = stream.next().await {
        match msg? {
            WsMessageEvent::ChannelData(value) => {
                let channel = value
                    .get("channel")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                match channel {
                    channels::EXECUTIONS => {
                        if let Ok(execs) = serde_json::from_value::<ExecutionsMessage>(value) {
                            println!("Executions: {}", execs.data.len());
                        }
                    }
                    channels::BALANCES => {
                        if let Ok(balances) = serde_json::from_value::<BalancesMessage>(value) {
                            println!("Balances: {}", balances.data.len());
                        }
                    }
                    _ => {}
                }
                seen += 1;
                if seen >= 25 {
                    break;
                }
            }
            WsMessageEvent::OrderAdded { req_id, result } => {
                println!("Order added (req_id={:?}): {}", req_id, result.order_id);
            }
            WsMessageEvent::OrderCancelled { req_id, result } => {
                println!("Order cancelled (req_id={:?}): {:?}", req_id, result.order_id);
            }
            WsMessageEvent::AllOrdersCancelled { req_id, result } => {
                println!("All orders cancelled (req_id={:?}): {}", req_id, result.count);
            }
            WsMessageEvent::OrderEdited { req_id, result } => {
                println!("Order edited (req_id={:?}): {}", req_id, result.order_id);
            }
            WsMessageEvent::Error { method, error, req_id } => {
                println!("WS error: method={}, error={}, req_id={:?}", method, error, req_id);
            }
            WsMessageEvent::Disconnected => break,
            _ => {}
        }
    }

    stream.close().await?;
    Ok(())
}

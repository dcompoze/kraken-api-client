//! Kraken Spot WebSocket v2 API client.
//!
//! Provides real-time market data and trading via WebSocket connections.
//!
//! # Example
//!
//! ```rust,ignore
//! use kraken_api_client::spot::ws::{SpotWsClient, WsMessageEvent};
//! use kraken_api_client::spot::ws::messages::{SubscribeParams, channels};
//! use futures_util::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = SpotWsClient::new();
//!     let mut stream = client.connect_public().await?;
//!
//!     // Subscribe to ticker updates
//!     stream.subscribe(SubscribeParams::public(channels::TICKER, vec!["BTC/USD".into()])).await?;
//!
//!     // Process messages
//!     while let Some(msg) = stream.next().await {
//!         match msg? {
//!             WsMessageEvent::ChannelData(data) => {
//!                 println!("Data: {:?}", data);
//!             }
//!             WsMessageEvent::Status(status) => {
//!                 println!("Status: {:?}", status);
//!             }
//!             WsMessageEvent::Disconnected => {
//!                 println!("Disconnected!");
//!                 break;
//!             }
//!             _ => {}
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```

mod client;
pub mod messages;
mod stream;

pub use client::{SpotWsClient, WsConfig, WsConfigBuilder};
pub use stream::{KrakenStream, WsMessageEvent};

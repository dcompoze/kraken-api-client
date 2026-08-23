//! Kraken Spot WebSocket v2 API client.
//!
//! Provides real-time market data and trading via WebSocket connections.

pub mod book;
mod client;
pub mod messages;
mod stream;

pub use book::{OrderBookState, book_checksum};
pub use client::{SpotWsClient, WsConfig, WsConfigBuilder};
pub use stream::{KrakenStream, WsMessageEvent};

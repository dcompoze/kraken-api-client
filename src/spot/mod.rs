//! Kraken Spot trading API clients.
//!
//! [`rest`] provides the REST client and [`ws`] provides the WebSocket v2 client.

pub mod rest;
pub mod ws;

pub use rest::SpotRestClient;
pub use ws::SpotWsClient;

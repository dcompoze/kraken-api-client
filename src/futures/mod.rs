//! Kraken Futures (derivatives) API client with REST and WebSocket interfaces.
//!
//! Uses the same `Credentials` types as Spot, but with a different signature scheme (see `auth`).
//! API docs: <https://docs.kraken.com/api/docs/futures-api>

mod auth;
pub mod rest;
pub mod types;
pub mod ws;

pub use auth::sign_futures_request;
pub use types::*;
pub use ws::{FuturesStream, FuturesWsClient, FuturesWsEvent};

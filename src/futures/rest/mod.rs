//! Futures REST API client.
//!
//! This module provides the REST API client for Kraken Futures trading.

mod client;
mod endpoints;
mod types;

pub use client::{FuturesRestClient, FuturesRestClientBuilder};
pub use endpoints::*;
pub use types::*;

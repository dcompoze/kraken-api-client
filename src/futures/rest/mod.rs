//! Futures REST API client.

mod client;
mod endpoints;
mod types;

pub use client::{FuturesRestClient, FuturesRestClientBuilder};
pub use endpoints::*;
pub use types::*;

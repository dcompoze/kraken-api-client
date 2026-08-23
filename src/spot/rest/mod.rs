//! Kraken Spot REST API client.
//!
//! The [`KrakenClient`] trait abstracts all REST operations for mocking and decoration.

mod client;
mod endpoints;
pub mod private;
pub mod public;
mod traits;

pub use client::{SpotRestClient, SpotRestClientBuilder};
pub use endpoints::*;
pub use traits::{KrakenClient, KrakenClientExt};

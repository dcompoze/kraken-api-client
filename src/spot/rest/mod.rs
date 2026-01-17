//! Kraken Spot REST API client.
//!
//! Provides access to all Kraken Spot trading REST endpoints.
//!
//! # Trait-based API
//!
//! The [`KrakenClient`] trait abstracts all REST API operations, enabling:
//! - Mock implementations for testing
//! - Decorator pattern (e.g., rate limiting wrapper)
//! - Alternative implementations
//!
//! ```rust,ignore
//! use kraken_api_client::spot::rest::{KrakenClient, SpotRestClient};
//!
//! async fn use_client<C: KrakenClient>(client: &C) -> Result<(), kraken_api_client::error::KrakenError> {
//!     let time = client.get_server_time().await?;
//!     println!("Server time: {}", time.unixtime);
//!     Ok(())
//! }
//! ```

mod client;
mod endpoints;
pub mod private;
pub mod public;
mod traits;

pub use client::{SpotRestClient, SpotRestClientBuilder};
pub use endpoints::*;
pub use traits::{KrakenClient, KrakenClientExt};

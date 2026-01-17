//! Kraken Spot trading API clients.
//!
//! This module provides:
//! - [`rest`] - REST API client for HTTP-based requests
//! - [`ws`] - WebSocket v2 API client for real-time streaming

pub mod rest;
pub mod ws;

pub use rest::SpotRestClient;
pub use ws::SpotWsClient;

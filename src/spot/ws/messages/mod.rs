//! WebSocket message types for Kraken v2 API.

mod admin;
mod base;
mod market_data;
mod trading;
mod user_data;

pub use admin::*;
pub use base::*;
pub use market_data::*;
pub use trading::*;
pub use user_data::*;

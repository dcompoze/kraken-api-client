//! Public REST API endpoints (no authentication required).

mod types;

pub use types::*;

use crate::error::KrakenError;
use crate::spot::rest::SpotRestClient;
use crate::spot::rest::endpoints::public;

impl SpotRestClient {
    /// Get the server time.
    ///
    /// This is useful for synchronizing local time and checking API availability.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use kraken_api_client::spot::rest::SpotRestClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let client = SpotRestClient::new();
    ///     let time = client.get_server_time().await?;
    ///     println!("Server time: {} ({})", time.unixtime, time.rfc1123);
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_server_time(&self) -> Result<ServerTime, KrakenError> {
        self.public_get(public::TIME).await
    }

    /// Get the system status.
    ///
    /// Returns the current system status and timestamp.
    pub async fn get_system_status(&self) -> Result<SystemStatus, KrakenError> {
        self.public_get(public::SYSTEM_STATUS).await
    }

    /// Get asset information.
    ///
    /// Returns information about the assets available on Kraken.
    ///
    /// # Arguments
    ///
    /// * `request` - Optional request parameters to filter assets.
    pub async fn get_assets(
        &self,
        request: Option<&AssetInfoRequest>,
    ) -> Result<std::collections::HashMap<String, AssetInfo>, KrakenError> {
        match request {
            Some(req) => self.public_get_with_params(public::ASSETS, req).await,
            None => self.public_get(public::ASSETS).await,
        }
    }

    /// Get tradable asset pairs.
    ///
    /// Returns information about the trading pairs available on Kraken.
    ///
    /// # Arguments
    ///
    /// * `request` - Optional request parameters to filter pairs.
    pub async fn get_asset_pairs(
        &self,
        request: Option<&AssetPairsRequest>,
    ) -> Result<std::collections::HashMap<String, AssetPair>, KrakenError> {
        match request {
            Some(req) => self.public_get_with_params(public::ASSET_PAIRS, req).await,
            None => self.public_get(public::ASSET_PAIRS).await,
        }
    }

    /// Get ticker information for one or more pairs.
    ///
    /// # Arguments
    ///
    /// * `pairs` - Comma-separated list of pairs (e.g., "XBTUSD,ETHUSD").
    pub async fn get_ticker(
        &self,
        pairs: &str,
    ) -> Result<std::collections::HashMap<String, TickerInfo>, KrakenError> {
        #[derive(serde::Serialize)]
        struct Params<'a> {
            pair: &'a str,
        }
        self.public_get_with_params(public::TICKER, &Params { pair: pairs })
            .await
    }

    /// Get OHLC (candlestick) data.
    ///
    /// Returns up to 720 OHLC data points for the specified pair and interval.
    ///
    /// # Arguments
    ///
    /// * `request` - OHLC request parameters.
    pub async fn get_ohlc(&self, request: &OhlcRequest) -> Result<OhlcResponse, KrakenError> {
        self.public_get_with_params(public::OHLC, request).await
    }

    /// Get order book for a pair.
    ///
    /// # Arguments
    ///
    /// * `request` - Order book request parameters.
    pub async fn get_order_book(
        &self,
        request: &OrderBookRequest,
    ) -> Result<std::collections::HashMap<String, OrderBook>, KrakenError> {
        self.public_get_with_params(public::DEPTH, request).await
    }

    /// Get recent trades for a pair.
    ///
    /// # Arguments
    ///
    /// * `request` - Recent trades request parameters.
    pub async fn get_recent_trades(
        &self,
        request: &RecentTradesRequest,
    ) -> Result<RecentTradesResponse, KrakenError> {
        self.public_get_with_params(public::TRADES, request).await
    }

    /// Get recent spreads for a pair.
    ///
    /// # Arguments
    ///
    /// * `request` - Recent spreads request parameters.
    pub async fn get_recent_spreads(
        &self,
        request: &RecentSpreadsRequest,
    ) -> Result<RecentSpreadsResponse, KrakenError> {
        self.public_get_with_params(public::SPREAD, request).await
    }
}

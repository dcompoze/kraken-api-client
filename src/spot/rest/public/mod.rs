//! Public REST API endpoints (no authentication required).

mod types;

pub use types::*;

use crate::error::KrakenError;
use crate::spot::rest::SpotRestClient;
use crate::spot::rest::endpoints::public;

impl SpotRestClient {
    /// Get the server time.
    pub async fn get_server_time(&self) -> Result<ServerTime, KrakenError> {
        self.public_get(public::TIME).await
    }

    /// Get the system status.
    pub async fn get_system_status(&self) -> Result<SystemStatus, KrakenError> {
        self.public_get(public::SYSTEM_STATUS).await
    }

    /// Get information about the assets available on Kraken.
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
    pub async fn get_asset_pairs(
        &self,
        request: Option<&AssetPairsRequest>,
    ) -> Result<std::collections::HashMap<String, AssetPair>, KrakenError> {
        match request {
            Some(req) => self.public_get_with_params(public::ASSET_PAIRS, req).await,
            None => self.public_get(public::ASSET_PAIRS).await,
        }
    }

    /// Get ticker information for a comma-separated list of pairs (e.g. "XBTUSD,ETHUSD").
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
    /// Returns up to 720 data points for the specified pair and interval.
    pub async fn get_ohlc(&self, request: &OhlcRequest) -> Result<OhlcResponse, KrakenError> {
        self.public_get_with_params(public::OHLC, request).await
    }

    /// Get order book for a pair.
    pub async fn get_order_book(
        &self,
        request: &OrderBookRequest,
    ) -> Result<std::collections::HashMap<String, OrderBook>, KrakenError> {
        self.public_get_with_params(public::DEPTH, request).await
    }

    /// Get recent trades for a pair.
    pub async fn get_recent_trades(
        &self,
        request: &RecentTradesRequest,
    ) -> Result<RecentTradesResponse, KrakenError> {
        self.public_get_with_params(public::TRADES, request).await
    }

    /// Get recent spreads for a pair.
    pub async fn get_recent_spreads(
        &self,
        request: &RecentSpreadsRequest,
    ) -> Result<RecentSpreadsResponse, KrakenError> {
        self.public_get_with_params(public::SPREAD, request).await
    }
}

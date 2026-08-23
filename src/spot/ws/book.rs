//! Local order book maintenance and CRC32 checksum validation.
//!
//! Kraken sends a `checksum` field with every book message.
//! The checksum is a CRC32 over the top 10 asks and top 10 bids, with prices
//! and quantities formatted to the pair's precision, decimal points removed,
//! and leading zeros stripped.
//! The required precisions come from the `AssetPairs` REST endpoint
//! (`pair_decimals` and `lot_decimals`).

use std::collections::BTreeMap;

use rust_decimal::Decimal;

use crate::spot::ws::messages::{BookData, BookLevel};

/// Format a value for checksum hashing.
///
/// The value is formatted with a fixed number of decimals, then the decimal
/// point and leading zeros are removed.
fn format_for_checksum(value: Decimal, decimals: u32) -> String {
    let formatted = format!("{:.*}", decimals as usize, value);
    let without_point = formatted.replace('.', "");
    let trimmed = without_point.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// Compute the CRC32 checksum for the given book sides.
///
/// `asks` must be sorted ascending by price and `bids` descending, which is
/// the order Kraken delivers them in.
/// Only the top 10 levels of each side are included.
pub fn book_checksum(
    asks: &[BookLevel],
    bids: &[BookLevel],
    price_decimals: u32,
    qty_decimals: u32,
) -> u32 {
    let mut hasher = crc32fast::Hasher::new();

    for level in asks.iter().take(10) {
        hasher.update(format_for_checksum(level.price, price_decimals).as_bytes());
        hasher.update(format_for_checksum(level.qty, qty_decimals).as_bytes());
    }
    for level in bids.iter().take(10) {
        hasher.update(format_for_checksum(level.price, price_decimals).as_bytes());
        hasher.update(format_for_checksum(level.qty, qty_decimals).as_bytes());
    }

    hasher.finalize()
}

impl BookData {
    /// Compute the CRC32 checksum of this book data.
    ///
    /// `price_decimals` and `qty_decimals` are the pair's `pair_decimals` and
    /// `lot_decimals` from the `AssetPairs` REST endpoint.
    pub fn compute_checksum(&self, price_decimals: u32, qty_decimals: u32) -> u32 {
        book_checksum(&self.asks, &self.bids, price_decimals, qty_decimals)
    }

    /// Validate this book data against its embedded checksum.
    ///
    /// Returns `None` when the message carries no checksum.
    pub fn validate_checksum(&self, price_decimals: u32, qty_decimals: u32) -> Option<bool> {
        self.checksum
            .map(|expected| self.compute_checksum(price_decimals, qty_decimals) == expected)
    }
}

/// A locally maintained order book for one symbol.
///
/// Apply snapshots and updates from the `book` channel, then validate the
/// book against the checksum from each update.
///
/// # Example
///
/// ```rust,ignore
/// let mut book = OrderBookState::new("BTC/USD", 10, 1, 8);
/// book.apply_snapshot(&snapshot_data);
/// book.apply_update(&update_data);
/// if !book.validate(update_data.checksum.unwrap()) {
///     // Resubscribe to recover.
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OrderBookState {
    /// Symbol this book tracks.
    pub symbol: String,
    depth: usize,
    price_decimals: u32,
    qty_decimals: u32,
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

impl OrderBookState {
    /// Create a new order book.
    ///
    /// `depth` is the subscribed book depth.
    /// `price_decimals` and `qty_decimals` are the pair's `pair_decimals` and
    /// `lot_decimals` from the `AssetPairs` REST endpoint.
    pub fn new(
        symbol: impl Into<String>,
        depth: usize,
        price_decimals: u32,
        qty_decimals: u32,
    ) -> Self {
        Self {
            symbol: symbol.into(),
            depth,
            price_decimals,
            qty_decimals,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Apply a snapshot, replacing the current book contents.
    pub fn apply_snapshot(&mut self, data: &BookData) {
        self.bids.clear();
        self.asks.clear();
        self.apply_update(data);
    }

    /// Apply an incremental update.
    ///
    /// A quantity of zero removes the price level.
    /// Both sides are truncated to the subscribed depth afterwards.
    pub fn apply_update(&mut self, data: &BookData) {
        for level in &data.bids {
            if level.qty.is_zero() {
                self.bids.remove(&level.price);
            } else {
                self.bids.insert(level.price, level.qty);
            }
        }
        for level in &data.asks {
            if level.qty.is_zero() {
                self.asks.remove(&level.price);
            } else {
                self.asks.insert(level.price, level.qty);
            }
        }

        // Bids keep the highest prices, asks keep the lowest.
        while self.bids.len() > self.depth {
            let lowest = *self.bids.keys().next().unwrap();
            self.bids.remove(&lowest);
        }
        while self.asks.len() > self.depth {
            let highest = *self.asks.keys().next_back().unwrap();
            self.asks.remove(&highest);
        }
    }

    /// Get the best bid as (price, quantity).
    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        self.bids.iter().next_back().map(|(p, q)| (*p, *q))
    }

    /// Get the best ask as (price, quantity).
    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        self.asks.iter().next().map(|(p, q)| (*p, *q))
    }

    /// Get the bid side sorted descending by price.
    pub fn bids(&self) -> Vec<BookLevel> {
        self.bids
            .iter()
            .rev()
            .map(|(p, q)| BookLevel { price: *p, qty: *q })
            .collect()
    }

    /// Get the ask side sorted ascending by price.
    pub fn asks(&self) -> Vec<BookLevel> {
        self.asks
            .iter()
            .map(|(p, q)| BookLevel { price: *p, qty: *q })
            .collect()
    }

    /// Compute the CRC32 checksum of the current book state.
    pub fn checksum(&self) -> u32 {
        book_checksum(
            &self.asks(),
            &self.bids(),
            self.price_decimals,
            self.qty_decimals,
        )
    }

    /// Validate the current book state against an expected checksum.
    pub fn validate(&self, expected: u32) -> bool {
        self.checksum() == expected
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn level(price: &str, qty: &str) -> BookLevel {
        BookLevel {
            price: Decimal::from_str(price).unwrap(),
            qty: Decimal::from_str(qty).unwrap(),
        }
    }

    fn snapshot_asks() -> Vec<BookLevel> {
        vec![
            level("29430.3", "8.25215653"),
            level("29430.4", "2.55637606"),
            level("29430.5", "0.2997598"),
            level("29430.9", "2.56473216"),
            level("29431.0", "0.00011662"),
            level("29431.1", "0.001"),
            level("29432.3", "0.0241"),
            level("29432.4", "0.02210682"),
            level("29432.5", "3.39761477"),
            level("29433.1", "0.01021497"),
        ]
    }

    fn snapshot_bids() -> Vec<BookLevel> {
        vec![
            level("29430.2", "0.18967538"),
            level("29429.6", "0.00011621"),
            level("29427.4", "0.001"),
            level("29426.7", "4.0"),
            level("29425.7", "0.67945399"),
            level("29423.5", "0.67950478"),
            level("29422.1", "0.16314267"),
            level("29421.8", "0.06797895"),
            level("29421.7", "0.23401846"),
            level("29421.5", "0.28639276"),
        ]
    }

    #[test]
    fn test_checksum_matches_known_snapshot() {
        // Fixture values from a real BTC/USD book snapshot.
        let checksum = book_checksum(&snapshot_asks(), &snapshot_bids(), 1, 8);
        assert_eq!(checksum, 2785033588);
    }

    #[test]
    fn test_book_data_validate_checksum() {
        let data = BookData {
            symbol: "BTC/USD".to_string(),
            bids: snapshot_bids(),
            asks: snapshot_asks(),
            checksum: Some(2785033588),
            timestamp: None,
        };
        assert_eq!(data.validate_checksum(1, 8), Some(true));
        assert_eq!(data.compute_checksum(1, 8), 2785033588);
    }

    #[test]
    fn test_order_book_state_snapshot_and_checksum() {
        let mut book = OrderBookState::new("BTC/USD", 10, 1, 8);
        let data = BookData {
            symbol: "BTC/USD".to_string(),
            bids: snapshot_bids(),
            asks: snapshot_asks(),
            checksum: Some(2785033588),
            timestamp: None,
        };
        book.apply_snapshot(&data);

        assert!(book.validate(2785033588));
        assert_eq!(
            book.best_bid().unwrap().0,
            Decimal::from_str("29430.2").unwrap()
        );
        assert_eq!(
            book.best_ask().unwrap().0,
            Decimal::from_str("29430.3").unwrap()
        );
    }

    #[test]
    fn test_order_book_state_update_and_truncate() {
        let mut book = OrderBookState::new("BTC/USD", 10, 1, 8);
        let snapshot = BookData {
            symbol: "BTC/USD".to_string(),
            bids: snapshot_bids(),
            asks: snapshot_asks(),
            checksum: None,
            timestamp: None,
        };
        book.apply_snapshot(&snapshot);

        // Remove the best bid and add a new deep bid.
        let update = BookData {
            symbol: "BTC/USD".to_string(),
            bids: vec![level("29430.2", "0"), level("29420.0", "1.5")],
            asks: vec![],
            checksum: None,
            timestamp: None,
        };
        book.apply_update(&update);

        assert_eq!(
            book.best_bid().unwrap().0,
            Decimal::from_str("29429.6").unwrap()
        );
        assert_eq!(book.bids().len(), 10);
    }

    #[test]
    fn test_format_strips_point_and_leading_zeros() {
        assert_eq!(
            format_for_checksum(Decimal::from_str("0.00011621").unwrap(), 8),
            "11621"
        );
        assert_eq!(
            format_for_checksum(Decimal::from_str("29430.2").unwrap(), 1),
            "294302"
        );
        // Trailing zeros are preserved by the fixed precision.
        assert_eq!(format_for_checksum(Decimal::from_str("4.0").unwrap(), 8), "400000000");
    }
}

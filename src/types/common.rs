//! Common domain types for Kraken API.

use serde::{Deserialize, Serialize};

/// Buy or sell side of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuySell {
    /// Buy order
    Buy,
    /// Sell order
    Sell,
}

impl std::fmt::Display for BuySell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuySell::Buy => write!(f, "buy"),
            BuySell::Sell => write!(f, "sell"),
        }
    }
}

/// Order type for trading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OrderType {
    /// Market order - execute immediately at best available price
    Market,
    /// Limit order - execute at specified price or better
    Limit,
    /// Stop-loss order - trigger market order when price reaches stop price
    StopLoss,
    /// Take-profit order - trigger market order when price reaches profit target
    TakeProfit,
    /// Stop-loss limit - trigger limit order when price reaches stop price
    StopLossLimit,
    /// Take-profit limit - trigger limit order when price reaches profit target
    TakeProfitLimit,
    /// Trailing stop order
    TrailingStop,
    /// Trailing stop limit order
    TrailingStopLimit,
    /// Settle position order
    SettlePosition,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            OrderType::Market => "market",
            OrderType::Limit => "limit",
            OrderType::StopLoss => "stop-loss",
            OrderType::TakeProfit => "take-profit",
            OrderType::StopLossLimit => "stop-loss-limit",
            OrderType::TakeProfitLimit => "take-profit-limit",
            OrderType::TrailingStop => "trailing-stop",
            OrderType::TrailingStopLimit => "trailing-stop-limit",
            OrderType::SettlePosition => "settle-position",
        };
        write!(f, "{}", s)
    }
}

/// Status of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// Order is pending (not yet submitted)
    Pending,
    /// Order is open and active
    Open,
    /// Order has been partially filled
    #[serde(alias = "partial")]
    PartiallyFilled,
    /// Order has been completely filled
    #[serde(alias = "filled")]
    Closed,
    /// Order has been canceled
    Canceled,
    /// Order has expired
    Expired,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderStatus::Pending => write!(f, "pending"),
            OrderStatus::Open => write!(f, "open"),
            OrderStatus::PartiallyFilled => write!(f, "partially_filled"),
            OrderStatus::Closed => write!(f, "closed"),
            OrderStatus::Canceled => write!(f, "canceled"),
            OrderStatus::Expired => write!(f, "expired"),
        }
    }
}

/// Time in force for orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TimeInForce {
    /// Good till canceled (default)
    #[default]
    GTC,
    /// Immediate or cancel - fill what's possible immediately, cancel rest
    IOC,
    /// Good till date - order expires at specified time
    GTD,
}

/// Order flags for special order behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderFlag {
    /// Post-only order - will only make liquidity, not take it
    Post,
    /// Fee in base currency
    #[serde(rename = "fcib")]
    FeeInBase,
    /// Fee in quote currency
    #[serde(rename = "fciq")]
    FeeInQuote,
    /// No market price protection
    #[serde(rename = "nompp")]
    NoMarketPriceProtection,
    /// Order volume in quote currency
    #[serde(rename = "viqc")]
    VolumeInQuote,
}

/// Trigger type for conditional orders.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TriggerType {
    /// Trigger on last trade price
    #[default]
    Last,
    /// Trigger on index price
    Index,
}

/// Self-trade prevention mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SelfTradePrevent {
    /// Cancel newest order
    CancelNewest,
    /// Cancel oldest order
    CancelOldest,
    /// Cancel both orders
    CancelBoth,
}

/// Asset class for trading pairs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetClass {
    /// Currency/forex
    Currency,
    /// Cryptocurrency
    #[serde(alias = "crypto")]
    Cryptocurrency,
}

/// Ledger entry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LedgerType {
    /// Trade execution
    Trade,
    /// Deposit
    Deposit,
    /// Withdrawal
    Withdrawal,
    /// Transfer between accounts
    Transfer,
    /// Margin trade
    Margin,
    /// Adjustment
    Adjustment,
    /// Rollover
    Rollover,
    /// Credit
    Credit,
    /// Settled
    Settled,
    /// Staking
    Staking,
    /// Dividend
    Dividend,
    /// Sale
    Sale,
    /// NFT-related
    #[serde(rename = "nft")]
    Nft,
}

/// Verification tier for rate limiting purposes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum VerificationTier {
    /// Starter tier (lowest limits)
    #[default]
    Starter,
    /// Intermediate tier
    Intermediate,
    /// Pro tier (highest limits)
    Pro,
}

impl VerificationTier {
    /// Get the rate limit parameters for this verification tier.
    ///
    /// Returns a tuple of (max_counter, decay_rate_per_sec).
    pub fn rate_limit_params(&self) -> (u32, f64) {
        match self {
            VerificationTier::Starter => (15, 0.33),
            VerificationTier::Intermediate => (20, 0.5),
            VerificationTier::Pro => (20, 1.0),
        }
    }
}

/// OHLC interval in minutes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(into = "u32", try_from = "u32")]
pub enum OhlcInterval {
    /// 1 minute
    Min1,
    /// 5 minutes
    Min5,
    /// 15 minutes
    Min15,
    /// 30 minutes
    Min30,
    /// 1 hour
    Hour1,
    /// 4 hours
    Hour4,
    /// 1 day
    Day1,
    /// 1 week
    Week1,
    /// 15 days
    Day15,
}

impl From<OhlcInterval> for u32 {
    fn from(interval: OhlcInterval) -> u32 {
        match interval {
            OhlcInterval::Min1 => 1,
            OhlcInterval::Min5 => 5,
            OhlcInterval::Min15 => 15,
            OhlcInterval::Min30 => 30,
            OhlcInterval::Hour1 => 60,
            OhlcInterval::Hour4 => 240,
            OhlcInterval::Day1 => 1440,
            OhlcInterval::Week1 => 10080,
            OhlcInterval::Day15 => 21600,
        }
    }
}

impl TryFrom<u32> for OhlcInterval {
    type Error = String;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(OhlcInterval::Min1),
            5 => Ok(OhlcInterval::Min5),
            15 => Ok(OhlcInterval::Min15),
            30 => Ok(OhlcInterval::Min30),
            60 => Ok(OhlcInterval::Hour1),
            240 => Ok(OhlcInterval::Hour4),
            1440 => Ok(OhlcInterval::Day1),
            10080 => Ok(OhlcInterval::Week1),
            21600 => Ok(OhlcInterval::Day15),
            _ => Err(format!("Invalid OHLC interval: {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buy_sell_serde() {
        assert_eq!(
            serde_json::to_string(&BuySell::Buy).unwrap(),
            r#""buy""#
        );
        assert_eq!(
            serde_json::from_str::<BuySell>(r#""sell""#).unwrap(),
            BuySell::Sell
        );
    }

    #[test]
    fn test_order_type_serde() {
        assert_eq!(
            serde_json::to_string(&OrderType::StopLoss).unwrap(),
            r#""stop-loss""#
        );
        assert_eq!(
            serde_json::from_str::<OrderType>(r#""take-profit-limit""#).unwrap(),
            OrderType::TakeProfitLimit
        );
    }

    #[test]
    fn test_ohlc_interval_conversion() {
        assert_eq!(u32::from(OhlcInterval::Hour1), 60);
        assert_eq!(OhlcInterval::try_from(1440).unwrap(), OhlcInterval::Day1);
        assert!(OhlcInterval::try_from(999).is_err());
    }
}

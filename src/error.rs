//! Error types for the Kraken client library.

use thiserror::Error;

/// The main error type for all Kraken client operations.
#[derive(Error, Debug)]
pub enum KrakenError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// HTTP request with middleware failed
    #[error("HTTP request failed: {0}")]
    HttpMiddleware(#[from] reqwest_middleware::Error),

    /// WebSocket protocol error
    #[error("WebSocket error: {0}")]
    WebSocket(#[from] tokio_tungstenite::tungstenite::Error),

    /// WebSocket communication error (with message)
    #[error("WebSocket error: {0}")]
    WebSocketMsg(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    Url(#[from] url::ParseError),

    /// Kraken API returned an error
    #[error("Kraken API error: {0}")]
    Api(ApiError),

    /// Rate limit exceeded
    #[error("Rate limit exceeded, retry after {retry_after_ms:?}ms")]
    RateLimitExceeded {
        /// Suggested wait time in milliseconds before retrying
        retry_after_ms: Option<u64>,
    },

    /// Authentication error
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid response from the API
    #[error("Invalid response: {0}")]
    InvalidResponse(String),

    /// WebSocket connection closed unexpectedly
    #[error("WebSocket connection closed: {reason}")]
    ConnectionClosed {
        /// Reason for the closure
        reason: String,
    },

    /// Request timeout
    #[error("Request timed out")]
    Timeout,

    /// Missing required credentials
    #[error("Missing credentials: API key and secret required for private endpoints")]
    MissingCredentials,
}

/// Kraken API error codes and messages.
///
/// These are errors returned by the Kraken API itself in the response body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    /// The error code/identifier from Kraken (e.g., "EGeneral:Invalid arguments")
    pub code: String,
    /// Human-readable error message
    pub message: String,
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl ApiError {
    /// Create a new API error from code and message.
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// Parse API error from Kraken's error array format.
    ///
    /// Kraken returns errors as an array like `["EGeneral:Invalid arguments"]`
    pub fn from_error_array(errors: &[String]) -> Option<Self> {
        errors.first().map(|e| {
            // Kraken errors are in format "ECategory:Message"
            let parts: Vec<&str> = e.splitn(2, ':').collect();
            if parts.len() == 2 {
                Self::new(parts[0], parts[1])
            } else {
                Self::new("Unknown", e.clone())
            }
        })
    }

    /// Get the full error string in Kraken's format (code:message).
    pub fn full_code(&self) -> String {
        format!("{}:{}", self.code, self.message)
    }

    /// Check if this is a rate limit error.
    pub fn is_rate_limit(&self) -> bool {
        (self.code == "EAPI" && self.message.contains("Rate limit"))
            || (self.code == "EOrder" && self.message.contains("Rate limit"))
    }

    /// Check if this is an invalid nonce error.
    pub fn is_invalid_nonce(&self) -> bool {
        self.code == "EAPI" && self.message.contains("Invalid nonce")
    }

    /// Check if this is an invalid key error.
    pub fn is_invalid_key(&self) -> bool {
        self.code == "EAPI" && self.message.contains("Invalid key")
    }

    /// Check if this is an invalid signature error.
    pub fn is_invalid_signature(&self) -> bool {
        self.code == "EAPI" && self.message.contains("Invalid signature")
    }

    /// Check if this is a permission denied error.
    pub fn is_permission_denied(&self) -> bool {
        self.code == "EGeneral" && self.message.contains("Permission denied")
    }

    /// Check if this is a service unavailable error.
    pub fn is_service_unavailable(&self) -> bool {
        self.code == "EService" && (self.message.contains("Unavailable") || self.message.contains("Busy"))
    }
}

/// Known Kraken error codes for pattern matching.
pub mod error_codes {
    /// General errors
    pub const INVALID_ARGUMENTS: &str = "EGeneral:Invalid arguments";
    pub const PERMISSION_DENIED: &str = "EGeneral:Permission denied";
    pub const UNKNOWN_METHOD: &str = "EGeneral:Unknown method";
    pub const INTERNAL_ERROR: &str = "EGeneral:Internal error";

    /// API errors
    pub const INVALID_KEY: &str = "EAPI:Invalid key";
    pub const INVALID_SIGNATURE: &str = "EAPI:Invalid signature";
    pub const INVALID_NONCE: &str = "EAPI:Invalid nonce";
    pub const RATE_LIMIT_EXCEEDED: &str = "EAPI:Rate limit exceeded";
    pub const FEATURE_DISABLED: &str = "EAPI:Feature disabled";

    /// Order errors
    pub const ORDER_RATE_LIMIT: &str = "EOrder:Rate limit exceeded";
    pub const INSUFFICIENT_FUNDS: &str = "EOrder:Insufficient funds";
    pub const INVALID_ORDER: &str = "EOrder:Invalid order";
    pub const ORDER_NOT_FOUND: &str = "EOrder:Unknown order";
    pub const MARGIN_LIMIT: &str = "EOrder:Margin limit exceeded";

    /// Service errors
    pub const SERVICE_UNAVAILABLE: &str = "EService:Unavailable";
    pub const SERVICE_BUSY: &str = "EService:Busy";
    pub const SERVICE_MARKET_IN_CANCEL_ONLY: &str = "EService:Market in cancel_only mode";
    pub const SERVICE_MARKET_IN_POST_ONLY: &str = "EService:Market in post_only mode";

    /// Query errors
    pub const UNKNOWN_ASSET_PAIR: &str = "EQuery:Unknown asset pair";
    pub const UNKNOWN_ASSET: &str = "EQuery:Unknown asset";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_error_from_array() {
        let errors = vec!["EAPI:Invalid key".to_string()];
        let error = ApiError::from_error_array(&errors).unwrap();
        assert_eq!(error.code, "EAPI");
        assert_eq!(error.message, "Invalid key");
        assert!(error.is_invalid_key());
    }

    #[test]
    fn test_api_error_display() {
        let error = ApiError::new("EOrder", "Insufficient funds");
        assert_eq!(error.to_string(), "EOrder: Insufficient funds");
    }
}

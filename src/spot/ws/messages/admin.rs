//! Admin WebSocket messages (heartbeat, status, ping/pong).

use serde::{Deserialize, Serialize};

/// Ping request message.
#[derive(Debug, Clone, Serialize)]
pub struct PingRequest {
    /// Request ID for correlation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub req_id: Option<u64>,
}

impl PingRequest {
    /// Create a new ping request.
    pub fn new() -> Self {
        Self { req_id: None }
    }

    /// Create a ping request with a request ID.
    pub fn with_req_id(req_id: u64) -> Self {
        Self {
            req_id: Some(req_id),
        }
    }
}

impl Default for PingRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Pong response message.
#[derive(Debug, Clone, Deserialize)]
pub struct PongResponse {
    /// Request ID (if provided in ping).
    #[serde(default)]
    pub req_id: Option<u64>,
    /// Time when message was received.
    #[serde(default)]
    pub time_in: Option<String>,
    /// Time when message was sent.
    #[serde(default)]
    pub time_out: Option<String>,
}

/// Heartbeat message from server.
#[derive(Debug, Clone, Deserialize)]
pub struct Heartbeat {
    /// Channel name (always "heartbeat").
    pub channel: String,
}

/// System status message.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemStatusMessage {
    /// Channel name (always "status").
    pub channel: String,
    /// Status data.
    pub data: Vec<SystemStatusData>,
}

/// System status data.
#[derive(Debug, Clone, Deserialize)]
pub struct SystemStatusData {
    /// API version.
    #[serde(default)]
    pub api_version: Option<String>,
    /// Connection ID.
    #[serde(default)]
    pub connection_id: Option<u64>,
    /// System status.
    pub system: String,
    /// System version.
    #[serde(default)]
    pub version: Option<String>,
}

impl SystemStatusData {
    /// Check if the system is online.
    pub fn is_online(&self) -> bool {
        self.system == "online"
    }

    /// Check if the system is in maintenance mode.
    pub fn is_maintenance(&self) -> bool {
        self.system == "maintenance"
    }
}

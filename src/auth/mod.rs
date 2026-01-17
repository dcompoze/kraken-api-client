//! Authentication module for Kraken API.
//!
//! This module provides:
//! - Credential management with secure secret storage
//! - Nonce generation for replay attack prevention
//! - HMAC-SHA512 signature generation for authenticated requests

mod credentials;
mod nonce;
mod signature;

pub use credentials::{Credentials, CredentialsProvider, EnvCredentials, StaticCredentials};
pub use nonce::{IncreasingNonce, NonceProvider};
pub use signature::sign_request;

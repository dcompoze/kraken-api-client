//! Authentication for the Kraken API: credential management, nonce generation, and HMAC-SHA512 request signing.

mod credentials;
mod nonce;
mod signature;

pub use credentials::{Credentials, CredentialsProvider, EnvCredentials, StaticCredentials};
pub use nonce::{IncreasingNonce, NonceProvider};
pub use signature::sign_request;

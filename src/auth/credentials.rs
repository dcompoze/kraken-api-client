//! Credential management for Kraken API authentication.

use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;

/// API credentials containing the key and secret.
#[derive(Clone)]
pub struct Credentials {
    /// The API key (public identifier)
    pub api_key: String,
    /// The API secret (private, used for signing)
    api_secret: SecretString,
}

impl Credentials {
    /// Create new credentials from an API key and secret.
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: SecretString::from(api_secret.into()),
        }
    }

    /// Get the API secret for signing.
    ///
    /// This method exposes the secret - use carefully.
    pub fn expose_secret(&self) -> &str {
        self.api_secret.expose_secret()
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credentials")
            .field("api_key", &self.api_key)
            .field("api_secret", &"[REDACTED]")
            .finish()
    }
}

/// Trait for providing API credentials.
///
/// Implement this trait to customize how credentials are retrieved,
/// for example from a secrets manager or environment variables.
pub trait CredentialsProvider: Send + Sync {
    /// Get the credentials.
    fn get_credentials(&self) -> &Credentials;
}

/// Static credentials provider that holds credentials directly.
#[derive(Clone)]
pub struct StaticCredentials {
    credentials: Credentials,
}

impl StaticCredentials {
    /// Create a new static credentials provider.
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            credentials: Credentials::new(api_key, api_secret),
        }
    }
}

impl CredentialsProvider for StaticCredentials {
    fn get_credentials(&self) -> &Credentials {
        &self.credentials
    }
}

impl CredentialsProvider for Arc<StaticCredentials> {
    fn get_credentials(&self) -> &Credentials {
        &self.credentials
    }
}

/// Credentials provider that reads from environment variables.
///
/// By default, reads from `KRAKEN_API_KEY` and `KRAKEN_API_SECRET`.
pub struct EnvCredentials {
    credentials: Credentials,
}

impl EnvCredentials {
    /// Create credentials from default environment variables.
    ///
    /// Reads `KRAKEN_API_KEY` and `KRAKEN_API_SECRET`.
    ///
    /// # Panics
    ///
    /// Panics if the environment variables are not set.
    pub fn from_env() -> Self {
        Self::from_env_vars("KRAKEN_API_KEY", "KRAKEN_API_SECRET")
    }

    /// Create credentials from custom environment variable names.
    ///
    /// # Panics
    ///
    /// Panics if the environment variables are not set.
    pub fn from_env_vars(key_var: &str, secret_var: &str) -> Self {
        let api_key = std::env::var(key_var)
            .unwrap_or_else(|_| panic!("Environment variable {key_var} not set"));
        let api_secret = std::env::var(secret_var)
            .unwrap_or_else(|_| panic!("Environment variable {secret_var} not set"));

        Self {
            credentials: Credentials::new(api_key, api_secret),
        }
    }

    /// Try to create credentials from default environment variables.
    ///
    /// Returns `None` if the environment variables are not set.
    pub fn try_from_env() -> Option<Self> {
        Self::try_from_env_vars("KRAKEN_API_KEY", "KRAKEN_API_SECRET")
    }

    /// Try to create credentials from custom environment variable names.
    ///
    /// Returns `None` if the environment variables are not set.
    pub fn try_from_env_vars(key_var: &str, secret_var: &str) -> Option<Self> {
        let api_key = std::env::var(key_var).ok()?;
        let api_secret = std::env::var(secret_var).ok()?;

        Some(Self {
            credentials: Credentials::new(api_key, api_secret),
        })
    }
}

impl CredentialsProvider for EnvCredentials {
    fn get_credentials(&self) -> &Credentials {
        &self.credentials
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credentials_debug_redacted() {
        let creds = Credentials::new("my_key", "super_secret");
        let debug_str = format!("{:?}", creds);
        assert!(debug_str.contains("my_key"));
        assert!(!debug_str.contains("super_secret"));
        assert!(debug_str.contains("[REDACTED]"));
    }

    #[test]
    fn test_static_credentials() {
        let provider = StaticCredentials::new("key", "secret");
        let creds = provider.get_credentials();
        assert_eq!(creds.api_key, "key");
        assert_eq!(creds.expose_secret(), "secret");
    }
}

//! Futures API authentication.
//!
//! The Futures API uses a different signature algorithm than the Spot API:
//!
//! ```text
//! Futures: SHA256(postData + nonce + endpointPath) -> HMAC-SHA512 -> Base64
//! Spot:    SHA256(nonce + postData) -> path + hash -> HMAC-SHA512 -> Base64
//! ```
//!
//! ## Authentication Headers
//!
//! Authenticated requests require these headers:
//! - `APIKey`: Your public API key
//! - `Authent`: The computed signature
//! - `Nonce` (optional): A unique incrementing integer
//!
//! ## Signature Computation
//!
//! 1. Concatenate: `postData + nonce + endpointPath`
//! 2. SHA-256 hash the concatenation
//! 3. HMAC-SHA-512 with base64-decoded API secret
//! 4. Base64 encode the result

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

use crate::auth::Credentials;
use crate::error::KrakenError;

type HmacSha512 = Hmac<Sha512>;

/// Sign a request for Kraken's Futures API.
///
/// This uses a different algorithm than the Spot API.
///
/// # Arguments
///
/// * `credentials` - API credentials containing the secret
/// * `endpoint_path` - The API endpoint path (e.g., "/api/v3/sendorder")
/// * `nonce` - The nonce value for this request
/// * `post_data` - The URL-encoded POST body (empty string for GET requests)
///
/// # Returns
///
/// Base64-encoded HMAC-SHA512 signature for the `Authent` header.
///
/// # Example
///
/// ```rust,no_run
/// use kraken_api_client::auth::Credentials;
/// use kraken_api_client::futures::sign_futures_request;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let credentials = Credentials::new("api_key", "YXBpX3NlY3JldA=="); // base64 encoded
/// let signature = sign_futures_request(
///     &credentials,
///     "/api/v3/sendorder",
///     1234567890,
///     "symbol=PI_XBTUSD&side=buy&orderType=lmt&size=1&limitPrice=10000"
/// )?;
/// # Ok(())
/// # }
/// ```
pub fn sign_futures_request(
    credentials: &Credentials,
    endpoint_path: &str,
    nonce: u64,
    post_data: &str,
) -> Result<String, KrakenError> {
    // Decode the API secret from base64.
    let secret_decoded = BASE64
        .decode(credentials.expose_secret())
        .map_err(|_| KrakenError::Auth("API secret must be valid base64.".to_string()))?;

    // Concatenate postData + nonce + endpointPath.
    let nonce_str = nonce.to_string();
    let message = format!("{}{}{}", post_data, nonce_str, endpoint_path);

    // SHA-256 hash the message.
    let sha256_hash = Sha256::digest(message.as_bytes());

    // HMAC-SHA-512 with the decoded secret.
    let mut hmac = HmacSha512::new_from_slice(&secret_decoded)
        .map_err(|e| KrakenError::Auth(format!("Invalid HMAC key: {e}")))?;
    hmac.update(&sha256_hash);
    let hmac_result = hmac.finalize().into_bytes();

    // Base64 encode the result.
    Ok(BASE64.encode(hmac_result))
}

/// Sign a GET request for Kraken's Futures API.
///
/// For GET requests with query parameters, the query string is used as the "post data"
/// for signature computation.
///
/// # Arguments
///
/// * `credentials` - API credentials containing the secret
/// * `endpoint_path` - The API endpoint path (e.g., "/api/v3/accounts")
/// * `nonce` - The nonce value for this request
///
/// # Returns
///
/// Base64-encoded HMAC-SHA512 signature for the `Authent` header.
#[allow(dead_code)]
pub fn sign_futures_get_request(
    credentials: &Credentials,
    endpoint_path: &str,
    nonce: u64,
) -> Result<String, KrakenError> {
    sign_futures_request(credentials, endpoint_path, nonce, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_futures_signature_generation() {
        // Test that signature is generated correctly
        let secret = BASE64.encode("test_secret_key_for_signing");
        let credentials = Credentials::new("test_key", secret);

        let signature = sign_futures_request(
            &credentials,
            "/api/v3/sendorder",
            1616492376594,
            "symbol=PI_XBTUSD&side=buy&orderType=lmt",
        )
        .unwrap();

        // The signature should be a valid base64 string
        assert!(BASE64.decode(&signature).is_ok());
        // HMAC-SHA512 produces 64 bytes, base64 encoded = 88 chars (with padding)
        assert_eq!(signature.len(), 88);
    }

    #[test]
    fn test_futures_signature_consistency() {
        // Same inputs should produce same signature
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_futures_signature_differs_from_spot() {
        // Futures signature should differ from Spot signature for same inputs
        // because the algorithm is different
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", &secret);

        let futures_sig =
            sign_futures_request(&credentials, "/api/v3/accounts", 12345, "nonce=12345").unwrap();

        // Spot signature would concatenate differently
        // This test documents that the algorithms are distinct
        let spot_sig =
            crate::auth::sign_request(&credentials, "/api/v3/accounts", 12345, "nonce=12345")
                .unwrap();

        assert_ne!(futures_sig, spot_sig);
    }

    #[test]
    fn test_futures_signature_changes_with_nonce() {
        // Different nonces should produce different signatures
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/accounts", 12346, "").unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_futures_signature_changes_with_path() {
        // Different paths should produce different signatures
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/openpositions", 12345, "").unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_futures_signature_changes_with_data() {
        // Different post data should produce different signatures
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 =
            sign_futures_request(&credentials, "/api/v3/sendorder", 12345, "symbol=PI_XBTUSD")
                .unwrap();
        let sig2 =
            sign_futures_request(&credentials, "/api/v3/sendorder", 12345, "symbol=PI_ETHUSD")
                .unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_get_request_signature() {
        // GET requests should use empty post_data
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_get_request(&credentials, "/api/v3/accounts", 12345).unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();

        assert_eq!(sig1, sig2);
    }
}

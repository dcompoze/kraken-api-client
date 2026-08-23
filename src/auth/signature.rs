//! HMAC-SHA512 signature generation for Kraken API authentication.
//!
//! Kraken private endpoints require a signature computed as:
//! ```text
//! HMAC-SHA512(path + SHA256(nonce + POST_data), base64_decode(api_secret))
//! ```
//!
//! The signature is then base64-encoded and sent in the `API-Sign` header.

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

use crate::auth::Credentials;
use crate::error::KrakenError;

type HmacSha512 = Hmac<Sha512>;

/// Sign a request for Kraken's private API, returning the base64-encoded signature.
///
/// Fails with `KrakenError::Auth` if the API secret is not valid base64.
pub fn sign_request(
    credentials: &Credentials,
    url_path: &str,
    nonce: u64,
    post_data: &str,
) -> Result<String, KrakenError> {
    let secret_decoded = BASE64
        .decode(credentials.expose_secret())
        .map_err(|_| KrakenError::Auth("API secret must be valid base64.".to_string()))?;

    let nonce_str = nonce.to_string();
    let mut sha256_hasher = Sha256::new();
    sha256_hasher.update(nonce_str.as_bytes());
    sha256_hasher.update(post_data.as_bytes());
    let sha256_hash = sha256_hasher.finalize();

    let mut hmac = HmacSha512::new_from_slice(&secret_decoded)
        .map_err(|e| KrakenError::Auth(format!("Invalid HMAC key: {e}")))?;
    hmac.update(url_path.as_bytes());
    hmac.update(&sha256_hash);
    let hmac_result = hmac.finalize().into_bytes();

    Ok(BASE64.encode(hmac_result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generation() {
        let secret = BASE64.encode("test_secret_key_for_signing");
        let credentials = Credentials::new("test_key", secret);

        let signature = sign_request(
            &credentials,
            "/0/private/Balance",
            1616492376594,
            "nonce=1616492376594",
        )
        .unwrap();

        assert!(BASE64.decode(&signature).is_ok());
        assert_eq!(signature.len(), 88);
    }

    #[test]
    fn test_signature_consistency() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_request(
            &credentials,
            "/0/private/TradeBalance",
            12345,
            "nonce=12345&asset=ZUSD",
        )
        .unwrap();
        let sig2 = sign_request(
            &credentials,
            "/0/private/TradeBalance",
            12345,
            "nonce=12345&asset=ZUSD",
        )
        .unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_signature_changes_with_nonce() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_request(&credentials, "/0/private/Balance", 12345, "nonce=12345").unwrap();
        let sig2 = sign_request(&credentials, "/0/private/Balance", 12346, "nonce=12346").unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_signature_changes_with_path() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_request(&credentials, "/0/private/Balance", 12345, "nonce=12345").unwrap();
        let sig2 = sign_request(
            &credentials,
            "/0/private/TradeBalance",
            12345,
            "nonce=12345",
        )
        .unwrap();

        assert_ne!(sig1, sig2);
    }
}

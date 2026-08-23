//! Futures API authentication.
//!
//! The Futures signature is `Base64(HMAC-SHA512(SHA256(postData + nonce + endpointPath), base64-decoded secret))`.
//! This differs from the Spot scheme, which hashes `nonce + postData` and prepends the path before the HMAC.

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256, Sha512};

use crate::auth::Credentials;
use crate::error::KrakenError;

type HmacSha512 = Hmac<Sha512>;

/// Sign a Futures API request and return the value for the `Authent` header.
///
/// `post_data` is the URL-encoded body, or the query string for GET requests.
/// Fails if the API secret is not valid base64.
pub fn sign_futures_request(
    credentials: &Credentials,
    endpoint_path: &str,
    nonce: u64,
    post_data: &str,
) -> Result<String, KrakenError> {
    let secret_decoded = BASE64
        .decode(credentials.expose_secret())
        .map_err(|_| KrakenError::Auth("API secret must be valid base64.".to_string()))?;

    let nonce_str = nonce.to_string();
    let message = format!("{}{}{}", post_data, nonce_str, endpoint_path);

    let sha256_hash = Sha256::digest(message.as_bytes());

    let mut hmac = HmacSha512::new_from_slice(&secret_decoded)
        .map_err(|e| KrakenError::Auth(format!("Invalid HMAC key: {e}")))?;
    hmac.update(&sha256_hash);
    let hmac_result = hmac.finalize().into_bytes();

    Ok(BASE64.encode(hmac_result))
}

/// Sign a Futures API GET request without query parameters.
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
        let secret = BASE64.encode("test_secret_key_for_signing");
        let credentials = Credentials::new("test_key", secret);

        let signature = sign_futures_request(
            &credentials,
            "/api/v3/sendorder",
            1616492376594,
            "symbol=PI_XBTUSD&side=buy&orderType=lmt",
        )
        .unwrap();

        assert!(BASE64.decode(&signature).is_ok());
        assert_eq!(signature.len(), 88);
    }

    #[test]
    fn test_futures_signature_consistency() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();

        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_futures_signature_differs_from_spot() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", &secret);

        let futures_sig =
            sign_futures_request(&credentials, "/api/v3/accounts", 12345, "nonce=12345").unwrap();

        let spot_sig =
            crate::auth::sign_request(&credentials, "/api/v3/accounts", 12345, "nonce=12345")
                .unwrap();

        assert_ne!(futures_sig, spot_sig);
    }

    #[test]
    fn test_futures_signature_changes_with_nonce() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/accounts", 12346, "").unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_futures_signature_changes_with_path() {
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/openpositions", 12345, "").unwrap();

        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_futures_signature_changes_with_data() {
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
        let secret = BASE64.encode("my_secret");
        let credentials = Credentials::new("key", secret);

        let sig1 = sign_futures_get_request(&credentials, "/api/v3/accounts", 12345).unwrap();
        let sig2 = sign_futures_request(&credentials, "/api/v3/accounts", 12345, "").unwrap();

        assert_eq!(sig1, sig2);
    }
}

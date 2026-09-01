//! Internal TLS backend selection and transport validation.

use tokio_tungstenite::Connector;

use crate::error::KrakenError;

/// Create the WebSocket connector selected by this crate's TLS feature.
#[cfg(all(feature = "native-tls", not(feature = "rustls-tls")))]
pub(crate) fn websocket_connector() -> Result<Connector, KrakenError> {
    let connector = native_tls::TlsConnector::new()
        .map_err(|error| KrakenError::TlsConfiguration(error.to_string()))?;
    Ok(Connector::NativeTls(connector))
}

/// Create the WebSocket connector selected by this crate's TLS feature.
#[cfg(all(feature = "rustls-tls", not(feature = "native-tls")))]
pub(crate) fn websocket_connector() -> Result<Connector, KrakenError> {
    let rustls_native_certs::CertificateResult { certs, errors, .. } =
        rustls_native_certs::load_native_certs();

    if !errors.is_empty() {
        tracing::warn!(?errors, "some native root certificates could not be loaded");
    }
    if certs.is_empty() {
        return Err(KrakenError::TlsConfiguration(format!(
            "no native root certificates were found: {errors:?}"
        )));
    }

    let mut roots = rustls::RootCertStore::empty();
    let total = certs.len();
    let (added, ignored) = roots.add_parsable_certificates(certs);
    tracing::debug!(added, ignored, total, "loaded native root certificates");

    if added == 0 {
        return Err(KrakenError::TlsConfiguration(
            "no native root certificates could be parsed".to_string(),
        ));
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();
    Ok(Connector::Rustls(std::sync::Arc::new(config)))
}

/// Select plaintext transport for `ws` and the configured TLS backend for `wss`.
pub(crate) fn websocket_connector_for_url(url: &str) -> Result<Connector, KrakenError> {
    match url::Url::parse(url)?.scheme() {
        "ws" => Ok(Connector::Plain),
        "wss" => websocket_connector(),
        scheme => Err(KrakenError::WebSocketMsg(format!(
            "unsupported WebSocket URL scheme: {scheme}"
        ))),
    }
}

/// Require a secure scheme unless the caller explicitly permits plaintext.
pub(crate) fn require_secure_url(
    url: &str,
    required_scheme: &'static str,
    allow_insecure: bool,
) -> Result<(), KrakenError> {
    let parsed = url::Url::parse(url)?;
    let insecure_scheme = match required_scheme {
        "https" => "http",
        "wss" => "ws",
        _ => "",
    };
    let scheme_is_allowed = parsed.scheme() == required_scheme
        || (allow_insecure && parsed.scheme() == insecure_scheme);

    if !scheme_is_allowed {
        return Err(KrakenError::InsecureTransport {
            required_scheme,
            actual_scheme: parsed.scheme().to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::auth::StaticCredentials;
    use crate::futures::rest::FuturesRestClient;
    use crate::futures::ws::FuturesWsClient;
    use crate::spot::rest::SpotRestClient;
    use crate::spot::ws::SpotWsClient;

    use super::*;

    fn credentials() -> Arc<StaticCredentials> {
        Arc::new(StaticCredentials::new("key", "c2VjcmV0"))
    }

    #[test]
    fn selected_websocket_connector_matches_feature() {
        let connector = websocket_connector().unwrap();

        #[cfg(feature = "rustls-tls")]
        assert!(matches!(connector, Connector::Rustls(_)));
        #[cfg(feature = "native-tls")]
        assert!(matches!(connector, Connector::NativeTls(_)));
    }

    #[test]
    fn ws_url_selects_plain_connector() {
        let connector = websocket_connector_for_url("ws://127.0.0.1:8080").unwrap();
        assert!(matches!(connector, Connector::Plain));
    }

    #[test]
    fn authenticated_rest_clients_reject_http() {
        let spot = SpotRestClient::builder()
            .base_url("http://127.0.0.1:8080")
            .credentials(credentials())
            .build();
        let futures = FuturesRestClient::builder()
            .base_url("http://127.0.0.1:8080")
            .credentials(credentials())
            .build();

        assert!(matches!(spot, Err(KrakenError::InsecureTransport { .. })));
        assert!(matches!(futures, Err(KrakenError::InsecureTransport { .. })));
    }

    #[test]
    fn explicit_test_option_allows_authenticated_http() {
        let spot = SpotRestClient::builder()
            .base_url("http://127.0.0.1:8080")
            .credentials(credentials())
            .danger_allow_insecure_transport()
            .build();
        let futures = FuturesRestClient::builder()
            .base_url("http://127.0.0.1:8080")
            .credentials(credentials())
            .danger_allow_insecure_transport()
            .build();

        assert!(spot.is_ok());
        assert!(futures.is_ok());
    }

    #[tokio::test]
    async fn private_websocket_clients_reject_ws() {
        let spot = SpotWsClient::with_urls(
            "ws://127.0.0.1:8080/public",
            "ws://127.0.0.1:8080/private",
        )
        .connect_private("token")
        .await;
        let futures = FuturesWsClient::with_url("ws://127.0.0.1:8080/private")
            .connect_private(credentials())
            .await;

        assert!(matches!(spot, Err(KrakenError::InsecureTransport { .. })));
        assert!(matches!(futures, Err(KrakenError::InsecureTransport { .. })));
    }
}

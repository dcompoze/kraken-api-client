use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use kraken_api_client::auth::StaticCredentials;
use kraken_api_client::spot::rest::SpotRestClient;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn authenticated_rest_requests_do_not_follow_redirects() {
    let source = MockServer::start().await;
    let destination = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/0/private/Balance"))
        .respond_with(
            ResponseTemplate::new(307)
                .insert_header("Location", format!("{}/stolen", destination.uri())),
        )
        .expect(1)
        .mount(&source)
        .await;

    Mock::given(method("POST"))
        .and(path("/stolen"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&destination)
        .await;

    let credentials = Arc::new(StaticCredentials::new(
        "test_key",
        STANDARD.encode("test_secret"),
    ));
    let client = SpotRestClient::builder()
        .base_url(source.uri())
        .credentials(credentials)
        .danger_allow_insecure_transport()
        .build()
        .unwrap();

    let result = client.get_account_balance().await;
    assert!(result.is_err());
    source.verify().await;
    destination.verify().await;
}

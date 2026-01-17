# kraken-api-client

`kraken-api-client` is an async Rust library for Kraken Spot and Futures APIs.

It includes REST clients, WebSocket clients, auth helpers, typed request/response models, and optional rate-limiting wrappers.

- Spot REST client for public and private endpoints.
- Spot WebSocket v2 client for market data and trading channels.
- Futures REST and WebSocket clients.
- Typed models for requests and responses.
- Auth support for signatures, credentials providers, and nonce generation.
- Rate limiting utilities for public, private, and trading flows.

# Usage

```rust,no_run
use kraken_api_client::spot::rest::SpotRestClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SpotRestClient::new();
    let time = client.get_server_time().await?;
    println!("Server time: {:?}", time);
    Ok(())
}
```

# Credentials

Private endpoints use credentials from your own source.

You can use `StaticCredentials` or `EnvCredentials`.

Expected env vars are `KRAKEN_API_KEY` and `KRAKEN_API_SECRET`.

# Examples and tests

- `examples/` contains focused usage snippets.
- Integration tests under `tests/` use mocked servers where possible.
- Live tests require credentials configured in your environment.

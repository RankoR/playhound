use super::*;
use crate::{
    test_support::fixtures::{app_html, search_html, search_item, search_rpc},
    transport::{HttpResponse, test_support::FakeTransport},
};

#[test]
fn client_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Client>();
}

#[test]
fn builder_rejects_invalid_limits_before_creating_a_transport() {
    assert_eq!(
        Client::builder()
            .request_timeout(Duration::ZERO)
            .build()
            .unwrap_err()
            .kind(),
        crate::ErrorKind::Configuration
    );
    assert_eq!(
        Client::builder()
            .max_response_bytes(0)
            .build()
            .unwrap_err()
            .kind(),
        crate::ErrorKind::Configuration
    );
}

#[allow(clippy::unnecessary_wraps)]
fn ok(body: String) -> Result<HttpResponse> {
    Ok(HttpResponse {
        status: 200,
        retry_after: None,
        body,
    })
}

#[tokio::test]
async fn app_uses_fake_transport_and_default_locale() {
    let transport = FakeTransport::new([ok(app_html())]);
    let client = Client::with_test_transport(transport.clone());
    let app = client.app("com.example.app").await.unwrap();
    assert_eq!(app.overview.title, "Example App");
    let requests = transport.requests.lock().unwrap();
    assert_eq!(requests.len(), 1);
    assert!(requests[0].query.contains(&("hl".into(), "en".into())));
    assert!(requests[0].query.contains(&("gl".into(), "us".into())));
}

#[tokio::test]
async fn search_paginates_and_honors_limit() {
    let first = search_html(
        vec![search_item(Some("com.example.one"), "First")],
        Some("NEXT"),
    );
    let second = search_rpc(
        vec![
            search_item(Some("com.example.two"), "Second"),
            search_item(Some("com.example.three"), "Third"),
        ],
        None,
    );
    let transport = FakeTransport::new([ok(first), ok(second)]);
    let client = Client::with_test_transport(transport.clone());
    let apps = client
        .search(SearchQuery::new("example").limit(2))
        .await
        .unwrap();
    assert_eq!(apps.len(), 2);
    assert_eq!(apps[1].title, "Second");
    assert_eq!(transport.requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn invalid_input_never_reaches_transport() {
    let transport = FakeTransport::new([]);
    let client = Client::with_test_transport(transport.clone());
    assert_eq!(
        client.app("").await.unwrap_err().kind(),
        crate::ErrorKind::InvalidInput
    );
    assert_eq!(
        client
            .search(SearchQuery::new(" "))
            .await
            .unwrap_err()
            .kind(),
        crate::ErrorKind::InvalidInput
    );
    assert!(transport.requests.lock().unwrap().is_empty());
}

#[tokio::test]
async fn retries_transient_status_when_explicitly_enabled() {
    let transport = FakeTransport::new([ok_status(503, "temporarily unavailable"), ok(app_html())]);
    let config = crate::config::ClientConfig {
        retry_policy: RetryPolicy::exponential(1)
            .base_delay(Duration::ZERO)
            .max_delay(Duration::ZERO),
        ..crate::config::ClientConfig::default()
    };
    let client = Client::with_test_config(config, transport.clone());
    let app = client.app("com.example.app").await.unwrap();
    assert_eq!(app.overview.app_id.as_str(), "com.example.app");
    assert_eq!(transport.requests.lock().unwrap().len(), 2);
}

#[tokio::test]
async fn retries_are_disabled_by_default() {
    let transport = FakeTransport::new([ok_status(503, "temporarily unavailable")]);
    let client = Client::with_test_transport(transport.clone());
    assert_eq!(
        client.app("com.example.app").await.unwrap_err().kind(),
        crate::ErrorKind::RateLimited
    );
    assert_eq!(transport.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn does_not_retry_non_transient_http_status() {
    let transport = FakeTransport::new([ok_status(500, "server error")]);
    let config = crate::config::ClientConfig {
        retry_policy: RetryPolicy::exponential(3)
            .base_delay(Duration::ZERO)
            .max_delay(Duration::ZERO),
        ..crate::config::ClientConfig::default()
    };
    let client = Client::with_test_config(config, transport.clone());
    assert_eq!(
        client.app("com.example.app").await.unwrap_err().kind(),
        crate::ErrorKind::HttpStatus
    );
    assert_eq!(transport.requests.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn maps_app_404_to_typed_not_found_error() {
    let transport = FakeTransport::new([ok_status(404, "not found")]);
    let client = Client::with_test_transport(transport);
    let error = client.app("com.example.missing").await.unwrap_err();
    assert!(matches!(error, Error::AppNotFound { app_id } if app_id == "com.example.missing"));
}

#[tokio::test]
async fn repeated_search_token_stops_pagination() {
    let first = search_html(
        vec![search_item(Some("com.example.one"), "First")],
        Some("REPEATED"),
    );
    let second = search_rpc(
        vec![search_item(Some("com.example.two"), "Second")],
        Some("REPEATED"),
    );
    let transport = FakeTransport::new([ok(first), ok(second)]);
    let client = Client::with_test_transport(transport.clone());
    let apps = client
        .search(SearchQuery::new("example").limit(3))
        .await
        .unwrap();
    assert_eq!(apps.len(), 2);
    assert_eq!(transport.requests.lock().unwrap().len(), 2);
}

#[allow(clippy::unnecessary_wraps)]
fn ok_status(status: u16, body: &str) -> Result<HttpResponse> {
    Ok(HttpResponse {
        status,
        retry_after: None,
        body: body.into(),
    })
}

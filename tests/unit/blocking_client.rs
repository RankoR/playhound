use super::*;
use std::collections::VecDeque;

use crate::{
    blocking::transport::BlockingTransport,
    test_support::fixtures::{
        app_html, list_rpc, reviews_rpc, search_html, search_item, suggestions_rpc,
    },
    transport::{HttpRequest, HttpResponse},
};

struct FakeBlockingTransport {
    requests: Mutex<Vec<HttpRequest>>,
    responses: Mutex<VecDeque<Result<HttpResponse>>>,
}

impl FakeBlockingTransport {
    fn new(bodies: impl IntoIterator<Item = String>) -> Arc<Self> {
        Arc::new(Self {
            requests: Mutex::new(Vec::new()),
            responses: Mutex::new(
                bodies
                    .into_iter()
                    .map(|body| {
                        Ok(HttpResponse {
                            status: 200,
                            retry_after: None,
                            body,
                        })
                    })
                    .collect(),
            ),
        })
    }
}

impl BlockingTransport for FakeBlockingTransport {
    fn execute(&self, request: HttpRequest) -> Result<HttpResponse> {
        self.requests.lock().unwrap().push(request);
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .expect("fake response")
    }
}

#[test]
fn blocking_client_is_send_and_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<Client>();
}

#[test]
fn blocking_builder_rejects_invalid_limits_before_creating_a_transport() {
    assert_eq!(
        Client::builder()
            .connect_timeout(Duration::ZERO)
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

#[test]
fn blocking_client_exercises_every_operation_without_network() {
    let transport = FakeBlockingTransport::new([
        app_html(),
        search_html(
            vec![search_item(Some("com.example.search"), "Search Result")],
            None,
        ),
        list_rpc(),
        reviews_rpc(),
        suggestions_rpc(),
    ]);
    let client = Client::with_test_transport(transport.clone());

    assert_eq!(
        client.app("com.example.app").unwrap().overview.title,
        "Example App"
    );
    assert_eq!(
        client.search(SearchQuery::new("example")).unwrap()[0]
            .app_id
            .as_str(),
        "com.example.search"
    );
    assert!(!client.list(ListQuery::default()).unwrap().is_empty());
    assert!(
        !client
            .reviews(ReviewQuery::new("com.example.app"))
            .unwrap()
            .items
            .is_empty()
    );
    assert!(!client.suggestions("example").unwrap().is_empty());
    assert_eq!(transport.requests.lock().unwrap().len(), 5);
}

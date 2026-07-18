use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use super::*;

pub(crate) struct FakeTransport {
    pub requests: Mutex<Vec<HttpRequest>>,
    responses: Mutex<VecDeque<Result<HttpResponse>>>,
}

impl FakeTransport {
    pub fn new(responses: impl IntoIterator<Item = Result<HttpResponse>>) -> Arc<Self> {
        Arc::new(Self {
            requests: Mutex::new(Vec::new()),
            responses: Mutex::new(responses.into_iter().collect()),
        })
    }
}

impl AsyncTransport for FakeTransport {
    fn execute<'a>(
        &'a self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse>> + Send + 'a>> {
        Box::pin(async move {
            self.requests.lock().unwrap().push(request);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .expect("fake response")
        })
    }
}

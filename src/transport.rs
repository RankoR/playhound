use std::{future::Future, pin::Pin, sync::Arc};

use futures_util::StreamExt;
use reqwest::{Method, StatusCode, header::RETRY_AFTER};

use crate::{
    Error, Result,
    config::{ClientConfig, ProxyScope},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HttpRequest {
    pub method: Method,
    pub path: String,
    pub query: Vec<(String, String)>,
    pub form: Option<Vec<(String, String)>>,
}

#[derive(Clone, Debug)]
pub(crate) struct HttpResponse {
    pub status: u16,
    pub retry_after: Option<std::time::Duration>,
    pub body: String,
}

pub(crate) trait AsyncTransport: Send + Sync {
    fn execute<'a>(
        &'a self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse>> + Send + 'a>>;
}

pub(crate) type DynAsyncTransport = Arc<dyn AsyncTransport>;

pub(crate) struct ReqwestTransport {
    client: reqwest::Client,
    base_url: url::Url,
    max_response_bytes: usize,
}

impl ReqwestTransport {
    pub fn new(config: &ClientConfig) -> Result<Self> {
        let mut builder = reqwest::Client::builder()
            .user_agent(concat!("playhound/", env!("CARGO_PKG_VERSION")))
            .timeout(config.request_timeout)
            .connect_timeout(config.connect_timeout)
            .danger_accept_invalid_certs(config.accept_invalid_certs)
            .redirect(reqwest::redirect::Policy::custom(|attempt| {
                if attempt.previous().len() >= 5 {
                    return attempt.error("too many redirects");
                }
                if attempt.url().host_str() != Some("play.google.com") {
                    return attempt.stop();
                }
                attempt.follow()
            }));

        if !config.proxies.is_empty() || !config.use_system_proxy {
            builder = builder.no_proxy();
        }
        let mut proxies = config.proxies.iter().collect::<Vec<_>>();
        proxies.sort_by_key(|proxy| matches!(proxy.scope, ProxyScope::All));
        for proxy in proxies {
            let url = proxy.url.as_str();
            let built = match proxy.scope {
                ProxyScope::All => reqwest::Proxy::all(url),
                ProxyScope::Http => reqwest::Proxy::http(url),
                ProxyScope::Https => reqwest::Proxy::https(url),
            }
            .map_err(|_| Error::Configuration {
                message: "invalid proxy configuration".into(),
            })?;
            builder = builder.proxy(built);
        }
        let client = builder.build().map_err(|error| Error::Configuration {
            message: redact_transport_error(&error.to_string()),
        })?;
        Ok(Self {
            client,
            base_url: url::Url::parse("https://play.google.com").expect("constant URL"),
            max_response_bytes: config.max_response_bytes,
        })
    }
}

impl AsyncTransport for ReqwestTransport {
    fn execute<'a>(
        &'a self,
        request: HttpRequest,
    ) -> Pin<Box<dyn Future<Output = Result<HttpResponse>> + Send + 'a>> {
        Box::pin(async move {
            let url = self
                .base_url
                .join(&request.path)
                .map_err(|_| Error::invalid("path", "invalid request path"))?;
            let mut builder = self
                .client
                .request(request.method, url)
                .query(&request.query);
            if let Some(form) = request.form {
                builder = builder.form(&form);
            }
            let response = builder.send().await.map_err(map_reqwest_error)?;
            let status = response.status();
            let retry_after = parse_retry_after(response.headers().get(RETRY_AFTER));
            if let Some(length) = response.content_length() {
                if length > self.max_response_bytes as u64 {
                    return Err(Error::ResponseTooLarge {
                        limit: self.max_response_bytes,
                    });
                }
            }
            let mut stream = response.bytes_stream();
            let mut bytes = Vec::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk.map_err(map_reqwest_error)?;
                if bytes.len().saturating_add(chunk.len()) > self.max_response_bytes {
                    return Err(Error::ResponseTooLarge {
                        limit: self.max_response_bytes,
                    });
                }
                bytes.extend_from_slice(&chunk);
            }
            let body = String::from_utf8(bytes).map_err(|_| Error::Parse {
                operation: "HTTP",
                message: "response is not valid UTF-8".into(),
            })?;
            Ok(HttpResponse {
                status: status.as_u16(),
                retry_after,
                body,
            })
        })
    }
}

fn map_reqwest_error(error: reqwest::Error) -> Error {
    Error::Transport {
        message: redact_transport_error(&error.to_string()),
    }
}

fn redact_transport_error(message: &str) -> String {
    // reqwest normally redacts URL passwords. Avoid returning any URL-shaped detail at all.
    if message.contains('@') {
        "request failed (credential-bearing URL redacted)".into()
    } else {
        message.to_owned()
    }
}

fn parse_retry_after(value: Option<&reqwest::header::HeaderValue>) -> Option<std::time::Duration> {
    value?
        .to_str()
        .ok()?
        .parse::<u64>()
        .ok()
        .map(std::time::Duration::from_secs)
}

pub(crate) fn classify(response: HttpResponse) -> Result<String> {
    match StatusCode::from_u16(response.status).ok() {
        Some(status) if status.is_success() => Ok(response.body),
        Some(StatusCode::TOO_MANY_REQUESTS | StatusCode::SERVICE_UNAVAILABLE) => {
            Err(Error::RateLimited {
                retry_after: response.retry_after,
            })
        }
        _ => Err(Error::HttpStatus {
            status: response.status,
        }),
    }
}

#[cfg(test)]
#[path = "../tests/support/transport.rs"]
pub(crate) mod test_support;

#[cfg(test)]
#[path = "../tests/unit/transport.rs"]
mod tests;

use std::io::Read;

use reqwest::{StatusCode, header::RETRY_AFTER};

use crate::{
    Error, Result,
    config::{ClientConfig, ProxyScope},
    transport::{HttpRequest, HttpResponse},
};

pub(super) trait BlockingTransport: Send + Sync {
    fn execute(&self, request: HttpRequest) -> Result<HttpResponse>;
}

pub(super) struct Transport {
    client: reqwest::blocking::Client,
    base_url: url::Url,
    max_response_bytes: usize,
}

impl Transport {
    pub(super) fn new(config: &ClientConfig) -> Result<Self> {
        let mut builder = reqwest::blocking::Client::builder()
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
            let built = match proxy.scope {
                ProxyScope::All => reqwest::Proxy::all(proxy.url.as_str()),
                ProxyScope::Http => reqwest::Proxy::http(proxy.url.as_str()),
                ProxyScope::Https => reqwest::Proxy::https(proxy.url.as_str()),
            }
            .map_err(|_| Error::Configuration {
                message: "invalid proxy configuration".into(),
            })?;
            builder = builder.proxy(built);
        }
        let client = builder.build().map_err(|error| Error::Configuration {
            message: error.to_string(),
        })?;
        Ok(Self {
            client,
            base_url: url::Url::parse("https://play.google.com").expect("constant URL"),
            max_response_bytes: config.max_response_bytes,
        })
    }
}

impl BlockingTransport for Transport {
    fn execute(&self, request: HttpRequest) -> Result<HttpResponse> {
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
        let mut response = builder.send().map_err(map_error)?;
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get(RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<u64>().ok())
            .map(std::time::Duration::from_secs);
        if response
            .content_length()
            .is_some_and(|length| length > self.max_response_bytes as u64)
        {
            return Err(Error::ResponseTooLarge {
                limit: self.max_response_bytes,
            });
        }
        let mut bytes = Vec::new();
        response
            .by_ref()
            .take(self.max_response_bytes as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|error| Error::Transport {
                message: error.to_string(),
            })?;
        if bytes.len() > self.max_response_bytes {
            return Err(Error::ResponseTooLarge {
                limit: self.max_response_bytes,
            });
        }
        let body = String::from_utf8(bytes).map_err(|_| Error::Parse {
            operation: "HTTP",
            message: "response is not valid UTF-8".into(),
        })?;
        Ok(HttpResponse {
            status,
            retry_after,
            body,
        })
    }
}

fn map_error(error: reqwest::Error) -> Error {
    let message = if error.to_string().contains('@') {
        "request failed (credential-bearing URL redacted)".into()
    } else {
        error.to_string()
    };
    Error::Transport { message }
}

pub(super) fn classify(response: HttpResponse) -> Result<String> {
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

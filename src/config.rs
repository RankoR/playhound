use std::{fmt, time::Duration};

use url::Url;

use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProxyScope {
    All,
    Http,
    Https,
}

/// Explicit proxy routing rule.
#[derive(Clone)]
pub struct Proxy {
    pub(crate) scope: ProxyScope,
    pub(crate) url: Url,
}

impl Proxy {
    /// Routes all HTTP and HTTPS traffic through the proxy.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is invalid or uses an unsupported scheme.
    pub fn all(value: impl AsRef<str>) -> Result<Self> {
        Self::parse(ProxyScope::All, value.as_ref())
    }
    /// Routes HTTP traffic through the proxy.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is invalid or uses an unsupported scheme.
    pub fn http(value: impl AsRef<str>) -> Result<Self> {
        Self::parse(ProxyScope::Http, value.as_ref())
    }
    /// Routes HTTPS traffic through the proxy.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is invalid or uses an unsupported scheme.
    pub fn https(value: impl AsRef<str>) -> Result<Self> {
        Self::parse(ProxyScope::Https, value.as_ref())
    }
    fn parse(scope: ProxyScope, value: &str) -> Result<Self> {
        let url = Url::parse(value)
            .map_err(|_| Error::invalid("proxy", "must be a valid absolute proxy URL"))?;
        match url.scheme() {
            "http" | "https" | "socks5" | "socks5h" => Ok(Self { scope, url }),
            _ => Err(Error::invalid("proxy", "unsupported proxy URL scheme")),
        }
    }
}

impl fmt::Debug for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut redacted = self.url.clone();
        if !redacted.username().is_empty() {
            let _ = redacted.set_username("***");
        }
        if redacted.password().is_some() {
            let _ = redacted.set_password(Some("***"));
        }
        f.debug_struct("Proxy")
            .field("scope", &self.scope)
            .field("url", &redacted.as_str())
            .finish()
    }
}

/// Bounded retry policy for transient read-only requests.
#[must_use = "a retry policy has no effect until passed to a client builder"]
#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub(crate) max_retries: u32,
    pub(crate) base_delay: Duration,
    pub(crate) max_delay: Duration,
    pub(crate) jitter_ratio: f64,
    pub(crate) honor_retry_after: bool,
}

impl RetryPolicy {
    /// No retries.
    pub const fn none() -> Self {
        Self {
            max_retries: 0,
            base_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(5),
            jitter_ratio: 0.2,
            honor_retry_after: true,
        }
    }
    /// Recommended bounded exponential policy.
    pub const fn exponential(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Self::none()
        }
    }
    /// Sets the delay used before the first retry.
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }
    /// Caps exponential backoff and server-requested delays.
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }
    /// Controls whether a valid `Retry-After` header takes precedence over backoff.
    pub fn honor_retry_after(mut self, honor: bool) -> Self {
        self.honor_retry_after = honor;
        self
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ClientConfig {
    pub locale: crate::Locale,
    pub proxies: Vec<Proxy>,
    pub use_system_proxy: bool,
    pub request_timeout: Duration,
    pub connect_timeout: Duration,
    pub max_response_bytes: usize,
    pub requests_per_second: Option<std::num::NonZeroU32>,
    pub retry_policy: RetryPolicy,
    pub accept_invalid_certs: bool,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            locale: crate::Locale::default(),
            proxies: Vec::new(),
            use_system_proxy: true,
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_response_bytes: 32 * 1024 * 1024,
            requests_per_second: None,
            retry_policy: RetryPolicy::none(),
            accept_invalid_certs: false,
        }
    }
}

#[cfg(test)]
#[path = "../tests/unit/config.rs"]
mod tests;

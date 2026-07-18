use std::{
    collections::HashSet,
    num::NonZeroU32,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use crate::{
    AppDetails, AppOverview, AppRequest, Error, ListQuery, Locale, Page, Proxy, Result,
    RetryPolicy, Review, ReviewQuery, SearchQuery, SuggestionQuery,
    config::ClientConfig,
    protocol,
    request::{validate_app_id, validate_limit, validate_nonempty},
    transport::HttpRequest,
};

use super::transport::{BlockingTransport, Transport, classify};

/// Builder for the synchronous [`Client`].
#[must_use = "a client builder does nothing until build is called"]
#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Sets the default response locale.
    pub fn default_locale(mut self, locale: Locale) -> Self {
        self.config.locale = locale;
        self
    }
    /// Adds or replaces an explicit proxy for its scope.
    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.config
            .proxies
            .retain(|existing| existing.scope != proxy.scope);
        self.config.proxies.push(proxy);
        self
    }
    /// Enables or disables environment/system proxy discovery.
    pub fn use_system_proxy(mut self, enabled: bool) -> Self {
        self.config.use_system_proxy = enabled;
        self
    }
    /// Sets the complete request timeout.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.config.request_timeout = timeout;
        self
    }
    /// Sets the connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }
    /// Sets the maximum accepted response size.
    pub fn max_response_bytes(mut self, bytes: usize) -> Self {
        self.config.max_response_bytes = bytes;
        self
    }
    /// Limits requests per second with a burst capacity of one.
    pub fn requests_per_second(mut self, rate: NonZeroU32) -> Self {
        self.config.requests_per_second = Some(rate);
        self
    }
    /// Sets the retry policy. Retries are disabled by default.
    pub fn retry_policy(mut self, policy: RetryPolicy) -> Self {
        self.config.retry_policy = policy;
        self
    }
    /// Disables server-certificate verification.
    ///
    /// This is dangerous and should only be used for controlled troubleshooting.
    pub fn danger_accept_invalid_certs(mut self, enabled: bool) -> Self {
        self.config.accept_invalid_certs = enabled;
        self
    }
    /// Creates the client and validates all configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when client or HTTP transport configuration is invalid.
    pub fn build(self) -> Result<Client> {
        if self.config.request_timeout.is_zero()
            || self.config.connect_timeout.is_zero()
            || self.config.max_response_bytes == 0
        {
            return Err(Error::Configuration {
                message: "timeouts and response size must be nonzero".into(),
            });
        }
        if self.config.retry_policy.max_delay < self.config.retry_policy.base_delay {
            return Err(Error::Configuration {
                message: "retry maximum delay must not be shorter than base delay".into(),
            });
        }
        let transport = Transport::new(&self.config)?;
        let spacing = self
            .config
            .requests_per_second
            .map(|rate| Duration::from_secs_f64(1.0 / f64::from(rate.get())));
        Ok(Client {
            config: Arc::new(self.config),
            transport: Arc::new(transport),
            throttle: Arc::new(Throttle {
                spacing,
                last: Mutex::new(None),
            }),
        })
    }
}

struct Throttle {
    spacing: Option<Duration>,
    last: Mutex<Option<Instant>>,
}
impl Throttle {
    fn wait(&self) {
        let Some(spacing) = self.spacing else {
            return;
        };
        let mut last = self.last.lock().expect("throttle lock poisoned");
        if let Some(previous) = *last {
            let remaining = spacing.saturating_sub(previous.elapsed());
            if !remaining.is_zero() {
                std::thread::sleep(remaining);
            }
        }
        *last = Some(Instant::now());
    }
}

/// Synchronous Google Play scraper client.
#[derive(Clone)]
pub struct Client {
    config: Arc<ClientConfig>,
    transport: Arc<dyn BlockingTransport>,
    throttle: Arc<Throttle>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a synchronous client builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }
    /// Creates a synchronous client with safe defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if the default HTTP transport cannot be constructed.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    #[cfg(test)]
    fn with_test_transport<T>(transport: Arc<T>) -> Self
    where
        T: BlockingTransport + 'static,
    {
        Self {
            config: Arc::new(ClientConfig::default()),
            transport,
            throttle: Arc::new(Throttle {
                spacing: None,
                last: Mutex::new(None),
            }),
        }
    }

    /// Fetches complete application metadata.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, a missing app, or
    /// an unrecognized upstream response.
    pub fn app(&self, request: impl Into<AppRequest>) -> Result<AppDetails> {
        let request = request.into();
        let app_id = validate_app_id(&request.app_id)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        let body = match self.execute(protocol::app_request(&app_id, locale)) {
            Err(Error::HttpStatus { status: 404 }) => {
                return Err(Error::AppNotFound {
                    app_id: app_id.to_string(),
                });
            }
            other => other?,
        };
        protocol::parse_app(&body, app_id)
    }

    /// Searches applications and follows continuation pages up to the requested limit.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, or an unrecognized
    /// upstream response.
    pub fn search(&self, request: impl Into<SearchQuery>) -> Result<Vec<AppOverview>> {
        let request = request.into();
        validate_nonempty("search term", &request.term)?;
        validate_limit("search limit", request.limit)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        let (mut items, mut token) = protocol::parse_initial_search(
            &self.execute(protocol::search_request(&request, locale))?,
        )?;
        let mut seen = HashSet::new();
        while items.len() < request.limit {
            let Some(next) = token.take() else {
                break;
            };
            if !seen.insert(next.expose().to_owned()) {
                tracing::warn!(
                    operation = "search",
                    "stopping after repeated continuation token"
                );
                break;
            }
            let (page, next_token) = protocol::parse_search_page(
                &self.execute(protocol::search_page_request(next.expose(), locale))?,
            )?;
            if page.is_empty() {
                break;
            }
            items.extend(page);
            token = next_token;
        }
        items.truncate(request.limit);
        Ok(items)
    }

    /// Fetches a Google Play application collection.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, or an unrecognized
    /// upstream response.
    pub fn list(&self, request: ListQuery) -> Result<Vec<AppOverview>> {
        validate_limit("list limit", request.limit)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        protocol::parse_list(&self.execute(protocol::list_request(&request, locale)?)?)
    }

    /// Fetches one review page.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, or an unrecognized
    /// upstream response.
    pub fn reviews(&self, request: ReviewQuery) -> Result<Page<Review>> {
        let app_id = validate_app_id(&request.app_id)?;
        validate_limit("review page size", request.page_size)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        protocol::parse_reviews(&self.execute(protocol::review_request(&request, &app_id, locale))?)
    }

    /// Fetches Google Play search suggestions.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, or an unrecognized
    /// upstream response.
    pub fn suggestions(&self, request: impl Into<SuggestionQuery>) -> Result<Vec<String>> {
        let request = request.into();
        validate_nonempty("suggestion term", &request.term)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        protocol::parse_suggestions(&self.execute(protocol::suggestion_request(&request, locale))?)
    }

    fn execute(&self, request: HttpRequest) -> Result<String> {
        let mut retry = 0;
        loop {
            self.throttle.wait();
            let result = self.transport.execute(request.clone()).and_then(classify);
            match result {
                Ok(body) => return Ok(body),
                Err(error)
                    if retry < self.config.retry_policy.max_retries
                        && matches!(error, Error::RateLimited { .. } | Error::Transport { .. }) =>
                {
                    let mut delay = self
                        .config
                        .retry_policy
                        .base_delay
                        .saturating_mul(1_u32 << retry.min(20))
                        .min(self.config.retry_policy.max_delay);
                    if self.config.retry_policy.honor_retry_after {
                        if let Error::RateLimited {
                            retry_after: Some(server),
                        } = &error
                        {
                            delay = (*server).min(self.config.retry_policy.max_delay);
                        }
                    }
                    std::thread::sleep(delay);
                    retry += 1;
                }
                Err(error) => return Err(error),
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/blocking_client.rs"]
mod tests;

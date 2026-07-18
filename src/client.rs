use std::{collections::HashSet, num::NonZeroU32, sync::Arc, time::Duration};

use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};

use crate::{
    AppDetails, AppOverview, AppRequest, Error, ListQuery, Locale, Page, Proxy, Result,
    RetryPolicy, Review, ReviewQuery, SearchQuery, SuggestionQuery,
    config::ClientConfig,
    protocol,
    request::{validate_app_id, validate_limit, validate_nonempty},
    transport::{DynAsyncTransport, HttpRequest, ReqwestTransport, classify},
};

/// Builder for the asynchronous [`Client`].
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
    /// Returns an error when a timeout, response limit, retry policy, proxy, or
    /// HTTP transport configuration is invalid.
    pub fn build(self) -> Result<Client> {
        validate_config(&self.config)?;
        let transport = Arc::new(ReqwestTransport::new(&self.config)?);
        Ok(Client::from_parts(self.config, transport))
    }
}

fn validate_config(config: &ClientConfig) -> Result<()> {
    if config.request_timeout.is_zero() {
        return Err(Error::Configuration {
            message: "request timeout must be nonzero".into(),
        });
    }
    if config.connect_timeout.is_zero() {
        return Err(Error::Configuration {
            message: "connect timeout must be nonzero".into(),
        });
    }
    if config.max_response_bytes == 0 {
        return Err(Error::Configuration {
            message: "maximum response size must be nonzero".into(),
        });
    }
    if config.retry_policy.max_delay < config.retry_policy.base_delay {
        return Err(Error::Configuration {
            message: "retry maximum delay must not be shorter than base delay".into(),
        });
    }
    Ok(())
}

/// Async Google Play scraper client.
#[derive(Clone)]
pub struct Client {
    config: Arc<ClientConfig>,
    transport: DynAsyncTransport,
    rate_limiter: Option<Arc<DefaultDirectRateLimiter>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl Client {
    /// Creates a client builder.
    pub fn builder() -> ClientBuilder {
        ClientBuilder::default()
    }

    /// Creates a client with safe defaults.
    ///
    /// # Errors
    ///
    /// Returns an error if the default HTTP transport cannot be constructed.
    pub fn new() -> Result<Self> {
        Self::builder().build()
    }

    fn from_parts(config: ClientConfig, transport: DynAsyncTransport) -> Self {
        let rate_limiter = config.requests_per_second.map(|rate| {
            Arc::new(RateLimiter::direct(
                Quota::per_second(rate).allow_burst(NonZeroU32::MIN),
            ))
        });
        Self {
            config: Arc::new(config),
            transport,
            rate_limiter,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_test_transport(transport: DynAsyncTransport) -> Self {
        Self::from_parts(ClientConfig::default(), transport)
    }

    #[cfg(test)]
    pub(crate) fn with_test_config(config: ClientConfig, transport: DynAsyncTransport) -> Self {
        Self::from_parts(config, transport)
    }

    /// Fetches complete application metadata.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, a missing app, or
    /// an unrecognized upstream response.
    pub async fn app(&self, request: impl Into<AppRequest>) -> Result<AppDetails> {
        let request = request.into();
        let app_id = validate_app_id(&request.app_id)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        let body = match self.execute(protocol::app_request(&app_id, locale)).await {
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
    pub async fn search(&self, request: impl Into<SearchQuery>) -> Result<Vec<AppOverview>> {
        let request = request.into();
        validate_nonempty("search term", &request.term)?;
        validate_limit("search limit", request.limit)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        let body = self
            .execute(protocol::search_request(&request, locale))
            .await?;
        let (mut items, mut token) = protocol::parse_initial_search(&body)?;
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
            let body = self
                .execute(protocol::search_page_request(next.expose(), locale))
                .await?;
            let (page, next_token) = protocol::parse_search_page(&body)?;
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
    pub async fn list(&self, request: ListQuery) -> Result<Vec<AppOverview>> {
        validate_limit("list limit", request.limit)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        let wire = protocol::list_request(&request, locale)?;
        protocol::parse_list(&self.execute(wire).await?)
    }

    /// Fetches one review page.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, or an unrecognized
    /// upstream response.
    pub async fn reviews(&self, request: ReviewQuery) -> Result<Page<Review>> {
        let app_id = validate_app_id(&request.app_id)?;
        validate_limit("review page size", request.page_size)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        let wire = protocol::review_request(&request, &app_id, locale);
        protocol::parse_reviews(&self.execute(wire).await?)
    }

    /// Fetches Google Play search suggestions.
    ///
    /// # Errors
    ///
    /// Returns an error for invalid input, transport failure, or an unrecognized
    /// upstream response.
    pub async fn suggestions(&self, request: impl Into<SuggestionQuery>) -> Result<Vec<String>> {
        let request = request.into();
        validate_nonempty("suggestion term", &request.term)?;
        let locale = request.locale.as_ref().unwrap_or(&self.config.locale);
        let wire = protocol::suggestion_request(&request, locale);
        protocol::parse_suggestions(&self.execute(wire).await?)
    }

    async fn execute(&self, request: HttpRequest) -> Result<String> {
        let policy = &self.config.retry_policy;
        let mut retry = 0;
        loop {
            if let Some(limiter) = &self.rate_limiter {
                limiter.until_ready().await;
            }
            let outcome = self
                .transport
                .execute(request.clone())
                .await
                .and_then(classify);
            match outcome {
                Ok(body) => return Ok(body),
                Err(error) if retry < policy.max_retries && retryable(&error) => {
                    let delay = retry_delay(policy, retry, &error);
                    tracing::warn!(attempt = retry + 1, delay_ms = delay.as_millis(), error_kind = ?error.kind(), "retrying request");
                    tokio_sleep(delay).await;
                    retry += 1;
                }
                Err(error) => return Err(error),
            }
        }
    }
}

fn retryable(error: &Error) -> bool {
    matches!(
        error,
        Error::RateLimited { .. } | Error::Transport { .. } | Error::HttpStatus { status: 503 }
    )
}

fn retry_delay(policy: &RetryPolicy, retry: u32, error: &Error) -> Duration {
    if policy.honor_retry_after {
        if let Error::RateLimited {
            retry_after: Some(delay),
        } = error
        {
            return (*delay).min(policy.max_delay);
        }
    }
    let exponent = retry.min(20);
    let base = policy
        .base_delay
        .saturating_mul(1_u32 << exponent)
        .min(policy.max_delay);
    let spread = policy.jitter_ratio;
    base.mul_f64((1.0 - spread) + fastrand::f64() * spread * 2.0)
        .min(policy.max_delay)
}

async fn tokio_sleep(delay: Duration) {
    #[cfg(feature = "cli")]
    {
        tokio::time::sleep(delay).await;
    }
    #[cfg(not(feature = "cli"))]
    {
        futures_timer::Delay::new(delay).await;
    }
}

#[cfg(test)]
#[path = "../tests/unit/client.rs"]
mod tests;

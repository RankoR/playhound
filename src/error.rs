use std::time::Duration;

/// The broad category of a [`Error`], useful for CLI exit-code mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Caller-provided input was invalid.
    InvalidInput,
    /// The requested application does not exist.
    NotFound,
    /// Google Play asked the caller to slow down.
    RateLimited,
    /// An HTTP request failed with a non-success status.
    HttpStatus,
    /// A connection, DNS, TLS, or timeout error occurred.
    Transport,
    /// The response exceeded the configured size bound.
    ResponseTooLarge,
    /// Google returned a response shape that is not recognized.
    UnexpectedResponse,
    /// A recognized response contained invalid data.
    Parse,
    /// Client construction failed.
    Configuration,
}

/// Errors returned by PlayHound.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Invalid caller input.
    #[error("invalid {field}: {message}")]
    InvalidInput {
        /// The invalid field.
        field: &'static str,
        /// A redacted explanation.
        message: String,
    },

    /// Application not found.
    #[error("application not found: {app_id}")]
    AppNotFound {
        /// Requested application ID.
        app_id: String,
    },

    /// Request rate-limited by the upstream service.
    #[error("request rate limited")]
    RateLimited {
        /// Server-provided delay, when present.
        retry_after: Option<Duration>,
    },

    /// An unexpected HTTP status.
    #[error("HTTP request failed with status {status}")]
    HttpStatus {
        /// Numeric HTTP status.
        status: u16,
    },

    /// Network or TLS failure.
    #[error("transport error: {message}")]
    Transport {
        /// Redacted transport message.
        message: String,
    },

    /// Response body larger than configured.
    #[error("response body exceeded configured limit of {limit} bytes")]
    ResponseTooLarge {
        /// Configured byte limit.
        limit: usize,
    },

    /// Upstream response no longer has a recognized shape.
    #[error("unexpected {operation} response: {message}")]
    UnexpectedResponse {
        /// Operation being parsed.
        operation: &'static str,
        /// A non-sensitive explanation.
        message: String,
    },

    /// Recognized data could not be converted.
    #[error("failed to parse {operation} response: {message}")]
    Parse {
        /// Operation being parsed.
        operation: &'static str,
        /// A non-sensitive explanation.
        message: String,
    },

    /// Invalid client configuration.
    #[error("client configuration error: {message}")]
    Configuration {
        /// A redacted explanation.
        message: String,
    },
}

impl Error {
    /// Returns the stable broad error category.
    pub const fn kind(&self) -> ErrorKind {
        match self {
            Self::InvalidInput { .. } => ErrorKind::InvalidInput,
            Self::AppNotFound { .. } => ErrorKind::NotFound,
            Self::RateLimited { .. } => ErrorKind::RateLimited,
            Self::HttpStatus { .. } => ErrorKind::HttpStatus,
            Self::Transport { .. } => ErrorKind::Transport,
            Self::ResponseTooLarge { .. } => ErrorKind::ResponseTooLarge,
            Self::UnexpectedResponse { .. } => ErrorKind::UnexpectedResponse,
            Self::Parse { .. } => ErrorKind::Parse,
            Self::Configuration { .. } => ErrorKind::Configuration,
        }
    }

    pub(crate) fn invalid(field: &'static str, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            field,
            message: message.into(),
        }
    }

    pub(crate) fn unexpected(operation: &'static str, message: impl Into<String>) -> Self {
        Self::UnexpectedResponse {
            operation,
            message: message.into(),
        }
    }
}

/// PlayHound result type.
pub type Result<T> = std::result::Result<T, Error>;

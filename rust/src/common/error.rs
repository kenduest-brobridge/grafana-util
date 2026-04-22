use thiserror::Error;

/// Canonical error type shared by all Rust CLI domains.
#[derive(Debug, Error)]
pub enum GrafanaCliError {
    #[error("{0}")]
    Message(String),
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    Tui(String),
    #[error("{0}")]
    Editor(String),
    #[error("Invalid URL for {context}: {details}")]
    Url { context: String, details: String },
    #[error("Invalid header name: {name}")]
    HeaderName { name: String },
    #[error("Invalid header value for {name}: {details}")]
    HeaderValue { name: String, details: String },
    #[error("Failed to parse {target}: {details}")]
    Parse { target: String, details: String },
    #[error("HTTP error {status_code} for {url}: {body}")]
    ApiResponse {
        status_code: u16,
        url: String,
        body: String,
    },
    #[error("{context}: {source}")]
    Context {
        context: String,
        #[source]
        source: Box<GrafanaCliError>,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("HTTP client error: {0}")]
    Http(#[from] reqwest::Error),
}

/// Repository-wide result alias using [`GrafanaCliError`].
pub type Result<T> = std::result::Result<T, GrafanaCliError>;

/// Build a plain user-facing CLI error message.
pub fn message(text: impl Into<String>) -> GrafanaCliError {
    GrafanaCliError::Message(text.into())
}

/// Build a structured local validation failure.
pub fn validation(text: impl Into<String>) -> GrafanaCliError {
    GrafanaCliError::Validation(text.into())
}

/// Build a structured terminal/TUI failure.
pub fn tui(text: impl Into<String>) -> GrafanaCliError {
    GrafanaCliError::Tui(text.into())
}

/// Build a structured external-editor failure.
pub fn editor(text: impl Into<String>) -> GrafanaCliError {
    GrafanaCliError::Editor(text.into())
}

/// Build a structured HTTP/API error with status code and response body context.
pub fn api_response(
    status_code: u16,
    url: impl Into<String>,
    body: impl Into<String>,
) -> GrafanaCliError {
    GrafanaCliError::ApiResponse {
        status_code,
        url: url.into(),
        body: body.into(),
    }
}

/// Build a structured URL parsing/validation failure.
pub fn invalid_url(context: impl Into<String>, source: impl std::fmt::Display) -> GrafanaCliError {
    GrafanaCliError::Url {
        context: context.into(),
        details: source.to_string(),
    }
}

/// Build a structured invalid-header-name failure.
pub fn invalid_header_name(name: impl Into<String>) -> GrafanaCliError {
    GrafanaCliError::HeaderName { name: name.into() }
}

/// Build a structured invalid-header-value failure.
pub fn invalid_header_value(
    name: impl Into<String>,
    source: impl std::fmt::Display,
) -> GrafanaCliError {
    GrafanaCliError::HeaderValue {
        name: name.into(),
        details: source.to_string(),
    }
}

/// Build a structured parsing failure for local text/value decoding.
pub fn parse_error(target: impl Into<String>, details: impl Into<String>) -> GrafanaCliError {
    GrafanaCliError::Parse {
        target: target.into(),
        details: details.into(),
    }
}

impl GrafanaCliError {
    /// Attach higher-level context while preserving the original typed error.
    pub fn with_context(self, context: impl Into<String>) -> Self {
        GrafanaCliError::Context {
            context: context.into(),
            source: Box::new(self),
        }
    }

    /// Return the HTTP status code for API errors and `None` for local failures.
    pub fn status_code(&self) -> Option<u16> {
        match self {
            GrafanaCliError::ApiResponse { status_code, .. } => Some(*status_code),
            GrafanaCliError::Context { source, .. } => source.status_code(),
            _ => None,
        }
    }

    /// Return a stable category label for shared error handling/reporting.
    pub fn kind(&self) -> &'static str {
        match self {
            GrafanaCliError::Message(_) => "message",
            GrafanaCliError::Validation(_) => "validation",
            GrafanaCliError::Tui(_) => "tui",
            GrafanaCliError::Editor(_) => "editor",
            GrafanaCliError::Url { .. } => "url",
            GrafanaCliError::HeaderName { .. } => "header-name",
            GrafanaCliError::HeaderValue { .. } => "header-value",
            GrafanaCliError::Parse { .. } => "parse",
            GrafanaCliError::Context { .. } => "context",
            GrafanaCliError::ApiResponse { .. } => "api-response",
            GrafanaCliError::Io(_) => "io",
            GrafanaCliError::Json(_) => "json",
            GrafanaCliError::Http(_) => "http",
        }
    }
}

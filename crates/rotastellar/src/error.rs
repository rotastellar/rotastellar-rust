//! RotaStellar SDK - Custom Errors
//!
//! All custom errors raised by the RotaStellar SDK.

use std::fmt;
use thiserror::Error;

/// Base error for all RotaStellar SDK errors.
#[derive(Error, Debug)]
pub enum RotaStellarError {
    /// Authentication error
    #[error("Authentication error: {0}")]
    Authentication(#[from] AuthenticationError),

    /// API error
    #[error("API error: {0}")]
    Api(#[from] ApiError),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}

/// Authentication errors.
#[derive(Error, Debug)]
pub enum AuthenticationError {
    /// API key is missing or empty.
    #[error("API key is required. Get your key at https://rotastellar.com/dashboard")]
    MissingApiKey,

    /// API key format is invalid.
    #[error("Invalid API key format: {masked_key}. Keys should start with 'rs_live_' or 'rs_test_'")]
    InvalidApiKey { masked_key: String },
}

impl AuthenticationError {
    /// Create a new InvalidApiKey error with a masked key.
    pub fn invalid_api_key(api_key: &str) -> Self {
        let masked = if api_key.len() > 10 {
            format!("{}...", &api_key[..10])
        } else {
            api_key.to_string()
        };
        Self::InvalidApiKey { masked_key: masked }
    }
}

/// API errors returned by the RotaStellar API.
#[derive(Error, Debug)]
pub struct ApiError {
    /// Error message
    pub message: String,
    /// HTTP status code
    pub status_code: u16,
    /// Request ID for debugging
    pub request_id: Option<String>,
    /// Additional details
    pub details: Option<serde_json::Value>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.status_code, self.message)?;
        if let Some(ref id) = self.request_id {
            write!(f, " (request_id: {})", id)?;
        }
        Ok(())
    }
}

impl ApiError {
    /// Create a new API error.
    pub fn new(message: impl Into<String>, status_code: u16) -> Self {
        Self {
            message: message.into(),
            status_code,
            request_id: None,
            details: None,
        }
    }

    /// Create a rate limit error.
    pub fn rate_limited(retry_after: Option<u32>) -> Self {
        Self {
            message: "Rate limit exceeded".to_string(),
            status_code: 429,
            request_id: None,
            details: retry_after.map(|r| serde_json::json!({ "retry_after": r })),
        }
    }

    /// Create a not found error.
    pub fn not_found(resource_type: &str, resource_id: &str) -> Self {
        Self {
            message: format!("{} not found: {}", resource_type, resource_id),
            status_code: 404,
            request_id: None,
            details: Some(serde_json::json!({
                "resource_type": resource_type,
                "resource_id": resource_id
            })),
        }
    }

    /// Set the request ID.
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }
}

/// Validation errors for input data.
#[derive(Error, Debug)]
#[error("Validation error on '{field}': {message}")]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Error message
    pub message: String,
}

impl ValidationError {
    /// Create a new validation error.
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
        }
    }
}

/// Network errors.
#[derive(Error, Debug)]
pub enum NetworkError {
    /// Request timed out.
    #[error("Request timed out after {0} seconds")]
    Timeout(f64),

    /// Connection failed.
    #[error("Connection failed: {0}")]
    Connection(String),

    /// Other network error.
    #[error("Network error: {0}")]
    Other(String),
}

/// Result type alias for RotaStellar operations.
pub type Result<T> = std::result::Result<T, RotaStellarError>;

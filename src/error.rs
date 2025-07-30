//! Error types for RSAirtable
//!
//! This module defines all error types that can occur when using the RSAirtable library.
//! The error types are designed to provide detailed information about what went wrong
//! and are compatible with the error patterns used in pyairtable.

use thiserror::Error;

/// Result type alias for RSAirtable operations
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for RSAirtable operations
#[derive(Error, Debug)]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response
    #[error("API error {status}: {message}")]
    Api { status: u16, message: String },

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Configuration error (missing API key, invalid base ID, etc.)
    #[error("Configuration error: {0}")]
    Config(String),

    /// URL building error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    Env(#[from] std::env::VarError),

    /// IO error (for file operations like reading .env files)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Rate limit exceeded
    #[error("Rate limit exceeded. Please wait {retry_after_ms}ms before retrying")]
    RateLimit { retry_after_ms: u64 },

    /// Record not found
    #[error("Record not found: {record_id}")]
    RecordNotFound { record_id: String },

    /// Table not found
    #[error("Table not found: {table_name}")]
    TableNotFound { table_name: String },

    /// Base not found or access denied
    #[error("Base not found or access denied: {base_id}")]
    BaseNotFound { base_id: String },

    /// Authentication failed
    #[error("Authentication failed: {message}")]
    Auth { message: String },

    /// Generic error for other cases
    #[error("Error: {0}")]
    Other(String),
}

impl Error {
    /// Create a new configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Error::Config(message.into())
    }

    /// Create a new API error from response
    pub fn api(status: u16, message: String) -> Self {
        Error::Api { status, message }
    }

    /// Create a new authentication error
    pub fn auth<S: Into<String>>(message: S) -> Self {
        Error::Auth {
            message: message.into(),
        }
    }

    /// Create a new rate limit error
    pub fn rate_limit(retry_after_ms: u64) -> Self {
        Error::RateLimit { retry_after_ms }
    }

    /// Create a new record not found error
    pub fn record_not_found<S: Into<String>>(record_id: S) -> Self {
        Error::RecordNotFound {
            record_id: record_id.into(),
        }
    }

    /// Create a new table not found error
    pub fn table_not_found<S: Into<String>>(table_name: S) -> Self {
        Error::TableNotFound {
            table_name: table_name.into(),
        }
    }

    /// Create a new base not found error
    pub fn base_not_found<S: Into<String>>(base_id: S) -> Self {
        Error::BaseNotFound {
            base_id: base_id.into(),
        }
    }
}

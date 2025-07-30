//! Configuration management for RSAirtable
//!
//! This module handles configuration loading with priority order:
//! 1. CLI arguments (highest priority)
//! 2. Environment variables
//! 3. .env file (lowest priority)
//!
//! This matches the configuration strategy used by pyairtable.

use crate::error::{Error, Result};
use std::env;

/// Configuration for RSAirtable client
#[derive(Debug, Clone)]
pub struct Config {
    /// Personal Access Token or API Key for Airtable
    pub api_key: String,
    /// Base URL for Airtable API (typically https://api.airtable.com/v0)
    pub endpoint_url: String,
    /// Default timeout for HTTP requests in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retries for failed requests
    pub max_retries: u32,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Config {
    /// Create a new configuration with the provided API key
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            endpoint_url: "https://api.airtable.com/v0".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            verbose: false,
        }
    }

    /// Load configuration from environment variables and .env file
    /// 
    /// This follows the pyairtable pattern of checking multiple possible env var names:
    /// - PERSONAL_ACCESS_TOKEN (preferred for Personal Access Tokens)
    /// - AIRTABLE_API_KEY (legacy API key support)
    /// - AIRTABLE_ACCESS_TOKEN (alternative name)
    pub fn from_env() -> Result<Self> {
        // Load .env file if it exists (ignore errors if file doesn't exist)
        let _ = dotenv::dotenv();

        // Try to get API key from environment variables in priority order
        let api_key = env::var("PERSONAL_ACCESS_TOKEN")
            .or_else(|_| env::var("AIRTABLE_API_KEY"))
            .or_else(|_| env::var("AIRTABLE_ACCESS_TOKEN"))
            .map_err(|_| {
                Error::config(
                    "API key not found. Set PERSONAL_ACCESS_TOKEN, AIRTABLE_API_KEY, or AIRTABLE_ACCESS_TOKEN environment variable"
                )
            })?;

        let mut config = Self::new(api_key);

        // Override defaults with environment variables if present
        if let Ok(url) = env::var("AIRTABLE_ENDPOINT_URL") {
            config.endpoint_url = url;
        }

        if let Ok(timeout) = env::var("AIRTABLE_TIMEOUT_SECONDS") {
            if let Ok(timeout_val) = timeout.parse::<u64>() {
                config.timeout_seconds = timeout_val;
            }
        }

        if let Ok(retries) = env::var("AIRTABLE_MAX_RETRIES") {
            if let Ok(retries_val) = retries.parse::<u32>() {
                config.max_retries = retries_val;
            }
        }

        if let Ok(verbose) = env::var("AIRTABLE_VERBOSE") {
            config.verbose = verbose.to_lowercase() == "true" || verbose == "1";
        }

        Ok(config)
    }

    /// Get API key from multiple possible sources
    pub fn api_key_from_env_or_file() -> Result<String> {
        // Load .env file if it exists
        let _ = dotenv::dotenv();

        env::var("PERSONAL_ACCESS_TOKEN")
            .or_else(|_| env::var("AIRTABLE_API_KEY"))
            .or_else(|_| env::var("AIRTABLE_ACCESS_TOKEN"))
            .map_err(|_| {
                Error::config(
                    "API key not found. Set PERSONAL_ACCESS_TOKEN, AIRTABLE_API_KEY, or AIRTABLE_ACCESS_TOKEN"
                )
            })
    }

    /// Get base ID from environment variable with fallback
    pub fn base_id_from_env(fallback: Option<&str>) -> Result<String> {
        env::var("AIRTABLE_BASE_ID")
            .or_else(|_| {
                if let Some(fallback_val) = fallback {
                    Ok(fallback_val.to_string())
                } else {
                    Err(env::VarError::NotPresent)
                }
            })
            .map_err(|_| Error::config("Base ID not found. Set AIRTABLE_BASE_ID environment variable or provide as argument"))
    }

    /// Get table name from environment variable with fallback
    pub fn table_name_from_env(fallback: Option<&str>) -> Result<String> {
        env::var("AIRTABLE_TABLE_NAME")
            .or_else(|_| {
                if let Some(fallback_val) = fallback {
                    Ok(fallback_val.to_string())
                } else {
                    Err(env::VarError::NotPresent)
                }
            })
            .map_err(|_| Error::config("Table name not found. Set AIRTABLE_TABLE_NAME environment variable or provide as argument"))
    }

    /// Set verbose logging
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Set custom endpoint URL
    pub fn with_endpoint_url<S: Into<String>>(mut self, url: S) -> Self {
        self.endpoint_url = url.into();
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Set max retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }
}
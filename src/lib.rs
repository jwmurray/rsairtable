//! # RSAirtable
//!
//! Rust client library for Airtable API, compatible with pyairtable.
//!
//! This library provides a comprehensive Rust interface to the Airtable API,
//! designed to be functionally compatible with the popular pyairtable Python library.
//!
//! ## Features
//!
//! - Full Airtable API coverage (records, schema, comments, attachments)
//! - Async/await support with tokio
//! - Type-safe serialization with serde
//! - CLI tool with pyairtable-compatible interface
//! - Flexible authentication (API key, environment variables, .env files)
//! - Rate limiting and error handling
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rsairtable::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::new("your_api_key".to_string());
//!     
//!     let (records, _offset) = client
//!         .base("appXXXXXXXXXXXXXX")
//!         .table("Table Name")
//!         .list()
//!         .max_records(10)
//!         .execute()
//!         .await?;
//!     
//!     println!("Retrieved {} records", records.len());
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod config;
pub mod error;
pub mod models;

pub use client::Client;
pub use config::Config;
pub use error::{Error, Result};
pub use models::*;

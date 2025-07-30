#![allow(dead_code)]
#![allow(unused_imports)]
//! Main client implementation for RSAirtable
//!
//! This module provides the core Client struct and its methods for interacting
//! with the Airtable API. The design closely follows pyairtable's client structure
//! for maximum compatibility.

use crate::config::Config;
use crate::error::{Error, Result};
use crate::models::*;
use reqwest::{header, Client as HttpClient};
use serde_json::json;
use std::time::Duration;
use url::Url;

/// Main client for interacting with Airtable API
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client for making requests
    http_client: HttpClient,
    /// Client configuration
    config: Config,
}

impl Client {
    /// Create a new client with the given API key
    pub fn new(api_key: String) -> Self {
        let config = Config::new(api_key);
        Self::from_config(config)
    }

    /// Create a new client from environment variables
    pub fn from_env() -> Result<Self> {
        let config = Config::from_env()?;
        Ok(Self::from_config(config))
    }

    /// Create a new client from configuration
    pub fn from_config(config: Config) -> Self {
        let mut headers = header::HeaderMap::new();

        // Set authorization header with Bearer token
        let auth_value = format!("Bearer {}", config.api_key);
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&auth_value).expect("Invalid API key format"),
        );

        // Set user agent
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("rsairtable/0.1.0"),
        );

        // Build HTTP client with timeout and headers
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .expect("Failed to create HTTP client");

        Self {
            http_client,
            config,
        }
    }

    /// Get a base handle for the given base ID
    pub fn base(&self, base_id: &str) -> BaseHandle {
        BaseHandle {
            client: self.clone(),
            base_id: base_id.to_string(),
        }
    }

    /// Get user information (whoami)
    pub async fn whoami(&self) -> Result<UserInfo> {
        let url = format!("{}/meta/whoami", self.config.endpoint_url);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let user_info: UserInfo = response.json().await?;
        Ok(user_info)
    }

    /// List all accessible bases
    pub async fn bases(&self) -> Result<Vec<BaseInfo>> {
        let url = format!("{}/meta/bases", self.config.endpoint_url);
        let response = self.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(self.parse_error_response(response).await);
        }

        let bases_response: BasesResponse = response.json().await?;
        Ok(bases_response.bases)
    }

    /// Parse error response from API
    async fn parse_error_response(&self, response: reqwest::Response) -> Error {
        let status = response.status().as_u16();

        // Try to parse structured error response
        if let Ok(error_response) = response.json::<ErrorResponse>().await {
            Error::api(status, error_response.error.message)
        } else {
            // Fallback to generic error message
            Error::api(status, format!("HTTP {}", status))
        }
    }

    /// Get the client configuration
    pub fn config(&self) -> &Config {
        &self.config
    }
}

/// Handle for operations on a specific base
#[derive(Debug, Clone)]
pub struct BaseHandle {
    client: Client,
    base_id: String,
}

impl BaseHandle {
    /// Get a table handle for the given table name
    pub fn table(&self, table_name: &str) -> TableHandle {
        TableHandle {
            base: self.clone(),
            table_name: table_name.to_string(),
        }
    }

    /// Get base schema information
    pub async fn schema(&self) -> Result<BaseSchema> {
        let url = format!(
            "{}/meta/bases/{}/tables",
            self.client.config.endpoint_url, self.base_id
        );
        let response = self.client.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(self.client.parse_error_response(response).await);
        }

        let schema: BaseSchema = response.json().await?;
        Ok(schema)
    }

    /// Get base ID
    pub fn id(&self) -> &str {
        &self.base_id
    }
}

/// Handle for operations on a specific table
#[derive(Debug, Clone)]
pub struct TableHandle {
    base: BaseHandle,
    table_name: String,
}

impl TableHandle {
    /// Create a query builder for listing records
    pub fn list(&self) -> ListRecordsQuery {
        ListRecordsQuery {
            table: self.clone(),
            max_records: None,
            page_size: None,
            fields: None,
            filter_by_formula: None,
            sort: None,
            view: None,
            cell_format: None,
            time_zone: None,
            user_locale: None,
            return_fields_by_field_id: None,
        }
    }

    /// Build URL for table operations
    fn build_url(&self, path: &str) -> String {
        format!(
            "{}/{}/{}/{}",
            self.base.client.config.endpoint_url,
            self.base.base_id,
            urlencoding::encode(&self.table_name),
            path
        )
    }

    /// Get table name
    pub fn name(&self) -> &str {
        &self.table_name
    }
}

/// Query builder for listing records with various filters and options
#[derive(Debug, Clone)]
pub struct ListRecordsQuery {
    table: TableHandle,
    max_records: Option<u32>,
    page_size: Option<u32>,
    fields: Option<Vec<String>>,
    filter_by_formula: Option<String>,
    sort: Option<Vec<String>>,
    view: Option<String>,
    cell_format: Option<String>,
    time_zone: Option<String>,
    user_locale: Option<String>,
    return_fields_by_field_id: Option<bool>,
}

impl ListRecordsQuery {
    /// Set maximum number of records to return
    pub fn max_records(mut self, max_records: u32) -> Self {
        self.max_records = Some(max_records);
        self
    }

    /// Set page size for pagination
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Set specific fields to return
    pub fn fields(mut self, fields: Vec<String>) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Set filter formula
    pub fn filter_by_formula<S: Into<String>>(mut self, formula: S) -> Self {
        self.filter_by_formula = Some(formula.into());
        self
    }

    /// Set sort order
    pub fn sort(mut self, sort: Vec<String>) -> Self {
        self.sort = Some(sort);
        self
    }

    /// Set view name
    pub fn view<S: Into<String>>(mut self, view: S) -> Self {
        self.view = Some(view.into());
        self
    }

    /// Execute the query and return all matching records
    pub async fn execute(self) -> Result<Vec<Record>> {
        let mut all_records = Vec::new();
        let mut offset: Option<String> = None;

        loop {
            let mut url = Url::parse(&self.table.build_url(""))?;

            // Add query parameters
            let mut query_pairs = url.query_pairs_mut();

            if let Some(max_records) = self.max_records {
                query_pairs.append_pair("maxRecords", &max_records.to_string());
            }

            if let Some(page_size) = self.page_size {
                query_pairs.append_pair("pageSize", &page_size.to_string());
            }

            if let Some(ref fields) = self.fields {
                for field in fields {
                    query_pairs.append_pair("fields[]", field);
                }
            }

            if let Some(ref formula) = self.filter_by_formula {
                query_pairs.append_pair("filterByFormula", formula);
            }

            if let Some(ref view) = self.view {
                query_pairs.append_pair("view", view);
            }

            if let Some(ref offset_val) = offset {
                query_pairs.append_pair("offset", offset_val);
            }

            drop(query_pairs);

            // Make the request
            let response = self
                .table
                .base
                .client
                .http_client
                .get(url.as_str())
                .send()
                .await?;

            if !response.status().is_success() {
                return Err(self.table.base.client.parse_error_response(response).await);
            }

            let list_response: ListRecordsResponse = response.json().await?;
            all_records.extend(list_response.records);

            // Check if we have more pages
            if let Some(next_offset) = list_response.offset {
                offset = Some(next_offset);

                // Respect max_records limit
                if let Some(max) = self.max_records {
                    if all_records.len() >= max as usize {
                        all_records.truncate(max as usize);
                        break;
                    }
                }
            } else {
                break;
            }
        }

        Ok(all_records)
    }
}

// Add urlencoding dependency to Cargo.toml when we test
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new("test_key".to_string());
        assert_eq!(client.config.api_key, "test_key");
    }

    #[test]
    fn test_base_handle() {
        let client = Client::new("test_key".to_string());
        let base = client.base("appTestBase123");
        assert_eq!(base.id(), "appTestBase123");
    }

    #[test]
    fn test_table_handle() {
        let client = Client::new("test_key".to_string());
        let table = client.base("appTestBase123").table("TestTable");
        assert_eq!(table.name(), "TestTable");
    }
}

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

fn parse_records(json: &serde_json::Value) -> Vec<Record> {
    json.get("records")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|rec| serde_json::from_value(rec.clone()).ok())
                .collect()
        })
        .unwrap_or_default()
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
            offset: None,
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

    /// Get all records from the table (convenience method)
    pub async fn all(&self) -> Result<Vec<Record>> {
        let mut all_records = Vec::new();
        let mut offset = None;
        loop {
            let (records, next_offset) = self.list().offset(offset.clone()).execute().await?;
            all_records.extend(records);
            if next_offset.is_none() {
                break;
            }
            offset = next_offset;
        }
        Ok(all_records)
    }

    /// Get a single record by ID
    pub async fn get(&self, record_id: &str) -> Result<Record> {
        let url = self.build_url(record_id);
        let response = self.base.client.http_client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let record: Record = response.json().await?;
        Ok(record)
    }

    /// Get the first record (optionally with filters)
    pub fn first(&self) -> FirstRecordQuery {
        FirstRecordQuery {
            table: self.clone(),
            fields: None,
            filter_by_formula: None,
            view: None,
        }
    }

    /// Create a single record
    pub async fn create(&self, fields: serde_json::Value) -> Result<Record> {
        let request_body = json!({
            "fields": fields,
            "typecast": false
        });

        let url = self.build_url("");
        let response = self
            .base
            .client
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let record: Record = response.json().await?;
        Ok(record)
    }

    /// Create a single record with typecast option
    pub async fn create_with_typecast(
        &self,
        fields: serde_json::Value,
        typecast: bool,
    ) -> Result<Record> {
        let request_body = json!({
            "fields": fields,
            "typecast": typecast
        });

        let url = self.build_url("");
        let response = self
            .base
            .client
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let record: Record = response.json().await?;
        Ok(record)
    }

    /// Batch create multiple records
    pub async fn batch_create(&self, records_data: Vec<serde_json::Value>) -> Result<Vec<Record>> {
        if records_data.is_empty() {
            return Err(Error::Api {
                status: 400,
                message: "Cannot create empty batch: records_data cannot be empty".to_string(),
            });
        }

        if records_data.len() > 10 {
            return Err(Error::Api {
                status: 400,
                message: "Batch size too large: Maximum 10 records per batch".to_string(),
            });
        }

        let records: Vec<serde_json::Value> = records_data
            .into_iter()
            .map(|fields| json!({"fields": fields}))
            .collect();

        let request_body = json!({
            "records": records,
            "typecast": false
        });

        let url = self.build_url("");
        let response = self
            .base
            .client
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let response_data: ListRecordsResponse = response.json().await?;
        Ok(response_data.records)
    }

    /// Batch create multiple records with options
    pub async fn batch_create_with_options(
        &self,
        records_data: Vec<serde_json::Value>,
        typecast: bool,
        _return_fields: &[&str],
    ) -> Result<Vec<Record>> {
        if records_data.is_empty() {
            return Err(Error::Api {
                status: 400,
                message: "Cannot create empty batch: records_data cannot be empty".to_string(),
            });
        }

        if records_data.len() > 10 {
            return Err(Error::Api {
                status: 400,
                message: "Batch size too large: Maximum 10 records per batch".to_string(),
            });
        }

        let records: Vec<serde_json::Value> = records_data
            .into_iter()
            .map(|fields| json!({"fields": fields}))
            .collect();

        let request_body = json!({
            "records": records,
            "typecast": typecast
        });

        // For now, ignore return_fields in batch creation to avoid API validation errors
        // The Airtable API may not support field filtering in batch creation

        let url = self.build_url("");
        let response = self
            .base
            .client
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let response_data: ListRecordsResponse = response.json().await?;
        Ok(response_data.records)
    }

    /// Update a single record
    pub async fn update(&self, record_id: &str, fields: serde_json::Value) -> Result<Record> {
        let request_body = json!({
            "fields": fields,
            "typecast": false
        });

        let url = self.build_url(record_id);
        let response = self
            .base
            .client
            .http_client
            .patch(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let record: Record = response.json().await?;
        Ok(record)
    }

    /// Update a single record with typecast option
    pub async fn update_with_typecast(
        &self,
        record_id: &str,
        fields: serde_json::Value,
        typecast: bool,
    ) -> Result<Record> {
        let request_body = json!({
            "fields": fields,
            "typecast": typecast
        });

        let url = self.build_url(record_id);
        let response = self
            .base
            .client
            .http_client
            .patch(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let record: Record = response.json().await?;
        Ok(record)
    }

    /// Batch update multiple records
    pub async fn batch_update(&self, records_data: Vec<serde_json::Value>) -> Result<Vec<Record>> {
        if records_data.is_empty() {
            return Err(Error::Api {
                status: 400,
                message: "Cannot update empty batch: records_data cannot be empty".to_string(),
            });
        }

        if records_data.len() > 10 {
            return Err(Error::Api {
                status: 400,
                message: "Batch size too large: Maximum 10 records per batch".to_string(),
            });
        }

        let request_body = json!({
            "records": records_data,
            "typecast": false
        });

        let url = self.build_url("");
        let response = self
            .base
            .client
            .http_client
            .patch(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let response_data: ListRecordsResponse = response.json().await?;
        Ok(response_data.records)
    }

    /// Batch upsert multiple records (create or update based on matching fields)
    pub async fn batch_upsert(
        &self,
        records_data: Vec<serde_json::Value>,
        fields_to_merge_on: &[&str],
    ) -> Result<Vec<Record>> {
        if records_data.is_empty() {
            return Err(Error::Api {
                status: 400,
                message: "Cannot upsert empty batch: records_data cannot be empty".to_string(),
            });
        }

        if records_data.len() > 10 {
            return Err(Error::Api {
                status: 400,
                message: "Batch size too large: Maximum 10 records per batch".to_string(),
            });
        }

        let request_body = json!({
            "records": records_data,
            "performUpsert": {
                "fieldsToMergeOn": fields_to_merge_on
            },
            "typecast": false
        });

        let url = self.build_url("");
        let response = self
            .base
            .client
            .http_client
            .patch(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        let response_data: ListRecordsResponse = response.json().await?;
        Ok(response_data.records)
    }

    /// Delete a single record
    pub async fn delete(&self, record_id: &str) -> Result<()> {
        let url = self.build_url(record_id);
        let response = self.base.client.http_client.delete(&url).send().await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        Ok(())
    }

    /// Batch delete multiple records
    pub async fn batch_delete(&self, record_ids: &[String]) -> Result<()> {
        if record_ids.is_empty() {
            return Err(Error::Api {
                status: 400,
                message: "Cannot delete empty batch: record_ids cannot be empty".to_string(),
            });
        }

        if record_ids.len() > 10 {
            return Err(Error::Api {
                status: 400,
                message: "Batch size too large: Maximum 10 records per batch".to_string(),
            });
        }

        let mut url = Url::parse(&self.build_url(""))?;
        {
            let mut query_pairs = url.query_pairs_mut();
            for record_id in record_ids {
                query_pairs.append_pair("records[]", record_id);
            }
        }

        let response = self
            .base
            .client
            .http_client
            .delete(url.as_str())
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.base.client.parse_error_response(response).await);
        }

        Ok(())
    }

    /// Create record using query builder pattern
    pub fn create_record(&self) -> CreateRecordQuery {
        CreateRecordQuery {
            table: self.clone(),
            fields: None,
            typecast: None,
            return_fields: None,
        }
    }

    /// Update record using query builder pattern
    pub fn update_record(&self, record_id: &str) -> UpdateRecordQuery {
        UpdateRecordQuery {
            table: self.clone(),
            record_id: record_id.to_string(),
            fields: None,
            typecast: None,
            return_fields: None,
        }
    }

    /// Create an iterator for paginated record retrieval
    pub fn iterate(&self) -> RecordIteratorBuilder {
        RecordIteratorBuilder {
            table: self.clone(),
            page_size: None,
            fields: None,
            filter_by_formula: None,
            view: None,
            sort: None,
        }
    }

    /// Create a query builder for complex record selection
    pub fn select(&self) -> SelectQueryBuilder {
        SelectQueryBuilder {
            table: self.clone(),
            fields: None,
            filter_by_formula: None,
            sort: None,
            view: None,
            max_records: None,
        }
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
    offset: Option<String>, // <-- Add this field
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
    pub fn fields(mut self, fields: &[&str]) -> Self {
        self.fields = Some(fields.iter().map(|s| s.to_string()).collect());
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
    pub async fn execute(self) -> Result<(Vec<Record>, Option<String>)> {
        let mut url = Url::parse(&self.table.build_url(""))?;
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

        if let Some(ref offset_val) = self.offset {
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

        let response_json: serde_json::Value = response.json().await?;
        let records = parse_records(&response_json); // Use your existing record parsing logic
        let next_offset = response_json
            .get("offset")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        Ok((records, next_offset))
    }

    pub fn offset(mut self, offset: Option<String>) -> Self {
        self.offset = offset;
        self
    }
}

// Add urlencoding dependency to Cargo.toml when we test
/// Query builder for getting the first record with optional filters
#[derive(Debug, Clone)]
pub struct FirstRecordQuery {
    table: TableHandle,
    fields: Option<Vec<String>>,
    filter_by_formula: Option<String>,
    view: Option<String>,
}

impl FirstRecordQuery {
    /// Specify fields to return
    pub fn fields(mut self, fields: &[&str]) -> Self {
        self.fields = Some(fields.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Add filter formula
    pub fn filter_by_formula<S: Into<String>>(mut self, formula: S) -> Self {
        self.filter_by_formula = Some(formula.into());
        self
    }

    /// Specify view to use
    pub fn view<S: Into<String>>(mut self, view: S) -> Self {
        self.view = Some(view.into());
        self
    }

    /// Execute and return the first matching record
    pub async fn execute(self) -> Result<Record> {
        let mut query = self.table.list().max_records(1);

        if let Some(ref fields) = self.fields {
            query = query.fields(&fields.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        }

        if let Some(formula) = self.filter_by_formula {
            query = query.filter_by_formula(&formula);
        }

        if let Some(view) = self.view {
            query = query.view(&view);
        }

        let (records, _) = query.execute().await?;
        records
            .into_iter()
            .next()
            .ok_or_else(|| crate::Error::RecordNotFound {
                record_id: "first record".to_string(),
            })
    }
}

/// Builder for creating paginated record iterators
#[derive(Debug, Clone)]
pub struct RecordIteratorBuilder {
    table: TableHandle,
    page_size: Option<u32>,
    fields: Option<Vec<String>>,
    filter_by_formula: Option<String>,
    view: Option<String>,
    sort: Option<Vec<String>>,
}

impl RecordIteratorBuilder {
    /// Set page size for pagination
    pub fn page_size(mut self, size: u32) -> Self {
        self.page_size = Some(size);
        self
    }

    /// Specify fields to return
    pub fn fields(mut self, fields: &[&str]) -> Self {
        self.fields = Some(fields.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Add filter formula
    pub fn filter_by_formula<S: Into<String>>(mut self, formula: S) -> Self {
        self.filter_by_formula = Some(formula.into());
        self
    }

    /// Specify view to use
    pub fn view<S: Into<String>>(mut self, view: S) -> Self {
        self.view = Some(view.into());
        self
    }

    /// Build the iterator
    pub async fn build(self) -> Result<RecordIterator> {
        Ok(RecordIterator {
            table: self.table,
            page_size: self.page_size.unwrap_or(100),
            fields: self.fields,
            filter_by_formula: self.filter_by_formula,
            view: self.view,
            sort: self.sort,
            offset: None,
            finished: false,
        })
    }
}

/// Iterator for paginated record retrieval
#[derive(Debug)]
pub struct RecordIterator {
    table: TableHandle,
    page_size: u32,
    fields: Option<Vec<String>>,
    filter_by_formula: Option<String>,
    view: Option<String>,
    sort: Option<Vec<String>>,
    offset: Option<String>,
    finished: bool,
}

impl RecordIterator {
    /// Get the next batch of records
    pub async fn next(&mut self) -> Option<Result<Vec<Record>>> {
        if self.finished {
            return None;
        }

        let mut query = self.table.list().page_size(self.page_size);

        if let Some(ref fields) = self.fields {
            query = query.fields(&fields.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        }

        if let Some(ref formula) = self.filter_by_formula {
            query = query.filter_by_formula(formula);
        }

        if let Some(ref view) = self.view {
            query = query.view(view);
        }

        if let Some(ref offset) = self.offset {
            query = query.offset(Some(offset.clone()));
        }

        // Execute single page
        let result = query.execute().await;
        match result {
            Ok((records, next_offset)) => {
                if next_offset.is_none() || records.len() < self.page_size as usize {
                    self.finished = true;
                }
                self.offset = next_offset;
                Some(Ok(records))
            }
            Err(e) => {
                self.finished = true;
                Some(Err(e))
            }
        }
    }
}

/// Query builder for complex record selection
#[derive(Debug, Clone)]
pub struct SelectQueryBuilder {
    table: TableHandle,
    fields: Option<Vec<String>>,
    filter_by_formula: Option<String>,
    sort: Option<Vec<(String, String)>>,
    view: Option<String>,
    max_records: Option<u32>,
}

impl SelectQueryBuilder {
    /// Specify fields to return
    pub fn fields(mut self, fields: &[&str]) -> Self {
        self.fields = Some(fields.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Add filter formula
    pub fn filter_by_formula<S: Into<String>>(mut self, formula: S) -> Self {
        self.filter_by_formula = Some(formula.into());
        self
    }

    /// Add sorting
    pub fn sort(mut self, sort_spec: &[(&str, &str)]) -> Self {
        self.sort = Some(
            sort_spec
                .iter()
                .map(|(field, direction)| (field.to_string(), direction.to_string()))
                .collect(),
        );
        self
    }

    /// Specify view to use
    pub fn view<S: Into<String>>(mut self, view: S) -> Self {
        self.view = Some(view.into());
        self
    }

    /// Set maximum number of records
    pub fn max_records(mut self, max: u32) -> Self {
        self.max_records = Some(max);
        self
    }

    /// Execute the query
    pub async fn execute(self) -> Result<Vec<Record>> {
        let mut query = self.table.list();

        if let Some(ref fields) = self.fields {
            query = query.fields(&fields.iter().map(|s| s.as_str()).collect::<Vec<_>>());
        }

        if let Some(formula) = self.filter_by_formula {
            query = query.filter_by_formula(&formula);
        }

        if let Some(view) = self.view {
            query = query.view(&view);
        }

        if let Some(max) = self.max_records {
            query = query.max_records(max);
        }

        let (records, _) = query.execute().await?;
        Ok(records)
    }
}

/// Query builder for record creation
#[derive(Debug, Clone)]
pub struct CreateRecordQuery {
    table: TableHandle,
    fields: Option<serde_json::Value>,
    typecast: Option<bool>,
    return_fields: Option<Vec<String>>,
}

impl CreateRecordQuery {
    /// Set fields for the new record
    pub fn fields(mut self, fields: serde_json::Value) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Enable typecast for the creation
    pub fn typecast(mut self, typecast: bool) -> Self {
        self.typecast = Some(typecast);
        self
    }

    /// Specify which fields to return in the response
    pub fn return_fields(mut self, fields: &[&str]) -> Self {
        self.return_fields = Some(fields.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Execute the record creation
    pub async fn execute(self) -> Result<Record> {
        let fields = self.fields.ok_or_else(|| Error::Api {
            status: 400,
            message: "Missing fields: fields must be specified for record creation".to_string(),
        })?;

        let request_body = json!({
            "fields": fields,
            "typecast": self.typecast.unwrap_or(false)
        });

        // Note: return_fields functionality not implemented for now
        // The Airtable API may not support field filtering in single record creation

        let url = self.table.build_url("");
        let response = self
            .table
            .base
            .client
            .http_client
            .post(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.table.base.client.parse_error_response(response).await);
        }

        let record: Record = response.json().await?;
        Ok(record)
    }
}

/// Query builder for record updates
#[derive(Debug, Clone)]
pub struct UpdateRecordQuery {
    table: TableHandle,
    record_id: String,
    fields: Option<serde_json::Value>,
    typecast: Option<bool>,
    return_fields: Option<Vec<String>>,
}

impl UpdateRecordQuery {
    /// Set fields to update
    pub fn fields(mut self, fields: serde_json::Value) -> Self {
        self.fields = Some(fields);
        self
    }

    /// Enable typecast for the update
    pub fn typecast(mut self, typecast: bool) -> Self {
        self.typecast = Some(typecast);
        self
    }

    /// Specify which fields to return in the response
    pub fn return_fields(mut self, fields: &[&str]) -> Self {
        self.return_fields = Some(fields.iter().map(|s| s.to_string()).collect());
        self
    }

    /// Execute the record update
    pub async fn execute(self) -> Result<Record> {
        let fields = self.fields.ok_or_else(|| Error::Api {
            status: 400,
            message: "Missing fields: fields must be specified for record update".to_string(),
        })?;

        let request_body = json!({
            "fields": fields,
            "typecast": self.typecast.unwrap_or(false)
        });

        // Note: return_fields functionality not implemented for now
        // The Airtable API may not support field filtering in single record updates

        let url = self.table.build_url(&self.record_id);
        let response = self
            .table
            .base
            .client
            .http_client
            .patch(&url)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.table.base.client.parse_error_response(response).await);
        }

        let record: Record = response.json().await?;
        Ok(record)
    }
}

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

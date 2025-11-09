//! Data models for Airtable API responses
//!
//! This module contains all the data structures used to represent Airtable data,
//! matching the schema defined in our OpenAPI specification and compatible with
//! pyairtable's data structures.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// A single Airtable record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Unique record identifier (e.g., "recXXXXXXXXXXXXXX")
    pub id: String,
    /// Record creation timestamp
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
    /// Record fields as key-value pairs
    pub fields: Fields,
}

/// Record fields - dynamic key-value structure
/// Uses BTreeMap to maintain alphabetical ordering of field names in JSON output
pub type Fields = BTreeMap<String, serde_json::Value>;

/// Response from list records API call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListRecordsResponse {
    /// Array of records
    pub records: Vec<Record>,
    /// Pagination offset for next page (if any)
    pub offset: Option<String>,
}

/// Request body for creating/updating records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecordRequest {
    /// Record fields to set
    pub fields: Fields,
    /// Whether to return field objects instead of string values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_fields_by_field_id: Option<bool>,
}

/// Request body for batch operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRequest {
    /// Array of records to process
    pub records: Vec<CreateRecordRequest>,
    /// Whether to return field objects instead of string values  
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_fields_by_field_id: Option<bool>,
}

/// Request body for batch upsert operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchUpsertRequest {
    /// Array of records to upsert
    pub records: Vec<CreateRecordRequest>,
    /// Fields to use for matching existing records
    #[serde(rename = "fieldsToMergeOn")]
    pub fields_to_merge_on: Vec<String>,
    /// Whether to return field objects instead of string values
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_fields_by_field_id: Option<bool>,
}

/// Comment on a record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    /// Comment ID
    pub id: String,
    /// Comment author information
    pub author: Collaborator,
    /// Comment text content
    pub text: String,
    /// Comment creation timestamp
    #[serde(rename = "createdTime")]
    pub created_time: DateTime<Utc>,
}

/// User/collaborator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collaborator {
    /// User ID
    pub id: String,
    /// User display name
    pub name: String,
    /// User email address
    pub email: String,
}

/// Table schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    /// Table ID
    pub id: String,
    /// Table name
    pub name: String,
    /// Primary field ID
    #[serde(rename = "primaryFieldId")]
    pub primary_field_id: String,
    /// Array of field definitions
    pub fields: Vec<FieldSchema>,
    /// Array of view definitions
    pub views: Vec<ViewSchema>,
}

/// Field schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSchema {
    /// Field ID
    pub id: String,
    /// Field name
    pub name: String,
    /// Field type (e.g., "singleLineText", "multipleRecordLinks", etc.)
    #[serde(rename = "type")]
    pub field_type: String,
    /// Field options (varies by field type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Value>,
    /// Field description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// View schema definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewSchema {
    /// View ID
    pub id: String,
    /// View name
    pub name: String,
    /// View type (e.g., "grid", "kanban", "gallery", etc.)
    #[serde(rename = "type")]
    pub view_type: String,
}

/// Base schema information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseSchema {
    /// Array of table schemas in the base
    pub tables: Vec<TableSchema>,
}

/// File attachment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment ID
    pub id: String,
    /// File URL
    pub url: String,
    /// Original filename
    pub filename: String,
    /// File size in bytes
    pub size: u64,
    /// MIME type
    #[serde(rename = "type")]
    pub content_type: String,
    /// Image dimensions (for image files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Thumbnail URLs (for image files)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnails: Option<HashMap<String, ThumbnailInfo>>,
}

/// Thumbnail information for image attachments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailInfo {
    /// Thumbnail URL
    pub url: String,
    /// Thumbnail width
    pub width: u32,
    /// Thumbnail height  
    pub height: u32,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error information
    pub error: ApiError,
}

/// API error details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error type/code
    #[serde(rename = "type")]
    pub error_type: String,
    /// Human-readable error message
    pub message: String,
}

/// User information from whoami endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    /// User ID
    pub id: String,
    /// User display name (optional - not always provided by Airtable API)
    pub name: Option<String>,
    /// User email
    pub email: String,
}

/// Base information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseInfo {
    /// Base ID
    pub id: String,
    /// Base name
    pub name: String,
    /// Permission level for current user
    #[serde(rename = "permissionLevel")]
    pub permission_level: String,
}

/// Response from bases list endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasesResponse {
    /// Array of accessible bases
    pub bases: Vec<BaseInfo>,
    /// Pagination offset (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
}
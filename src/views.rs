//! View system for specialized data extraction and formatting
//!
//! This module provides a pluggable view system that allows for specialized
//! data extraction, filtering, and formatting of Airtable records.

use crate::models::Record;
use serde_json::Value;
use std::error::Error as StdError;
use std::fmt;

/// Trait for implementing specialized view processors
///
/// Views handle data extraction, filtering, and formatting for specific use cases.
/// Each view can define its own filtering logic and output format.
pub trait ViewProcessor: Send + Sync {
    /// Returns the name of this view (used for CLI matching)
    fn name(&self) -> &'static str;

    /// Returns a description of what this view does
    fn description(&self) -> &'static str;

    /// Returns the list of required field names for this view
    /// These fields must exist in the table schema
    fn required_fields(&self) -> Vec<&'static str>;

    /// Returns the list of optional field names for this view
    /// These fields will be included if they exist, but won't cause errors if missing
    fn optional_fields(&self) -> Vec<&'static str> {
        Vec::new()
    }

    /// Determines if a record should be included in the view output
    /// This allows views to implement custom filtering logic
    fn should_include_record(&self, record: &Record) -> bool;

    /// Processes a collection of records and returns the formatted output
    /// This is where the view-specific formatting logic is implemented
    fn process_records(&self, records: Vec<Record>) -> Result<Value, ViewError>;
}

/// Errors that can occur during view processing
#[derive(Debug)]
pub enum ViewError {
    /// A required field was not found in the record
    MissingRequiredField {
        field_name: String,
        record_id: String,
    },
    /// JSON serialization failed
    SerializationError(serde_json::Error),
    /// General processing error
    ProcessingError(String),
}

impl fmt::Display for ViewError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ViewError::MissingRequiredField {
                field_name,
                record_id,
            } => {
                write!(
                    f,
                    "Missing required field '{}' in record '{}'",
                    field_name, record_id
                )
            }
            ViewError::SerializationError(e) => {
                write!(f, "JSON serialization error: {}", e)
            }
            ViewError::ProcessingError(msg) => {
                write!(f, "View processing error: {}", msg)
            }
        }
    }
}

impl StdError for ViewError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            ViewError::SerializationError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ViewError {
    fn from(err: serde_json::Error) -> Self {
        ViewError::SerializationError(err)
    }
}

/// Registry for managing available views
pub struct ViewRegistry {
    views: Vec<Box<dyn ViewProcessor>>,
}

impl ViewRegistry {
    /// Create a new view registry with all available views
    pub fn new() -> Self {
        let mut registry = ViewRegistry { views: Vec::new() };

        // Register all available views
        registry.register(Box::new(crate::views::clio::ClioView::new()));

        registry
    }

    /// Register a new view processor
    pub fn register(&mut self, view: Box<dyn ViewProcessor>) {
        self.views.push(view);
    }

    /// Get a view by name
    pub fn get_view(&self, name: &str) -> Option<&dyn ViewProcessor> {
        self.views
            .iter()
            .find(|view| view.name() == name)
            .map(|view| view.as_ref())
    }

    /// List all available views
    pub fn list_views(&self) -> Vec<(&str, &str)> {
        self.views
            .iter()
            .map(|view| (view.name(), view.description()))
            .collect()
    }
}

impl Default for ViewRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Process records using the specified view
pub fn process_with_view(view_name: &str, records: Vec<Record>) -> Result<Value, ViewError> {
    let registry = ViewRegistry::new();

    let view = registry.get_view(view_name).ok_or_else(|| {
        let available_views: Vec<_> = registry
            .list_views()
            .iter()
            .map(|(name, _)| *name)
            .collect();
        ViewError::ProcessingError(format!(
            "Unknown view '{}'. Available views: {}",
            view_name,
            available_views.join(", ")
        ))
    })?;

    // Filter records based on view criteria
    let filtered_records: Vec<Record> = records
        .into_iter()
        .filter(|record| view.should_include_record(record))
        .collect();

    // Process the filtered records
    view.process_records(filtered_records)
}

/// Helper function to extract field value from a record with error handling
pub fn extract_field_value(
    record: &Record,
    field_name: &str,
    required: bool,
) -> Result<Option<Value>, ViewError> {
    match record.fields.get(field_name) {
        Some(value) => {
            // Handle null values
            if value.is_null() {
                if required {
                    return Err(ViewError::MissingRequiredField {
                        field_name: field_name.to_string(),
                        record_id: record.id.clone(),
                    });
                }
                Ok(None)
            } else {
                Ok(Some(value.clone()))
            }
        }
        None => {
            if required {
                Err(ViewError::MissingRequiredField {
                    field_name: field_name.to_string(),
                    record_id: record.id.clone(),
                })
            } else {
                Ok(None)
            }
        }
    }
}

// Re-export views
pub mod clio;

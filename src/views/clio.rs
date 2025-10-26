//! Clio view for extracting Matter records with Clio-specific formatting
//!
//! This view extracts Matter records that have Clio Matter IDs and formats them
//! with the specific fields needed for Clio integration.

use crate::models::Record;
use crate::views::{extract_field_value, ViewError, ViewProcessor};
use serde_json::{json, Value};

/// View processor for Clio Matter integration
///
/// This view:
/// - Filters records to only include those with non-null/non-empty "Clio Matter ID"
/// - Extracts 6 specific fields related to Clio integration
/// - Formats output as structured JSON with normalized field names
pub struct ClioView;

impl ClioView {
    pub fn new() -> Self {
        ClioView
    }
}

impl ViewProcessor for ClioView {
    fn name(&self) -> &'static str {
        "clio"
    }

    fn description(&self) -> &'static str {
        "Extract Matter records with Clio integration fields (filters null Clio Matter IDs)"
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec![
            "Clio Matter ID", // Primary filter field - must be non-null/non-empty
        ]
    }

    fn optional_fields(&self) -> Vec<&'static str> {
        vec![
            "Matter Title",
            "Clio Matter Url",
            "Clio Drive Folder",
            "Open in Google drive (from Clio Drive Folder)",
        ]
    }

    fn should_include_record(&self, record: &Record) -> bool {
        // Only include records that have a non-null, non-empty Clio Matter ID
        match record.fields.get("Clio Matter ID") {
            Some(value) => {
                // Check if value is not null and not an empty string
                !value.is_null() && value.as_str().map_or(false, |s| !s.trim().is_empty())
            }
            None => false,
        }
    }

    fn process_records(&self, records: Vec<Record>) -> Result<Value, ViewError> {
        let mut result_records = Vec::new();

        for record in records {
            // Extract required field (we know it exists due to filtering)
            let clio_matter_id = extract_field_value(&record, "Clio Matter ID", true)?
                .unwrap() // Safe to unwrap because we filtered for non-null values
                .as_str()
                .unwrap_or("") // Convert to string, empty if not a string
                .to_string();

            // Extract optional fields
            let matter_title = extract_field_value(&record, "Matter Title", false)?
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();

            let clio_matter_url = extract_field_value(&record, "Clio Matter Url", false)?
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();

            let clio_drive_folder = extract_field_value(&record, "Clio Drive Folder", false)?
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();

            let google_drive_link = extract_field_value(
                &record,
                "Open in Google drive (from Clio Drive Folder)",
                false,
            )?
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_default();

            // Create the formatted record
            let formatted_record = json!({
                "record_id": record.id,
                "matter_title": matter_title,
                "clio_matter_id": clio_matter_id,
                "clio_matter_url": clio_matter_url,
                "clio_drive_folder": clio_drive_folder,
                "google_drive_link": google_drive_link
            });

            result_records.push(formatted_record);
        }

        // Return as JSON array
        Ok(Value::Array(result_records))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn create_test_record(id: &str, fields: HashMap<String, Value>) -> Record {
        use chrono::{DateTime, Utc};
        Record {
            id: id.to_string(),
            fields,
            created_time: DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .with_timezone(&Utc),
        }
    }

    #[test]
    fn test_should_include_record_with_valid_clio_id() {
        let view = ClioView::new();
        let mut fields = HashMap::new();
        fields.insert("Clio Matter ID".to_string(), json!("12345"));

        let record = create_test_record("rec123", fields);
        assert!(view.should_include_record(&record));
    }

    #[test]
    fn test_should_exclude_record_with_null_clio_id() {
        let view = ClioView::new();
        let mut fields = HashMap::new();
        fields.insert("Clio Matter ID".to_string(), Value::Null);

        let record = create_test_record("rec123", fields);
        assert!(!view.should_include_record(&record));
    }

    #[test]
    fn test_should_exclude_record_with_empty_clio_id() {
        let view = ClioView::new();
        let mut fields = HashMap::new();
        fields.insert("Clio Matter ID".to_string(), json!(""));

        let record = create_test_record("rec123", fields);
        assert!(!view.should_include_record(&record));
    }

    #[test]
    fn test_should_exclude_record_with_whitespace_only_clio_id() {
        let view = ClioView::new();
        let mut fields = HashMap::new();
        fields.insert("Clio Matter ID".to_string(), json!("   "));

        let record = create_test_record("rec123", fields);
        assert!(!view.should_include_record(&record));
    }

    #[test]
    fn test_should_exclude_record_without_clio_id_field() {
        let view = ClioView::new();
        let fields = HashMap::new(); // No Clio Matter ID field

        let record = create_test_record("rec123", fields);
        assert!(!view.should_include_record(&record));
    }

    #[test]
    fn test_process_records_with_all_fields() {
        let view = ClioView::new();
        let mut fields = HashMap::new();
        fields.insert("Clio Matter ID".to_string(), json!("12345"));
        fields.insert("Matter Title".to_string(), json!("Test Matter"));
        fields.insert(
            "Clio Matter Url".to_string(),
            json!("https://example.com/matter/12345"),
        );
        fields.insert("Clio Drive Folder".to_string(), json!("Test Folder"));
        fields.insert(
            "Open in Google drive (from Clio Drive Folder)".to_string(),
            json!("https://drive.google.com/folder123"),
        );

        let record = create_test_record("rec123", fields);
        let result = view.process_records(vec![record]).unwrap();

        let expected = json!([{
            "record_id": "rec123",
            "matter_title": "Test Matter",
            "clio_matter_id": "12345",
            "clio_matter_url": "https://example.com/matter/12345",
            "clio_drive_folder": "Test Folder",
            "google_drive_link": "https://drive.google.com/folder123"
        }]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_process_records_with_missing_optional_fields() {
        let view = ClioView::new();
        let mut fields = HashMap::new();
        fields.insert("Clio Matter ID".to_string(), json!("12345"));
        // Missing all optional fields

        let record = create_test_record("rec123", fields);
        let result = view.process_records(vec![record]).unwrap();

        let expected = json!([{
            "record_id": "rec123",
            "matter_title": "",
            "clio_matter_id": "12345",
            "clio_matter_url": "",
            "clio_drive_folder": "",
            "google_drive_link": ""
        }]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_process_records_filters_multiple_records() {
        let view = ClioView::new();

        // Record with valid Clio ID
        let mut fields1 = HashMap::new();
        fields1.insert("Clio Matter ID".to_string(), json!("12345"));
        fields1.insert("Matter Title".to_string(), json!("Valid Matter"));
        let record1 = create_test_record("rec123", fields1);

        // Record with null Clio ID (should be excluded)
        let mut fields2 = HashMap::new();
        fields2.insert("Clio Matter ID".to_string(), Value::Null);
        fields2.insert("Matter Title".to_string(), json!("Invalid Matter"));
        let record2 = create_test_record("rec456", fields2);

        // Process both records
        let records = vec![record1, record2];
        let filtered_records: Vec<Record> = records
            .into_iter()
            .filter(|r| view.should_include_record(r))
            .collect();
        let result = view.process_records(filtered_records).unwrap();

        // Should only contain the first record
        assert_eq!(result.as_array().unwrap().len(), 1);
        assert_eq!(result[0]["record_id"], "rec123");
        assert_eq!(result[0]["matter_title"], "Valid Matter");
    }
}

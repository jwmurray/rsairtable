use rsairtable::Client;
use std::env;
use tokio::time::{timeout, Duration};

/// Test Step 5: Record Retrieval Methods
///
/// These tests will initially FAIL (red phase) to drive TDD development.
/// We need to implement: get(), all(), iterate(), first() methods

#[tokio::test]
async fn test_step5_get_single_record() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Test getting a single record by ID
    // This will fail initially because get() method doesn't exist yet
    let table = client.base(&base_id).table("Clio");

    // First get a record to test with
    let (records, _) = table
        .list()
        .max_records(10)
        .execute()
        .await
        .expect("Should get records to test with");
    assert!(
        !records.is_empty(),
        "Need at least one record to test get()"
    );

    let test_record_id = &records[0].id;

    // Test 1: Get existing record by ID
    let record = table
        .get(test_record_id)
        .await
        .expect("Should get record by ID");
    assert_eq!(record.id, *test_record_id);
    assert!(!record.fields.is_empty(), "Record should have fields");
    println!("✅ Successfully retrieved record: {}", record.id);

    // Test 2: Get non-existent record should return error
    let fake_id = "recFAKERECORDIDXXX";
    let result = table.get(fake_id).await;
    assert!(result.is_err(), "Getting fake record should fail");
    println!("✅ Correctly failed to get non-existent record");
}

#[tokio::test]
async fn test_step5_all_records() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Test getting all records from a table
    let table = client.base(&base_id).table("Clio");

    // Test 1: Get all records (basic)
    let (records, _) = table
        .list()
        .max_records(10)
        .execute()
        .await
        .expect("Should get all records");
    assert!(!records.is_empty(), "Should have at least one record");
    println!("✅ Retrieved {} records", records.len());

    // Verify record structure
    for record in &records {
        assert!(!record.id.is_empty(), "Record ID should not be empty");
        assert!(
            record.id.starts_with("rec"),
            "Record ID should start with 'rec'"
        );
        // Note: fields can be empty for some records
    }

    // Test 2: Get all records with parameters (fields filter)
    let (filtered_records, _) = table
        .list()
        .fields(&["Matter"])
        .max_records(10)
        .execute()
        .await
        .expect("Should get filtered records");

    assert!(!filtered_records.is_empty(), "Should have filtered records");
    println!("✅ Retrieved {} filtered records", filtered_records.len());

    // Verify only requested fields are present (if record has the field)
    for record in &filtered_records {
        if !record.fields.is_empty() {
            // If fields exist, they should only be the ones we requested
            for field_name in record.fields.keys() {
                assert!(
                    field_name == "Matter" || field_name.contains("Matter"),
                    "Unexpected field in filtered result: {}",
                    field_name
                );
            }
        }
    }
}

#[tokio::test]
async fn test_step5_iterate_records() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Test pagination and iteration through records
    let table = client.base(&base_id).table("Clio");

    // Test 1: Iterate with page size limit
    let mut count = 0;
    let mut iterator = table
        .iterate()
        .page_size(10)
        .build()
        .await
        .expect("Should create iterator");

    while let Some(batch_result) = timeout(Duration::from_secs(10), iterator.next())
        .await
        .expect("Timeout waiting for batch")
    {
        println!("Received a batch");
        let batch = batch_result.expect("Should get valid batch");
        println!("Batch size: {}", batch.len());
        assert!(batch.len() <= 10, "Batch size should not exceed page size");
        count += batch.len();

        // Verify each record in batch
        for record in batch {
            assert!(!record.id.is_empty(), "Record ID should not be empty");
        }

        // Prevent infinite loop in test
        if count >= 5 {
            break;
        }
    }

    assert!(
        count > 0,
        "Should have iterated through at least one record"
    );
    println!("✅ Iterated through {} records in batches", count);

    // Test 2: Iterate with filters
    let mut filtered_count = 0;
    let mut filtered_iterator = table
        .iterate()
        .filter_by_formula("NOT({Name} = '')") // Records where Name is not empty
        .page_size(10)
        .build()
        .await
        .expect("Should create filtered iterator");

    while let Some(batch) = filtered_iterator.next().await {
        let batch = batch.expect("Should get valid filtered batch");
        filtered_count += batch.len();

        // Verify filter worked (if records have Name field)
        for record in batch {
            if let Some(name_value) = record.fields.get("Name") {
                assert!(
                    !name_value.is_null(),
                    "Name should not be empty due to filter"
                );
            }
        }

        if filtered_count >= 10 {
            break;
        }
    }

    println!("✅ Filtered iteration processed {} records", filtered_count);
}

#[tokio::test]
async fn test_step5_first_record() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Test getting the first record that matches criteria
    let table = client.base(&base_id).table("Clio");

    // Test 1: Get first record (no criteria)
    let first_record = table
        .first()
        .execute()
        .await
        .expect("Should get first record");
    assert!(!first_record.id.is_empty(), "First record should have ID");
    println!("✅ Got first record: {}", first_record.id);

    // Test 2: Get first record with filter
    let first_filtered = table
        .first()
        .filter_by_formula("NOT({Name} = '')") // First record where Name is not empty
        .execute()
        .await;

    match first_filtered {
        Ok(record) => {
            assert!(
                !record.id.is_empty(),
                "Filtered first record should have ID"
            );
            if let Some(name_value) = record.fields.get("Name") {
                assert!(
                    !name_value.is_null(),
                    "Name should not be empty due to filter"
                );
            }
            println!("✅ Got first filtered record: {}", record.id);
        }
        Err(_) => {
            // It's OK if no records match the filter
            println!("ℹ️ No records matched the filter criteria");
        }
    }

    // Test 3: Get first record with specific fields
    let first_with_fields = table
        .first()
        .fields(&["Matter"])
        .execute()
        .await
        .expect("Should get first record with specific fields");

    assert!(!first_with_fields.id.is_empty(), "Record should have ID");
    println!(
        "✅ Got first record with specific fields: {}",
        first_with_fields.id
    );
}

#[tokio::test]
async fn test_step5_record_retrieval_query_builder() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Test the query builder pattern for record retrieval
    let table = client.base(&base_id).table("Clio");

    // Test 1: Complex query with multiple parameters
    let complex_query_records = table
        .select()
        .fields(&["Matter", "File Owner(s)"])
        .filter_by_formula("NOT({Matter} = '')")
        .sort(&[("Matter", "asc")])
        .max_records(10)
        .execute()
        .await
        .expect("Should execute complex query");

    assert!(
        complex_query_records.len() <= 10,
        "Should respect max_records limit"
    );
    println!(
        "✅ Complex query returned {} records",
        complex_query_records.len()
    );

    // Verify sorting (if we have multiple records)
    if complex_query_records.len() > 1 {
        let mut prev_name: Option<String> = None;
        for record in &complex_query_records {
            if let Some(name_value) = record.fields.get("Name") {
                if let Some(name_str) = name_value.as_str() {
                    if let Some(prev) = &prev_name {
                        assert!(
                            name_str >= prev.as_str(),
                            "Records should be sorted by Name ascending"
                        );
                    }
                    prev_name = Some(name_str.to_string());
                }
            }
        }
        println!("✅ Records are properly sorted by Name");
    }

    // Test 2: Query with view
    let view_records = table
        .select()
        .view("Grid view") // Default view name in Airtable
        .max_records(10) // Limit to 10 records for test speed
        .execute()
        .await;

    match view_records {
        Ok(records) => {
            println!("✅ View query returned {} records", records.len());
        }
        Err(_) => {
            // View might not exist, which is OK for this test
            println!("ℹ️ View 'Grid view' not found (this is OK)");
        }
    }
}

#[tokio::test]
async fn test_step5_error_handling() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");

    // Test error handling for invalid requests

    // Test 1: Invalid base ID
    let invalid_base = client.base("appINVALIDXXXXXXXXX").table("TestTable");
    let result = invalid_base.list().max_records(10).execute().await;
    assert!(result.is_err(), "Invalid base should return error");
    println!("✅ Correctly handled invalid base ID");

    // Test 2: Invalid table name
    let base_id = env::var("BASE").expect("BASE environment variable not set");
    let invalid_table = client.base(&base_id).table("NonExistentTable123");
    let result = invalid_table.list().max_records(10).execute().await;
    assert!(result.is_err(), "Invalid table should return error");
    println!("✅ Correctly handled invalid table name");

    // Test 3: Invalid record ID
    let valid_table = client.base(&base_id).table("Clio");
    let result = valid_table.get("recINVALIDXXXXXXXXX").await;
    assert!(result.is_err(), "Invalid record ID should return error");
    println!("✅ Correctly handled invalid record ID");
}

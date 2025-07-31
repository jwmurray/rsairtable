use rsairtable::Client;
use serde_json::json;
use std::env;

/// Test Step 6: Record Creation Methods
///
/// These tests will initially FAIL (red phase) to drive TDD development.
/// We need to implement: create(), batch_create() methods

#[tokio::test]
async fn test_step6_create_single_record() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Test creating a single record
    let table = client.base(&base_id).table("TestCaseLaw");

    // Test 1: Create a basic record with fields
    let fields = json!({
        "Name": "Test Record - Single Create",
        "Status": "Todo",
        "Notes": "This record was created by the API test suite"
    });

    let record = table.create(fields).await.expect("Should create record");

    assert!(!record.id.is_empty(), "Record ID should not be empty");
    assert!(
        record.id.starts_with("rec"),
        "Record ID should start with 'rec'"
    );
    assert_eq!(
        record.fields.get("Name").unwrap().as_str().unwrap(),
        "Test Record - Single Create"
    );
    assert_eq!(
        record.fields.get("Status").unwrap().as_str().unwrap(),
        "Todo"
    );
    println!("✅ Successfully created record: {}", record.id);

    // Clean up: delete the created record
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
    println!("✅ Cleaned up test record: {}", record.id);
}

#[tokio::test]
async fn test_step6_create_record_with_typecast() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    let table = client.base(&base_id).table("TestCaseLaw");

    // Test 2: Create record with typecast option
    let fields = json!({
        "Name": "Test Record - Typecast",
        "Status": "In progress",
        "Notes": "Testing typecast functionality"
    });

    let record = table
        .create_with_typecast(fields, true)
        .await
        .expect("Should create record with typecast");

    assert!(!record.id.is_empty(), "Record ID should not be empty");
    assert_eq!(
        record.fields.get("Name").unwrap().as_str().unwrap(),
        "Test Record - Typecast"
    );
    println!(
        "✅ Successfully created record with typecast: {}",
        record.id
    );

    // Clean up
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
}

#[tokio::test]
async fn test_step6_batch_create_records() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    let table = client.base(&base_id).table("TestCaseLaw");

    // Test 3: Batch create multiple records
    let records_data = vec![
        json!({
            "Name": "Test Record - Batch 1",
            "Status": "Todo",
            "Notes": "First batch test record"
        }),
        json!({
            "Name": "Test Record - Batch 2",
            "Status": "In progress",
            "Notes": "Second batch test record"
        }),
        json!({
            "Name": "Test Record - Batch 3",
            "Status": "Done",
            "Notes": "Third batch test record"
        }),
    ];

    let records = table
        .batch_create(records_data)
        .await
        .expect("Should batch create records");

    assert_eq!(records.len(), 3, "Should create 3 records");

    for (i, record) in records.iter().enumerate() {
        assert!(!record.id.is_empty(), "Record ID should not be empty");
        assert!(
            record.id.starts_with("rec"),
            "Record ID should start with 'rec'"
        );
        assert_eq!(
            record.fields.get("Name").unwrap().as_str().unwrap(),
            format!("Test Record - Batch {}", i + 1)
        );
    }

    println!("✅ Successfully batch created {} records", records.len());

    // Clean up: delete all created records
    let record_ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();
    table
        .batch_delete(&record_ids)
        .await
        .expect("Should batch delete test records");
    println!("✅ Cleaned up {} test records", record_ids.len());
}

#[tokio::test]
async fn test_step6_batch_create_with_options() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    let table = client.base(&base_id).table("TestCaseLaw");

    // Test 4: Batch create with typecast and return fields options
    let records_data = vec![
        json!({
            "Name": "Test Record - Options 1",
            "Status": "Todo",
            "Notes": "First options test record"
        }),
        json!({
            "Name": "Test Record - Options 2",
            "Status": "In progress",
            "Notes": "Second options test record"
        }),
    ];

    let records = table
        .batch_create_with_options(records_data, true, &["Name", "Status", "Notes"])
        .await
        .expect("Should batch create records with options");

    assert_eq!(records.len(), 2, "Should create 2 records");

    for record in &records {
        assert!(!record.id.is_empty(), "Record ID should not be empty");
        // Verify only requested fields are returned (plus auto fields)
        assert!(record.fields.contains_key("Name"), "Should have Name field");
        assert!(
            record.fields.contains_key("Status"),
            "Should have Status field"
        );
        assert!(
            record.fields.contains_key("Notes"),
            "Should have Notes field"
        );
    }

    println!("✅ Successfully batch created records with options");

    // Clean up
    let record_ids: Vec<String> = records.iter().map(|r| r.id.clone()).collect();
    table
        .batch_delete(&record_ids)
        .await
        .expect("Should batch delete test records");
}

#[tokio::test]
async fn test_step6_creation_error_handling() {
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
    let fields = json!({"Name": "Test"});
    let result = invalid_base.create(fields).await;
    assert!(result.is_err(), "Invalid base should return error");
    println!("✅ Correctly handled invalid base ID");

    // Test 2: Invalid table name
    let base_id = env::var("BASE").expect("BASE environment variable not set");
    let invalid_table = client.base(&base_id).table("NonExistentTable123");
    let fields = json!({"Name": "Test"});
    let result = invalid_table.create(fields).await;
    assert!(result.is_err(), "Invalid table should return error");
    println!("✅ Correctly handled invalid table name");

    // Test 3: Invalid field data (wrong field name)
    let table = client.base(&base_id).table("TestCaseLaw");
    let invalid_fields = json!({"NonExistentField": "Test Value"});
    let result = table.create(invalid_fields).await;
    assert!(result.is_err(), "Invalid field should return error");
    println!("✅ Correctly handled invalid field name");

    // Test 4: Empty batch create
    let empty_batch: Vec<serde_json::Value> = vec![];
    let result = table.batch_create(empty_batch).await;
    assert!(result.is_err(), "Empty batch should return error");
    println!("✅ Correctly handled empty batch creation");

    // Test 5: Batch create with too many records (Airtable limit is 10 per batch)
    let large_batch: Vec<serde_json::Value> = (0..15)
        .map(|i| {
            json!({
                "Name": format!("Test Record {}", i),
                "Status": "Todo",
                "Notes": format!("Large batch test record {}", i)
            })
        })
        .collect();

    let result = table.batch_create(large_batch).await;
    assert!(result.is_err(), "Large batch should return error");
    println!("✅ Correctly handled oversized batch creation");
}

#[tokio::test]
async fn test_step6_create_query_builder() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    let table = client.base(&base_id).table("TestCaseLaw");

    // Test 6: Create using query builder pattern
    let fields = json!({
        "Name": "Test Record - Query Builder",
        "Status": "Done",
        "Notes": "Testing query builder pattern"
    });

    let record = table
        .create_record()
        .fields(fields)
        .typecast(true)
        .return_fields(&["Name", "Status", "Notes"])
        .execute()
        .await
        .expect("Should create record using query builder");

    assert!(!record.id.is_empty(), "Record ID should not be empty");
    assert_eq!(
        record.fields.get("Name").unwrap().as_str().unwrap(),
        "Test Record - Query Builder"
    );
    println!(
        "✅ Successfully created record using query builder: {}",
        record.id
    );

    // Clean up
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
}

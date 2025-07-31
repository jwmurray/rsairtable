use rsairtable::Client;
use serde_json::json;
use std::env;

/// Test Step 7: Record Update and Delete Methods
///
/// These tests will initially FAIL (red phase) to drive TDD development.
/// We need to implement: update(), batch_update(), batch_upsert(), delete(), batch_delete()

#[tokio::test]
async fn test_step7_update_single_record() {
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

    // First, create a record to update
    let initial_fields = json!({
        "Name": "Test Record - Update",
        "Status": "Todo",
        "Notes": "Initial notes for update test"
    });

    let record = table
        .create(initial_fields)
        .await
        .expect("Should create record");
    println!("✅ Created record for update test: {}", record.id);

    // Test 1: Update the record
    let update_fields = json!({
        "Status": "In progress",
        "Notes": "Updated notes after modification"
    });

    let updated_record = table
        .update(&record.id, update_fields)
        .await
        .expect("Should update record");

    assert_eq!(
        updated_record.id, record.id,
        "Record ID should remain the same"
    );
    assert_eq!(
        updated_record.fields.get("Name").unwrap().as_str().unwrap(),
        "Test Record - Update",
        "Name should remain unchanged"
    );
    assert_eq!(
        updated_record
            .fields
            .get("Status")
            .unwrap()
            .as_str()
            .unwrap(),
        "In progress",
        "Status should be updated"
    );
    assert_eq!(
        updated_record
            .fields
            .get("Notes")
            .unwrap()
            .as_str()
            .unwrap(),
        "Updated notes after modification",
        "Notes should be updated"
    );

    println!("✅ Successfully updated record: {}", updated_record.id);

    // Clean up: delete the test record
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
    println!("✅ Cleaned up test record: {}", record.id);
}

#[tokio::test]
async fn test_step7_update_with_typecast() {
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

    // Create a record to update
    let initial_fields = json!({
        "Name": "Test Record - Typecast Update",
        "Status": "Todo",
        "Notes": "Initial notes"
    });

    let record = table
        .create(initial_fields)
        .await
        .expect("Should create record");

    // Test 2: Update with typecast option
    let update_fields = json!({
        "Status": "Done",
        "Notes": "Updated with typecast functionality"
    });

    let updated_record = table
        .update_with_typecast(&record.id, update_fields, true)
        .await
        .expect("Should update record with typecast");

    assert_eq!(
        updated_record
            .fields
            .get("Status")
            .unwrap()
            .as_str()
            .unwrap(),
        "Done"
    );
    assert_eq!(
        updated_record
            .fields
            .get("Notes")
            .unwrap()
            .as_str()
            .unwrap(),
        "Updated with typecast functionality"
    );

    println!(
        "✅ Successfully updated record with typecast: {}",
        updated_record.id
    );

    // Clean up
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
}

#[tokio::test]
async fn test_step7_batch_update_records() {
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

    // Create multiple records to update
    let initial_records = vec![
        json!({
            "Name": "Batch Update Test 1",
            "Status": "Todo",
            "Notes": "Initial notes 1"
        }),
        json!({
            "Name": "Batch Update Test 2",
            "Status": "Todo",
            "Notes": "Initial notes 2"
        }),
        json!({
            "Name": "Batch Update Test 3",
            "Status": "Todo",
            "Notes": "Initial notes 3"
        }),
    ];

    let created_records = table
        .batch_create(initial_records)
        .await
        .expect("Should create records for batch update test");

    println!(
        "✅ Created {} records for batch update test",
        created_records.len()
    );

    // Test 3: Batch update the records
    let update_data = vec![
        json!({
            "id": created_records[0].id,
            "fields": {
                "Status": "In progress",
                "Notes": "Updated notes 1"
            }
        }),
        json!({
            "id": created_records[1].id,
            "fields": {
                "Status": "Done",
                "Notes": "Updated notes 2"
            }
        }),
        json!({
            "id": created_records[2].id,
            "fields": {
                "Status": "In progress",
                "Notes": "Updated notes 3"
            }
        }),
    ];

    let updated_records = table
        .batch_update(update_data)
        .await
        .expect("Should batch update records");

    assert_eq!(updated_records.len(), 3, "Should update 3 records");

    // Verify updates
    for (i, record) in updated_records.iter().enumerate() {
        assert_eq!(record.id, created_records[i].id, "Record IDs should match");
        let expected_status = match i {
            0 | 2 => "In progress",
            1 => "Done",
            _ => unreachable!(),
        };
        assert_eq!(
            record.fields.get("Status").unwrap().as_str().unwrap(),
            expected_status
        );
        assert_eq!(
            record.fields.get("Notes").unwrap().as_str().unwrap(),
            format!("Updated notes {}", i + 1)
        );
    }

    println!(
        "✅ Successfully batch updated {} records",
        updated_records.len()
    );

    // Clean up: delete all test records
    let record_ids: Vec<String> = created_records.iter().map(|r| r.id.clone()).collect();
    table
        .batch_delete(&record_ids)
        .await
        .expect("Should batch delete test records");
    println!("✅ Cleaned up {} test records", record_ids.len());
}

#[tokio::test]
async fn test_step7_batch_upsert_records() {
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

    // Create one record that will be updated, and prepare one that will be created
    let existing_record = table
        .create(json!({
            "Name": "Existing Record for Upsert",
            "Status": "Todo",
            "Notes": "Will be updated via upsert"
        }))
        .await
        .expect("Should create existing record");

    // Test 4: Batch upsert (mix of update existing and create new)
    let upsert_data = vec![
        json!({
            "id": existing_record.id,
            "fields": {
                "Status": "Done",
                "Notes": "Updated via upsert operation"
            }
        }),
        json!({
            "fields": {
                "Name": "New Record via Upsert",
                "Status": "In progress",
                "Notes": "Created via upsert operation"
            }
        }),
    ];

    let upserted_records = table
        .batch_upsert(upsert_data, &["Name"])
        .await
        .expect("Should batch upsert records");

    assert_eq!(upserted_records.len(), 2, "Should upsert 2 records");

    // Verify the existing record was updated
    let updated_existing = &upserted_records[0];
    assert_eq!(
        updated_existing.id, existing_record.id,
        "Should update existing record"
    );
    assert_eq!(
        updated_existing
            .fields
            .get("Status")
            .unwrap()
            .as_str()
            .unwrap(),
        "Done"
    );
    assert_eq!(
        updated_existing
            .fields
            .get("Notes")
            .unwrap()
            .as_str()
            .unwrap(),
        "Updated via upsert operation"
    );

    // Verify the new record was created
    let new_record = &upserted_records[1];
    assert_ne!(
        new_record.id, existing_record.id,
        "Should create new record"
    );
    assert_eq!(
        new_record.fields.get("Name").unwrap().as_str().unwrap(),
        "New Record via Upsert"
    );

    println!(
        "✅ Successfully upserted {} records",
        upserted_records.len()
    );

    // Clean up
    let record_ids: Vec<String> = upserted_records.iter().map(|r| r.id.clone()).collect();
    table
        .batch_delete(&record_ids)
        .await
        .expect("Should batch delete test records");
}

#[tokio::test]
async fn test_step7_delete_single_record() {
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

    // Create a record to delete
    let record = table
        .create(json!({
            "Name": "Test Record - Delete",
            "Status": "Todo",
            "Notes": "This record will be deleted"
        }))
        .await
        .expect("Should create record for delete test");

    println!("✅ Created record for delete test: {}", record.id);

    // Test 5: Delete the record
    table
        .delete(&record.id)
        .await
        .expect("Should delete record");

    println!("✅ Successfully deleted record: {}", record.id);

    // Verify the record is gone by trying to retrieve it
    let get_result = table.get(&record.id).await;
    assert!(get_result.is_err(), "Record should no longer exist");
    println!("✅ Verified record was deleted");
}

#[tokio::test]
async fn test_step7_batch_delete_records() {
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

    // Create multiple records to delete
    let records_to_create = vec![
        json!({
            "Name": "Batch Delete Test 1",
            "Status": "Todo",
            "Notes": "Will be deleted in batch 1"
        }),
        json!({
            "Name": "Batch Delete Test 2",
            "Status": "In progress",
            "Notes": "Will be deleted in batch 2"
        }),
        json!({
            "Name": "Batch Delete Test 3",
            "Status": "Done",
            "Notes": "Will be deleted in batch 3"
        }),
    ];

    let created_records = table
        .batch_create(records_to_create)
        .await
        .expect("Should create records for batch delete test");

    println!(
        "✅ Created {} records for batch delete test",
        created_records.len()
    );

    // Test 6: Batch delete the records
    let record_ids: Vec<String> = created_records.iter().map(|r| r.id.clone()).collect();

    table
        .batch_delete(&record_ids)
        .await
        .expect("Should batch delete records");

    println!("✅ Successfully batch deleted {} records", record_ids.len());

    // Verify all records are gone
    for record_id in &record_ids {
        let get_result = table.get(record_id).await;
        assert!(
            get_result.is_err(),
            "Record {} should no longer exist",
            record_id
        );
    }
    println!("✅ Verified all records were deleted");
}

#[tokio::test]
async fn test_step7_update_delete_error_handling() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");

    // Test error handling for update/delete operations

    // Test 1: Update non-existent record
    let base_id = env::var("BASE").expect("BASE environment variable not set");
    let table = client.base(&base_id).table("TestCaseLaw");

    let result = table
        .update("recNONEXISTENT123", json!({"Status": "Done"}))
        .await;
    assert!(
        result.is_err(),
        "Should return error for non-existent record"
    );
    println!("✅ Correctly handled update of non-existent record");

    // Test 2: Delete non-existent record
    let result = table.delete("recNONEXISTENT456").await;
    assert!(
        result.is_err(),
        "Should return error for non-existent record"
    );
    println!("✅ Correctly handled delete of non-existent record");

    // Test 3: Invalid base ID
    let invalid_base = client.base("appINVALIDXXXXXXXXX").table("TestCaseLaw");
    let result = invalid_base
        .update("recSOMERECORD", json!({"Status": "Done"}))
        .await;
    assert!(result.is_err(), "Invalid base should return error");
    println!("✅ Correctly handled invalid base ID for update");

    // Test 4: Invalid table name
    let invalid_table = client.base(&base_id).table("NonExistentTable456");
    let result = invalid_table.delete("recSOMERECORD").await;
    assert!(result.is_err(), "Invalid table should return error");
    println!("✅ Correctly handled invalid table name for delete");

    // Test 5: Empty batch update
    let empty_batch: Vec<serde_json::Value> = vec![];
    let result = table.batch_update(empty_batch).await;
    assert!(result.is_err(), "Empty batch update should return error");
    println!("✅ Correctly handled empty batch update");

    // Test 6: Empty batch delete
    let empty_ids: Vec<String> = vec![];
    let result = table.batch_delete(&empty_ids).await;
    assert!(result.is_err(), "Empty batch delete should return error");
    println!("✅ Correctly handled empty batch delete");

    // Test 7: Batch update with too many records (Airtable limit is 10 per batch)
    let large_batch: Vec<serde_json::Value> = (0..15)
        .map(|i| {
            json!({
                "id": format!("recFAKE{:03}", i),
                "fields": {
                    "Status": "Done",
                    "Notes": format!("Large batch test {}", i)
                }
            })
        })
        .collect();

    let result = table.batch_update(large_batch).await;
    assert!(result.is_err(), "Large batch update should return error");
    println!("✅ Correctly handled oversized batch update");

    // Test 8: Batch delete with too many records
    let large_ids: Vec<String> = (0..15).map(|i| format!("recFAKE{:03}", i)).collect();

    let result = table.batch_delete(&large_ids).await;
    assert!(result.is_err(), "Large batch delete should return error");
    println!("✅ Correctly handled oversized batch delete");
}

#[tokio::test]
async fn test_step7_update_query_builder() {
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

    // Create a record to update using query builder
    let record = table
        .create(json!({
            "Name": "Test Record - Query Builder Update",
            "Status": "Todo",
            "Notes": "Initial notes for query builder test"
        }))
        .await
        .expect("Should create record");

    // Test 8: Update using query builder pattern
    let update_fields = json!({
        "Status": "Done",
        "Notes": "Updated using query builder pattern"
    });

    let updated_record = table
        .update_record(&record.id)
        .fields(update_fields)
        .typecast(true)
        .return_fields(&["Name", "Status", "Notes"])
        .execute()
        .await
        .expect("Should update record using query builder");

    assert_eq!(updated_record.id, record.id);
    assert_eq!(
        updated_record
            .fields
            .get("Status")
            .unwrap()
            .as_str()
            .unwrap(),
        "Done"
    );
    assert_eq!(
        updated_record
            .fields
            .get("Notes")
            .unwrap()
            .as_str()
            .unwrap(),
        "Updated using query builder pattern"
    );

    println!(
        "✅ Successfully updated record using query builder: {}",
        updated_record.id
    );

    // Clean up
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
}

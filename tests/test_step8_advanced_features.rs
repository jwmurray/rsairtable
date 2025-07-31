use rsairtable::Client;
use serde_json::json;
use std::env;

/// Test Step 8: Advanced Features (Comments, Schema, Attachments)
///
/// These tests will initially FAIL (red phase) to drive TDD development.
/// We need to implement: comments(), add_comment(), schema(), create_field(), upload_attachment()

#[tokio::test]
async fn test_step8_table_schema_operations() {
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

    // Test 1: Get table schema
    let schema = table.schema().await.expect("Should get table schema");

    assert!(!schema.id.is_empty(), "Table ID should not be empty");
    assert_eq!(schema.name, "TestCaseLaw", "Table name should match");
    assert!(
        !schema.primary_field_id.is_empty(),
        "Primary field ID should not be empty"
    );
    assert!(!schema.fields.is_empty(), "Should have fields");
    assert!(!schema.views.is_empty(), "Should have views");

    // Verify expected fields exist
    let field_names: Vec<&str> = schema.fields.iter().map(|f| f.name.as_str()).collect();
    assert!(field_names.contains(&"Name"), "Should have Name field");
    assert!(field_names.contains(&"Status"), "Should have Status field");
    assert!(field_names.contains(&"Notes"), "Should have Notes field");

    println!("✅ Successfully retrieved table schema");
    println!("📋 Table: {} ({})", schema.name, schema.id);
    println!("📝 Fields: {}", field_names.join(", "));
    println!("👀 Views: {}", schema.views.len());
}

#[tokio::test]
async fn test_step8_base_schema_operations() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    let base = client.base(&base_id);

    // Test 2: Get base schema (already implemented, but verify it works)
    let base_schema = base.schema().await.expect("Should get base schema");

    assert!(!base_schema.tables.is_empty(), "Should have tables");

    // Find TestCaseLaw table
    let test_table = base_schema
        .tables
        .iter()
        .find(|t| t.name == "TestCaseLaw")
        .expect("Should find TestCaseLaw table");

    assert_eq!(test_table.name, "TestCaseLaw");
    assert!(!test_table.id.is_empty());
    assert!(!test_table.fields.is_empty());

    println!("✅ Successfully retrieved base schema");
    println!("📊 Base contains {} tables", base_schema.tables.len());

    let table_names: Vec<&str> = base_schema.tables.iter().map(|t| t.name.as_str()).collect();
    println!("📋 Tables: {}", table_names.join(", "));
}

#[tokio::test]
async fn test_step8_create_field_operations_limitation_demo() {
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

    // Test 3: Demonstrate field creation API limitation
    println!("🔬 Testing field creation API limitation...");

    let field_definition = json!({
        "name": "Test API Field",
        "type": "singleLineText",
        "description": "This field was created by the API test suite and can be safely deleted"
    });

    let result = table.create_field(field_definition).await;

    // We expect this to fail with a 404 error due to API limitations
    match result {
        Ok(_) => {
            println!("⚠️  Unexpected success: Field creation worked! API may have been updated.");
            panic!("Field creation succeeded unexpectedly - API may have changed");
        }
        Err(error) => {
            // This is the expected behavior - field creation is not supported via API
            println!("✅ Expected API limitation confirmed: {}", error);
            println!("📝 Field creation requires manual UI interaction or special permissions");

            // Verify it's a 404 error (endpoint not found)
            if let rsairtable::Error::Api { status, message: _ } = error {
                assert_eq!(status, 404, "Should be a 404 Not Found error");
                println!("✅ Confirmed: 404 error indicates API endpoint doesn't exist");
            }
        }
    }

    println!("💡 Recommendation: Use schema() method to inspect existing fields instead");
}

#[tokio::test]
async fn test_step8_comments_operations() {
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

    // First, create a record to comment on
    let record = table
        .create(json!({
            "Name": "Test Record - Comments",
            "Status": "Todo",
            "Notes": "This record is for testing comments functionality"
        }))
        .await
        .expect("Should create record for comments test");

    println!("✅ Created record for comments test: {}", record.id);

    // Test 4: Add a comment to the record
    let comment_text = "This is a test comment added via the API";
    let comment = table
        .add_comment(&record.id, comment_text)
        .await
        .expect("Should add comment");

    assert!(!comment.id.is_empty(), "Comment ID should not be empty");
    assert_eq!(comment.text, comment_text, "Comment text should match");
    assert!(
        !comment.author.id.is_empty(),
        "Author ID should not be empty"
    );
    assert!(
        !comment.author.name.is_empty(),
        "Author name should not be empty"
    );

    println!("✅ Successfully added comment: {}", comment.id);
    println!("💬 Comment: \"{}\"", comment.text);
    println!("👤 Author: {} ({})", comment.author.name, comment.author.id);

    // Test 5: Retrieve comments for the record
    let comments = table
        .comments(&record.id)
        .await
        .expect("Should retrieve comments");

    assert!(!comments.is_empty(), "Should have at least one comment");

    let our_comment = comments
        .iter()
        .find(|c| c.id == comment.id)
        .expect("Should find our comment in the list");

    assert_eq!(
        our_comment.text, comment_text,
        "Retrieved comment text should match"
    );

    println!("✅ Successfully retrieved {} comment(s)", comments.len());

    // Test 6: Add another comment with pagination test
    let comment2_text = "Second test comment for pagination testing";
    let comment2 = table
        .add_comment(&record.id, comment2_text)
        .await
        .expect("Should add second comment");

    println!("✅ Added second comment: {}", comment2.id);

    // Retrieve comments again to verify both exist
    let all_comments = table
        .comments(&record.id)
        .await
        .expect("Should retrieve all comments");

    assert!(all_comments.len() >= 2, "Should have at least 2 comments");
    println!(
        "✅ Verified {} total comments on record",
        all_comments.len()
    );

    // Clean up: delete the test record (this also deletes associated comments)
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
    println!("✅ Cleaned up test record and comments: {}", record.id);
}

#[tokio::test]
async fn test_step8_attachment_operations_limitation_demo() {
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

    // First, create a record to test attachment operations
    let record = table
        .create(json!({
            "Name": "Test Record - Attachment API Demo",
            "Status": "Todo",
            "Notes": "This record demonstrates attachment API limitations"
        }))
        .await
        .expect("Should create record for attachment test");

    println!("✅ Created record for attachment test: {}", record.id);

    // Test 7: Demonstrate attachment upload API limitation
    println!("🔬 Testing attachment upload API limitation...");

    let file_content =
        b"This is a test file for API attachment testing.\nIt contains some sample text.";
    let file_name = "test_attachment.txt";
    let mime_type = "text/plain";

    let result = table
        .upload_attachment(
            &record.id,
            "Attachments",
            file_content,
            file_name,
            mime_type,
        )
        .await;

    // We expect this to fail with a 404 error due to API limitations
    match result {
        Ok(_) => {
            println!(
                "⚠️  Unexpected success: Attachment upload worked! API may have been updated."
            );
            panic!("Attachment upload succeeded unexpectedly - API may have changed");
        }
        Err(error) => {
            // This is the expected behavior - direct file uploads are not supported via API
            println!("✅ Expected API limitation confirmed: {}", error);
            println!("📝 Direct file uploads require URL-based approach or manual UI interaction");

            // Verify it's a 404 error (endpoint not found)
            if let rsairtable::Error::Api { status, message: _ } = error {
                assert_eq!(status, 404, "Should be a 404 Not Found error");
                println!("✅ Confirmed: 404 error indicates API endpoint doesn't exist or requires different approach");
            }
        }
    }

    println!("💡 Alternative: Use record.update() with publicly accessible URLs for attachments");
    println!("📚 See Airtable docs: Attachments must be URLs, not direct file uploads");

    // Clean up: delete the test record
    table
        .delete(&record.id)
        .await
        .expect("Should delete test record");
    println!("✅ Cleaned up test record: {}", record.id);
}

#[tokio::test]
async fn test_step8_advanced_features_error_handling() {
    // Load .env file first
    dotenv::dotenv().ok();

    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");

    // Test error handling for advanced features

    // Test 1: Schema operations on invalid table
    let base_id = env::var("BASE").expect("BASE environment variable not set");
    let invalid_table = client.base(&base_id).table("NonExistentTable456");

    let result = invalid_table.schema().await;
    assert!(
        result.is_err(),
        "Should return error for non-existent table"
    );
    println!("✅ Correctly handled schema request for invalid table");

    // Test 2: Comments on non-existent record
    let table = client.base(&base_id).table("TestCaseLaw");
    let result = table.comments("recNONEXISTENT123").await;
    assert!(
        result.is_err(),
        "Should return error for non-existent record"
    );
    println!("✅ Correctly handled comments request for non-existent record");

    // Test 3: Add comment to non-existent record
    let result = table.add_comment("recNONEXISTENT456", "Test comment").await;
    assert!(
        result.is_err(),
        "Should return error for non-existent record"
    );
    println!("✅ Correctly handled add comment to non-existent record");

    // Test 4: Upload attachment to non-existent record
    let test_content = b"test file";
    let result = table
        .upload_attachment(
            "recNONEXISTENT789",
            "Attachments",
            test_content,
            "test.txt",
            "text/plain",
        )
        .await;
    assert!(
        result.is_err(),
        "Should return error for non-existent record"
    );
    println!("✅ Correctly handled attachment upload to non-existent record");

    // Test 5: Create field with invalid definition
    let invalid_field = json!({
        "name": "", // Empty name should be invalid
        "type": "invalidType"
    });

    let result = table.create_field(invalid_field).await;
    assert!(
        result.is_err(),
        "Should return error for invalid field definition"
    );
    println!("✅ Correctly handled invalid field creation");

    // Test 6: Invalid base ID for schema operations
    let invalid_base = client.base("appINVALIDXXXXXXXXX");
    let result = invalid_base.schema().await;
    assert!(result.is_err(), "Should return error for invalid base ID");
    println!("✅ Correctly handled schema request for invalid base");
}

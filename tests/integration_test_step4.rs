use rsairtable::{Client, Config};
use std::env;

#[tokio::test]
async fn test_step4_real_api_authentication() {
    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    // Test that our client can authenticate and make a real API call
    let client = Client::from_env().expect("Should create client from environment");

    // Use the implemented whoami method to test authentication
    match client.whoami().await {
        Ok(user_info) => {
            println!("✅ Authentication successful!");
            println!("User ID: {}", user_info.id);
            println!("User Name: {}", user_info.name.as_deref().unwrap_or("N/A"));
            println!("User Email: {}", user_info.email);

            // Verify we got valid user info
            assert!(!user_info.id.is_empty(), "User ID should not be empty");
            assert!(
                !user_info.email.is_empty(),
                "User email should not be empty"
            );
        }
        Err(e) => {
            panic!("Authentication failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_step4_bases_retrieval() {
    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");

    // Test retrieving bases list
    match client.bases().await {
        Ok(bases) => {
            println!("✅ Retrieved {} bases", bases.len());

            // Should have at least one base if the token is valid
            assert!(
                !bases.is_empty(),
                "Should have at least one accessible base"
            );

            // Check first base structure
            let first_base = &bases[0];
            println!("First base: {} ({})", first_base.name, first_base.id);

            assert!(!first_base.id.is_empty(), "Base ID should not be empty");
            assert!(!first_base.name.is_empty(), "Base name should not be empty");
            assert!(
                first_base.id.starts_with("app"),
                "Base ID should start with 'app'"
            );
        }
        Err(e) => {
            panic!("Failed to retrieve bases: {}", e);
        }
    }
}

#[tokio::test]
async fn test_step4_base_schema_retrieval() {
    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    let client = Client::from_env().expect("Should create client from environment");

    // Use the test base ID from environment or default
    let base_id = env::var("AIRTABLE_BASE_ID").unwrap_or_else(|_| "appS0LhkvZkx6CCOQ".to_string());

    // Test retrieving base schema
    match client.base(&base_id).schema().await {
        Ok(schema) => {
            println!("✅ Retrieved schema for base {}", base_id);
            println!("Found {} tables", schema.tables.len());

            // Should have at least one table
            assert!(
                !schema.tables.is_empty(),
                "Base should have at least one table"
            );

            // Check first table structure
            let first_table = &schema.tables[0];
            println!("First table: {} ({})", first_table.name, first_table.id);

            assert!(!first_table.id.is_empty(), "Table ID should not be empty");
            assert!(
                !first_table.name.is_empty(),
                "Table name should not be empty"
            );
            assert!(!first_table.fields.is_empty(), "Table should have fields");

            // Check first field
            let first_field = &first_table.fields[0];
            println!(
                "First field: {} ({})",
                first_field.name, first_field.field_type
            );

            assert!(!first_field.id.is_empty(), "Field ID should not be empty");
            assert!(
                !first_field.name.is_empty(),
                "Field name should not be empty"
            );
            assert!(
                !first_field.field_type.is_empty(),
                "Field type should not be empty"
            );
        }
        Err(e) => {
            panic!("Failed to retrieve base schema: {}", e);
        }
    }
}

#[tokio::test]
async fn test_step4_invalid_token_handling() {
    // Test that our client properly handles authentication errors
    let client = Client::new("invalid_token_12345".to_string());

    // Try whoami with invalid token
    match client.whoami().await {
        Ok(_) => {
            panic!("Should not succeed with invalid token");
        }
        Err(e) => {
            println!("✅ Properly handled invalid token: {}", e);
            // Should be an authentication or API error
            let error_string = e.to_string();
            assert!(
                error_string.contains("401")
                    || error_string.contains("Authentication")
                    || error_string.contains("Unauthorized"),
                "Error should indicate authentication failure: {}",
                error_string
            );
        }
    }
}

#[tokio::test]
async fn test_step4_config_integration() {
    // Skip if no API key is available
    if env::var("PERSONAL_ACCESS_TOKEN").is_err() {
        println!("Skipping API test - no PERSONAL_ACCESS_TOKEN found");
        return;
    }

    // Test creating client with custom configuration
    let config = Config::from_env()
        .expect("Should load config from environment")
        .with_verbose(true)
        .with_timeout(30);

    let client = Client::from_config(config);

    // Test that configured client works
    match client.whoami().await {
        Ok(user_info) => {
            println!("✅ Custom configured client works!");
            println!(
                "User: {} ({})",
                user_info.name.as_deref().unwrap_or("N/A"),
                user_info.email
            );
            assert!(!user_info.id.is_empty(), "Should get valid user info");
        }
        Err(e) => {
            panic!("Custom configured client failed: {}", e);
        }
    }
}

#[test]
fn test_step4_client_configuration() {
    // Test client creation and configuration without API calls

    // Test direct client creation
    let _client1 = Client::new("test_token_123".to_string());
    // Just verify it was created (we can't inspect internals)

    // Test config-based creation
    let config = Config::new("test_token_456".to_string())
        .with_timeout(60)
        .with_verbose(true)
        .with_endpoint_url("https://api.custom.airtable.com/v0");

    let _client2 = Client::from_config(config);
    // Verify creation succeeded

    // Test environment-based creation (if env vars available)
    if env::var("PERSONAL_ACCESS_TOKEN").is_ok() {
        let _client3 = Client::from_env().expect("Should create from environment");
        // Verify creation succeeded
    }

    println!("✅ All client creation methods work");
}

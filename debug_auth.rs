use rsairtable::{Client, Config};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check environment first
    println!("Environment check:");
    if let Ok(token) = env::var("PERSONAL_ACCESS_TOKEN") {
        println!("  PERSONAL_ACCESS_TOKEN found: {}...", &token[..20]);
    } else {
        println!("  PERSONAL_ACCESS_TOKEN not found");
    }

    // Load from environment
    let config = Config::from_env()?;
    println!("\nConfig loaded:");
    println!("  API Key: {}...", &config.api_key[..20]);
    println!("  Endpoint: {}", config.endpoint_url);

    let client = Client::from_config(config.clone());

    // Try to manually debug the request
    println!("\nTesting whoami call...");

    match client.whoami().await {
        Ok(user_info) => {
            println!("✅ Success!");
            println!("User ID: {}", user_info.id);
            println!("User Email: {}", user_info.email);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("Error type: {:?}", e);
        }
    }

    Ok(())
}

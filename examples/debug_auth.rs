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

    // Let's also test with a direct HTTP call to compare
    println!("\nTesting direct reqwest call...");
    let http_client = reqwest::Client::new();
    let auth_header = format!("Bearer {}", config.api_key);
    println!("  Auth header: {}", auth_header);

    let response = http_client
        .get("https://api.airtable.com/v0/meta/whoami")
        .header("Authorization", &auth_header)
        .header("User-Agent", "rsairtable-debug/0.1.0")
        .send()
        .await?;

    println!("  Direct call status: {}", response.status());
    if response.status().is_success() {
        let text = response.text().await?;
        println!("  Direct call response: {}", text);
    }

    // Try to manually debug the request
    println!("\nTesting whoami call through our client...");

    match client.whoami().await {
        Ok(user_info) => {
            println!("✅ Success!");
            println!("User ID: {}", user_info.id);
            println!("User Name: {}", user_info.name.as_deref().unwrap_or("N/A"));
            println!("User Email: {}", user_info.email);
        }
        Err(e) => {
            println!("❌ Error: {}", e);
            println!("Error type: {:?}", e);
        }
    }

    Ok(())
}

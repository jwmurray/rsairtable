use rsairtable::Client;
use serde_json::json;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();

    let client = Client::from_env()?;
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Create a record in TestCaseLaw table (this will stay in the table)
    let table = client.base(&base_id).table("TestCaseLaw");

    let fields = json!({
        "Name": "Demo Record - Created via API",
        "Status": "Todo",
        "Notes": "This record was created manually via the Rust API and will remain in the table for you to see."
    });

    let record = table.create(fields).await?;

    println!("✅ Successfully created permanent record!");
    println!("Record ID: {}", record.id);
    println!(
        "Record fields: {}",
        serde_json::to_string_pretty(&record.fields)?
    );
    println!("\nYou can now see this record in your TestCaseLaw table in Airtable.");
    println!(
        "To delete it later, run: cargo run --example delete_record -- {}",
        record.id
    );

    Ok(())
}

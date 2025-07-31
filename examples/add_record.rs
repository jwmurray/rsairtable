use rsairtable::Client;
use serde_json::json;
use std::env;

/// Create a record in TestCaseLaw table with custom field values
/// Usage: cargo run --example add_record -- "Record Name" "Status" "Notes"
/// Example: cargo run --example add_record -- "My Case" "Todo" "Initial notes for this case"

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load .env file
    dotenv::dotenv().ok();

    let args: Vec<String> = env::args().collect();

    // Parse command line arguments
    let (name, status, notes) = if args.len() >= 4 {
        (args[1].clone(), args[2].clone(), args[3].clone())
    } else {
        println!("Usage: cargo run --example add_record -- \"Record Name\" \"Status\" \"Notes\"");
        println!("Status options: Todo, In progress, Done");
        println!("\nExample:");
        println!(
            "cargo run --example add_record -- \"My Legal Case\" \"Todo\" \"Initial case notes\""
        );
        return Ok(());
    };

    // Validate status
    let valid_statuses = ["Todo", "In progress", "Done"];
    if !valid_statuses.contains(&status.as_str()) {
        println!(
            "❌ Invalid status: '{}'. Must be one of: {}",
            status,
            valid_statuses.join(", ")
        );
        return Ok(());
    }

    let client = Client::from_env()?;
    let base_id = env::var("BASE").expect("BASE environment variable not set");

    // Create the record
    let table = client.base(&base_id).table("TestCaseLaw");

    let fields = json!({
        "Name": name,
        "Status": status,
        "Notes": notes
    });

    let record = table.create(fields).await?;

    println!("✅ Successfully created record in TestCaseLaw table!");
    println!("📋 Record ID: {}", record.id);
    println!(
        "📝 Name: {}",
        record.fields.get("Name").unwrap().as_str().unwrap()
    );
    println!(
        "📊 Status: {}",
        record.fields.get("Status").unwrap().as_str().unwrap()
    );
    println!(
        "📄 Notes: {}",
        record.fields.get("Notes").unwrap().as_str().unwrap()
    );

    Ok(())
}

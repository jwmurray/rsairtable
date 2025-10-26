//! Command-line interface for RSAirtable
//!
//! This module implements a CLI that's functionally compatible with pyairtable's CLI,
//! using the same command structure and arguments.

use clap::{Arg, ArgMatches, Command};
use rsairtable::{BaseSchema, Client, Config};
use rsairtable::views::process_with_view;
use std::process;

#[tokio::main]
async fn main() {
    // Load .env file BEFORE clap processes arguments to ensure env vars are available
    dotenv::dotenv().ok();

    let matches = build_cli().get_matches();

    if let Err(e) = run_command(matches).await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn build_cli() -> Command {
    Command::new("rsairtable")
        .version("0.1.0")
        .about("Rust client for Airtable API - compatible with pyairtable")
        .after_help("HINT: Use 'rsairtable base [BASE_ID] --help' to see table operations, then 'rsairtable base [BASE_ID] table <TABLE_NAME> --help' for record operations\nBASE_ID can be omitted if BASE environment variable is set or if you have only one base")
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .value_name("KEY")
                .help("Your API key/token")
                .env("PERSONAL_ACCESS_TOKEN")
                .global(true),
        )
        .arg(
            Arg::new("key-file")
                .short('f')
                .long("key-file")
                .value_name("PATH")
                .help("File containing your API key")
                .global(true),
        )
        .arg(
            Arg::new("key-env")
                .short('e')
                .long("key-env")
                .value_name("VAR")
                .help("Environment variable containing your API key")
                .global(true),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Print verbose output")
                .global(true),
        )
        .arg(
            Arg::new("help-detail")
                .long("help-detail")
                .action(clap::ArgAction::SetTrue)
                .help("Show detailed examples and usage patterns")
                .global(true),
        )
        .subcommand(Command::new("whoami").about("Print current user information"))
        .subcommand(Command::new("bases").about("List all available bases"))
        .subcommand(
            Command::new("base")
                .about("Base operations")
                .after_help("HINT: Use 'rsairtable base [BASE_ID] --help' to see table and record operations\nBASE_ID can be omitted if BASE environment variable is set or if you have only one base")
                .arg(
                    Arg::new("base-id")
                        .value_name("BASE_ID")
                        .help("Base ID (e.g., appXXXXXXXXXXXXXX) [env: BASE]")
                        .env("BASE")
                        .required(false),
                )
                .subcommand(Command::new("schema").about("Print base schema"))
                .subcommand(Command::new("collaborators").about("Print base collaborators"))
                .subcommand(Command::new("shares").about("Print base shares"))
                .subcommand(Command::new("orm").about("Generate Rust structs for base tables"))
                .subcommand(
                    Command::new("table")
                        .about("Table operations")
                        .after_help("HINT: Use 'rsairtable base [BASE_ID] table <TABLE_NAME> --help' to see record operations (create, update, delete, records)\nBASE_ID can be omitted if BASE environment variable is set or if you have only one base")
                        .arg(
                            Arg::new("table-name")
                                .value_name("TABLE_NAME")
                                .help("Table name")
                                .required(true),
                        )
                        .subcommand(
                            Command::new("records")
                                .about("Retrieve records from table")
                                .arg(
                                    Arg::new("formula")
                                        .short('w')
                                        .long("formula")
                                        .value_name("FORMULA")
                                        .help("Filter records with a formula"),
                                )
                                .arg(
                                    Arg::new("view")
                                        .short('u')
                                        .long("view")
                                        .value_name("VIEW")
                                        .help("Filter records by a view"),
                                )
                                .arg(
                                    Arg::new("data-view")
                                        .long("data-view")
                                        .value_name("DATA_VIEW")
                                        .help("Apply specialized data view formatting (e.g., 'clio')"),
                                )
                                .arg(
                                    Arg::new("limit")
                                        .short('n')
                                        .long("limit")
                                        .value_name("NUMBER")
                                        .help("Limit the number of records returned")
                                        .value_parser(clap::value_parser!(u32)),
                                )
                                .arg(
                                    Arg::new("sort")
                                        .short('S')
                                        .long("sort")
                                        .value_name("FIELD")
                                        .help("Sort records by field(s)")
                                        .action(clap::ArgAction::Append),
                                )
                                .arg(
                                    Arg::new("field")
                                        .short('F')
                                        .long("field")
                                        .value_name("FIELD")
                                        .help("Limit output to certain field(s)")
                                        .action(clap::ArgAction::Append),
                                )
                                .arg(
                                    Arg::new("direction")
                                        .short('D')
                                        .long("direction")
                                        .value_name("DIRECTION")
                                        .help("Sort direction (asc/desc)")
                                        .value_parser(["asc", "desc"])
                                        .action(clap::ArgAction::Append),
                                )
                                .arg(
                                    Arg::new("offset")
                                        .long("offset")
                                        .value_name("OFFSET_TOKEN")
                                        .help("Continue from specified offset for pagination"),
                                )
                                .arg(
                                    Arg::new("all")
                                        .long("all")
                                        .action(clap::ArgAction::SetTrue)
                                        .help("Retrieve all records by automatically handling pagination")
                                        .conflicts_with_all(["offset", "limit"]),
                                ),
                        )
                        .subcommand(Command::new("schema").about("Print table schema"))
                        .subcommand(
                            Command::new("create")
                                .about("Create a new record")
                                .arg(
                                    Arg::new("fields")
                                        .short('j')
                                        .long("fields")
                                        .value_name("JSON")
                                        .help("Record fields as JSON")
                                        .required(true),
                                )
                                .arg(
                                    Arg::new("typecast")
                                        .short('t')
                                        .long("typecast")
                                        .action(clap::ArgAction::SetTrue)
                                        .help("Enable automatic typecasting"),
                                ),
                        )
                        .subcommand(
                            Command::new("update")
                                .about("Update an existing record")
                                .arg(
                                    Arg::new("record-id")
                                        .value_name("RECORD_ID")
                                        .help("Record ID to update")
                                        .required(true),
                                )
                                .arg(
                                    Arg::new("fields")
                                        .short('j')
                                        .long("fields")
                                        .value_name("JSON")
                                        .help("Record fields as JSON")
                                        .required(true),
                                )
                                .arg(
                                    Arg::new("typecast")
                                        .short('t')
                                        .long("typecast")
                                        .action(clap::ArgAction::SetTrue)
                                        .help("Enable automatic typecasting"),
                                ),
                        )
                        .subcommand(
                            Command::new("delete").about("Delete a record").arg(
                                Arg::new("record-id")
                                    .value_name("RECORD_ID")
                                    .help("Record ID to delete")
                                    .required(true),
                            ),
                        ),
                ),
        )
        .subcommand(
            Command::new("enterprise")
                .about("Enterprise operations")
                .subcommand(Command::new("audit-log").about("Retrieve audit log"))
                .subcommand(Command::new("users").about("List enterprise users"))
                .subcommand(Command::new("claims").about("List enterprise claims")),
        )
}

async fn run_command(matches: ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    // Check for help-detail flag first
    if matches.get_flag("help-detail") {
        print_detailed_help();
        return Ok(());
    }

    // Get API key from various sources with priority order
    let api_key = get_api_key(&matches)?;

    // Create client
    let mut config = Config::new(api_key);
    if matches.get_flag("verbose") {
        config = config.with_verbose(true);
    }

    let client = Client::from_config(config);

    match matches.subcommand() {
        Some(("whoami", _)) => {
            let user_info = client.whoami().await?;
            println!("{}", serde_json::to_string_pretty(&user_info)?);
        }
        Some(("bases", _)) => {
            let bases = client.bases().await?;
            for base in bases {
                println!("{} - {}", base.id, base.name);
            }
        }
        Some(("base", base_matches)) => {
            let base_id = match base_matches.get_one::<String>("base-id") {
                Some(id) => id.clone(),
                None => {
                    // Try to auto-detect base if only one is available
                    let bases = client.bases().await?;
                    if bases.len() == 1 {
                        println!("Auto-detected base: {} - {}", bases[0].id, bases[0].name);
                        bases[0].id.clone()
                    } else if bases.is_empty() {
                        return Err(Box::new(rsairtable::Error::config("No bases found. Check your API key and permissions.")));
                    } else {
                        println!("Multiple bases available:");
                        for base in &bases {
                            println!("  {} - {}", base.id, base.name);
                        }
                        return Err(Box::new(rsairtable::Error::config(
                            "Multiple bases found. Please specify BASE_ID or set BASE environment variable."
                        )));
                    }
                }
            };
            let base = client.base(&base_id);

            match base_matches.subcommand() {
                Some(("schema", _)) => {
                    let schema = base.schema().await?;
                    println!("{}", serde_json::to_string_pretty(&schema)?);
                }
                Some(("collaborators", _)) => {
                    // Note: Collaborators endpoint is not available in the public API
                    println!(
                        "Collaborators information is not available via the public Airtable API"
                    );
                    println!("Use the Airtable web interface to manage collaborators");
                }
                Some(("shares", _)) => {
                    // Note: Shares endpoint is not available in the public API
                    println!("Shares information is not available via the public Airtable API");
                    println!("Use the Airtable web interface to manage shares");
                }
                Some(("orm", _)) => {
                    let schema = base.schema().await?;
                    generate_rust_structs(&schema);
                }
                Some(("table", table_matches)) => {
                    let table_name = table_matches.get_one::<String>("table-name").unwrap();
                    let table = base.table(table_name);

                    match table_matches.subcommand() {
                        Some(("records", record_matches)) => {
                            let mut query = table.list();

                            if let Some(limit) = record_matches.get_one::<u32>("limit") {
                                query = query.max_records(*limit);
                            }

                            if let Some(formula) = record_matches.get_one::<String>("formula") {
                                query = query.filter_by_formula(formula);
                            }

                            if let Some(view) = record_matches.get_one::<String>("view") {
                                query = query.view(view);
                            }

                            if let Some(fields) = record_matches.get_many::<String>("field") {
                                let field_list: Vec<String> = fields.cloned().collect();
                                let field_refs: Vec<&str> =
                                    field_list.iter().map(|s| s.as_str()).collect();
                                query = query.fields(&field_refs);
                            }

                            if let Some(sorts) = record_matches.get_many::<String>("sort") {
                                let sort_list: Vec<String> = sorts.cloned().collect();
                                query = query.sort(sort_list);
                            }

                            if let Some(offset) = record_matches.get_one::<String>("offset") {
                                query = query.offset(Some(offset.clone()));
                            }

                            if record_matches.get_flag("all") {
                                // Implement automatic pagination to retrieve all records
                                let mut all_records = Vec::new();
                                let mut current_offset: Option<String> = None;
                                let verbose = matches.get_flag("verbose");

                                if verbose {
                                    eprintln!("Starting pagination to retrieve all records...");
                                }

                                loop {
                                    let mut current_query = table.list();

                                    // Apply all the same filters as the base query
                                    if let Some(formula) =
                                        record_matches.get_one::<String>("formula")
                                    {
                                        current_query = current_query.filter_by_formula(formula);
                                    }
                                    if let Some(view) = record_matches.get_one::<String>("view") {
                                        current_query = current_query.view(view);
                                    }
                                    if let Some(fields) = record_matches.get_many::<String>("field")
                                    {
                                        let field_list: Vec<String> = fields.cloned().collect();
                                        let field_refs: Vec<&str> =
                                            field_list.iter().map(|s| s.as_str()).collect();
                                        current_query = current_query.fields(&field_refs);
                                    }
                                    if let Some(sorts) = record_matches.get_many::<String>("sort") {
                                        let sort_list: Vec<String> = sorts.cloned().collect();
                                        current_query = current_query.sort(sort_list);
                                    }

                                    // Apply current offset
                                    if let Some(ref offset) = current_offset {
                                        current_query = current_query.offset(Some(offset.clone()));
                                    }

                                    // Execute query for this batch
                                    let (batch_records, next_offset) =
                                        current_query.execute().await?;

                                    let batch_count = batch_records.len();

                                    // Add batch to our collection
                                    all_records.extend(batch_records);

                                    if verbose {
                                        eprintln!(
                                            "Retrieved {} records (total: {})",
                                            batch_count,
                                            all_records.len()
                                        );
                                    }

                                    // Check if we should continue
                                    match next_offset {
                                        Some(offset) if !offset.is_empty() => {
                                            current_offset = Some(offset);
                                            if verbose {
                                                eprintln!("Fetching more records...");
                                            }
                                        }
                                        _ => break, // No more pages
                                    }
                                }

                                if verbose {
                                    eprintln!(
                                        "Completed! Retrieved {} records total",
                                        all_records.len()
                                    );
                                }

                                // Process with data view if specified
                                if let Some(data_view) = record_matches.get_one::<String>("data-view") {
                                    let view_result = process_with_view(data_view, all_records)?;
                                    println!("{}", serde_json::to_string_pretty(&view_result)?);
                                } else {
                                    // Output all records in the same format as regular queries
                                    let result = (all_records, None::<String>);
                                    println!("{}", serde_json::to_string_pretty(&result)?);
                                }
                            } else {
                                let records = query.execute().await?;

                                if matches.get_flag("verbose") {
                                    let record_count = records.0.len();
                                    let has_more = records.1.is_some();
                                    if has_more {
                                        eprintln!("Retrieved {} records (more available, use --all or --offset)", record_count);
                                    } else {
                                        eprintln!(
                                            "Retrieved {} records (all records from table)",
                                            record_count
                                        );
                                    }
                                }

                                // Process with data view if specified
                                if let Some(data_view) = record_matches.get_one::<String>("data-view") {
                                    let view_result = process_with_view(data_view, records.0)?;
                                    println!("{}", serde_json::to_string_pretty(&view_result)?);
                                } else {
                                    println!("{}", serde_json::to_string_pretty(&records)?);
                                }
                            }
                        }
                        Some(("schema", _)) => {
                            let schema = table.schema().await?;
                            println!("{}", serde_json::to_string_pretty(&schema)?);
                        }
                        Some(("create", create_matches)) => {
                            let fields_json = create_matches.get_one::<String>("fields").unwrap();
                            let fields: serde_json::Value = serde_json::from_str(fields_json)?;
                            let typecast = create_matches.get_flag("typecast");

                            let record = if typecast {
                                table.create_with_typecast(fields, true).await?
                            } else {
                                table.create(fields).await?
                            };

                            println!("✅ Created record: {}", record.id);
                            println!("{}", serde_json::to_string_pretty(&record)?);
                        }
                        Some(("update", update_matches)) => {
                            let record_id = update_matches.get_one::<String>("record-id").unwrap();
                            let fields_json = update_matches.get_one::<String>("fields").unwrap();
                            let fields: serde_json::Value = serde_json::from_str(fields_json)?;
                            let typecast = update_matches.get_flag("typecast");

                            let record = if typecast {
                                table.update_with_typecast(record_id, fields, true).await?
                            } else {
                                table.update(record_id, fields).await?
                            };

                            println!("✅ Updated record: {}", record.id);
                            println!("{}", serde_json::to_string_pretty(&record)?);
                        }
                        Some(("delete", delete_matches)) => {
                            let record_id = delete_matches.get_one::<String>("record-id").unwrap();
                            table.delete(record_id).await?;
                            println!("✅ Deleted record: {}", record_id);
                        }
                        _ => {
                            eprintln!("No table subcommand specified");
                            process::exit(1);
                        }
                    }
                }
                _ => {
                    eprintln!("No base subcommand specified");
                    process::exit(1);
                }
            }
        }
        Some(("enterprise", enterprise_matches)) => {
            match enterprise_matches.subcommand() {
                Some(("audit-log", _)) => {
                    println!("Enterprise audit log is not available via the public Airtable API");
                    println!("Use the Airtable Enterprise Admin Panel for audit logs");
                }
                Some(("users", _)) => {
                    println!(
                        "Enterprise users management is not available via the public Airtable API"
                    );
                    println!("Use the Airtable Enterprise Admin Panel for user management");
                }
                Some(("claims", _)) => {
                    println!("Enterprise claims information is not available via the public Airtable API");
                    println!("Use the Airtable Enterprise Admin Panel for claims management");
                }
                _ => {
                    eprintln!("No enterprise subcommand specified");
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("No command specified. Use --help for usage information.");
            process::exit(1);
        }
    }

    Ok(())
}

fn get_api_key(matches: &ArgMatches) -> Result<String, Box<dyn std::error::Error>> {
    // Priority order: CLI arg > key-file > key-env > environment variables

    // 1. Direct CLI argument
    if let Some(key) = matches.get_one::<String>("key") {
        return Ok(key.clone());
    }

    // 2. Key from file
    if let Some(key_file) = matches.get_one::<String>("key-file") {
        let key = std::fs::read_to_string(key_file)?.trim().to_string();
        return Ok(key);
    }

    // 3. Key from specified environment variable
    if let Some(key_env) = matches.get_one::<String>("key-env") {
        let key = std::env::var(key_env)?;
        return Ok(key);
    }

    // 4. Key from standard environment variables
    Config::api_key_from_env_or_file().map_err(|e| e.into())
}

fn print_detailed_help() {
    print!(
        r#"
🚀 RSAIRTABLE - DETAILED EXAMPLES AND USAGE GUIDE
===============================================

This is a comprehensive Rust client for the Airtable API, compatible with pyairtable.
Below are practical examples for all major operations.

📋 SETUP AND AUTHENTICATION
---------------------------

# Method 1: Environment variable (recommended)
export PERSONAL_ACCESS_TOKEN="patXXXXXXXXXXXXXX"
export BASE="appXXXXXXXXXXXXXX"

# Method 2: CLI argument
rsairtable --key "patXXXXXXXXXXXXXX" whoami

# Method 3: Key file
echo "patXXXXXXXXXXXXXX" > ~/.airtable_key
rsairtable --key-file ~/.airtable_key whoami

👤 USER INFORMATION
------------------

# Get current user information
rsairtable whoami

📚 BASE OPERATIONS
-----------------

# List all bases you have access to
rsairtable bases

# Get base schema (all tables and fields)
rsairtable base schema                          # Uses BASE env var or auto-detects if only one base
rsairtable base appXXXXXXXXXXXXXX schema        # Explicit base ID

# Generate Rust structs from base schema (ORM)
rsairtable base orm > models.rs                # Uses BASE env var or auto-detects if only one base
rsairtable base appXXXXXXXXXXXXXX orm > models.rs   # Explicit base ID

🗂️  TABLE OPERATIONS
-------------------

# Get table schema
rsairtable base table "TableName" schema               # Uses BASE env var or auto-detects if only one base
rsairtable base appXXXXXXXXXXXXXX table "TableName" schema  # Explicit base ID

📄 RECORD OPERATIONS
-------------------

# List all records (default: first 100)
rsairtable base table "TableName" records              # Uses BASE env var or auto-detects if only one base
rsairtable base appXXXXXXXXXXXXXX table "TableName" records  # Explicit base ID

# Get specific record by ID (equivalent to a "get" command)
rsairtable base table "TableName" records \\          # Uses BASE env var or auto-detects if only one base
  -w "RECORD_ID()='recXXXXXXXXXXXXX'"
rsairtable base appXXXXXXXXXXXXXX table "TableName" records \\  # Explicit base ID
  -w "RECORD_ID()='recXXXXXXXXXXXXX'"

# List specific number of records
rsairtable base table "TableName" records -n 10       # Uses BASE env var or auto-detects if only one base

# List records with filtering
rsairtable base appXXXXXXXXXXXXXX table "TableName" records -w "Status = 'Active'"

# List records from specific view
rsairtable base appXXXXXXXXXXXXXX table "TableName" records -u "My View"

# List records with sorting
rsairtable base appXXXXXXXXXXXXXX table "TableName" records -S "Name asc"

# List specific fields only
rsairtable base appXXXXXXXXXXXXXX table "TableName" records -F "Name" -F "Status"

# Get records in descending order
rsairtable base appXXXXXXXXXXXXXX table "TableName" records -D -n 5

📖 PAGINATION - RETRIEVING ALL RECORDS
--------------------------------------

# Automatic pagination - get ALL records from a table (may take time for large tables)
rsairtable base table "TableName" records --all            # Uses BASE env var or auto-detects if only one base
rsairtable base appXXXXXXXXXXXXXX table "TableName" records --all  # Explicit base ID

# Get all records with filtering (retrieve all matching records)
rsairtable base table "TableName" records --all -w "Status = 'Active'"
rsairtable base table "TableName" records --all -F "Name" -F "Email"

# Manual pagination - continue from specific offset token
rsairtable base table "TableName" records --offset "itrABC123/recXYZ789"

# Verbose mode - shows pagination progress
rsairtable -v base table "TableName" records --all
# Output: "Starting pagination to retrieve all records..."
#         "Retrieved 100 records (total: 100)"
#         "Fetching more records..."
#         "Retrieved 50 records (total: 150)"
#         "Completed! Retrieved 150 records total"

# Combine pagination with all filtering options
rsairtable -v base table "Customers" records --all \\
  -w "Status = 'Active'" \\
  -F "Name" -F "Email" -F "Status" \\
  -u "Customer View" \\
  -S "Name"

⚠️  PAGINATION NOTES:
- Default: Returns first 100 records only
- --all: Automatically retrieves ALL records (can be thousands)
- --offset: Manual pagination using token from previous request  
- --all conflicts with --limit and --offset
- Use -v (verbose) to see progress for large datasets

✏️  RECORD CREATION
------------------

# Create a simple record
rsairtable base table "TableName" create \\              # Uses BASE env var or auto-detects if only one base
  -j '{{"Name": "New Record", "Status": "Active"}}'

# Create record with typecast (automatic type conversion)  
rsairtable base table "TableName" create \\              # Uses BASE env var or auto-detects if only one base
  -j '{{"Name": "Auto Convert", "Date": "2024-01-15"}}' \\
  --typecast

# Create record with multiple field types
rsairtable base appXXXXXXXXXXXXXX table "TableName" create \\
  -j '{{
    "Name": "Complex Record",
    "Status": "In Progress", 
    "Priority": 5,
    "Notes": "This is a detailed note",
    "Tags": ["urgent", "review"]
  }}'

📝 RECORD UPDATES
----------------

# Update specific fields of a record
rsairtable base appXXXXXXXXXXXXXX table "TableName" update recXXXXXXXXXXXXX \\
  -j '{{"Status": "Completed", "Notes": "Finished today"}}'

# Update with typecast
rsairtable base appXXXXXXXXXXXXXX table "TableName" update recXXXXXXXXXXXXX \\
  -j '{{"Priority": "High", "Due Date": "2024-12-31"}}' \\
  --typecast

🗑️  RECORD DELETION
------------------

# Delete a specific record
rsairtable base appXXXXXXXXXXXXXX table "TableName" delete recXXXXXXXXXXXXX

🏢 ADVANCED FEATURES
-------------------

# Enterprise operations (limitations noted)
rsairtable enterprise audit-log
rsairtable enterprise users

# Base collaboration info (limitations noted)
rsairtable base appXXXXXXXXXXXXXX collaborators
rsairtable base appXXXXXXXXXXXXXX shares

🔗 REAL-WORLD EXAMPLES
---------------------

# Example 1: Customer Management
rsairtable base appCustomers table "Customers" create \\
  -j '{{
    "Company": "Tech Corp", 
    "Contact": "John Smith",
    "Email": "john@techcorp.com",
    "Status": "Active",
    "Revenue": 50000
  }}'

# Example 2: Get Specific Matter by ID
rsairtable base table "Matters" records \\
  -w "RECORD_ID()='recrh79HILmmNEepC'"

# Example 3: Task Tracking
rsairtable base appTasks table "Tasks" records \\
  -w "AND(Status != 'Done', {{Assignee}} = 'John')" \\
  -S "Priority desc, Due Date asc" \\
  -n 20

# Example 4: Inventory Update
rsairtable base appInventory table "Products" update recProductXYZ \\
  -j '{{
    "Stock": 150,
    "Last Updated": "2024-01-15T10:30:00Z",
    "Status": "In Stock"
  }}' \\
  --typecast

# Example 5: Bulk Data Export to JSON
rsairtable base appSales table "Orders" records \\
  -F "Order ID" -F "Customer" -F "Amount" -F "Date" \\
  > orders_export.json

# Example 6: Complete Data Migration - Get ALL records with pagination
rsairtable -v base table "Customers" records --all \\
  -F "Name" -F "Email" -F "Status" \\
  -w "Status != 'Deleted'" \\
  > complete_customer_export.json

# Example 7: Large Dataset Processing with Progress
rsairtable -v base appInventory table "Products" records --all \\
  | jq '.[[]][].fields | {{name: .Name, price: .Price, stock: .Stock}}' \\
  > inventory_summary.json

# Example 8: Continuing from Previous Export (Manual Pagination)
# First, get initial batch and note the offset in response[1]
rsairtable base table "LargeTable" records -n 100 > batch1.json
# Then continue from that offset
rsairtable base table "LargeTable" records \\
  --offset "itrABC123/recXYZ789" > batch2.json

🚀 RUST CODE GENERATION
-----------------------

# Generate complete Rust structs for your base
rsairtable base appXXXXXXXXXXXXXX orm > src/airtable_models.rs

# This creates structs like:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {{
    pub id: String,
    pub created_time: Option<String>,
    pub company: Option<String>,
    pub contact: Option<String>,
    pub email: Option<String>,
    // ... all your fields
}}

⚠️  API LIMITATIONS
------------------

Some advanced features are not available via Airtable's public API:
• Field creation/deletion (use Airtable web interface)
• Direct file uploads (use URL-based attachments)
• Base collaborator management (use Airtable web interface)
• Enterprise audit logs (use Enterprise Admin Panel)

🛠️  TROUBLESHOOTING
------------------

# Test authentication
rsairtable whoami

# Check base access
rsairtable bases

# Validate table name
rsairtable base appXXXXXXXXXXXXXX schema | grep -i "tablename"

# Test with verbose output
rsairtable -v base appXXXXXXXXXXXXXX table "TableName" records -n 1

📖 MORE HELP
-----------

# Basic help
rsairtable --help

# Command-specific help
rsairtable base --help
rsairtable base <BASE_ID> table --help
rsairtable base <BASE_ID> table <TABLE_NAME> --help

For more information, visit: https://github.com/your-repo/rsairtable
"#
    );
}

/// Generate Rust structs for all tables in a base (ORM-like functionality)
fn generate_rust_structs(schema: &BaseSchema) {
    println!("// Generated Rust structs for Airtable base");
    println!("// This is equivalent to pyairtable's ORM generation");
    println!();
    println!("use serde::{{{}, {}}};", "Deserialize", "Serialize");
    println!("use rsairtable::Record;");
    println!();

    for table in &schema.tables {
        let struct_name = to_pascal_case(&table.name);

        println!(
            "/// Struct representing records from the '{}' table",
            table.name
        );
        println!("#[derive(Debug, Clone, Serialize, Deserialize)]");
        println!("pub struct {} {{", struct_name);
        println!("    pub id: String,");
        println!("    pub created_time: Option<String>,");

        for field in &table.fields {
            let field_name = to_snake_case(&field.name);
            let rust_type = field_type_to_rust_type(&field.field_type);

            println!("    /// {} field: {}", field.name, field.field_type);
            println!("    pub {}: Option<{}>,", field_name, rust_type);
        }

        println!("}}\n");

        // Generate implementation
        println!("impl {} {{", struct_name);
        println!("    /// Convert from a generic Airtable Record");
        println!("    pub fn from_record(record: Record) -> Self {{");
        println!("        Self {{");
        println!("            id: record.id,");
        println!("            created_time: record.created_time,");

        for field in &table.fields {
            let field_name = to_snake_case(&field.name);
            println!("            {}: record.fields.get(\"{}\").and_then(|v| serde_json::from_value(v.clone()).ok()),", 
                     field_name, field.name);
        }

        println!("        }}");
        println!("    }}");
        println!("}}\n");
    }

    println!("// Usage example:");
    println!("// let client = rsairtable::Client::from_env()?;");
    println!(
        "// let table = client.base(\"your_base_id\").table(\"{}\");",
        schema
            .tables
            .first()
            .map(|t| &t.name)
            .unwrap_or(&"TableName".to_string())
    );
    println!("// let records = table.list().execute().await?;");
    println!(
        "// let typed_records: Vec<{}> = records.into_iter().map({}::from_record).collect();",
        to_pascal_case(
            &schema
                .tables
                .first()
                .map(|t| &t.name)
                .unwrap_or(&"TableName".to_string())
        ),
        to_pascal_case(
            &schema
                .tables
                .first()
                .map(|t| &t.name)
                .unwrap_or(&"TableName".to_string())
        )
    );
}

/// Convert a string to PascalCase for struct names
fn to_pascal_case(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// Convert a string to snake_case for field names
fn to_snake_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_whitespace() || c == '-' {
                '_'
            } else {
                c.to_lowercase().next().unwrap_or(c)
            }
        })
        .collect()
}

/// Convert Airtable field types to Rust types
fn field_type_to_rust_type(field_type: &str) -> &'static str {
    match field_type {
        "singleLineText" | "multilineText" | "richText" | "email" | "url" | "phoneNumber" => {
            "String"
        }
        "number" | "currency" | "percent" | "duration" | "rating" | "autonumber" => "f64",
        "checkbox" => "bool",
        "date" | "dateTime" | "createdTime" | "lastModifiedTime" => "String",
        "singleSelect" => "String",
        "multipleSelects" => "Vec<String>",
        "singleCollaborator" => "serde_json::Value", // Collaborator object
        "multipleCollaborators" => "Vec<serde_json::Value>",
        "multipleRecordLinks" => "Vec<String>", // Array of record IDs
        "lookup" | "rollup" | "formula" | "count" => "serde_json::Value", // Computed fields
        "multipleAttachments" => "Vec<serde_json::Value>", // Array of attachment objects
        "barcode" => "serde_json::Value",       // Barcode object
        "button" => "serde_json::Value",        // Button object
        _ => "serde_json::Value",               // Fallback for unknown types
    }
}

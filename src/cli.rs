//! Command-line interface for RSAirtable
//!
//! This module implements a CLI that's functionally compatible with pyairtable's CLI,
//! using the same command structure and arguments.

use clap::{Arg, ArgMatches, Command};
use rsairtable::{BaseSchema, Client, Config};
use std::process;

#[tokio::main]
async fn main() {
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
        .after_help("HINT: Use 'rsairtable base <BASE_ID> --help' to see table operations, then 'rsairtable base <BASE_ID> table <TABLE_NAME> --help' for record operations")
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
        .subcommand(Command::new("whoami").about("Print current user information"))
        .subcommand(Command::new("bases").about("List all available bases"))
        .subcommand(
            Command::new("base")
                .about("Base operations")
                .after_help("HINT: Use 'rsairtable base <BASE_ID> --help' to see table and record operations")
                .arg(
                    Arg::new("base-id")
                        .value_name("BASE_ID")
                        .help("Base ID (e.g., appXXXXXXXXXXXXXX)")
                        .required(true),
                )
                .subcommand(Command::new("schema").about("Print base schema"))
                .subcommand(Command::new("collaborators").about("Print base collaborators"))
                .subcommand(Command::new("shares").about("Print base shares"))
                .subcommand(Command::new("orm").about("Generate Rust structs for base tables"))
                .subcommand(
                    Command::new("table")
                        .about("Table operations")
                        .after_help("HINT: Use 'rsairtable base <BASE_ID> table <TABLE_NAME> --help' to see record operations (create, update, delete, records)")
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
            let base_id = base_matches.get_one::<String>("base-id").unwrap();
            let base = client.base(base_id);

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

                            let records = query.execute().await?;
                            println!("{}", serde_json::to_string_pretty(&records)?);
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

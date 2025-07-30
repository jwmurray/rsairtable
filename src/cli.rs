//! Command-line interface for RSAirtable
//!
//! This module implements a CLI that's functionally compatible with pyairtable's CLI,
//! using the same command structure and arguments.

use clap::{Arg, ArgMatches, Command};
use rsairtable::{Client, Config};
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
        .arg(
            Arg::new("key")
                .short('k')
                .long("key")
                .value_name("KEY")
                .help("Your API key/token")
                .env("PERSONAL_ACCESS_TOKEN"),
        )
        .arg(
            Arg::new("key-file")
                .short('f')
                .long("key-file")
                .value_name("PATH")
                .help("File containing your API key"),
        )
        .arg(
            Arg::new("key-env")
                .short('e')
                .long("key-env")
                .value_name("VAR")
                .help("Environment variable containing your API key"),
        )
        .arg(
            Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::ArgAction::SetTrue)
                .help("Print verbose output"),
        )
        .subcommand(Command::new("whoami").about("Print current user information"))
        .subcommand(Command::new("bases").about("List all available bases"))
        .subcommand(
            Command::new("base")
                .about("Base operations")
                .arg(
                    Arg::new("base-id")
                        .value_name("BASE_ID")
                        .help("Base ID (e.g., appXXXXXXXXXXXXXX)")
                        .required(true),
                )
                .subcommand(Command::new("schema").about("Print base schema"))
                .subcommand(Command::new("collaborators").about("Print base collaborators"))
                .subcommand(Command::new("shares").about("Print base shares"))
                .subcommand(
                    Command::new("table")
                        .about("Table operations")
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
                                        .short('f')
                                        .long("formula")
                                        .value_name("FORMULA")
                                        .help("Filter records with a formula"),
                                )
                                .arg(
                                    Arg::new("view")
                                        .short('v')
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
                                ),
                        )
                        .subcommand(Command::new("schema").about("Print table schema")),
                ),
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
                    println!("Collaborators command not yet implemented");
                }
                Some(("shares", _)) => {
                    println!("Shares command not yet implemented");
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
                            println!("Table schema command not yet implemented");
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

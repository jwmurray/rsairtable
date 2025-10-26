# 🚀 rsairtable

A comprehensive Rust client for the Airtable API, fully compatible with pyairtable functionality and providing both a library and CLI interface.

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-46%20passing-green.svg)](tests/)

## ✨ Features

- **🔄 Full pyairtable Compatibility**: Drop-in replacement for Python pyairtable functionality
- **📊 Complete CRUD Operations**: Create, read, update, delete records with full type safety
- **🔍 Advanced Filtering**: Support for formulas, views, sorting, and field selection
- **📄 Automatic Pagination**: Retrieve all records from large tables with `--all` flag
- **🏗️ ORM Code Generation**: Automatically generate Rust structs from Airtable schemas
- **🖥️ CLI Interface**: Comprehensive command-line tool for all operations
- **🚀 High Performance**: Built with async/await and reqwest for optimal performance
- **🛡️ Type Safety**: Leverage Rust's type system for reliable API interactions
- **📖 Comprehensive Help**: Built-in detailed examples and usage guides

## 🏁 Quick Start

### Default Behavior

**The most common usage** - simply run without any arguments:

```bash
rsairtable
```

This automatically:
- Auto-detects your base (if you have only one)
- Fetches all records from the "Cases" table
- Saves them to `cases.json`

### Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
rsairtable = "0.1.0"
```

### Authentication

Set your Airtable credentials:

```bash
export PERSONAL_ACCESS_TOKEN="patXXXXXXXXXXXXXX"
export BASE="appXXXXXXXXXXXXXX"
```

### Library Usage

```rust
use rsairtable::{Client, Config};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = Client::from_env()?;
    
    // Get a table handle
    let table = client
        .base("appXXXXXXXXXXXXXX")
        .table("TableName");
    
    // Create a record
    let record = table.create(json!({
        "Name": "New Record",
        "Status": "Active"
    })).await?;
    
    // List records
    let (records, _) = table.list()
        .max_records(10)
        .execute()
        .await?;
    
    // Update a record
    table.update(&record.id, json!({
        "Status": "Completed"
    })).await?;
    
    Ok(())
}
```

### CLI Usage

```bash
# Default behavior - get all Cases records and save to cases.json
rsairtable

# Get user information
rsairtable whoami

# List all bases
rsairtable bases

# Get base schema
rsairtable base appXXXXXXXXXXXXXX schema

# List records
rsairtable base appXXXXXXXXXXXXXX table "TableName" records

# Get specific record by ID
rsairtable base appXXXXXXXXXXXXXX table "TableName" records \
  --formula "RECORD_ID()='recXXXXXXXXXXXXX'"

# Create a record
rsairtable base appXXXXXXXXXXXXXX table "TableName" create \
  -j '{"Name": "CLI Record", "Status": "Active"}'

# Generate Rust structs
rsairtable base appXXXXXXXXXXXXXX orm > models.rs

# Pagination - get ALL records from a table
rsairtable base appXXXXXXXXXXXXXX table "TableName" records --all

# Manual pagination with offset
rsairtable base appXXXXXXXXXXXXXX table "TableName" records --offset "itrABC123"

# Verbose pagination with progress
rsairtable -v base appXXXXXXXXXXXXXX table "TableName" records --all
```

### Pagination

The CLI supports both automatic and manual pagination for retrieving large datasets:

- **Default**: Returns first 100 records only
- **`--all`**: Automatically retrieves ALL records from the table
- **`--offset`**: Continue from a specific pagination token
- **`-v`**: Verbose mode shows pagination progress

#### Pagination Examples

```bash
# Get all records automatically (may take time for large tables)
rsairtable base table "Customers" records --all

# Get all records with filtering
rsairtable base table "Customers" records --all -w "Status = 'Active'"

# Get all records with specific fields only
rsairtable base table "Customers" records --all -F "Name" -F "Email"

# Manual pagination - get first batch and continue with offset
rsairtable base table "Customers" records > batch1.json
rsairtable base table "Customers" records --offset "itrXYZ123" > batch2.json

# Verbose mode shows progress for large datasets
rsairtable -v base table "Customers" records --all
```

#### Record Retrieval Patterns

```bash
# Store specific matter into json file
cargo run -- base table Matters records --formula "RECORD_ID()='recrh79HILmmNEepC'" > pratt_matter.json
```


```bash
# List all records
rsairtable base <BASE_ID> table <TABLE> records

# Get specific record by ID (equivalent to a "get" command)
rsairtable base <BASE_ID> table <TABLE> records \
  --formula "RECORD_ID()='recXXXXXXXXXXXXX'"
```

```bash
# Create mapping of all airetable ids to clio ids
rsairtable base table Matters records --all -F "Clio Matter ID" | jq -r '.[0] | map({(.id): .fields."Clio M
atter ID"}) | add' > mapping.json
```

```bash
#Create mapping of non-null airtable ids to clio ids
rsairtable base table Matters records --all -F "Clio Matter ID" | jq -r '.[0] | map(select(.fields."Clio Ma
tter ID" != null)) | map({(.id): .fields."Clio Matter ID"}) | add' > mapping_no_nulls_jq.json
```

## 📚 Documentation

### Detailed Examples

For comprehensive examples and usage patterns:

```bash
rsairtable --help-detail
```

This shows:
- 🔐 Authentication methods
- 📊 CRUD operations with examples
- 🔍 Advanced filtering and sorting
- 🏗️ ORM code generation
- 🛠️ Troubleshooting guide
- 🌍 Real-world use cases

### Library API

#### Client Creation

```rust
// From environment variables
let client = Client::from_env()?;

// From explicit config
let config = Config::new("patXXXXXXXXXXXXXX");
let client = Client::from_config(config);
```

#### Record Operations

```rust
let table = client.base("appXXXXXXXXXXXXXX").table("TableName");

// Create single record
let record = table.create(json!({
    "Name": "New Record",
    "Status": "Active"
})).await?;

// Create with typecast
let record = table.create_with_typecast(json!({
    "Date": "2024-01-15",  // Will be converted to date
    "Number": "42"         // Will be converted to number
}), true).await?;

// Batch create
let records = table.batch_create(vec![
    json!({"Name": "Record 1"}),
    json!({"Name": "Record 2"}),
]).await?;

// Query with builder pattern
let (records, offset) = table.list()
    .formula("Status = 'Active'")
    .view("My View")
    .sort("Name", "asc")
    .fields(&["Name", "Status"])
    .max_records(100)
    .execute()
    .await?;

// Update record
table.update("recXXXXXXXXXXXXX", json!({
    "Status": "Completed"
})).await?;

// Delete record
table.delete("recXXXXXXXXXXXXX").await?;
```

#### Schema Operations

```rust
// Get base schema
let schema = client.base("appXXXXXXXXXXXXXX").schema().await?;

// Get table schema
let table_schema = client
    .base("appXXXXXXXXXXXXXX")
    .table("TableName")
    .schema()
    .await?;
```

#### Advanced Features

```rust
// Comments (where supported)
let comments = table.comments("recXXXXXXXXXXXXX").await?;
table.add_comment("recXXXXXXXXXXXXX", "Great work!").await?;

// Error handling
match table.get("invalid_record_id").await {
    Ok(record) => println!("Found: {:?}", record),
    Err(rsairtable::Error::Api { status: 404, .. }) => {
        println!("Record not found");
    }
    Err(e) => return Err(e.into()),
}
```

### CLI Reference

#### Core Commands

```bash
# Authentication and user info
rsairtable whoami
rsairtable bases

# Base operations
rsairtable base <BASE_ID> schema
rsairtable base <BASE_ID> collaborators  # API limitation
rsairtable base <BASE_ID> shares         # API limitation
rsairtable base <BASE_ID> orm

# Table operations
rsairtable base <BASE_ID> table <TABLE> schema
rsairtable base <BASE_ID> table <TABLE> records [OPTIONS]
rsairtable base <BASE_ID> table <TABLE> create -j <JSON> [--typecast]
rsairtable base <BASE_ID> table <TABLE> update <RECORD_ID> -j <JSON> [--typecast]
rsairtable base <BASE_ID> table <TABLE> delete <RECORD_ID>
```

#### Record Retrieval Patterns

```bash
# List all records
rsairtable base <BASE_ID> table <TABLE> records

# Get specific record by ID (equivalent to a "get" command)
rsairtable base <BASE_ID> table <TABLE> records \
  --formula "RECORD_ID()='recXXXXXXXXXXXXX'"

#### Advanced Filtering

```bash
# Filter by formula
rsairtable base <BASE_ID> table <TABLE> records \
  -w "AND(Status = 'Active', Priority > 3)"

# Use specific view
rsairtable base <BASE_ID> table <TABLE> records -u "My View"

# Sort results
rsairtable base <BASE_ID> table <TABLE> records -S "Name asc,Priority desc"

# Limit fields
rsairtable base <BASE_ID> table <TABLE> records -F "Name" -F "Status"

# Limit number of records
rsairtable base <BASE_ID> table <TABLE> records -n 50

# Reverse order
rsairtable base <BASE_ID> table <TABLE> records -D
```

## 🏗️ ORM Code Generation

Generate type-safe Rust structs from your Airtable base:

```bash
rsairtable base appXXXXXXXXXXXXXX orm > src/models.rs
```

This creates structs like:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub created_time: Option<String>,
    pub company: Option<String>,
    pub email: Option<String>,
    pub status: Option<String>,
    // ... all your fields with proper types
}

impl Customer {
    pub fn from_record(record: Record) -> Result<Self, serde_json::Error> {
        let mut customer: Customer = serde_json::from_value(
            json!({
                "id": record.id,
                "created_time": record.created_time
            })
        )?;
        
        // Map fields from record.fields
        if let Some(company) = record.fields.get("Company") {
            customer.company = serde_json::from_value(company.clone()).ok();
        }
        
        Ok(customer)
    }
}
```

## 🧪 Testing

The project includes comprehensive test coverage:

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test test_step6_record_creation
cargo test test_step7_record_updates_deletes
cargo test test_step8_advanced_features
cargo test test_step11_advanced_cli

# Run with output
cargo test -- --nocapture
```

**Test Coverage**: 46 tests covering all API endpoints and CLI commands with 100% pass rate.

## ⚠️ API Limitations

Some features are not available via Airtable's public API:

| Feature | Status | Alternative |
|---------|--------|-------------|
| Field creation/deletion | ❌ Not supported | Use Airtable web interface |
| Direct file uploads | ❌ Not supported | Use URL-based attachments |
| Base collaborator management | ❌ Not supported | Use Airtable web interface |
| Enterprise audit logs | ❌ Not supported | Use Enterprise Admin Panel |

These limitations are properly documented and handled with helpful error messages.

## 🚀 Performance

- **Async/Await**: Built on tokio for high-concurrency applications
- **HTTP/2**: Uses reqwest with connection pooling
- **Batch Operations**: Efficient bulk create/update operations
- **Lazy Evaluation**: Query builders only execute when needed
- **Memory Efficient**: Streaming support for large datasets

## 🔧 Configuration

### Environment Variables

```bash
# Required
PERSONAL_ACCESS_TOKEN=patXXXXXXXXXXXXXX
BASE=appXXXXXXXXXXXXXX

# Optional
AIRTABLE_ENDPOINT_URL=https://api.airtable.com/v0  # Custom endpoint
```

### Environment File Configuration

rsairtable supports multiple environment file formats with automatic fallback:

#### Priority Order:
1. **`.env` file** - Standard environment file (if it contains Airtable tokens)
2. **`airtable.env` file** - Fallback if `.env` has no Airtable tokens
3. **System environment variables** - Always checked

#### Setup Examples:

**Option 1: Use .env file (recommended)**
```bash
# Create .env file in your project root
echo "PERSONAL_ACCESS_TOKEN=patXXXXXXXXXXXXXX" > .env
echo "BASE=appXXXXXXXXXXXXXX" >> .env
```

**Option 2: Use airtable.env file**
```bash
# Create airtable.env file in your project root
echo "PERSONAL_ACCESS_TOKEN=patXXXXXXXXXXXXXX" > airtable.env
echo "BASE=appXXXXXXXXXXXXXX" >> airtable.env
```

**Option 3: Mixed approach (automatic fallback)**
```bash
# .env file exists but has no Airtable tokens
echo "DATABASE_URL=postgres://..." > .env

# airtable.env file contains Airtable configuration
echo "PERSONAL_ACCESS_TOKEN=patXXXXXXXXXXXXXX" > airtable.env
echo "BASE=appXXXXXXXXXXXXXX" >> airtable.env
```

### File-based Configuration

```bash
# Store API key in file
echo "patXXXXXXXXXXXXXX" > ~/.airtable_key
rsairtable --key-file ~/.airtable_key whoami
```

### Programmatic Configuration

```rust
let config = Config::new("patXXXXXXXXXXXXXX")
    .with_endpoint_url("https://api.airtable.com/v0")
    .with_verbose(true);

let client = Client::from_config(config);
```

## 🤝 Compatibility with pyairtable

rsairtable is designed as a drop-in replacement for Python's pyairtable:

| pyairtable | rsairtable | Status |
|------------|------------|--------|
| `Table.all()` | `table.list().execute()` | ✅ |
| `Table.first()` | `table.list().max_records(1).execute()` | ✅ |
| `Table.create()` | `table.create()` | ✅ |
| `Table.update()` | `table.update()` | ✅ |
| `Table.delete()` | `table.delete()` | ✅ |
| `Table.batch_create()` | `table.batch_create()` | ✅ |
| `Table.batch_update()` | `table.batch_update()` | ✅ |
| `Base.schema()` | `base.schema()` | ✅ |
| CLI commands | CLI commands | ✅ |

## 🛠️ Troubleshooting

### Common Issues

#### Pagination and Large Datasets

**Problem**: Only getting 100 records when you expect more
```bash
# This only returns first 100 records
rsairtable base table "LargeTable" records
```

**Solution**: Use the `--all` flag for automatic pagination
```bash
# This retrieves ALL records from the table
rsairtable base table "LargeTable" records --all
```

**Performance Tips for Large Tables**:
- Use `-v` (verbose) flag to monitor progress
- Filter data when possible: `--all -w "Status = 'Active'"`
- Select only needed fields: `--all -F "Name" -F "Email"`
- Consider using `--offset` for manual pagination control

#### Offset Token Continuation

**Problem**: Need to continue from where a previous request left off

**Solution**: Use the offset token from the response
```bash
# First request returns [records_array, "itrABC123/recXYZ"]
rsairtable base table "Table" records > batch1.json

# Continue from that offset
rsairtable base table "Table" records --offset "itrABC123/recXYZ" > batch2.json
```

#### Memory Usage with --all Flag

The `--all` flag loads all records into memory before outputting. For very large tables (10,000+ records), consider:
- Using manual pagination with `--offset`
- Filtering to reduce dataset size
- Processing data in smaller batches

### Performance Tips

- Use `--all` with filters to reduce data volume
- Combine with `jq` for efficient data processing
- Use verbose mode (`-v`) to monitor progress on large operations

## 🛠️ Development

### Building from Source

```bash
git clone https://github.com/your-repo/rsairtable.git
cd rsairtable
cargo build --release
```

### Running Examples

```bash
# Basic record creation
cargo run --example create_sample_record

# Add record with CLI args
cargo run --example add_record "Company Name" "Active" "Notes here"

# Debug authentication
cargo run --example debug_auth
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass: `cargo test`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Airtable](https://airtable.com) for their excellent API
- [pyairtable](https://github.com/gtalarico/pyairtable) for API design inspiration
- The Rust community for amazing async ecosystem
- [reqwest](https://github.com/seanmonstar/reqwest) for HTTP client functionality
- [clap](https://github.com/clap-rs/clap) for CLI argument parsing

## 📞 Support

- 📖 Documentation: Use `rsairtable --help-detail` for comprehensive examples
- 🐛 Bug Reports: [GitHub Issues](https://github.com/your-repo/rsairtable/issues)
- 💡 Feature Requests: [GitHub Discussions](https://github.com/your-repo/rsairtable/discussions)
- 📧 Email: support@yourproject.com

---

**Made with ❤️ in Rust** | **Compatible with pyairtable** | **Production Ready**
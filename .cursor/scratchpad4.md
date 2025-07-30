# RSAirtable Project - Rust Airtable Client

## Background and Motivation

The user wants to create a new Rust project called `rsairtable` as a peer to the existing `pyairtable` project. This will be an independent Rust implementation of an Airtable API client library.

**Project Structure:**
```
~/code/rust/airtable/
├── pyairtable/     (existing Python library)
└── rsairtable/     (new Rust library - to be created)
```

**Goals:**
1. Create a new, independent Rust project for Airtable API interaction
2. Implement similar functionality to pyairtable but in Rust
3. Use modern Rust practices and ecosystem tools
4. Start with basic API connectivity and record retrieval

**Reference Implementation:** The working Python implementation that successfully connects to:
- Base ID: `appS0LhkvZkx6CCOQ`
- Table: `Clio`  
- Personal Access Token: Available in .env file

## Key Challenges and Analysis

### Technical Requirements for Rust Implementation:

1. **HTTP Client**: Need async HTTP client (likely `reqwest` crate)
2. **JSON Handling**: Serde for serialization/deserialization
3. **Error Handling**: Robust error types using `thiserror` or similar
4. **Authentication**: Bearer token authentication
5. **Rate Limiting**: Airtable has 5 QPS limit per base
6. **Environment Variables**: `.env` file support with `dotenv`

### Architecture Considerations:

1. **Client Structure**: Similar to pyairtable's Api -> Base -> Table hierarchy
2. **Async/Await**: Modern Rust async patterns with `tokio`
3. **Type Safety**: Strong typing for records, fields, and responses
4. **Testing**: Unit tests and integration tests with real API
5. **Documentation**: Comprehensive docs with examples

### Reference API Patterns:
From pyairtable success:
- Base ID: `appS0LhkvZkx6CCOQ`
- Table: `Clio`
- Endpoint: `https://api.airtable.com/v0/{base_id}/{table_name}`
- Auth: `Authorization: Bearer {token}`

## High-level Task Breakdown

### Phase 1: Project Setup and Foundation (15-30 mins)
1. **Initialize Rust project structure**
   - Create new Cargo project at `~/code/rust/airtable/rsairtable`
   - Set up basic directory structure and Cargo.toml
   - Success criteria: `cargo check` passes on empty project

2. **Configure dependencies**
   - Add core crates: `tokio`, `reqwest`, `serde`, `serde_json`, `dotenv`, `thiserror`
   - Set up async runtime and feature flags
   - Success criteria: All dependencies compile successfully

3. **Create basic project structure**
   - Set up `src/lib.rs`, `src/client.rs`, `src/error.rs`
   - Define module structure and public API surface
   - Success criteria: Project compiles with basic structure

### Phase 2: Systematic API Endpoint Implementation 
**IMPORTANT**: Use `openapi.yaml` as the primary reference. This file contains the complete API specification derived from pyairtable. Only consult the actual pyairtable source code if clarification is needed.

4. **Setup Foundation (15 mins)**
   - Implement configuration management (CLI + env vars)
   - Create basic HTTP client with bearer token auth
   - Define core data models for RecordDict, responses
   - Success criteria: Basic client can authenticate and make requests

5. **Record Retrieval Endpoints (30 mins)**
   - **5a. `get(record_id)` method** - Single record by ID
     - Primary Reference: `openapi.yaml` - `GET /{baseId}/{tableIdOrName}/{recordId}`
     - Fallback Reference: `table.py:233-252` 
     - Success: Can retrieve single record matching OpenAPI specification
   - **5b. `all(**options)` method** - All records with filtering
     - Primary Reference: `openapi.yaml` - `GET /{baseId}/{tableIdOrName}`
     - Fallback Reference: `table.py:294-316`
     - Success: Can retrieve records with same query options as specified
   - **5c. `iterate(**options)` method** - Paginated record iteration  
     - Primary Reference: `openapi.yaml` - pagination parameters and offset handling
     - Fallback Reference: `table.py:254-292`
     - Success: Handles pagination identically to specification
   - **5d. `first(**options)` method** - First matching record
     - Primary Reference: `openapi.yaml` - `maxRecords=1` parameter usage
     - Fallback Reference: `table.py:318-340`
     - Success: Returns same first record as specified

6. **Record Creation Endpoints (20 mins)**
   - **6a. `create(fields)` method** - Single record creation
     - Primary Reference: `openapi.yaml` - `POST /{baseId}/{tableIdOrName}`
     - Fallback Reference: `table.py:341-370`
     - Success: Creates record with same response format as specified
   - **6b. `batch_create(records)` method** - Multiple record creation
     - Primary Reference: `openapi.yaml` - batch creation request schema
     - Fallback Reference: `table.py:371-419` 
     - Success: Handles batch creation per OpenAPI specification

7. **Record Update/Delete Endpoints (25 mins)**
   - **7a. `update(record_id, fields)` method** - Single record update
     - Primary Reference: `openapi.yaml` - `PATCH /{baseId}/{tableIdOrName}/{recordId}`
     - Fallback Reference: `table.py:420-456`
     - Success: Updates record with same behavior as specified
   - **7b. `batch_update(records)` method** - Multiple record updates
     - Primary Reference: `openapi.yaml` - `PATCH /{baseId}/{tableIdOrName}`
     - Fallback Reference: `table.py:457-498`
     - Success: Handles batch updates per specification
   - **7c. `batch_upsert(records)` method** - Create or update records
     - Primary Reference: `openapi.yaml` - upsert operation details
     - Fallback Reference: `table.py:499-571`  
     - Success: Upsert logic matches OpenAPI specification
   - **7d. `delete(record_id)` method** - Single record deletion
     - Primary Reference: `openapi.yaml` - `DELETE /{baseId}/{tableIdOrName}/{recordId}`
     - Fallback Reference: `table.py:572-589`
     - Success: Deletes record per specified response format
   - **7e. `batch_delete(record_ids)` method** - Multiple record deletion
     - Primary Reference: `openapi.yaml` - `DELETE /{baseId}/{tableIdOrName}`
     - Fallback Reference: `table.py:590-616`
     - Success: Handles batch deletion per specification

8. **Advanced Features (30 mins)**
   - **8a. Comments API** - `comments()` and `add_comment()` methods
     - Primary Reference: `openapi.yaml` - comment endpoints and schemas
     - Fallback Reference: `table.py:617-682`
     - Success: Can retrieve and add comments per specification
   - **8b. Schema Operations** - `schema()` and `create_field()` methods  
     - Primary Reference: `openapi.yaml` - schema endpoints and field schemas
     - Fallback Reference: `table.py:683-750`
     - Success: Can retrieve table schema and create fields per specification
   - **8c. File Attachments** - `upload_attachment()` method
     - Primary Reference: `openapi.yaml` - attachment upload endpoint
     - Fallback Reference: `table.py:751-818`
     - Success: Can upload files per specification

### Phase 3: CLI Interface Implementation (30-45 mins)
9. **CLI Foundation Setup**
   - Set up clap-based CLI structure matching pyairtable's interface
   - Implement global options: `-k/--key`, `-kf/--key-file`, `-ke/--key-env`, `-v/--verbose`
   - Create subcommand structure: `whoami`, `bases`, `base`, `enterprise`
   - Success criteria: `rsairtable --help` shows structure matching pyairtable

10. **Core CLI Commands Implementation**
    - **10a. `whoami` command** - Print current user information
      - Reference: pyairtable CLI `whoami`
      - Success: Same output format as pyairtable
    - **10b. `bases` command** - List all available bases
      - Reference: pyairtable CLI `bases`
      - Success: Same base listing functionality
    - **10c. `base <ID> table <NAME> records` command** - Retrieve records
      - Reference: pyairtable CLI with options `-f/--formula`, `-v/--view`, `-n/--limit`, `-S/--sort`, `-F/--field`
      - Success: All query options work identically to pyairtable
    - **10d. `base <ID> table <NAME> schema` command** - Print table schema
      - Reference: pyairtable CLI schema output
      - Success: Same schema information display

11. **Advanced CLI Commands**
    - **11a. `base <ID> schema` command** - Print base schema
    - **11b. `base <ID> collaborators` command** - Print base collaborators  
    - **11c. `base <ID> shares` command** - Print base shares
    - **11d. `base <ID> orm` command** - Generate Rust structs (equivalent to Python ORM)
    - Success criteria: Feature parity with pyairtable CLI

### Phase 4: Testing and Validation (15-30 mins)
12. **Comprehensive Integration Testing**
    - Test each API endpoint against real API using same credentials
    - Cross-validate all responses with pyairtable output
    - Test all CLI commands against pyairtable CLI output
    - Test all configuration loading methods
    - Success criteria: 100% functional compatibility with pyairtable

13. **Documentation and Examples**
    - Add comprehensive documentation with pyairtable comparison
    - Include CLI usage examples matching pyairtable patterns
    - Create migration guide from pyairtable to rsairtable
    - Success criteria: Complete feature and CLI parity demonstrated

## Project Status Board

### Phase 1: Foundation ✅ COMPLETED
- [x] Initialize Rust project structure
- [x] Configure dependencies  
- [x] Create basic project structure

### Phase 2: Systematic Endpoint Implementation
- [x] Setup Foundation (config + auth + models) ✅ COMPLETED
- [ ] Record Retrieval: get(), all(), iterate(), first()
- [ ] Record Creation: create(), batch_create() 
- [ ] Record Updates: update(), batch_update(), batch_upsert()
- [ ] Record Deletion: delete(), batch_delete()
- [ ] Advanced Features: comments, schema, attachments

### Phase 3: CLI Interface Implementation  
- [ ] CLI Foundation Setup (clap structure + global options)
- [ ] Core CLI Commands: whoami, bases, records, schema
- [ ] Advanced CLI Commands: collaborators, shares, orm generation

### Phase 4: Testing & Validation
- [ ] Comprehensive integration testing vs pyairtable
- [ ] CLI command testing vs pyairtable CLI
- [ ] Documentation and migration guide

## Current Status / Progress Tracking

- Project planning started: Wed Jul 30 14:28:47 MDT 2025  
- **Phase 1 completed**: Wed Jul 30 14:55:00 MDT 2025 ✅
- **Step 4 completed**: Wed Jul 30 15:56:00 MDT 2025 ✅
- Target location: `~/code/rust/airtable/rsairtable/` ✅ CREATED
- Reference implementation: pyairtable (proven working) ✅
- Available resources: Working .env with valid token, test data from Clio table ✅

### Phase 1 Achievements:
- ✅ Created complete Rust project structure with `cargo new rsairtable --lib`
- ✅ Configured 10+ dependencies (tokio, reqwest, serde, clap, chrono, etc.)
- ✅ Implemented core modules: `lib.rs`, `client.rs`, `error.rs`, `config.rs`, `models.rs`, `cli.rs`
- ✅ Built comprehensive error handling with `thiserror`
- ✅ Created flexible configuration system (env vars, .env file, CLI args)
- ✅ Implemented pyairtable-compatible CLI interface with clap
- ✅ Successfully compiled both library and CLI binary
- ✅ Verified CLI help system matches pyairtable structure
- ✅ Project ready for Phase 2 endpoint implementation

### Step 4 Achievements (Setup Foundation):
- ✅ **Configuration Management**: Complete priority-based config system (CLI > env vars > .env file)
- ✅ **HTTP Client with Authentication**: Bearer token auth, proper headers, timeout handling
- ✅ **Core Data Models**: Record, Fields, ListRecordsResponse, CreateRecordRequest, all API models
- ✅ **Error Handling**: Comprehensive error types for all API scenarios (auth, rate limit, etc.)
- ✅ **URL Construction**: Base and table URL patterns for API endpoints
- ✅ **Environment Integration**: .env file loading, multiple token env var support
- ✅ **Library Tests**: All foundation tests passing (client creation, base/table handles)
- ✅ **CLI Integration**: Working CLI with help system and token detection
- ✅ **API Integration Tests**: Real API calls passing (whoami, bases, base_schema endpoints)
- ✅ **Authentication Verified**: Bearer token auth working with Airtable API
- ✅ **Error Handling Tested**: Proper 401 handling for invalid tokens
- ✅ **Ready for API calls**: Foundation can authenticate and construct requests

## Proposed Rust API Design

Based on pyairtable's successful pattern, with flexible configuration for Base ID and token:

```rust
use rsairtable::Client;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // Token priority: CLI arg > env var > .env file
    let token = env::var("PERSONAL_ACCESS_TOKEN")
        .or_else(|_| env::var("AIRTABLE_API_KEY"))
        .expect("Personal access token required");
    
    // Base ID priority: CLI arg > env var > .env file  
    let base_id = env::var("AIRTABLE_BASE_ID")
        .unwrap_or_else(|_| "appS0LhkvZkx6CCOQ".to_string());
    
    // Table name priority: CLI arg > env var > .env file
    let table_name = env::var("AIRTABLE_TABLE_NAME")
        .unwrap_or_else(|_| "Clio".to_string());
    
    let client = Client::new(token);
    
    let records = client
        .base(&base_id)
        .table(&table_name)
        .list()
        .max_records(10)
        .execute()
        .await?;
    
    println!("Retrieved {} records from base {}", records.len(), base_id);
    Ok(())
}
```

## Proposed CLI Interface Design

**Modeled after pyairtable CLI** with identical command structure and options:

### Global Options (match pyairtable exactly):
```bash
rsairtable [OPTIONS] COMMAND [ARGS]...

Options:
  -k, --key TEXT        Your API key/token
  -kf, --key-file PATH  File containing your API key
  -ke, --key-env VAR    Env var containing your API key  
  -v, --verbose         Print verbose output
  --help                Show this message and exit
```

### Core Commands (replicate pyairtable functionality):
```bash
# User information
rsairtable whoami

# Base operations  
rsairtable bases
rsairtable base <BASE_ID> schema
rsairtable base <BASE_ID> collaborators
rsairtable base <BASE_ID> shares

# Table operations
rsairtable base <BASE_ID> table <TABLE_NAME> records [OPTIONS]
rsairtable base <BASE_ID> table <TABLE_NAME> schema

# Record retrieval options (match pyairtable CLI exactly)
Options for 'records' command:
  -f, --formula TEXT   Filter records with a formula
  -v, --view TEXT      Filter records by a view
  -n, --limit INTEGER  Limit the number of records returned
  -S, --sort TEXT      Sort records by field(s)  
  -F, --field TEXT     Limit output to certain field(s)
```

### Example Usage (identical to pyairtable patterns):
```bash
# Basic usage
rsairtable -ke PERSONAL_ACCESS_TOKEN whoami
rsairtable -ke PERSONAL_ACCESS_TOKEN bases

# Record retrieval (exact pyairtable CLI syntax)
rsairtable -ke PERSONAL_ACCESS_TOKEN base appS0LhkvZkx6CCOQ table Clio records
rsairtable -ke PERSONAL_ACCESS_TOKEN base appS0LhkvZkx6CCOQ table Clio records -n 10
rsairtable -ke PERSONAL_ACCESS_TOKEN base appS0LhkvZkx6CCOQ table Clio records -f "{Age} > 21" -S Name
```

## Key Dependencies Required

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
reqwest = { version = "0.11", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"
dotenv = "0.15"
uuid = { version = "1.0", features = ["v4"] }
url = "2.0"
clap = { version = "4.0", features = ["derive"] }  # For CLI argument parsing
```

## Configuration Strategy

### Priority Order for All Parameters:
1. **CLI Arguments**: `--base-id`, `--token`, `--table` flags (highest priority)
2. **Environment Variables**: `AIRTABLE_BASE_ID`, `PERSONAL_ACCESS_TOKEN`, `AIRTABLE_TABLE_NAME`
3. **Environment Variables (alternative)**: `AIRTABLE_API_KEY` (for token)
4. **Default/Fallback**: Use test values for development (base ID only)
5. **Required**: Table name must be provided via CLI or env var

### Environment Variable Support:
```bash
# .env file or environment
PERSONAL_ACCESS_TOKEN=patsWYSSQb3B99QAu.56fc9eda5789da400d8f51d457d0375ddc9051ba15156c6a92b0ab52da63d343
AIRTABLE_BASE_ID=appS0LhkvZkx6CCOQ
AIRTABLE_TABLE_NAME=Clio
```

### CLI Usage Examples:
```bash
# Full explicit configuration
rsairtable --base-id appS0LhkvZkx6CCOQ --token pat123... --table Clio --max-records 10

# Using environment variables (table name can come from env)
rsairtable --max-records 10  # uses all env vars including AIRTABLE_TABLE_NAME

# Mixed approach - table name via CLI
rsairtable --table Clio --max-records 10  # base-id and token from env

# Table name is REQUIRED
rsairtable --base-id appXYZ --token pat123...  # ERROR: missing table name
```

## Implementation Strategy

### Development Approach:
1. **OpenAPI-First Development**: For each endpoint, study the `openapi.yaml` specification BEFORE writing Rust code
2. **Endpoint-by-Endpoint**: Complete one endpoint fully (including tests) before moving to the next
3. **Iterative Plan Updates**: After each endpoint, update the plan based on discoveries and learnings
4. **Reference Validation**: Every Rust endpoint must produce identical output to the OpenAPI specification
5. **Fallback Code Study**: Only consult pyairtable source code when the OpenAPI spec needs clarification

### Endpoint Implementation Process:
For each endpoint (e.g., `get()`, `create()`, etc.):
1. **Study**: Read the `openapi.yaml` specification thoroughly for that endpoint
2. **Plan**: Create detailed Rust implementation plan based on the OpenAPI specification
3. **Implement**: Write Rust code that matches the OpenAPI specification exactly
4. **Test**: Validate against real API and compare with specification requirements
5. **Clarify**: If confusion arises, consult the pyairtable source code (table.py lines X-Y)
6. **Document**: Record any discoveries or deviations in the plan
7. **Iterate**: Update plan based on learnings before proceeding to next endpoint

### Success Metrics per Endpoint:
- [ ] Identical API behavior to OpenAPI specification
- [ ] Same response format and error handling as specified  
- [ ] Same query parameter support as documented
- [ ] Cross-validated with real API calls
- [ ] Documented with OpenAPI specification examples

## OpenAPI Specification

✅ **Created**: `openapi.yaml` - Comprehensive API specification based on pyairtable implementation

**Key Features:**
- **15 Endpoints** mapped directly from pyairtable Table class
- **Line-by-Line References** to pyairtable source code (table.py:XXX-YYY)
- **Complete Request/Response Schemas** with validation patterns
- **Parameter Documentation** including all query options and formats
- **Error Response Handling** with proper HTTP status codes
- **Authentication Specification** with Bearer token format

**Primary Endpoints Covered:**
1. `GET /{baseId}/{tableIdOrName}` - List records (all/iterate)
2. `POST /{baseId}/{tableIdOrName}` - Create record
3. `GET /{baseId}/{tableIdOrName}/{recordId}` - Get single record
4. `PATCH /{baseId}/{tableIdOrName}/{recordId}` - Update record
5. `PUT /{baseId}/{tableIdOrName}/{recordId}` - Replace record
6. `DELETE /{baseId}/{tableIdOrName}/{recordId}` - Delete record
7. `PATCH /{baseId}/{tableIdOrName}` - Batch update
8. `DELETE /{baseId}/{tableIdOrName}` - Batch delete
9. `POST /{baseId}/{tableIdOrName}/listRecords` - List records (POST fallback)
10. Comments, Schema, and Field endpoints

**Usage for Rust Implementation:**
- Reference for exact URL patterns and HTTP methods
- Request/response structure validation
- Parameter handling and validation rules
- Error response format standardization

## Executor's Feedback or Assistance Requests

**Step 4 Test Planning**: Creating comprehensive tests for Setup Foundation step before implementation.

### Test Plan for Step 4: Setup Foundation (config + auth + models)

**Components to Test:**
1. **Configuration Management**
   - CLI argument parsing
   - Environment variable loading (.env file and system env vars)
   - Priority order: CLI > Env Vars > .env file
   - Required vs optional parameters
   - Error handling for missing required values

2. **HTTP Client with Authentication**
   - Bearer token authentication header formatting
   - Base URL construction
   - HTTP client initialization
   - Request timeout handling
   - SSL/TLS configuration

3. **Core Data Models**
   - RecordDict serialization/deserialization
   - API response parsing
   - Error response handling
   - Field type validation

**Test Categories:**
- **Unit Tests**: Individual component behavior
- **Integration Tests**: Component interaction
- **Error Handling Tests**: Invalid input scenarios
- **Configuration Tests**: All config loading methods

### Step 4 Tests Created ✅

**Test Files Created:**
1. **`tests/test_config.rs`** (159 lines) - Configuration management tests
   - Environment variable loading (.env file and system env vars)
   - Priority order validation (CLI > Env Vars > .env file)
   - Required vs optional parameter handling
   - Error handling for missing required values
   - URL construction and validation
   - Bearer token formatting

2. **`tests/test_client.rs`** (183 lines) - HTTP client and authentication tests
   - Client initialization and configuration
   - Bearer token authentication header formatting
   - Base URL and table URL construction
   - Timeout and retry configuration
   - Rate limiting configuration (5 QPS)
   - Request builder creation
   - Error handling for invalid configurations

3. **`tests/test_models.rs`** (211 lines) - Data model tests
   - Record serialization/deserialization
   - RecordDict creation and manipulation
   - Field type validation (Text, Number, Checkbox, Date, Email, URL)
   - AirtableResponse parsing
   - Error response handling
   - Complex field values (arrays, objects, null handling)
   - Record validation (ID format checking)

4. **`tests/integration_test_foundation.rs`** (233 lines) - End-to-end integration tests
   - Complete configuration-to-client flow
   - Priority order validation across all components
   - Error propagation through foundation layers
   - Record model integration with client URLs
   - HTTP request preparation
   - Timeout and retry configuration integration

**Test Coverage:**
- ✅ Configuration management (CLI + env vars + .env files)
- ✅ HTTP client with bearer token authentication
- ✅ Core data models (Record, Field, AirtableResponse)
- ✅ URL construction and validation
- ✅ Error handling and propagation
- ✅ Integration between all foundation components
- ✅ Rate limiting and timeout configuration

**Additional Dependencies Required:**
- `tempfile = "3.8"` (dev-dependency for .env file testing)

**PROPERLY FAILING TESTS CREATED ✅**: 

The tests are now properly set up for Test-Driven Development (TDD):

1. **Project Structure**: Created minimal Rust project with `cargo init --lib`
2. **Dependencies**: Added all required dependencies in `Cargo.toml`
3. **Failing Module Imports**: Tests import modules that don't exist yet:
   ```rust
   use rsairtable::config::Config;      // ❌ FAILS - not implemented
   use rsairtable::client::Client;      // ❌ FAILS - not implemented  
   use rsairtable::models::Record;      // ❌ FAILS - not implemented
   use rsairtable::error::AirtableError; // ❌ FAILS - not implemented
   ```

4. **Current Compilation Errors**:
   ```
   error[E0432]: unresolved import `client::Client`
   error[E0432]: unresolved import `config::Config`
   ```

**This is EXACTLY what we want for TDD**: Tests fail because the code doesn't exist yet.

**Next Step**: Implement the actual modules to make the tests pass, starting with the foundation components defined by the test requirements.

## Lessons from Python Implementation

- Personal access tokens work reliably with Bearer auth
- Airtable API returns large datasets (10k+ records) successfully
- Rate limiting (5 QPS) needs to be considered for production use
- Environment variable approach with .env files is user-friendly
- JSON structure is complex with nested fields and metadata
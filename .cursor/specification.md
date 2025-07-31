# RSAirtable Pagination Enhancement - Technical Specification

## Overview

This specification defines the requirements for adding pagination support to the RSAirtable CLI, enabling users to retrieve all records from any Airtable table, not just the first 100.

## Functional Requirements

### FR1: Offset Parameter Support
**Requirement**: The CLI shall accept an `--offset` parameter for manual pagination control.

**Behavior**:
- Parameter: `--offset <OFFSET_TOKEN>`
- When provided, the CLI uses the offset token in the API request
- When not provided, starts from the beginning (existing behavior)
- The offset token is an opaque string returned by Airtable API

**Test Cases**:
```bash
# Should start from beginning (existing behavior)
cargo run -- base table TestTable records

# Should start from specified offset
cargo run -- base table TestTable records --offset "itrABC123/recDEF456"

# Should work with filters
cargo run -- base table TestTable records --offset "token" -F "Name"
```

### FR2: Automatic Pagination Flag
**Requirement**: The CLI shall provide an `--all` flag to automatically retrieve all records.

**Behavior**:
- Parameter: `--all` (boolean flag)
- Automatically handles pagination until all records are retrieved
- Combines results from multiple API requests into single output
- Maintains same output format as single-page requests
- Works with all existing filters and parameters

**Test Cases**:
```bash
# Should get all records automatically
cargo run -- base table TestTable records --all

# Should work with field filtering
cargo run -- base table TestTable records --all -F "Name" -F "Status"

# Should work with formulas
cargo run -- base table TestTable records --all -w "Status = 'Active'"
```

### FR3: Pagination Status Reporting
**Requirement**: The CLI shall provide progress information during pagination operations.

**Behavior**:
- Default: Silent operation (existing behavior)
- Verbose mode (`-v`): Shows pagination progress
- Always shows final record count when complete
- Indicates if more records are available (when not using `--all`)

**Test Cases**:
```bash
# Verbose mode shows progress
cargo run -- base table TestTable records --all -v
# Expected output: "Retrieved 100 records, fetching more..." → "Retrieved 250 records total"

# Non-all mode indicates more available
cargo run -- base table TestTable records -v
# Expected output: "Retrieved 100 records (more available, use --all or --offset)"
```

## Non-Functional Requirements

### NFR1: Performance
- Memory usage shall remain constant regardless of dataset size when using `--all`
- Network requests shall be made sequentially to avoid rate limiting
- Response time for first 100 records shall not be impacted

### NFR2: Backward Compatibility
- Existing CLI usage shall continue to work unchanged
- Default behavior (no new flags) shall retrieve first 100 records as before
- Output format shall remain identical for single-page requests

### NFR3: Error Handling
- Network failures during pagination shall be reported with the current offset
- Invalid offset tokens shall return appropriate error messages
- Partial results shall be preserved when possible

## API Integration Requirements

### AIR1: Offset Token Handling
**Current API Response Structure**:
```json
[
  [record1, record2, ...],  // Records array (max 100)
  "offset_token_string"     // Null if no more records
]
```

**Required Integration**:
- Extract offset token from `response[1]`
- Pass offset token in subsequent requests
- Stop pagination when offset token is `null`

### AIR2: Request Parameters
**Current Request**: Uses library's `list()` method with various parameters

**Required Enhancement**:
- Add offset parameter to underlying library calls
- Preserve all existing parameters (formula, view, fields, sort, etc.)
- Combine results from multiple requests appropriately

## Implementation Requirements

### IR1: CLI Argument Structure
**New Parameters**:
```
--offset <OFFSET_TOKEN>   Continue from specified offset
--all                     Retrieve all records automatically
```

**Parameter Validation**:
- `--offset` and `--all` are mutually exclusive
- `--offset` requires a non-empty string value
- `--all` is a boolean flag (no value)

### IR2: Code Organization
**Files to Modify**:
- `src/cli.rs`: Add new parameters to argument parser
- `src/cli.rs`: Implement pagination logic in `run_command()`

**Design Patterns**:
- Use SOLID principles for pagination logic
- Extract pagination into separate function/module if complex
- Maintain single responsibility for each function

### IR3: Data Flow
**Single Page Request** (existing):
```
CLI Args → API Request → Response → JSON Output
```

**Paginated Request** (new):
```
CLI Args → Loop(API Request → Accumulate Response) → Combined JSON Output
```

## Test Requirements

### TR1: Unit Tests
- Test offset parameter parsing and validation
- Test pagination logic with mock API responses
- Test error handling for invalid offsets

### TR2: Integration Tests
**Test Cases**:
1. **Basic Pagination**: Verify `--all` retrieves more than 100 records
2. **Offset Continuation**: Verify `--offset` starts from correct position
3. **Parameter Combinations**: Test `--all` with filters, fields, sorting
4. **Error Scenarios**: Invalid offset, network failures, empty tables
5. **Large Datasets**: Test with 1000+ records for performance

### TR3: Compatibility Tests
- Verify existing CLI usage continues to work
- Verify output format remains unchanged for single-page requests
- Test with various table schemas and data types

## Documentation Requirements

### DR1: CLI Help Updates
**Commands to Update**:
```bash
cargo run -- base table <TABLE> records --help
cargo run -- --help-detail
```

**Required Content**:
- Description of `--offset` and `--all` parameters
- Examples of pagination usage
- Performance considerations for large datasets

### DR2: README Updates
**Sections to Add**:
- Pagination section in CLI reference
- Examples of retrieving all records
- Performance notes and best practices

**Example Content**:
```bash
# Get all records from a table
rsairtable base table Customers records --all

# Continue from specific offset
rsairtable base table Customers records --offset "itrXYZ123/recABC456"

# Get all active customers with specific fields
rsairtable base table Customers records --all -w "Status='Active'" -F "Name" -F "Email"
```

## Success Criteria

### SC1: Functional Success
- [ ] CLI accepts `--offset` parameter and uses it correctly
- [ ] CLI accepts `--all` parameter and retrieves all records
- [ ] Pagination works with all existing filters and parameters
- [ ] Output format matches existing single-page format

### SC2: Quality Success
- [ ] All unit tests pass
- [ ] Integration tests verify end-to-end functionality
- [ ] Memory usage remains constant during large dataset retrieval
- [ ] Documentation is complete and accurate

### SC3: User Experience Success
- [ ] Existing usage patterns continue to work unchanged
- [ ] New functionality is discoverable through help system
- [ ] Error messages are clear and actionable
- [ ] Performance is acceptable for typical use cases

## Implementation Notes

### IN1: Library Layer Requirements
The underlying rsairtable library must support:
- Passing offset tokens to the `list()` method
- Returning offset tokens in responses
- Maintaining compatibility with existing parameter combinations

### IN2: Memory Management Strategy
For `--all` flag implementation:
- Stream results to output rather than accumulating in memory
- Process records in batches to maintain constant memory usage
- Use iterative approach rather than recursive for pagination

### IN3: Error Recovery Strategy
- Preserve partial results when network errors occur
- Provide offset token in error messages for manual continuation
- Implement retry logic for transient network failures (optional enhancement)
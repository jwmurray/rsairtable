# RSAirtable Enhancement - View-Based Data Extraction

## Project Overview

**Goal**: Add a `--view` option with specialized data views, starting with a "clio" view that extracts and formats Matter records with Clio-specific fields.

**User Request**: Create a single Rust command equivalent to:
```bash
rsairtable base table Matters records --all -F "Clio Matter ID" | jq -r '.[0] | map(select(.fields."Clio Matter ID" != null)) | map({(.id): .fields."Clio Matter ID"}) | add'
```

**Required Output Fields for Clio View**:
- Matter Title
- Record ID  
- Clio Matter ID
- Clio Matter Url
- Clio Drive Folder
- Open in Google drive (from Clio Drive Folder)

## Architecture Analysis (SOLID Principles)

### Single Responsibility Principle (SRP)
- **ViewProcessor**: Handle view-specific logic and formatting
- **FieldExtractor**: Extract and validate specific fields from records
- **OutputFormatter**: Format data according to view requirements

### Open/Closed Principle (OCP)
- Design view system to be extensible for future views ("billing", "client", etc.)
- Use trait-based approach for view processors

### Liskov Substitution Principle (LSP)
- All view implementations should be interchangeable through a common interface

### Interface Segregation Principle (ISP)
- Separate view processing from general record retrieval
- Create focused interfaces for different aspects (filtering, formatting, output)

### Dependency Inversion Principle (DIP)
- CLI should depend on view abstractions, not concrete implementations
- Allow for easy testing and extensibility

## High-Level Task Breakdown

### Task 1: Design View System Architecture (20-25 minutes)
- **Goal**: Create extensible view system with trait-based design
- **Success Criteria**:
  - Define `ViewProcessor` trait for pluggable views
  - Create view registry/factory pattern
  - Design clean separation between CLI args and view logic
- **Files**: `src/views.rs`, `src/cli.rs`

### Task 2: Implement Core View Infrastructure (25-30 minutes) 
- **Goal**: Add `--view` CLI option and view processing pipeline
- **Success Criteria**:
  - CLI accepts `--view <VIEW_NAME>` parameter
  - View system integrates with existing record retrieval
  - Maintains all existing filtering capabilities
- **Files**: `src/cli.rs`, `src/views.rs`

### Task 3: Create Clio View Implementation (30-35 minutes)
- **Goal**: Implement the "clio" view with required fields and filtering
- **Success Criteria**:
  - Filters out records without Clio Matter ID (null filtering)
  - Extracts all 6 required fields  
  - Formats output as structured JSON
  - Equivalent functionality to jq pipeline
- **Files**: `src/views/clio.rs`

### Task 4: Add Field Validation and Error Handling (15-20 minutes)
- **Goal**: Robust handling of missing fields and data validation
- **Success Criteria**:
  - Graceful handling of missing optional fields
  - Clear error messages for required fields
  - Validation that required fields exist in table schema
- **Files**: `src/views/clio.rs`, `src/error.rs`

### Task 5: Integration Testing and Validation (20-25 minutes)
- **Goal**: Comprehensive testing of view system
- **Success Criteria**:
  - Unit tests for view processor
  - Integration test comparing output to jq pipeline
  - Performance testing with large datasets
- **Files**: `tests/test_views.rs`

### Task 6: Documentation and Help Updates (15-20 minutes)
- **Goal**: Document new view system and clio view
- **Success Criteria**:
  - Updated CLI help with `--view` option
  - Detailed help for available views
  - README examples for clio view usage
- **Files**: `README.md`, `src/cli.rs`

## Technical Implementation Plan

### 1. CLI Integration Point
```rust
// Add to existing records command
#[arg(long, value_name = "VIEW_NAME", help = "Apply specialized view formatting")]
view: Option<String>,
```

### 2. View Trait Design
```rust
pub trait ViewProcessor {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn required_fields(&self) -> Vec<&'static str>;
    fn process_records(&self, records: Vec<Record>) -> Result<serde_json::Value, Error>;
    fn should_include_record(&self, record: &Record) -> bool;
}
```

### 3. Clio View Specification
- **Filter Logic**: `record.fields.get("Clio Matter ID").is_some()` and not empty
- **Output Format**: Array of objects with the 6 required fields
- **Field Mapping**: 
  - `Matter Title` → `matter_title`
  - `Record ID` → `record_id` (from `record.id`)
  - `Clio Matter ID` → `clio_matter_id`
  - `Clio Matter Url` → `clio_matter_url`
  - `Clio Drive Folder` → `clio_drive_folder`
  - `Open in Google drive` → `google_drive_link`

### 4. Error Handling Strategy
- **Missing Fields**: Log warning but continue processing
- **Invalid View**: Clear error with list of available views
- **Empty Results**: Informative message about filtering criteria

## Project Status Board

- [ ] **Task 1**: Design View System Architecture
- [ ] **Task 2**: Implement Core View Infrastructure  
- [ ] **Task 3**: Create Clio View Implementation
- [ ] **Task 4**: Add Field Validation and Error Handling
- [ ] **Task 5**: Integration Testing and Validation
- [ ] **Task 6**: Documentation and Help Updates

## Success Metrics

### Functional Requirements
- ✅ Single command replaces complex jq pipeline
- ✅ Filters null Clio Matter IDs automatically
- ✅ Outputs structured JSON with 6 required fields
- ✅ Maintains all existing CLI capabilities (pagination, filtering, etc.)

### Non-Functional Requirements  
- ✅ Performance: No significant slowdown vs current CLI
- ✅ Memory: Efficient processing of large datasets
- ✅ Extensibility: Easy to add new views (billing, client, etc.)
- ✅ Usability: Clear help and error messages

### Target Command Signature
```bash
# New streamlined command
cargo run -- base table Matters records --all --view clio

# Equivalent to current complex pipeline:
# cargo run -- base table Matters records --all -F "Clio Matter ID" | 
# jq -r '.[0] | map(select(.fields."Clio Matter ID" != null)) | 
# map({(.id): .fields."Clio Matter ID"}) | add'
```

## Current Status

**Project Initiated**: July 31, 2025 10:35:20 MDT  
**Phase**: Planning Complete - Ready for Implementation

## Executor Notes

*Ready for implementation - all tasks defined with clear success criteria*

## Dependencies and Assumptions

- **Table Structure**: Assumes "Matters" table has the 6 required fields
- **Field Names**: Field names are exact matches (case-sensitive)
- **Backwards Compatibility**: All existing CLI functionality must remain unchanged
- **Performance**: Should handle 2,275+ records efficiently (current dataset size)
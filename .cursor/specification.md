# RSAirtable View System Specification

## Overview

This specification defines the requirements for implementing a view-based data extraction system in RSAirtable, starting with a "clio" view for extracting Matter records with Clio-specific formatting.

## Functional Requirements

### FR-1: View System Architecture
- **FR-1.1**: CLI SHALL accept a `--view <VIEW_NAME>` parameter
- **FR-1.2**: View system SHALL be extensible for future view types
- **FR-1.3**: View processing SHALL integrate with existing record retrieval pipeline
- **FR-1.4**: View system SHALL support all existing filtering options (formula, fields, etc.)

### FR-2: Clio View Implementation  
- **FR-2.1**: "clio" view SHALL filter records to only include those with non-null/non-empty "Clio Matter ID"
- **FR-2.2**: "clio" view SHALL extract exactly these 6 fields:
  - Matter Title
  - Record ID (from Airtable record.id)
  - Clio Matter ID  
  - Clio Matter Url
  - Clio Drive Folder
  - Open in Google drive (from Clio Drive Folder)
- **FR-2.3**: "clio" view SHALL output structured JSON format
- **FR-2.4**: "clio" view SHALL be functionally equivalent to this jq pipeline:
  ```bash
  | jq -r '.[0] | map(select(.fields."Clio Matter ID" != null)) | map({(.id): .fields."Clio Matter ID"}) | add'
  ```

### FR-3: Field Handling
- **FR-3.1**: System SHALL handle missing optional fields gracefully (null/empty values)
- **FR-3.2**: System SHALL validate that required fields exist in table schema
- **FR-3.3**: System SHALL provide clear error messages for missing required fields
- **FR-3.4**: System SHALL log warnings for missing optional fields but continue processing

### FR-4: Output Format
- **FR-4.1**: View output SHALL be valid JSON
- **FR-4.2**: Each record SHALL be represented as a JSON object with field mappings:
  - `matter_title`: String from "Matter Title" field
  - `record_id`: String from Airtable record ID
  - `clio_matter_id`: String from "Clio Matter ID" field  
  - `clio_matter_url`: String from "Clio Matter Url" field
  - `clio_drive_folder`: String from "Clio Drive Folder" field
  - `google_drive_link`: String from "Open in Google drive" field
- **FR-4.3**: Output SHALL be an array of record objects

### FR-5: Integration Requirements
- **FR-5.1**: View system SHALL work with `--all` pagination flag
- **FR-5.2**: View system SHALL work with existing `--formula` filtering
- **FR-5.3**: View system SHALL work with existing `--fields` selection
- **FR-5.4**: View processing SHALL occur after record retrieval but before output

## Non-Functional Requirements

### NFR-1: Performance
- **NFR-1.1**: View processing SHALL add no more than 10% overhead to record retrieval time
- **NFR-1.2**: Memory usage SHALL remain constant regardless of dataset size
- **NFR-1.3**: System SHALL efficiently process 2,275+ records (current dataset size)

### NFR-2: Usability  
- **NFR-2.1**: CLI help SHALL document all available views and their purposes
- **NFR-2.2**: Error messages SHALL be clear and actionable
- **NFR-2.3**: Invalid view names SHALL display list of available views
- **NFR-2.4**: View processing SHALL provide progress indication for large datasets

### NFR-3: Maintainability
- **NFR-3.1**: View implementations SHALL follow SOLID principles
- **NFR-3.2**: Adding new views SHALL require minimal changes to core CLI code
- **NFR-3.3**: View system SHALL have comprehensive unit test coverage
- **NFR-3.4**: View interfaces SHALL be well-documented with examples

### NFR-4: Compatibility
- **NFR-4.1**: All existing CLI functionality SHALL remain unchanged
- **NFR-4.2**: Existing commands SHALL continue to work without modification
- **NFR-4.3**: View system SHALL be optional - CLI SHALL work without view specification
- **NFR-4.4**: Output format for non-view commands SHALL remain identical

## API Integration Requirements

### AIR-1: Airtable API Interaction
- **AIR-1.1**: View system SHALL use existing record retrieval mechanisms
- **AIR-1.2**: No additional API calls SHALL be required for view processing
- **AIR-1.3**: Field extraction SHALL work with current API response format
- **AIR-1.4**: View system SHALL respect Airtable rate limits (via existing mechanisms)

### AIR-2: Data Processing
- **AIR-2.1**: Record filtering SHALL occur on client side (in Rust code)
- **AIR-2.2**: Field extraction SHALL handle various Airtable field types
- **AIR-2.3**: Empty/null field handling SHALL be consistent across all views
- **AIR-2.4**: View processing SHALL preserve record ordering from API

## Implementation Requirements

### IR-1: Code Structure
- **IR-1.1**: Views SHALL be implemented as separate modules under `src/views/`
- **IR-1.2**: Common view interface SHALL be defined in `src/views.rs`
- **IR-1.3**: View registration SHALL use factory pattern or trait objects
- **IR-1.4**: CLI integration SHALL occur in `src/cli.rs` with minimal view-specific code

### IR-2: Error Handling
- **IR-2.1**: View-specific errors SHALL extend existing error types
- **IR-2.2**: Field validation errors SHALL be recoverable where possible
- **IR-2.3**: View processing errors SHALL not crash the application
- **IR-2.4**: Error context SHALL include view name and affected fields

### IR-3: Testing Strategy
- **IR-3.1**: Each view SHALL have dedicated unit tests
- **IR-3.2**: Integration tests SHALL verify end-to-end view functionality
- **IR-3.3**: Performance tests SHALL validate large dataset handling
- **IR-3.4**: Comparison tests SHALL verify equivalence to manual jq processing

### IR-4: Configuration
- **IR-4.1**: View behavior SHALL be configurable via CLI arguments only
- **IR-4.2**: No additional configuration files SHALL be required
- **IR-4.3**: View-specific options SHALL be namespaced and optional
- **IR-4.4**: Default behavior SHALL be backward compatible

## Test Cases

### TC-1: Basic Functionality
```bash
# Test basic clio view functionality
cargo run -- base table Matters records --view clio
# Expected: JSON array of records with 6 required fields, null Clio IDs filtered out
```

### TC-2: Integration with Existing Options
```bash  
# Test view with pagination
cargo run -- base table Matters records --all --view clio
# Expected: All records processed with view formatting

# Test view with field selection (should work together)
cargo run -- base table Matters records --view clio -F "Matter Title,Clio Matter ID"
# Expected: View formatting applied to selected fields
```

### TC-3: Error Scenarios
```bash
# Test invalid view name
cargo run -- base table Matters records --view invalid
# Expected: Clear error with list of available views

# Test view on table missing required fields  
cargo run -- base table WrongTable records --view clio
# Expected: Informative error about missing fields
```

### TC-4: Performance Validation
```bash
# Test with large dataset
time cargo run -- base table Matters records --all --view clio
# Expected: Completes within reasonable time, memory usage stable
```

### TC-5: Output Format Validation
```bash
# Test output is valid JSON
cargo run -- base table Matters records --view clio | jq .
# Expected: Valid JSON parsing, proper structure

# Test equivalence to manual pipeline
diff <(cargo run -- base table Matters records --all --view clio) \
     <(cargo run -- base table Matters records --all -F "Clio Matter ID" | \
       jq -r '.[0] | map(select(.fields."Clio Matter ID" != null)) | map({(.id): .fields."Clio Matter ID"}) | add')
# Expected: Equivalent output (allowing for format differences)
```

## Acceptance Criteria

### Primary Success Criteria
1. ✅ Single command replaces complex jq pipeline
2. ✅ All 6 required fields extracted correctly  
3. ✅ Null Clio Matter IDs automatically filtered
4. ✅ Output matches jq pipeline functionality
5. ✅ Integration with existing CLI options works seamlessly

### Quality Criteria  
1. ✅ No breaking changes to existing functionality
2. ✅ Clear, helpful error messages and documentation
3. ✅ Performance impact < 10% for record processing
4. ✅ Extensible design for future views
5. ✅ Comprehensive test coverage

### Operational Criteria
1. ✅ Help system documents new functionality clearly
2. ✅ Error scenarios handled gracefully
3. ✅ Memory usage remains efficient for large datasets
4. ✅ View system integrates cleanly with TDD workflow

---

**Document Version**: 1.0  
**Created**: July 31, 2025 10:35:20 MDT  
**Status**: Ready for Implementation
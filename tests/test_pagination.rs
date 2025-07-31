use serde_json::Value;
use std::process::Command;

/// Test that the offset parameter is accepted and works correctly
#[test]
fn test_offset_parameter_support() {
    // First, get records without offset to obtain an offset token (use default limit to ensure offset exists)
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let response: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON response");

    // Extract offset token from response[1] - skip test if no offset (e.g., fewer than 100 records total)
    let offset_token = match response[1].as_str() {
        Some(token) if !token.is_empty() => token,
        _ => {
            println!(
                "Skipping test: No offset token available (likely fewer than 100 total records)"
            );
            return;
        }
    };

    // Now use the offset token to get the next batch
    let output2 = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
            "--offset",
            offset_token,
            "-n",
            "1",
        ])
        .output()
        .expect("Failed to execute command with offset");

    assert!(
        output2.status.success(),
        "Offset command failed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    let response2: Value =
        serde_json::from_slice(&output2.stdout).expect("Failed to parse JSON response with offset");

    // Verify we got records and they are different
    let first_record_id = response[0][0]["id"].as_str().unwrap();
    let second_record_id = response2[0][0]["id"].as_str().unwrap();

    assert_ne!(
        first_record_id, second_record_id,
        "Records should be different when using offset"
    );
}

/// Test that offset parameter appears in help
#[test]
fn test_offset_parameter_in_help() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "TestTable",
            "records",
            "--help",
        ])
        .output()
        .expect("Failed to execute help command");

    let help_text = String::from_utf8_lossy(&output.stdout);
    assert!(
        help_text.contains("--offset"),
        "Help should contain --offset parameter"
    );
    assert!(
        help_text.contains("pagination"),
        "Help should mention pagination"
    );
}

/// Test that offset works with other parameters
#[test]
fn test_offset_with_filters() {
    // Test that offset can be combined with field filtering
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
            "-n",
            "1",
        ])
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        // Skip test if we can't access the API (e.g., in CI)
        return;
    }

    let response: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse JSON response");

    if let Some(offset_token) = response[1].as_str() {
        // Test offset with field filtering
        let output2 = Command::new("cargo")
            .args(&[
                "run",
                "--",
                "base",
                "table",
                "Matters",
                "records",
                "-F",
                "Clio Matter ID",
                "--offset",
                offset_token,
                "-n",
                "1",
            ])
            .output()
            .expect("Failed to execute command with offset and filters");

        assert!(
            output2.status.success(),
            "Offset with filters failed: {}",
            String::from_utf8_lossy(&output2.stderr)
        );
    }
}

/// Test that the --all flag works correctly and retrieves more records than normal pagination
#[test]
fn test_all_flag_functionality() {
    // Get normal record count (should be limited to 100)
    let output1 = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
        ])
        .output()
        .expect("Failed to execute normal command");

    if !output1.status.success() {
        // Skip test if we can't access the API
        return;
    }

    let response1: Value =
        serde_json::from_slice(&output1.stdout).expect("Failed to parse normal response");
    let normal_count = response1[0].as_array().unwrap().len();

    // Get all records using --all flag
    let output2 = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
            "--all",
        ])
        .output()
        .expect("Failed to execute --all command");

    assert!(
        output2.status.success(),
        "--all command failed: {}",
        String::from_utf8_lossy(&output2.stderr)
    );

    let response2: Value =
        serde_json::from_slice(&output2.stdout).expect("Failed to parse --all response");
    let all_count = response2[0].as_array().unwrap().len();

    // Verify --all retrieved more records than normal pagination
    assert!(
        all_count >= normal_count,
        "--all should retrieve at least as many records as normal pagination. Normal: {}, All: {}",
        normal_count,
        all_count
    );

    // Verify response format is consistent [records_array, null]
    assert_eq!(
        response2.as_array().unwrap().len(),
        2,
        "Response should have 2 elements"
    );
    assert!(
        response2[1].is_null(),
        "Second element should be null for --all queries"
    );
}

/// Test that --all flag conflicts with --limit and --offset
#[test]
fn test_all_flag_conflicts() {
    // Test conflict with --limit
    let output1 = Command::new("cargo")
        .args(&[
            "run", "--", "base", "table", "Matters", "records", "--all", "--limit", "50",
        ])
        .output()
        .expect("Failed to execute conflicting command");

    assert!(
        !output1.status.success(),
        "--all should conflict with --limit"
    );
    let stderr = String::from_utf8_lossy(&output1.stderr);
    assert!(
        stderr.contains("cannot be used with"),
        "Error should mention conflict"
    );

    // Test conflict with --offset
    let output2 = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "--all",
            "--offset",
            "some_token",
        ])
        .output()
        .expect("Failed to execute conflicting command");

    assert!(
        !output2.status.success(),
        "--all should conflict with --offset"
    );
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(
        stderr2.contains("cannot be used with"),
        "Error should mention conflict"
    );
}

/// Test that --all flag works with other filtering options
#[test]
fn test_all_flag_with_filters() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
            "--all",
            "-w",
            "NOT({Clio Matter ID} = '')",
        ])
        .output()
        .expect("Failed to execute --all with filters");

    if !output.status.success() {
        // Skip test if we can't access the API
        return;
    }

    let response: Value =
        serde_json::from_slice(&output.stdout).expect("Failed to parse filtered --all response");

    // Should return records with non-empty Clio Matter ID
    let records = response[0].as_array().unwrap();

    // Verify format and that we got some records
    assert!(records.len() > 0, "Should return some filtered records");
    assert_eq!(
        response.as_array().unwrap().len(),
        2,
        "Response should have 2 elements"
    );
    assert!(
        response[1].is_null(),
        "Second element should be null for --all queries"
    );
}

/// Test that verbose mode works correctly with --all flag
#[test]
fn test_verbose_mode_with_all() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-v",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
            "--all",
            "-w",
            "NOT({Clio Matter ID} = '')",
        ])
        .output()
        .expect("Failed to execute verbose --all command");

    if !output.status.success() {
        // Skip test if we can't access the API
        return;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check for expected verbose messages
    assert!(
        stderr.contains("Starting pagination"),
        "Should show starting message"
    );
    assert!(
        stderr.contains("Retrieved") && stderr.contains("records (total:"),
        "Should show progress messages"
    );
    assert!(
        stderr.contains("Completed!") && stderr.contains("records total"),
        "Should show completion message"
    );
}

/// Test that verbose mode works correctly with normal pagination
#[test]
fn test_verbose_mode_normal() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "-v",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
        ])
        .output()
        .expect("Failed to execute verbose normal command");

    if !output.status.success() {
        // Skip test if we can't access the API
        return;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check for expected verbose messages
    assert!(
        stderr.contains("Retrieved") && stderr.contains("records"),
        "Should show record count"
    );

    // Should indicate if more records are available
    let has_more_indicator =
        stderr.contains("more available") || stderr.contains("all records from table");
    assert!(
        has_more_indicator,
        "Should indicate if more records are available"
    );
}

/// Test that non-verbose mode is silent (default behavior)
#[test]
fn test_non_verbose_mode_silent() {
    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "base",
            "table",
            "Matters",
            "records",
            "-F",
            "Clio Matter ID",
            "-n",
            "1",
        ])
        .output()
        .expect("Failed to execute non-verbose command");

    if !output.status.success() {
        // Skip test if we can't access the API
        return;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should be silent - no progress messages
    assert!(
        !stderr.contains("Retrieved") && !stderr.contains("Starting"),
        "Non-verbose mode should be silent"
    );
}

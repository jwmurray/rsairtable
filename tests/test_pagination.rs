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

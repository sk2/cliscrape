use assert_cmd::Command;
use predicates::prelude::*;

/// Test that list-templates shows all embedded templates with metadata
#[test]
fn test_list_embedded_templates() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("list-templates").arg("--format").arg("json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let templates: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Should be an array
    assert!(templates.is_array(), "Output should be JSON array");
    let templates = templates.as_array().unwrap();

    // Should have at least 5 templates (our new ones, plus any existing)
    assert!(
        templates.len() >= 5,
        "Should have at least 5 templates, found {}",
        templates.len()
    );

    // Check that our templates are present
    let names: Vec<String> = templates
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str().map(|s| s.to_string())))
        .collect();

    assert!(
        names.contains(&"cisco_ios_show_version.yaml".to_string()),
        "Should contain cisco_ios_show_version.yaml"
    );
    assert!(
        names.contains(&"cisco_ios_show_interfaces.yaml".to_string()),
        "Should contain cisco_ios_show_interfaces.yaml"
    );
    assert!(
        names.contains(&"juniper_junos_show_version.yaml".to_string()),
        "Should contain juniper_junos_show_version.yaml"
    );
    assert!(
        names.contains(&"arista_eos_show_version.yaml".to_string()),
        "Should contain arista_eos_show_version.yaml"
    );
    assert!(
        names.contains(&"cisco_nxos_show_version.yaml".to_string()),
        "Should contain cisco_nxos_show_version.yaml"
    );

    // Check that templates have metadata fields
    for template in templates {
        assert!(
            template.get("name").is_some(),
            "Template should have name field"
        );
        assert!(
            template.get("description").is_some(),
            "Template should have description field"
        );
        assert!(
            template.get("compatibility").is_some(),
            "Template should have compatibility field"
        );
        assert!(
            template.get("version").is_some(),
            "Template should have version field"
        );
        assert!(
            template.get("source").is_some(),
            "Template should have source field"
        );
    }
}

/// Test that show-template displays metadata and field information
#[test]
fn test_show_template_details() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("show-template")
        .arg("cisco_ios_show_version.yaml");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Description:"))
        .stdout(predicate::str::contains("Compatibility:"))
        .stdout(predicate::str::contains("Version:"))
        .stdout(predicate::str::contains("Author:"))
        .stdout(predicate::str::contains("Source: Embedded"))
        .stdout(predicate::str::contains("Fields Extracted:"));
}

/// Test that parsing with embedded template works without file paths
#[test]
fn test_parse_with_embedded_template() {
    // Sample Cisco IOS show version output
    let sample_input = r#"Router1 uptime is 5 weeks, 2 days, 3 hours, 45 minutes
Cisco IOS Software, C2960 Software (C2960-LANBASEK9-M), Version 15.0(2)SE11, RELEASE SOFTWARE (fc3)
System serial number: FOC1234ABCD
cisco WS-C2960-48TT-L (PowerPC405) processor (revision V02) with 65536K bytes of memory.
"#;

    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("parse")
        .arg("--template")
        .arg("cisco_ios_show_version.yaml")
        .arg("--format")
        .arg("json")
        .write_stdin(sample_input);

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();

    // Verify JSON output contains expected fields
    assert!(
        stdout.contains("\"hostname\""),
        "Output should contain hostname field"
    );
    assert!(
        stdout.contains("\"version\""),
        "Output should contain version field"
    );
    assert!(
        stdout.contains("Router1"),
        "Output should contain hostname value"
    );
    assert!(
        stdout.contains("15.0(2)SE11"),
        "Output should contain version value"
    );
}

/// Test that template name security validation prevents path traversal
#[test]
fn test_template_name_security_validation() {
    // Test path traversal attempt with non-existent path (to trigger resolver validation)
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("parse")
        .arg("--template")
        .arg("../nonexistent/template")
        .arg("--input")
        .arg("/dev/null");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));

    // Test template name with path separators (should be rejected by resolver)
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("parse")
        .arg("--template")
        .arg("evil/../../etc/passwd")
        .arg("--input")
        .arg("/dev/null");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test that filtering templates works with glob patterns
#[test]
fn test_filter_templates() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("list-templates")
        .arg("--filter")
        .arg("cisco*")
        .arg("--format")
        .arg("json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let templates: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let templates = templates.as_array().unwrap();

    // Should have at least 3 Cisco templates
    assert!(
        templates.len() >= 3,
        "Should have at least 3 Cisco templates, found {}",
        templates.len()
    );

    // All results should be Cisco templates
    for template in templates {
        let name = template
            .get("name")
            .and_then(|n| n.as_str())
            .unwrap_or("");
        assert!(
            name.starts_with("cisco"),
            "Filtered template name should start with 'cisco', found: {}",
            name
        );
    }

    // Verify Juniper and Arista templates are NOT in output
    let names: Vec<String> = templates
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str().map(|s| s.to_string())))
        .collect();

    assert!(
        !names.contains(&"juniper_junos_show_version.yaml".to_string()),
        "Juniper template should not be in Cisco filter results"
    );
    assert!(
        !names.contains(&"arista_eos_show_version.yaml".to_string()),
        "Arista template should not be in Cisco filter results"
    );
}

/// Test that show-template includes source code when --source flag is used
#[test]
fn test_show_template_source_flag() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("show-template")
        .arg("cisco_ios_show_version.yaml")
        .arg("--source");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Template Source:"))
        .stdout(predicate::str::contains("metadata:"))
        .stdout(predicate::str::contains("version: 1"))
        .stdout(predicate::str::contains("fields:"));
}

/// Test that nonexistent template returns appropriate error
#[test]
fn test_nonexistent_template_error() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("show-template").arg("nonexistent_template.yaml");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

/// Test CSV format returns error (not supported)
#[test]
fn test_list_templates_csv_format_error() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("list-templates")
        .arg("--format")
        .arg("csv");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("CSV format not supported"));
}

/// Test that embedded templates have correct metadata
#[test]
fn test_embedded_template_metadata() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("list-templates").arg("--format").arg("json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let templates: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let templates = templates.as_array().unwrap();

    // Find cisco_ios_show_version.yaml and verify its metadata
    let cisco_ios = templates
        .iter()
        .find(|t| {
            t.get("name")
                .and_then(|n| n.as_str())
                == Some("cisco_ios_show_version.yaml")
        })
        .expect("Should find cisco_ios_show_version.yaml");

    assert_eq!(
        cisco_ios.get("description").and_then(|d| d.as_str()),
        Some("Parse output of 'show version' command"),
        "Description should match"
    );
    assert_eq!(
        cisco_ios.get("compatibility").and_then(|c| c.as_str()),
        Some("Cisco IOS 12.x, 15.x, IOS-XE"),
        "Compatibility should match"
    );
    assert_eq!(
        cisco_ios.get("version").and_then(|v| v.as_str()),
        Some("1.0.0"),
        "Version should match"
    );
    assert_eq!(
        cisco_ios.get("source").and_then(|s| s.as_str()),
        Some("Embedded"),
        "Source should be Embedded"
    );
}

/// Test that template list is sorted alphabetically
#[test]
fn test_template_list_sorted() {
    let mut cmd = Command::cargo_bin("cliscrape").unwrap();
    cmd.arg("list-templates").arg("--format").arg("json");

    let output = cmd.assert().success();
    let stdout = String::from_utf8(output.get_output().stdout.clone()).unwrap();
    let templates: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    let templates = templates.as_array().unwrap();
    let names: Vec<String> = templates
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str().map(|s| s.to_string())))
        .collect();

    let mut sorted_names = names.clone();
    sorted_names.sort();

    assert_eq!(
        names, sorted_names,
        "Template list should be sorted alphabetically"
    );
}

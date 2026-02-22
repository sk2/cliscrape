use cliscrape::FsmParser;
use insta::assert_yaml_snapshot;

/// Helper function for positive test cases
/// Parses fixture input and snapshots the JSON output
fn test_positive_case(snapshot_name: &str, template_path: &str, fixture_path: &str) {
    let parser = FsmParser::from_file(template_path).unwrap();
    let input = std::fs::read_to_string(fixture_path).unwrap();
    let results = parser.parse(&input).unwrap();

    // Snapshot the JSON output for regression detection
    assert_yaml_snapshot!(snapshot_name, results);
}

/// Helper function for negative test cases
/// Documents parser behavior on malformed/incomplete input via snapshots
/// Note: Parser may return Ok([]) for incomplete data - this is by design
fn test_negative_case(snapshot_name: &str, template_path: &str, fixture_path: &str) {
    let parser = FsmParser::from_file(template_path).unwrap();
    let input = std::fs::read_to_string(fixture_path).unwrap();
    let result = parser.parse(&input);

    // Snapshot the result (whether Ok or Err) to detect regressions
    // This documents current behavior and catches unintended changes
    match result {
        Ok(records) => {
            // Parser succeeded with partial/empty captures - snapshot what was captured
            assert_yaml_snapshot!(snapshot_name, records);
        }
        Err(err) => {
            // Parser returned error - snapshot the error message
            assert_yaml_snapshot!(snapshot_name, format!("{:?}", err));
        }
    }
}

// ============================================================================
// POSITIVE TEST CASES - Snapshot testing for all embedded templates
// ============================================================================

// Cisco IOS show version positive tests
#[test]
fn test_cisco_ios_show_version_ios_15_standard() {
    test_positive_case(
        "cisco_ios_show_version_ios_15_standard",
        "templates/cisco_ios_show_version.yaml",
        "tests/fixtures/cisco/ios_show_version/ios_15_standard.txt",
    );
}

#[test]
fn test_cisco_ios_show_version_ios_12_legacy() {
    test_positive_case(
        "cisco_ios_show_version_ios_12_legacy",
        "templates/cisco_ios_show_version.yaml",
        "tests/fixtures/cisco/ios_show_version/ios_12_legacy.txt",
    );
}

// Cisco IOS show interfaces positive tests
#[test]
fn test_cisco_ios_show_interfaces_ios_15_standard() {
    test_positive_case(
        "cisco_ios_show_interfaces_ios_15_standard",
        "templates/cisco_ios_show_interfaces.yaml",
        "tests/fixtures/cisco/ios_show_interfaces/ios_15_standard.txt",
    );
}

// Cisco NX-OS show version positive tests
#[test]
fn test_cisco_nxos_show_version_nxos_9_standard() {
    test_positive_case(
        "cisco_nxos_show_version_nxos_9_standard",
        "templates/cisco_nxos_show_version.yaml",
        "tests/fixtures/cisco/nxos_show_version/nxos_9_standard.txt",
    );
}

// Juniper JunOS show version positive tests
#[test]
fn test_juniper_junos_show_version_junos_12_standard() {
    test_positive_case(
        "juniper_junos_show_version_junos_12_standard",
        "templates/juniper_junos_show_version.yaml",
        "tests/fixtures/juniper/junos_show_version/junos_12_standard.txt",
    );
}

// Arista EOS show version positive tests
#[test]
fn test_arista_eos_show_version_eos_4_standard() {
    test_positive_case(
        "arista_eos_show_version_eos_4_standard",
        "templates/arista_eos_show_version.yaml",
        "tests/fixtures/arista/eos_show_version/eos_4_standard.txt",
    );
}

// ============================================================================
// NEGATIVE TEST CASES - Error handling validation
// ============================================================================

// Cisco IOS show version negative tests
#[test]
fn test_cisco_ios_show_version_truncated_output() {
    test_negative_case(
        "cisco_ios_show_version_truncated",
        "templates/cisco_ios_show_version.yaml",
        "tests/fixtures/cisco/ios_show_version/negative/truncated_output.txt",
    );
}

#[test]
fn test_cisco_ios_show_version_malformed_version() {
    test_negative_case(
        "cisco_ios_show_version_malformed_version",
        "templates/cisco_ios_show_version.yaml",
        "tests/fixtures/cisco/ios_show_version/negative/malformed_version.txt",
    );
}

#[test]
fn test_cisco_ios_show_version_empty_input() {
    test_negative_case(
        "cisco_ios_show_version_empty",
        "templates/cisco_ios_show_version.yaml",
        "tests/fixtures/cisco/ios_show_version/negative/empty_input.txt",
    );
}

#[test]
fn test_cisco_ios_show_version_invalid_uptime() {
    test_negative_case(
        "cisco_ios_show_version_invalid_uptime",
        "templates/cisco_ios_show_version.yaml",
        "tests/fixtures/cisco/ios_show_version/negative/invalid_uptime.txt",
    );
}

// Cisco IOS show interfaces negative tests
#[test]
fn test_cisco_ios_show_interfaces_truncated_output() {
    test_negative_case(
        "cisco_ios_show_interfaces_truncated",
        "templates/cisco_ios_show_interfaces.yaml",
        "tests/fixtures/cisco/ios_show_interfaces/negative/truncated_output.txt",
    );
}

#[test]
fn test_cisco_ios_show_interfaces_missing_status() {
    test_negative_case(
        "cisco_ios_show_interfaces_missing_status",
        "templates/cisco_ios_show_interfaces.yaml",
        "tests/fixtures/cisco/ios_show_interfaces/negative/missing_status.txt",
    );
}

// Cisco NX-OS show version negative tests
#[test]
fn test_cisco_nxos_show_version_truncated_output() {
    test_negative_case(
        "cisco_nxos_show_version_truncated",
        "templates/cisco_nxos_show_version.yaml",
        "tests/fixtures/cisco/nxos_show_version/negative/truncated_output.txt",
    );
}

#[test]
fn test_cisco_nxos_show_version_malformed_serial() {
    test_negative_case(
        "cisco_nxos_show_version_malformed_serial",
        "templates/cisco_nxos_show_version.yaml",
        "tests/fixtures/cisco/nxos_show_version/negative/malformed_serial.txt",
    );
}

// Juniper JunOS show version negative tests
#[test]
fn test_juniper_junos_show_version_malformed_hostname() {
    test_negative_case(
        "juniper_junos_show_version_malformed_hostname",
        "templates/juniper_junos_show_version.yaml",
        "tests/fixtures/juniper/junos_show_version/negative/malformed_hostname.txt",
    );
}

#[test]
fn test_juniper_junos_show_version_empty_input() {
    test_negative_case(
        "juniper_junos_show_version_empty",
        "templates/juniper_junos_show_version.yaml",
        "tests/fixtures/juniper/junos_show_version/negative/empty_input.txt",
    );
}

// Arista EOS show version negative tests
#[test]
fn test_arista_eos_show_version_truncated_output() {
    test_negative_case(
        "arista_eos_show_version_truncated",
        "templates/arista_eos_show_version.yaml",
        "tests/fixtures/arista/eos_show_version/negative/truncated_output.txt",
    );
}

#[test]
fn test_arista_eos_show_version_malformed_model() {
    test_negative_case(
        "arista_eos_show_version_malformed_model",
        "templates/arista_eos_show_version.yaml",
        "tests/fixtures/arista/eos_show_version/negative/malformed_model.txt",
    );
}

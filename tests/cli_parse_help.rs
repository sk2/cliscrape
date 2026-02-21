use assert_cmd::Command;

#[test]
fn parse_help_includes_phase2_contract_flags_and_defaults() {
    let output = Command::cargo_bin("cliscrape")
        .expect("cliscrape binary builds")
        .args(["parse", "--help"])
        .output()
        .expect("run cliscrape parse --help");

    assert!(output.status.success(), "--help should succeed");

    let help = String::from_utf8_lossy(&output.stdout);

    // Template
    assert!(help.contains("-t, --template"));
    assert!(help.contains("Template spec (path or identifier)"));
    assert!(help.contains("--template-format"));
    assert!(help.contains("auto"));
    assert!(help.contains("textfsm"));
    assert!(help.contains("yaml"));
    assert!(help.contains("toml"));
    assert!(help.contains("[default: auto]"));

    // Inputs
    assert!(help.contains("[INPUTS]..."));
    assert!(help.contains("--input <PATH>"));
    assert!(help.contains("--input-glob <PATTERN>"));
    assert!(help.contains("--stdin"));

    // Output
    assert!(help.contains("-f, --format"));
    assert!(help.contains("- auto"));
    assert!(help.contains("- json"));
    assert!(help.contains("- csv"));
    assert!(help.contains("- table"));
    assert!(help.contains("[default: auto]"));

    // Errors
    assert!(help.contains("--error-format <ERROR_FORMAT>"));
    assert!(help.contains("- human"));
    assert!(help.contains("- json"));
    assert!(help.contains("[default: human]"));

    // Status
    assert!(help.contains("--quiet"));
}

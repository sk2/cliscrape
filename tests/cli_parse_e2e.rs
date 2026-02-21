use assert_cmd::Command;

#[test]
fn parse_file_input_emits_json_with_hostname() {
    let output = Command::cargo_bin("cliscrape")
        .expect("cliscrape binary builds")
        .args([
            "parse",
            "-t",
            "templates/modern/simple_hostname.toml",
            "tests/fixtures/inputs/hostname_file.txt",
        ])
        .output()
        .expect("run cliscrape parse with file input");

    assert!(output.status.success(), "parse file input should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("stdout is valid JSON");

    let records = json.as_array().expect("JSON is array");
    assert_eq!(records.len(), 1, "should have 1 record");
    assert_eq!(
        records[0]["hostname"],
        serde_json::json!("FileHost"),
        "hostname should be FileHost"
    );
}

#[test]
fn parse_piped_stdin_emits_json_with_hostname() {
    let output = Command::cargo_bin("cliscrape")
        .expect("cliscrape binary builds")
        .args([
            "parse",
            "-t",
            "templates/modern/simple_hostname.toml",
            "--stdin",
        ])
        .write_stdin("Hostname: StdinHost\n")
        .output()
        .expect("run cliscrape parse with piped stdin");

    assert!(output.status.success(), "parse stdin should succeed");

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("stdout is valid JSON");

    let records = json.as_array().expect("JSON is array");
    assert_eq!(records.len(), 1, "should have 1 record");
    assert_eq!(
        records[0]["hostname"],
        serde_json::json!("StdinHost"),
        "hostname should be StdinHost"
    );
}

#[test]
fn parse_stdin_plus_file_ordering_file_first_stdin_last() {
    let output = Command::cargo_bin("cliscrape")
        .expect("cliscrape binary builds")
        .args([
            "parse",
            "-t",
            "templates/modern/simple_hostname.toml",
            "tests/fixtures/inputs/hostname_file.txt",
            "--stdin",
        ])
        .write_stdin("Hostname: StdinHost\n")
        .output()
        .expect("run cliscrape parse with file + stdin");

    assert!(
        output.status.success(),
        "parse file + stdin should succeed"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("stdout is valid JSON");

    let records = json.as_array().expect("JSON is array");
    assert_eq!(records.len(), 2, "should have 2 records");
    
    // Phase-2 contract: files first, stdin last
    assert_eq!(
        records[0]["hostname"],
        serde_json::json!("FileHost"),
        "first record should be from file"
    );
    assert_eq!(
        records[1]["hostname"],
        serde_json::json!("StdinHost"),
        "second record should be from stdin"
    );
}

#[test]
fn parse_textfsm_required_filldown_interaction() {
    let output = Command::cargo_bin("cliscrape")
        .expect("cliscrape binary builds")
        .args([
            "parse",
            "-t",
            "tests/fixtures/textfsm/test_required.textfsm",
            "tests/fixtures/inputs/textfsm_required_ok.txt",
        ])
        .output()
        .expect("run cliscrape parse with Required+Filldown template");

    assert!(
        output.status.success(),
        "parse Required+Filldown should succeed"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("stdout is valid JSON");

    let records = json.as_array().expect("JSON is array");
    assert!(records.len() >= 2, "should have at least 2 records");

    // Check that Required+Filldown works: later records should have INTERFACE
    // populated via Filldown even though only ADDRESS lines match
    assert!(
        records[1].get("INTERFACE").is_some(),
        "second record should have INTERFACE via Filldown"
    );
    assert!(
        records[1]["INTERFACE"].as_str().is_some(),
        "second record INTERFACE should be populated"
    );
}

#[test]
fn parse_identifier_resolution_via_cwd_only_search() {
    // Run from tests/fixtures/textfsm directory, use identifier instead of path
    let output = Command::cargo_bin("cliscrape")
        .expect("cliscrape binary builds")
        .current_dir("tests/fixtures/textfsm")
        .args([
            "parse",
            "-t",
            "test_required",
            "../inputs/textfsm_required_ok.txt",
        ])
        .output()
        .expect("run cliscrape parse with identifier");

    assert!(
        output.status.success(),
        "parse with identifier should succeed"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid UTF-8");
    let json: serde_json::Value = serde_json::from_str(stdout.trim())
        .expect("stdout is valid JSON");

    let records = json.as_array().expect("JSON is array");
    assert!(records.len() >= 2, "should have at least 2 records");
    
    // Verify it's the same result as explicit path test
    assert!(
        records[1].get("INTERFACE").is_some(),
        "identifier resolution should produce same results as explicit path"
    );
}

#[test]
fn parse_no_partial_stdout_on_later_failure() {
    // First input succeeds, second triggers Error action
    // Verify: exit code 1, stdout empty (no partial JSON)
    let output = Command::cargo_bin("cliscrape")
        .expect("cliscrape binary builds")
        .args([
            "parse",
            "-t",
            "tests/fixtures/textfsm/error_action.textfsm",
            "tests/fixtures/inputs/textfsm_error_ok.txt",
            "tests/fixtures/inputs/textfsm_error_trigger.txt",
        ])
        .output()
        .expect("run cliscrape parse with error trigger");

    assert!(
        !output.status.success(),
        "parse with Error action should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(1),
        "exit code should be 1 on Error"
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is valid UTF-8");

    // Stdout should be empty or parse as empty array
    // (no partial results emitted before failure)
    let trimmed = stdout.trim();
    if !trimmed.is_empty() {
        let json: Result<serde_json::Value, _> = serde_json::from_str(trimmed);
        if let Ok(json) = json {
            if let Some(records) = json.as_array() {
                assert!(
                    records.is_empty(),
                    "no partial records should be emitted before failure"
                );
            }
        }
    }
}

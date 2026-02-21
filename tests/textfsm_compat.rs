use cliscrape::FsmParser;

#[test]
fn error_action_aborts_and_discards_rows() {
    let parser = FsmParser::from_file("tests/fixtures/textfsm/error_action.textfsm")
        .expect("fixture template should load");

    let input = "OK first\nBOOM now\nOK second\n";
    let err = parser
        .parse(input)
        .expect_err("Error action should fail-fast");
    let msg = err.to_string();
    assert!(
        msg.contains("Error action"),
        "error message should mention Error action, got: {msg}"
    );
}

#[test]
fn clear_preserves_filldown_and_clearall_clears_it() {
    let parser = FsmParser::from_file("tests/fixtures/textfsm/clear_vs_clearall.textfsm")
        .expect("fixture template should load");

    let input = "Chassis: R1\nSlot: 1\nCLEAR\nSlot: 2\nCLEARALL\nSlot: 3\n";
    let results = parser.parse(input).expect("parse should succeed");

    assert_eq!(results.len(), 3);

    assert_eq!(results[0]["CHASSIS"], "R1");
    assert_eq!(results[0]["SLOT"], serde_json::json!(1));

    assert_eq!(results[1]["CHASSIS"], "R1");
    assert_eq!(results[1]["SLOT"], serde_json::json!(2));

    assert_eq!(results[2]["CHASSIS"], serde_json::json!(""));
    assert_eq!(results[2]["SLOT"], serde_json::json!(3));
}

#[test]
fn undefined_placeholder_errors_at_template_load() {
    let template = r#"Value INTERFACE (\S+)

Start
  ^Interface ${INTERFACE} status ${MISSING}
"#;

    use cliscrape::template::loader::TextFsmLoader;
    use cliscrape::engine::Template;
    let ir = TextFsmLoader::parse_str(template).expect("parse should succeed");
    let err = Template::from_ir(ir).expect_err("template with undefined placeholder should error");
    let msg = err.to_string();
    assert!(
        msg.contains("${MISSING}") || msg.contains("MISSING"),
        "error should mention undefined placeholder MISSING, got: {msg}"
    );
}

#[test]
fn undefined_macro_errors_at_template_load() {
    let template = r#"Value INTERFACE (\S+)

Start
  ^Interface ${INTERFACE} is {{missing_macro}}
"#;

    use cliscrape::template::loader::TextFsmLoader;
    use cliscrape::engine::Template;
    let ir = TextFsmLoader::parse_str(template).expect("parse should succeed");
    let err = Template::from_ir(ir).expect_err("template with undefined macro should error");
    let msg = err.to_string();
    assert!(
        msg.contains("missing_macro") || msg.contains("Unknown macro"),
        "error should mention undefined macro missing_macro, got: {msg}"
    );
}

#[test]
fn explicit_eof_empty_suppresses_implicit_record() {
    let parser = FsmParser::from_file("tests/fixtures/textfsm/explicit_eof_empty.textfsm")
        .expect("fixture template should load");

    // Without explicit empty EOF, this would emit 1 record at EOF
    // With explicit empty EOF, no record should be emitted
    let input = "Data value";
    let results = parser.parse(input).expect("parse should succeed");
    assert_eq!(
        results.len(),
        0,
        "explicit empty EOF state should suppress implicit EOF record"
    );
}

#[test]
fn explicit_eof_rules_execute_once() {
    let parser = FsmParser::from_file("tests/fixtures/textfsm/explicit_eof_rules.textfsm")
        .expect("fixture template should load");

    // EOF rule matches empty string and records
    let input = "Item 1\nItem 2\nItem 3";
    let results = parser.parse(input).expect("parse should succeed");

    // Should emit one record at EOF containing the last captured COUNT
    assert_eq!(results.len(), 1, "EOF rules should execute once at end of input");
    assert_eq!(results[0]["COUNT"], serde_json::json!(3));
}

#[test]
fn warn_skip_constructs_returns_warnings_and_parses() {
    let (parser, warnings) = FsmParser::from_file_with_warnings(
        "tests/fixtures/textfsm/warn_skip_constructs.textfsm"
    ).expect("fixture template should load with warnings");

    // Should have warnings for unknown flag and unknown action
    assert!(
        !warnings.is_empty(),
        "should have warnings for unknown constructs"
    );

    // Check for unknown flag warning
    let has_flag_warning = warnings.iter().any(|w|
        w.kind == "unknown_value_flag" && w.message.contains("UnknownFlag")
    );
    assert!(has_flag_warning, "should warn about unknown Value flag");

    // Check for unknown action warning
    let has_action_warning = warnings.iter().any(|w|
        (w.kind == "unknown_record_action" || w.kind == "unknown_line_action")
        && w.message.contains("UnknownAction")
    );
    assert!(has_action_warning, "should warn about unknown action");

    // Template should still parse input correctly (skipping bad rule)
    let input = "Data value\nTrigger\n";
    let results = parser.parse(input).expect("parse should succeed despite warnings");

    assert_eq!(results.len(), 1, "should emit one record from valid rule");
    assert_eq!(results[0]["DATA"], "value");
}

#[test]
fn comment_lines_are_ignored() {
    let parser = FsmParser::from_file("tests/fixtures/textfsm/comment_lines_ignored.textfsm")
        .expect("fixture template with comments should load");

    // Comments should not affect template behavior
    let input = "Interface Eth0 is up";
    let results = parser.parse(input).expect("parse should succeed");

    assert_eq!(results.len(), 1, "should emit one record");
    assert_eq!(results[0]["INTERFACE"], "Eth0");
    assert_eq!(results[0]["STATUS"], "up");
}

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

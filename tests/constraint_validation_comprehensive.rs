use assert_cmd::prelude::*;
use cliscrape::engine::records::RecordBuffer;
use cliscrape::engine::types::{FieldConstraints, FieldType, Value as TemplateValue};
use cliscrape::engine::validate::validate_value;
use predicates::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

fn constraints(
    min: Option<f64>,
    max: Option<f64>,
    choices: Option<Vec<&str>>,
    regex: Option<&str>,
) -> FieldConstraints {
    FieldConstraints {
        min,
        max,
        choices: choices.map(|items| items.into_iter().map(str::to_string).collect()),
        regex: regex.map(str::to_string),
    }
}

fn template_value(
    name: &str,
    list: bool,
    type_hint: Option<FieldType>,
    constraints: Option<FieldConstraints>,
) -> TemplateValue {
    TemplateValue {
        name: name.to_string(),
        regex: String::new(),
        filldown: false,
        required: false,
        list,
        identity: false,
        ignore: false,
        common_schema: None,
        constraints,
        type_hint,
    }
}

fn temp_file(name: &str, content: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "cliscrape-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        name
    ));
    std::fs::write(&path, content).unwrap();
    path
}

#[test]
fn min_and_max_boundaries_behave_as_expected() {
    let in_range = validate_value(
        "mtu",
        &json!(1500),
        &constraints(Some(1500.0), Some(9000.0), None, None),
    );
    let below = validate_value(
        "mtu",
        &json!(1499),
        &constraints(Some(1500.0), Some(9000.0), None, None),
    );
    let above = validate_value(
        "mtu",
        &json!(9001),
        &constraints(Some(1500.0), Some(9000.0), None, None),
    );

    assert!(in_range.is_empty());
    assert_eq!(below.len(), 1);
    assert!(below[0].message.contains("less than minimum 1500"));
    assert_eq!(above.len(), 1);
    assert!(above[0].message.contains("greater than maximum 9000"));
}

#[test]
fn choices_validation_handles_pass_fail_and_empty_choices() {
    let allowed = constraints(None, None, Some(vec!["up", "down"]), None);
    let empty = constraints(None, None, Some(vec![]), None);

    assert!(validate_value("status", &json!("up"), &allowed).is_empty());

    let disallowed = validate_value("status", &json!("admin-up"), &allowed);
    assert_eq!(disallowed.len(), 1);
    assert!(disallowed[0].message.contains("not in allowed choices"));

    let empty_choices = validate_value("status", &json!("up"), &empty);
    assert_eq!(empty_choices.len(), 1);
    assert!(empty_choices[0].message.contains("not in allowed choices"));
}

#[test]
fn regex_validation_handles_match_mismatch_and_invalid_pattern() {
    let regex_constraint = constraints(None, None, None, Some(r"^Gi\d+/\d+$"));
    let invalid_constraint = constraints(None, None, None, Some("("));

    assert!(validate_value("interface", &json!("Gi0/1"), &regex_constraint).is_empty());

    let mismatch = validate_value("interface", &json!("Loopback0"), &regex_constraint);
    assert_eq!(mismatch.len(), 1);
    assert_eq!(mismatch[0].kind, "ConstraintViolation");

    let invalid = validate_value("interface", &json!("Gi0/1"), &invalid_constraint);
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0].kind, "InvalidConstraint");
}

#[test]
fn multiple_constraints_can_emit_multiple_warnings() {
    let warnings = validate_value(
        "status",
        &json!("unknown"),
        &constraints(None, None, Some(vec!["up", "down"]), Some(r"^(up|down)$")),
    );

    assert_eq!(warnings.len(), 2);
    assert!(
        warnings
            .iter()
            .all(|warning| warning.kind == "ConstraintViolation")
    );
}

#[test]
fn list_fields_validate_each_element_during_emit() {
    let mut rb = RecordBuffer::new();
    let mut values = HashMap::new();
    values.insert(
        "mtu".to_string(),
        template_value(
            "mtu",
            true,
            Some(FieldType::Int),
            Some(constraints(Some(1500.0), None, None, None)),
        ),
    );

    rb.insert("mtu".to_string(), "1500".to_string(), true);
    rb.insert("mtu".to_string(), "1400".to_string(), true);

    let (record, warnings) = rb.emit(&values).unwrap();

    assert_eq!(record["mtu"], json!([1500, 1400]));
    assert_eq!(warnings.len(), 1);
    assert!(warnings[0].message.contains("less than minimum 1500"));
}

#[test]
fn strict_policy_fails_on_regex_constraint_violation() {
    let template = temp_file(
        "constraints-regex.yaml",
        r#"version: 1
fields:
  interface:
    type: string
    pattern: '\S+'
    constraints:
      regex: '^Gi\d+/\d+$'
patterns:
  - regex: '^Interface: ${interface}'
    record: true
"#,
    );
    let input = temp_file("constraints-regex.txt", "Interface: Loopback0\n");

    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("cliscrape"));
    cmd.args([
        "parse",
        input.to_str().unwrap(),
        "--template",
        template.to_str().unwrap(),
        "--strict-policy",
    ]);

    cmd.assert().failure().stderr(predicate::str::contains(
        "Constraint violation failed strict policy",
    ));

    let _ = std::fs::remove_file(template);
    let _ = std::fs::remove_file(input);
}

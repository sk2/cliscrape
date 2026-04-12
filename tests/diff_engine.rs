use cliscrape::engine::diff::{DiffOp, SemanticDiffEngine};
use cliscrape::engine::types::Value as TemplateValue;
use insta::assert_debug_snapshot;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashMap};

fn template_value(name: &str, identity: bool, ignore: bool) -> TemplateValue {
    TemplateValue {
        name: name.to_string(),
        regex: String::new(),
        filldown: false,
        required: false,
        list: false,
        identity,
        ignore,
        common_schema: None,
        constraints: None,
        type_hint: None,
    }
}

fn mock_template_values(identities: &[&str], ignores: &[&str]) -> HashMap<String, TemplateValue> {
    let mut values = HashMap::new();
    for name in identities {
        values.insert((*name).to_string(), template_value(name, true, false));
    }
    for name in ignores {
        values.insert((*name).to_string(), template_value(name, false, true));
    }
    values
}

fn record(value: Value) -> BTreeMap<String, Value> {
    value.as_object().unwrap().clone().into_iter().collect()
}

#[test]
fn identical_inputs_produce_empty_diff() {
    let values = mock_template_values(&["name"], &[]);
    let engine = SemanticDiffEngine::new(&values);
    let before = vec![record(json!({"name": "Gi0/1", "status": "up"}))];
    let after = vec![record(json!({"name": "Gi0/1", "status": "up"}))];

    let diffs = engine.diff(before, after);

    assert!(diffs.is_empty());
}

#[test]
fn added_and_removed_records_are_detected() {
    let values = mock_template_values(&["name"], &[]);
    let engine = SemanticDiffEngine::new(&values);
    let before = vec![record(json!({"name": "Gi0/1", "status": "up"}))];
    let after = vec![record(json!({"name": "Gi0/2", "status": "down"}))];

    let diffs = engine.diff(before, after);

    assert_eq!(diffs.len(), 2);
    assert!(diffs.iter().any(
        |op| matches!(op, DiffOp::Removed { before } if before.get("name") == Some(&json!("Gi0/1")))
    ));
    assert!(diffs.iter().any(
        |op| matches!(op, DiffOp::Added { after } if after.get("name") == Some(&json!("Gi0/2")))
    ));
}

#[test]
fn reordered_records_match_by_identity() {
    let values = mock_template_values(&["name"], &[]);
    let engine = SemanticDiffEngine::new(&values);
    let before = vec![
        record(json!({"name": "Gi0/1", "status": "up"})),
        record(json!({"name": "Gi0/2", "status": "down"})),
    ];
    let after = vec![
        record(json!({"name": "Gi0/2", "status": "down"})),
        record(json!({"name": "Gi0/1", "status": "up"})),
    ];

    let diffs = engine.diff(before, after);

    assert!(diffs.is_empty());
}

#[test]
fn ignored_fields_do_not_generate_modifications() {
    let values = mock_template_values(&["name"], &["uptime"]);
    let engine = SemanticDiffEngine::new(&values);
    let before = vec![record(
        json!({"name": "Gi0/1", "status": "up", "uptime": "10s"}),
    )];
    let after = vec![record(
        json!({"name": "Gi0/1", "status": "up", "uptime": "20s"}),
    )];

    let diffs = engine.diff(before, after);

    assert!(diffs.is_empty());
}

#[test]
fn nested_list_changes_are_reported() {
    let values = mock_template_values(&["name"], &[]);
    let engine = SemanticDiffEngine::new(&values);
    let before = vec![record(json!({"name": "Gi0/1", "vlans": [10, 20]}))];
    let after = vec![record(json!({"name": "Gi0/1", "vlans": [10, 30]}))];

    let diffs = engine.diff(before, after);

    assert_eq!(diffs.len(), 1);
    match &diffs[0] {
        DiffOp::Modified { identity, changes } => {
            assert_eq!(identity.get("name"), Some(&json!("Gi0/1")));
            assert_eq!(changes["vlans"].before, json!([10, 20]));
            assert_eq!(changes["vlans"].after, json!([10, 30]));
        }
        other => panic!("expected modified diff, got {other:?}"),
    }
}

#[test]
fn no_identity_fields_fall_back_to_record_content_without_ignored_fields() {
    let values = mock_template_values(&[], &["timestamp"]);
    let engine = SemanticDiffEngine::new(&values);
    let before = vec![record(
        json!({"interface": "Gi0/1", "status": "up", "timestamp": "t1"}),
    )];
    let after = vec![record(
        json!({"interface": "Gi0/1", "status": "up", "timestamp": "t2"}),
    )];

    let diffs = engine.diff(before, after);

    assert!(diffs.is_empty());
}

#[test]
fn diff_output_format_is_stable() {
    let values = mock_template_values(&["name"], &[]);
    let engine = SemanticDiffEngine::new(&values);
    let before = vec![record(
        json!({"name": "Gi0/1", "status": "up", "description": "old"}),
    )];
    let after = vec![record(
        json!({"name": "Gi0/1", "status": "down", "description": "new"}),
    )];

    let diffs = engine.diff(before, after);

    assert_debug_snapshot!(diffs, @r#"
    [
        Modified {
            identity: {
                "name": String("Gi0/1"),
            },
            changes: {
                "description": ValueChange {
                    before: String("old"),
                    after: String("new"),
                },
                "status": ValueChange {
                    before: String("up"),
                    after: String("down"),
                },
            },
        },
    ]
    "#);
}

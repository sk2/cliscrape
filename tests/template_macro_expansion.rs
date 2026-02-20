use std::collections::HashMap;

use cliscrape::engine::{Action, Rule, State, Template, TemplateIR, Value};

#[test]
fn test_template_from_ir_expands_macros_in_rule_regex() {
    let mut values = HashMap::new();
    values.insert(
        "Mac".to_string(),
        Value {
            name: "Mac".to_string(),
            // Not used in this test, but Template::from_ir expects a values map.
            regex: r#"\S+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
            type_hint: None,
        },
    );

    let mut states = HashMap::new();
    states.insert(
        "Start".to_string(),
        State {
            name: "Start".to_string(),
            rules: vec![Rule {
                regex: r#"MAC (?P<Mac>{{mac_address}})"#.to_string(),
                line_action: Action::Next,
                record_action: Action::Record,
                next_state: None,
            }],
        },
    );

    let ir = TemplateIR {
        values,
        states,
        macros: HashMap::new(),
    };

    let template = Template::from_ir(ir).unwrap();
    let compiled = &template.states["Start"][0].regex;

    // Prove macro expansion occurred before regex compilation.
    assert!(!compiled.as_str().contains("{{mac_address}}"));

    let input = "MAC aa:bb:cc:dd:ee:ff";
    let results = template.parse(input).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["Mac"], "aa:bb:cc:dd:ee:ff");
}

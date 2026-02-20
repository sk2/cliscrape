use crate::engine::macros::expand_macros;
use crate::engine::records::RecordBuffer;
use crate::engine::types::*;
use crate::engine::{convert::convert_scalar, debug::*};
use crate::ScraperError;
use regex::Regex;
use std::collections::HashMap;

impl Template {
    pub fn from_ir(ir: TemplateIR) -> Result<Self, ScraperError> {
        let mut compiled_states = HashMap::new();

        for (state_name, state) in &ir.states {
            let mut compiled_rules = Vec::new();
            for rule in &state.rules {
                // 1. Expand macros {{name}}
                let expanded_macros = expand_macros(&rule.regex, &ir.macros).map_err(|e| {
                    let msg = match e {
                        ScraperError::Parse(m) => m,
                        other => other.to_string(),
                    };
                    ScraperError::Parse(format!(
                        "Macro expansion error in state '{}': {}",
                        state_name, msg
                    ))
                })?;

                // 2. Expand values ${ValueName}
                let mut final_regex_str = expanded_macros;
                for (val_name, val) in &ir.values {
                    let placeholder = format!("${{{}}}", val_name);
                    let replacement = format!("(?P<{}>{})", val_name, val.regex);
                    final_regex_str = final_regex_str.replace(&placeholder, &replacement);
                }

                // 3. Compile regex
                let regex = Regex::new(&final_regex_str).map_err(|e| {
                    ScraperError::Parse(format!(
                        "Invalid regex '{}' in state '{}': {}",
                        final_regex_str, state_name, e
                    ))
                })?;

                // 4. Validate next_state
                if let Some(ref next) = rule.next_state {
                    if !ir.states.contains_key(next) && next != "End" {
                        return Err(ScraperError::Parse(format!(
                            "State '{}' transitions to unknown state '{}'",
                            state_name, next
                        )));
                    }
                }

                compiled_rules.push(CompiledRule {
                    regex,
                    line_action: rule.line_action.clone(),
                    record_action: rule.record_action.clone(),
                    next_state: rule.next_state.clone(),
                });
            }
            compiled_states.insert(state_name.clone(), compiled_rules);
        }

        // Validate that "Start" state exists
        if !compiled_states.contains_key("Start") {
            return Err(ScraperError::Parse(
                "Template missing 'Start' state".to_string(),
            ));
        }

        Ok(Template {
            states: compiled_states,
            values: ir.values,
        })
    }

    fn parse_internal(
        &self,
        input: &str,
        mut debug: Option<&mut DebugReport>,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>, ScraperError> {
        let mut current_state = "Start".to_string();
        let mut results = Vec::new();
        let mut record_buffer = RecordBuffer::new();

        let want_debug = debug.is_some();

        let lines: Vec<&str> = input.lines().collect();
        let mut line_idx = 0;

        while line_idx < lines.len() {
            let line = lines[line_idx];
            let mut rule_idx = 0;

            loop {
                let rules = self.states.get(&current_state).ok_or_else(|| {
                    ScraperError::Parse(format!("Entered invalid state: {}", current_state))
                })?;

                if rule_idx >= rules.len() {
                    line_idx += 1;
                    break;
                }

                let rule = &rules[rule_idx];
                if let Some(caps) = rule.regex.captures(line) {
                    let prev_state = current_state.clone();

                    let mut capture_spans: Vec<CaptureSpan> = Vec::new();

                    // Capture named groups into record buffer (and spans in debug mode)
                    for name in rule.regex.capture_names().flatten() {
                        if let Some(m) = caps.name(name) {
                            let def = self.values.get(name);
                            let is_list = def.map(|v| v.list).unwrap_or(false);
                            record_buffer.insert(name.to_string(), m.as_str().to_string(), is_list);

                            if want_debug {
                                let typed =
                                    convert_scalar(m.as_str(), def.and_then(|v| v.type_hint));
                                capture_spans.push(CaptureSpan {
                                    name: name.to_string(),
                                    start_byte: m.start(),
                                    end_byte: m.end(),
                                    raw: m.as_str().to_string(),
                                    typed,
                                    is_list,
                                });
                            }
                        }
                    }

                    // Handle record action
                    match rule.record_action {
                        Action::Record => {
                            if let Some(record) = record_buffer.emit(&self.values) {
                                if want_debug {
                                    if let Some(d) = debug.as_mut() {
                                        d.records.push(EmittedRecord {
                                            line_idx,
                                            record: record.clone(),
                                        });
                                    }
                                }
                                results.push(record);
                            }
                        }
                        Action::Clear => {
                            record_buffer.clear();
                        }
                        _ => {}
                    }

                    // Handle next state
                    let mut state_after = prev_state.clone();
                    if let Some(ref next) = rule.next_state {
                        if next == "End" {
                            state_after = "End".to_string();
                        } else {
                            current_state = next.clone();
                            state_after = current_state.clone();
                        }
                    }

                    // Record successful match for this line before advancing
                    if want_debug {
                        if let Some(d) = debug.as_mut() {
                            if let Some(matches) = d.matches_by_line.get_mut(line_idx) {
                                matches.push(LineMatch {
                                    line_idx,
                                    state_before: prev_state.clone(),
                                    state_after: state_after.clone(),
                                    rule_idx,
                                    line_action: format!("{:?}", rule.line_action),
                                    record_action: format!("{:?}", rule.record_action),
                                    next_state: rule.next_state.clone(),
                                    captures: capture_spans,
                                });
                            }
                        }
                    }

                    if rule.next_state.as_deref() == Some("End") {
                        return Ok(results);
                    }

                    // Handle line action
                    if rule.line_action == Action::Continue {
                        // Move to next rule. If state changed, restart from 0
                        if current_state != prev_state {
                            rule_idx = 0;
                        } else {
                            rule_idx += 1;
                        }
                        continue;
                    } else {
                        // Default is Next: move to next line
                        line_idx += 1;
                        break;
                    }
                } else {
                    // No match, try next rule
                    rule_idx += 1;
                }
            }
        }

        // Implicit Record on EOF
        if let Some(record) = record_buffer.emit(&self.values) {
            if want_debug {
                if let Some(d) = debug.as_mut() {
                    d.records.push(EmittedRecord {
                        line_idx: lines.len(),
                        record: record.clone(),
                    });
                }
            }
            results.push(record);
        }

        Ok(results)
    }

    pub fn parse(
        &self,
        input: &str,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>, ScraperError> {
        self.parse_internal(input, None)
    }

    pub fn debug_parse(&self, input: &str) -> Result<DebugReport, ScraperError> {
        let lines: Vec<String> = input.lines().map(|s| s.to_string()).collect();
        let mut report = DebugReport::new(lines);
        let _ = self.parse_internal(input, Some(&mut report))?;
        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_expansion() {
        let mut values = HashMap::new();
        values.insert(
            "Interface".to_string(),
            Value {
                name: "Interface".to_string(),
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
                    regex: r#"Interface ${Interface}"#.to_string(),
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
        let rule = &template.states["Start"][0];

        // Check if regex contains named capture group
        assert!(rule.regex.as_str().contains(r"(?P<Interface>\S+)"));

        let input = "Interface GigabitEthernet0/1";
        let results = template.parse(input).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["Interface"], "GigabitEthernet0/1");
    }

    #[test]
    fn test_continue_action() {
        let mut values = HashMap::new();
        values.insert(
            "Vlan".to_string(),
            Value {
                name: "Vlan".to_string(),
                regex: r#"\d+"#.to_string(),
                filldown: false,
                required: false,
                list: false,
                type_hint: None,
            },
        );
        values.insert(
            "Status".to_string(),
            Value {
                name: "Status".to_string(),
                regex: r#"\w+"#.to_string(),
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
                rules: vec![
                    Rule {
                        regex: r#"VLAN ${Vlan}"#.to_string(),
                        line_action: Action::Continue,
                        record_action: Action::Next, // Acts as NoRecord
                        next_state: None,
                    },
                    Rule {
                        regex: r#"is ${Status}"#.to_string(),
                        line_action: Action::Next,
                        record_action: Action::Record,
                        next_state: None,
                    },
                ],
            },
        );

        let ir = TemplateIR {
            values,
            states,
            macros: HashMap::new(),
        };

        let template = Template::from_ir(ir).unwrap();
        let input = "VLAN 10 is up";
        let results = template.parse(input).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["Vlan"],
            serde_json::Value::Number(serde_json::Number::from(10_i64))
        );
        assert_eq!(results[0]["Status"], "up");
    }

    fn build_continue_template() -> Template {
        let mut values = HashMap::new();
        values.insert(
            "Vlan".to_string(),
            Value {
                name: "Vlan".to_string(),
                regex: r#"\d+"#.to_string(),
                filldown: false,
                required: false,
                list: false,
                type_hint: None,
            },
        );
        values.insert(
            "Status".to_string(),
            Value {
                name: "Status".to_string(),
                regex: r#"\w+"#.to_string(),
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
                rules: vec![
                    Rule {
                        regex: r#"VLAN ${Vlan}"#.to_string(),
                        line_action: Action::Continue,
                        record_action: Action::Next,
                        next_state: None,
                    },
                    Rule {
                        regex: r#"is ${Status}"#.to_string(),
                        line_action: Action::Next,
                        record_action: Action::Record,
                        next_state: None,
                    },
                ],
            },
        );

        let ir = TemplateIR {
            values,
            states,
            macros: HashMap::new(),
        };

        Template::from_ir(ir).unwrap()
    }

    #[test]
    fn debug_parse_records_continue_stacking_per_line() {
        let template = build_continue_template();
        let input = "VLAN 10 is up";
        let report = template.debug_parse(input).unwrap();

        assert_eq!(report.lines.len(), 1);
        assert_eq!(report.matches_by_line.len(), 1);
        assert_eq!(report.matches_by_line[0].len(), 2);
        assert_eq!(report.matches_by_line[0][0].rule_idx, 0);
        assert_eq!(report.matches_by_line[0][1].rule_idx, 1);
    }

    #[test]
    fn debug_parse_capture_spans_slice_back_to_raw() {
        let template = build_continue_template();
        let input = "VLAN 10 is up";
        let report = template.debug_parse(input).unwrap();

        let line = &report.lines[0];
        let first = &report.matches_by_line[0][0];
        let cap = first
            .captures
            .iter()
            .find(|c| c.name == "Vlan")
            .expect("Vlan capture");

        assert_eq!(&line[cap.start_byte..cap.end_byte], cap.raw);
    }

    #[test]
    fn debug_parse_emitted_records_match_parse_output() {
        let template = build_continue_template();
        let input = "VLAN 10 is up";

        let parsed = template.parse(input).unwrap();
        let report = template.debug_parse(input).unwrap();
        let emitted: Vec<_> = report.records.into_iter().map(|r| r.record).collect();

        assert_eq!(emitted, parsed);
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut states = HashMap::new();
        states.insert(
            "Start".to_string(),
            State {
                name: "Start".to_string(),
                rules: vec![Rule {
                    regex: "test".to_string(),
                    line_action: Action::Next,
                    record_action: Action::Record,
                    next_state: Some("Invalid".to_string()),
                }],
            },
        );

        let ir = TemplateIR {
            values: HashMap::new(),
            states,
            macros: HashMap::new(),
        };

        let result = Template::from_ir(ir);
        assert!(result.is_err());
    }

    #[test]
    fn test_state_transition_start_to_state2() {
        let mut values = HashMap::new();
        values.insert(
            "A".to_string(),
            Value {
                name: "A".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: false,
                type_hint: None,
            },
        );
        values.insert(
            "B".to_string(),
            Value {
                name: "B".to_string(),
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
                    regex: r#"A ${A}"#.to_string(),
                    line_action: Action::Next,
                    record_action: Action::Next,
                    next_state: Some("STATE2".to_string()),
                }],
            },
        );
        states.insert(
            "STATE2".to_string(),
            State {
                name: "STATE2".to_string(),
                rules: vec![Rule {
                    regex: r#"B ${B}"#.to_string(),
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
        let input = "A first\nB second";
        let results = template.parse(input).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["A"], "first");
        assert_eq!(results[0]["B"], "second");
    }

    #[test]
    fn test_end_state_terminates_parse() {
        let mut values = HashMap::new();
        values.insert(
            "X".to_string(),
            Value {
                name: "X".to_string(),
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
                    regex: r#"X ${X}"#.to_string(),
                    line_action: Action::Next,
                    record_action: Action::Record,
                    next_state: Some("End".to_string()),
                }],
            },
        );

        let ir = TemplateIR {
            values,
            states,
            macros: HashMap::new(),
        };

        let template = Template::from_ir(ir).unwrap();
        let input = "X one\nX two\nX three";
        let results = template.parse(input).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["X"], "one");
    }

    #[test]
    fn test_filldown() {
        let mut values = HashMap::new();
        values.insert(
            "Chassis".to_string(),
            Value {
                name: "Chassis".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: true,
                required: false,
                list: false,
                type_hint: None,
            },
        );
        values.insert(
            "Slot".to_string(),
            Value {
                name: "Slot".to_string(),
                regex: r#"\d+"#.to_string(),
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
                rules: vec![
                    Rule {
                        regex: r#"Chassis ${Chassis}"#.to_string(),
                        line_action: Action::Next,
                        record_action: Action::Next,
                        next_state: None,
                    },
                    Rule {
                        regex: r#"Slot ${Slot}"#.to_string(),
                        line_action: Action::Next,
                        record_action: Action::Record,
                        next_state: None,
                    },
                ],
            },
        );

        let ir = TemplateIR {
            values,
            states,
            macros: HashMap::new(),
        };

        let template = Template::from_ir(ir).unwrap();
        let input = "Chassis Router1\nSlot 1\nSlot 2";
        let results = template.parse(input).unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0]["Chassis"], "Router1");
        assert_eq!(
            results[0]["Slot"],
            serde_json::Value::Number(serde_json::Number::from(1_i64))
        );
        assert_eq!(results[1]["Chassis"], "Router1");
        assert_eq!(
            results[1]["Slot"],
            serde_json::Value::Number(serde_json::Number::from(2_i64))
        );
    }

    #[test]
    fn test_required() {
        let mut values = HashMap::new();
        values.insert(
            "Interface".to_string(),
            Value {
                name: "Interface".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: true,
                list: false,
                type_hint: None,
            },
        );
        values.insert(
            "IP".to_string(),
            Value {
                name: "IP".to_string(),
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
                rules: vec![
                    Rule {
                        regex: r#"Interface ${Interface}"#.to_string(),
                        line_action: Action::Continue,
                        record_action: Action::Next,
                        next_state: None,
                    },
                    Rule {
                        regex: r#"IP ${IP}"#.to_string(),
                        line_action: Action::Next,
                        record_action: Action::Record,
                        next_state: None,
                    },
                    Rule {
                        regex: r#"NO_INTERFACE"#.to_string(),
                        line_action: Action::Next,
                        record_action: Action::Record,
                        next_state: None,
                    },
                ],
            },
        );

        let ir = TemplateIR {
            values,
            states,
            macros: HashMap::new(),
        };

        let template = Template::from_ir(ir).unwrap();
        let input = "Interface Eth1 IP 1.1.1.1\nNO_INTERFACE";
        let results = template.parse(input).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["Interface"], "Eth1");
        assert_eq!(results[0]["IP"], "1.1.1.1");
    }

    #[test]
    fn test_eof_record() {
        let mut values = HashMap::new();
        values.insert(
            "Value".to_string(),
            Value {
                name: "Value".to_string(),
                regex: r#"\w+"#.to_string(),
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
                    regex: r#"Set ${Value}"#.to_string(),
                    line_action: Action::Next,
                    record_action: Action::Next, // NoRecord
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
        let input = "Set Data";
        let results = template.parse(input).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["Value"], "Data");
    }

    #[test]
    fn test_list_support() {
        let mut values = HashMap::new();
        values.insert(
            "Inter".to_string(),
            Value {
                name: "Inter".to_string(),
                regex: r#"\S+"#.to_string(),
                filldown: false,
                required: false,
                list: true,
                type_hint: None,
            },
        );

        let mut states = HashMap::new();
        states.insert(
            "Start".to_string(),
            State {
                name: "Start".to_string(),
                rules: vec![Rule {
                    regex: r#"Interface ${Inter}"#.to_string(),
                    line_action: Action::Next,
                    record_action: Action::Next,
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
        let input = "Interface Eth1\nInterface Eth2";
        let results = template.parse(input).unwrap();

        assert_eq!(results.len(), 1);
        assert!(results[0]["Inter"].is_array());
        let arr = results[0]["Inter"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0], "Eth1");
        assert_eq!(arr[1], "Eth2");
    }
}

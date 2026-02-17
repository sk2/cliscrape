use std::collections::HashMap;
use crate::engine::types::*;
use crate::engine::macros::expand_macros;
use crate::engine::records::RecordBuffer;
use crate::ScraperError;
use regex::Regex;

impl Template {
    pub fn from_ir(ir: TemplateIR) -> Result<Self, ScraperError> {
        let mut compiled_states = HashMap::new();
        
        for (state_name, state) in &ir.states {
            let mut compiled_rules = Vec::new();
            for rule in &state.rules {
                // 1. Expand macros {{name}}
                let expanded_macros = expand_macros(&rule.regex, &ir.macros);
                
                // 2. Expand values ${ValueName}
                let mut final_regex_str = expanded_macros;
                for (val_name, val) in &ir.values {
                    let placeholder = format!("${{{}}}", val_name);
                    let replacement = format!("(?P<{}>{})", val_name, val.regex);
                    final_regex_str = final_regex_str.replace(&placeholder, &replacement);
                }
                
                // 3. Compile regex
                let regex = Regex::new(&final_regex_str)
                    .map_err(|e| ScraperError::Parse(format!("Invalid regex '{}' in state '{}': {}", final_regex_str, state_name, e)))?;
                
                // 4. Validate next_state
                if let Some(ref next) = rule.next_state {
                    if !ir.states.contains_key(next) && next != "End" {
                        return Err(ScraperError::Parse(format!("State '{}' transitions to unknown state '{}'", state_name, next)));
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
             return Err(ScraperError::Parse("Template missing 'Start' state".to_string()));
        }
        
        Ok(Template {
            states: compiled_states,
            values: ir.values,
        })
    }

    pub fn parse(&self, input: &str) -> Result<Vec<HashMap<String, serde_json::Value>>, ScraperError> {
        let mut current_state = "Start".to_string();
        let mut results = Vec::new();
        let mut record_buffer = RecordBuffer::new();
        
        let lines: Vec<&str> = input.lines().collect();
        let mut line_idx = 0;
        
        while line_idx < lines.len() {
            let line = lines[line_idx];
            let mut rule_idx = 0;
            
            loop {
                let rules = self.states.get(&current_state)
                    .ok_or_else(|| ScraperError::Parse(format!("Entered invalid state: {}", current_state)))?;
                
                if rule_idx >= rules.len() {
                    line_idx += 1;
                    break;
                }

                let rule = &rules[rule_idx];
                if let Some(caps) = rule.regex.captures(line) {
                    
                    // Capture named groups into current_record
                    for name in rule.regex.capture_names().flatten() {
                        if let Some(m) = caps.name(name) {
                            let is_list = self.values.get(name).map(|v| v.list).unwrap_or(false);
                            record_buffer.insert(name.to_string(), m.as_str().to_string(), is_list);
                        }
                    }
                    
                    // Handle record action
                    match rule.record_action {
                        Action::Record => {
                            if let Some(record) = record_buffer.emit(&self.values) {
                                results.push(record);
                            }
                        }
                        Action::Clear => {
                            record_buffer.clear();
                        }
                        _ => {}
                    }
                    
                    let prev_state = current_state.clone();
                    // Handle next state
                    if let Some(ref next) = rule.next_state {
                        if next == "End" {
                            return Ok(results);
                        }
                        current_state = next.clone();
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
            results.push(record);
        }
        
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_expansion() {
        let mut values = HashMap::new();
        values.insert("Interface".to_string(), Value {
            name: "Interface".to_string(),
            regex: r#"\S+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
        });

        let mut states = HashMap::new();
        states.insert("Start".to_string(), State {
            name: "Start".to_string(),
            rules: vec![Rule {
                regex: r#"Interface ${Interface}"#.to_string(),
                line_action: Action::Next,
                record_action: Action::Record,
                next_state: None,
            }],
        });

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
        values.insert("Vlan".to_string(), Value {
            name: "Vlan".to_string(),
            regex: r#"\d+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
        });
        values.insert("Status".to_string(), Value {
            name: "Status".to_string(),
            regex: r#"\w+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
        });

        let mut states = HashMap::new();
        states.insert("Start".to_string(), State {
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
                }
            ],
        });

        let ir = TemplateIR {
            values,
            states,
            macros: HashMap::new(),
        };

        let template = Template::from_ir(ir).unwrap();
        let input = "VLAN 10 is up";
        let results = template.parse(input).unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["Vlan"], "10");
        assert_eq!(results[0]["Status"], "up");
    }

    #[test]
    fn test_invalid_state_transition() {
        let mut states = HashMap::new();
        states.insert("Start".to_string(), State {
            name: "Start".to_string(),
            rules: vec![Rule {
                regex: "test".to_string(),
                line_action: Action::Next,
                record_action: Action::Record,
                next_state: Some("Invalid".to_string()),
            }],
        });

        let ir = TemplateIR {
            values: HashMap::new(),
            states,
            macros: HashMap::new(),
        };

        let result = Template::from_ir(ir);
        assert!(result.is_err());
    }

    #[test]
    fn test_filldown() {
        let mut values = HashMap::new();
        values.insert("Chassis".to_string(), Value {
            name: "Chassis".to_string(),
            regex: r#"\S+"#.to_string(),
            filldown: true,
            required: false,
            list: false,
        });
        values.insert("Slot".to_string(), Value {
            name: "Slot".to_string(),
            regex: r#"\d+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
        });

        let mut states = HashMap::new();
        states.insert("Start".to_string(), State {
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
        });

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
        assert_eq!(results[0]["Slot"], "1");
        assert_eq!(results[1]["Chassis"], "Router1");
        assert_eq!(results[1]["Slot"], "2");
    }

    #[test]
    fn test_required() {
        let mut values = HashMap::new();
        values.insert("Interface".to_string(), Value {
            name: "Interface".to_string(),
            regex: r#"\S+"#.to_string(),
            filldown: false,
            required: true,
            list: false,
        });
        values.insert("IP".to_string(), Value {
            name: "IP".to_string(),
            regex: r#"\S+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
        });

        let mut states = HashMap::new();
        states.insert("Start".to_string(), State {
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
                }
            ],
        });

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
        values.insert("Value".to_string(), Value {
            name: "Value".to_string(),
            regex: r#"\w+"#.to_string(),
            filldown: false,
            required: false,
            list: false,
        });

        let mut states = HashMap::new();
        states.insert("Start".to_string(), State {
            name: "Start".to_string(),
            rules: vec![
                Rule {
                    regex: r#"Set ${Value}"#.to_string(),
                    line_action: Action::Next,
                    record_action: Action::Next, // NoRecord
                    next_state: None,
                }
            ],
        });

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
        values.insert("Inter".to_string(), Value {
            name: "Inter".to_string(),
            regex: r#"\S+"#.to_string(),
            filldown: false,
            required: false,
            list: true,
        });

        let mut states = HashMap::new();
        states.insert("Start".to_string(), State {
            name: "Start".to_string(),
            rules: vec![
                Rule {
                    regex: r#"Interface ${Inter}"#.to_string(),
                    line_action: Action::Next,
                    record_action: Action::Next,
                    next_state: None,
                }
            ],
        });

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

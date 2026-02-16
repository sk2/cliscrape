use std::collections::HashMap;
use crate::engine::types::*;
use crate::engine::macros::expand_macros;
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

    pub fn parse(&self, input: &str) -> Result<Vec<HashMap<String, String>>, ScraperError> {
        let mut current_state = "Start".to_string();
        let mut results = Vec::new();
        let mut current_record = HashMap::new();
        
        let lines: Vec<&str> = input.lines().collect();
        let mut line_idx = 0;
        
        while line_idx < lines.len() {
            let line = lines[line_idx];
            let mut consumed = false;
            
            // TextFSM-like loop for Continue action
            loop {
                let rules = self.states.get(&current_state)
                    .ok_or_else(|| ScraperError::Parse(format!("Entered invalid state: {}", current_state)))?;
                
                let mut matched_in_this_pass = false;
                for rule in rules {
                    if let Some(caps) = rule.regex.captures(line) {
                        matched_in_this_pass = true;
                        
                        // Capture named groups into current_record
                        for name in rule.regex.capture_names().flatten() {
                            if let Some(m) = caps.name(name) {
                                current_record.insert(name.to_string(), m.as_str().to_string());
                            }
                        }
                        
                        // Handle record action
                        match rule.record_action {
                            Action::Record => {
                                results.push(current_record.clone());
                                // Clear non-filldown values
                                let mut next_record = HashMap::new();
                                for (name, val) in &self.values {
                                    if val.filldown {
                                        if let Some(v) = current_record.get(name) {
                                            next_record.insert(name.clone(), v.clone());
                                        }
                                    }
                                }
                                current_record = next_record;
                            }
                            Action::Clear => {
                                current_record.clear();
                            }
                            _ => {}
                        }
                        
                        // Handle next state
                        if let Some(ref next) = rule.next_state {
                            if next == "End" {
                                return Ok(results);
                            }
                            current_state = next.clone();
                        }
                        
                        // Handle line action
                        if rule.line_action == Action::Continue {
                            // Try rules again (possibly in new state) on same line
                            // But we need to break the inner rule loop to re-fetch rules for current_state
                            break; 
                        } else {
                            // Default is Next: move to next line
                            line_idx += 1;
                            consumed = true;
                            break;
                        }
                    }
                }
                
                if !matched_in_this_pass {
                    // No more rules matched on this line
                    if !consumed {
                        line_idx += 1;
                    }
                    break; // break the loop for this line
                } else if consumed {
                    break; // break the loop for this line if it was consumed
                }
                // If it matched but wasn't consumed (Action::Continue), it continues to the next iteration of the inner loop
            }
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
        });
        values.insert("Status".to_string(), Value {
            name: "Status".to_string(),
            regex: r#"\w+"#.to_string(),
            filldown: false,
            required: false,
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

        // Use Action::Next as dummy for NoRecord if needed, but in our logic we only Record if it's Action::Record
        // Let's add NoRecord to Action enum? Plan didn't say so.
        // Actually, current RecordAction handling:
        // match rule.record_action { Action::Record => ..., Action::Clear => ..., _ => {} }
        // So any other Action (Next, Continue) acts as "NoRecord".

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
}

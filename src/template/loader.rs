use crate::engine::types::*;
use crate::template::{Rule as PestRule, TextFsmParser};
use crate::{ScraperError, TemplateWarning};
use pest::iterators::Pair;
use pest::Parser;
use std::collections::HashMap;

pub struct TextFsmLoader;

impl TextFsmLoader {
    pub fn parse_str(input: &str) -> Result<TemplateIR, ScraperError> {
        let (ir, _warnings) = Self::parse_str_with_warnings(input)?;
        Ok(ir)
    }

    pub fn parse_str_with_warnings(
        input: &str,
    ) -> Result<(TemplateIR, Vec<TemplateWarning>), ScraperError> {
        let mut pairs = TextFsmParser::parse(PestRule::file, input)
            .map_err(|e| ScraperError::Parse(format!("Pest error: {}", e)))?;

        let mut values = HashMap::new();
        let mut states = HashMap::new();
        let mut warnings = Vec::new();

        let file_pair = pairs.next().unwrap();
        for pair in file_pair.into_inner() {
            match pair.as_rule() {
                PestRule::val_def => {
                    let (value, val_warnings) = self::parse_definition_with_warnings(pair)?;
                    warnings.extend(val_warnings);
                    values.insert(value.name.clone(), value);
                }
                PestRule::state_block => {
                    let (state, state_warnings) = self::parse_state_block_with_warnings(pair)?;
                    warnings.extend(state_warnings);
                    states.insert(state.name.clone(), state);
                }
                PestRule::EOI => {}
                _ => {}
            }
        }

        Ok((
            TemplateIR {
                values,
                states,
                macros: HashMap::new(),
            },
            warnings,
        ))
    }
}

fn parse_definition(pair: Pair<PestRule>) -> Result<Value, ScraperError> {
    let (value, _warnings) = parse_definition_with_warnings(pair)?;
    Ok(value)
}

fn parse_definition_with_warnings(
    pair: Pair<PestRule>,
) -> Result<(Value, Vec<TemplateWarning>), ScraperError> {
    let mut name = String::new();
    let mut regex = String::new();
    let mut filldown = false;
    let mut required = false;
    let mut list = false;
    let mut warnings = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::value_tokens => {
                let tokens: Vec<&str> = inner.into_inner().map(|t| t.as_str()).collect();
                // Last token is the name, rest are flags
                if !tokens.is_empty() {
                    name = tokens.last().unwrap().to_string();
                    for flag in &tokens[..tokens.len() - 1] {
                        match *flag {
                            "Filldown" => filldown = true,
                            "Required" => required = true,
                            "List" => list = true,
                            _ => {
                                // Unknown flag: warn and ignore
                                warnings.push(TemplateWarning {
                                    kind: "unknown_value_flag".to_string(),
                                    message: format!(
                                        "Unknown Value flag '{}' on field '{}' - ignoring",
                                        flag, name
                                    ),
                                });
                            }
                        }
                    }
                }
            }
            PestRule::regex => regex = inner.as_str().to_string(),
            _ => {}
        }
    }

    Ok((
        Value {
            name,
            regex,
            filldown,
            required,
            list,
            type_hint: None,
        },
        warnings,
    ))
}

fn parse_state_block(pair: Pair<PestRule>) -> Result<State, ScraperError> {
    let (state, _warnings) = parse_state_block_with_warnings(pair)?;
    Ok(state)
}

fn parse_state_block_with_warnings(
    pair: Pair<PestRule>,
) -> Result<(State, Vec<TemplateWarning>), ScraperError> {
    let mut name = String::new();
    let mut rules = Vec::new();
    let mut warnings = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::state_name => name = inner.as_str().to_string(),
            PestRule::fsm_rule => {
                let (rule, rule_warnings) = parse_rule_with_warnings(inner)?;
                warnings.extend(rule_warnings);
                if let Some(r) = rule {
                    rules.push(r);
                }
            }
            _ => {}
        }
    }

    Ok((State { name, rules }, warnings))
}

fn parse_rule(pair: Pair<PestRule>) -> Result<Rule, ScraperError> {
    let (rule, _warnings) = parse_rule_with_warnings(pair)?;
    Ok(rule.expect("parse_rule should return Some"))
}

fn parse_rule_with_warnings(
    pair: Pair<PestRule>,
) -> Result<(Option<Rule>, Vec<TemplateWarning>), ScraperError> {
    let mut regex = String::new();
    let mut line_action = Action::Next;
    let mut record_action = Action::Next; // Next acts as NoRecord in our engine
    let mut next_state = None;
    let mut warnings = Vec::new();
    let mut skip_rule = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::rule_regex => regex = inner.as_str().trim_end().to_string(),
            PestRule::action => {
                let (la, ra, ns, action_warnings, skip) = parse_action_with_warnings(inner)?;
                warnings.extend(action_warnings);
                if skip {
                    skip_rule = true;
                } else {
                    line_action = la;
                    record_action = ra;
                    next_state = ns;
                }
            }
            _ => {}
        }
    }

    if skip_rule {
        Ok((None, warnings))
    } else {
        Ok((
            Some(Rule {
                regex,
                line_action,
                record_action,
                next_state,
            }),
            warnings,
        ))
    }
}

fn parse_action(pair: Pair<PestRule>) -> Result<(Action, Action, Option<String>), ScraperError> {
    let (la, ra, ns, _warnings, _skip) = parse_action_with_warnings(pair)?;
    Ok((la, ra, ns))
}

fn parse_action_with_warnings(
    pair: Pair<PestRule>,
) -> Result<(Action, Action, Option<String>, Vec<TemplateWarning>, bool), ScraperError> {
    let mut line_action = Action::Next;
    let mut record_action = Action::Next;
    let mut next_state = None;
    let mut warnings = Vec::new();
    let mut skip_rule = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::line_action => {
                let action_str = inner.as_str();
                // Check if this is actually a record action keyword that was misparsed as line_action
                if matches!(action_str, "Record" | "Clear" | "Clearall" | "Error" | "NoRecord") {
                    // This was misparsed - treat it as a record action
                    record_action = match action_str {
                        "Record" => Action::Record,
                        "Clear" => Action::Clear,
                        "Clearall" => Action::ClearAll,
                        "Error" => Action::Error,
                        "NoRecord" => Action::Next,
                        _ => unreachable!(),
                    };
                } else if matches!(action_str, "Continue" | "Next") {
                    line_action = match action_str {
                        "Continue" => Action::Continue,
                        "Next" => Action::Next,
                        _ => unreachable!(),
                    };
                } else {
                    // This looks like a state name that was misparsed as line_action
                    // Treat it as next_state instead
                    next_state = Some(action_str.to_string());
                }
            }
            PestRule::record_action => {
                let action_str = inner.as_str();
                record_action = match action_str {
                    "Record" => Action::Record,
                    "Clear" => Action::Clear,
                    "Clearall" => Action::ClearAll,
                    "Error" => Action::Error,
                    "NoRecord" => Action::Next,
                    _ => {
                        // Unknown record action: warn and skip this rule
                        warnings.push(TemplateWarning {
                            kind: "unknown_record_action".to_string(),
                            message: format!(
                                "Unknown record action '{}' - skipping rule",
                                action_str
                            ),
                        });
                        skip_rule = true;
                        Action::Next
                    }
                };
            }
            PestRule::next_state => {
                next_state = Some(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok((line_action, record_action, next_state, warnings, skip_rule))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_template() {
        let input = r#"Value INTERFACE (\S+)
Value STATUS (up|down)

Start
  ^Interface ${INTERFACE} is ${STATUS} -> Record
"#;
        let ir = TextFsmLoader::parse_str(input).unwrap();
        assert_eq!(ir.values.len(), 2);
        assert!(ir.values.contains_key("INTERFACE"));
        assert!(ir.states.contains_key("Start"));

        let start_state = &ir.states["Start"];
        assert_eq!(start_state.rules.len(), 1);
        assert_eq!(start_state.rules[0].record_action, Action::Record);
    }

    #[test]
    fn test_complex_actions() {
        let input = r#"Start
  ^rule1 -> Continue.Record NextState
  ^rule2 -> Clear
  ^rule3 -> NextState
"#;
        let ir = TextFsmLoader::parse_str(input).unwrap();
        let rules = &ir.states["Start"].rules;

        assert_eq!(rules[0].line_action, Action::Continue);
        assert_eq!(rules[0].record_action, Action::Record);
        assert_eq!(rules[0].next_state, Some("NextState".to_string()));

        assert_eq!(rules[1].record_action, Action::Clear);

        assert_eq!(rules[2].next_state, Some("NextState".to_string()));
        assert_eq!(rules[2].line_action, Action::Next);
    }
}

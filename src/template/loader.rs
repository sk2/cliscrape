use pest::Parser;
use pest::iterators::Pair;
use crate::template::{TextFsmParser, Rule as PestRule};
use crate::engine::types::*;
use crate::ScraperError;
use std::collections::HashMap;

pub struct TextFsmLoader;

impl TextFsmLoader {
    pub fn parse_str(input: &str) -> Result<TemplateIR, ScraperError> {
        let mut pairs = TextFsmParser::parse(PestRule::file, input)
            .map_err(|e| ScraperError::Parse(format!("Pest error: {}", e)))?;

        let mut values = HashMap::new();
        let mut states = HashMap::new();

        let file_pair = pairs.next().unwrap();
        for pair in file_pair.into_inner() {
            match pair.as_rule() {
                PestRule::definition => {
                    let value = self::parse_definition(pair)?;
                    values.insert(value.name.clone(), value);
                }
                PestRule::state_block => {
                    let state = self::parse_state_block(pair)?;
                    states.insert(state.name.clone(), state);
                }
                PestRule::EOI => {}
                _ => {}
            }
        }

        Ok(TemplateIR {
            values,
            states,
            macros: HashMap::new(),
        })
    }
}

fn parse_definition(pair: Pair<PestRule>) -> Result<Value, ScraperError> {
    let mut name = String::new();
    let mut regex = String::new();
    let mut filldown = false;
    let mut required = false;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::flags => {
                for flag in inner.into_inner() {
                    match flag.as_str() {
                        "Filldown" => filldown = true,
                        "Required" => required = true,
                        _ => {} // Handle others if needed
                    }
                }
            }
            PestRule::name => name = inner.as_str().to_string(),
            PestRule::regex => regex = inner.as_str().to_string(),
            _ => {}
        }
    }

    Ok(Value {
        name,
        regex,
        filldown,
        required,
    })
}

fn parse_state_block(pair: Pair<PestRule>) -> Result<State, ScraperError> {
    let mut name = String::new();
    let mut rules = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::state_name => name = inner.as_str().to_string(),
            PestRule::rule => {
                rules.push(parse_rule(inner)?);
            }
            _ => {}
        }
    }

    Ok(State { name, rules })
}

fn parse_rule(pair: Pair<PestRule>) -> Result<Rule, ScraperError> {
    let mut regex = String::new();
    let mut line_action = Action::Next;
    let mut record_action = Action::Next; // Next acts as NoRecord in our engine
    let mut next_state = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::regex_rule => regex = inner.as_str().trim_end().to_string(),
            PestRule::action => {
                let (la, ra, ns) = parse_action(inner)?;
                line_action = la;
                record_action = ra;
                next_state = ns;
            }
            _ => {}
        }
    }

    Ok(Rule {
        regex,
        line_action,
        record_action,
        next_state,
    })
}

fn parse_action(pair: Pair<PestRule>) -> Result<(Action, Action, Option<String>), ScraperError> {
    let mut line_action = Action::Next;
    let mut record_action = Action::Next;
    let mut next_state = None;

    for inner in pair.into_inner() {
        match inner.as_rule() {
            PestRule::line_action => {
                line_action = match inner.as_str() {
                    "Continue" => Action::Continue,
                    _ => Action::Next,
                };
            }
            PestRule::record_action => {
                record_action = match inner.as_str() {
                    "Record" => Action::Record,
                    "Clear" | "Clearall" => Action::Clear,
                    _ => Action::Next, // NoRecord
                };
            }
            PestRule::next_state => {
                next_state = Some(inner.as_str().to_string());
            }
            _ => {}
        }
    }

    Ok((line_action, record_action, next_state))
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

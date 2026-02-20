use crate::engine::types::{Action, FieldType, Rule, State, TemplateIR, Value};
use crate::ScraperError;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModernFormat {
    Yaml,
    Toml,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModernDoc {
    version: u32,

    #[serde(default)]
    macros: HashMap<String, String>,

    #[serde(default)]
    fields: BTreeMap<String, FieldDef>,

    states: Option<BTreeMap<String, Vec<StateRuleDef>>>,
    patterns: Option<Vec<PatternRuleDef>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FieldDef {
    #[serde(default)]
    r#type: Option<FieldTypeDef>,

    #[serde(default)]
    pattern: Option<String>,

    #[serde(default)]
    filldown: bool,

    #[serde(default)]
    required: bool,

    #[serde(default)]
    list: bool,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum FieldTypeDef {
    Int,
    String,
}

impl From<FieldTypeDef> for FieldType {
    fn from(t: FieldTypeDef) -> Self {
        match t {
            FieldTypeDef::Int => FieldType::Int,
            FieldTypeDef::String => FieldType::String,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StateRuleDef {
    regex: String,

    #[serde(default)]
    action: Option<ActionDef>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ActionDef {
    #[serde(default)]
    line: Option<LineActionDef>,

    #[serde(default)]
    record: Option<RecordActionDef>,

    #[serde(default)]
    next: Option<String>,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum LineActionDef {
    Next,
    Continue,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum RecordActionDef {
    None,
    Record,
    Clear,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PatternRuleDef {
    regex: String,

    #[serde(default)]
    record: bool,
}

pub fn load_str(format: ModernFormat, input: &str) -> Result<TemplateIR, ScraperError> {
    let doc: ModernDoc = match format {
        ModernFormat::Toml => {
            let de = toml::de::Deserializer::parse(input)
                .map_err(|e| ScraperError::Parse(format!("TOML parse error: {e}")))?;
            serde_path_to_error::deserialize(de)
                .map_err(|e| ScraperError::Parse(format!("TOML schema error: {e}")))?
        }
        ModernFormat::Yaml => {
            let de = serde_yaml_ng::Deserializer::from_str(input);
            serde_path_to_error::deserialize(de)
                .map_err(|e| ScraperError::Parse(format!("YAML schema error: {e}")))?
        }
    };

    doc.validate()?;
    doc.lower()
}

pub fn load_toml_str(input: &str) -> Result<TemplateIR, ScraperError> {
    load_str(ModernFormat::Toml, input)
}

pub fn load_yaml_str(input: &str) -> Result<TemplateIR, ScraperError> {
    load_str(ModernFormat::Yaml, input)
}

impl ModernDoc {
    fn validate(&self) -> Result<(), ScraperError> {
        if self.version != 1 {
            return Err(ScraperError::Parse(format!(
                "Unsupported modern template version {} (supported: 1)",
                self.version
            )));
        }

        let has_states = self.states.as_ref().is_some_and(|m| !m.is_empty());
        let has_patterns = self.patterns.as_ref().is_some_and(|v| !v.is_empty());
        match (has_states, has_patterns) {
            (true, false) => {
                let states = self.states.as_ref().unwrap();
                if !states.contains_key("Start") {
                    return Err(ScraperError::Parse(
                        "Modern templates with explicit states must define a 'Start' state"
                            .to_string(),
                    ));
                }
            }
            (false, true) => {}
            (true, true) => {
                return Err(ScraperError::Parse(
                    "Modern template must define exactly one of 'states' or 'patterns'".to_string(),
                ));
            }
            (false, false) => {
                return Err(ScraperError::Parse(
                    "Modern template must define either 'states' or 'patterns'".to_string(),
                ));
            }
        }

        // Field reference validation
        let mut placeholders = HashSet::<String>::new();
        let mut named_groups = HashSet::<String>::new();

        if let Some(states) = &self.states {
            for rules in states.values() {
                for rule in rules {
                    collect_placeholders(&rule.regex, &mut placeholders);
                    collect_named_groups(&rule.regex, &mut named_groups);
                }
            }
        }

        if let Some(patterns) = &self.patterns {
            for p in patterns {
                collect_placeholders(&p.regex, &mut placeholders);
                collect_named_groups(&p.regex, &mut named_groups);
            }
        }

        for name in placeholders.iter() {
            let def = self.fields.get(name).ok_or_else(|| {
                ScraperError::Parse(format!(
                    "Rule references placeholder '${{{}}}' but 'fields.{}' is not defined",
                    name, name
                ))
            })?;
            let missing_pattern = def
                .pattern
                .as_ref()
                .map(|p| p.trim().is_empty())
                .unwrap_or(true);
            if missing_pattern {
                return Err(ScraperError::Parse(format!(
                    "Rule references placeholder '${{{}}}' but 'fields.{}.pattern' is missing",
                    name, name
                )));
            }
        }

        for name in named_groups.iter() {
            if !self.fields.contains_key(name) {
                return Err(ScraperError::Parse(format!(
                    "Rule contains named capture group '{name}' but 'fields.{name}' is not defined"
                )));
            }
        }

        Ok(())
    }

    fn lower(&self) -> Result<TemplateIR, ScraperError> {
        let mut values = HashMap::new();
        for (name, def) in &self.fields {
            let hint = match def.r#type.unwrap_or(FieldTypeDef::String) {
                FieldTypeDef::Int => FieldType::Int,
                FieldTypeDef::String => FieldType::String,
            };

            values.insert(
                name.clone(),
                Value {
                    name: name.clone(),
                    regex: def.pattern.clone().unwrap_or_else(|| r#".*?"#.to_string()),
                    filldown: def.filldown,
                    required: def.required,
                    list: def.list,
                    type_hint: Some(hint),
                },
            );
        }

        let mut states = HashMap::new();
        if let Some(s) = &self.states {
            for (state_name, rule_defs) in s {
                let rules = rule_defs
                    .iter()
                    .map(|rd| rd.lower())
                    .collect::<Result<Vec<_>, _>>()?;
                states.insert(
                    state_name.clone(),
                    State {
                        name: state_name.clone(),
                        rules,
                    },
                );
            }
        } else if let Some(pats) = &self.patterns {
            let mut rules = Vec::new();
            for p in pats {
                rules.push(Rule {
                    regex: p.regex.clone(),
                    line_action: Action::Next,
                    record_action: if p.record {
                        Action::Record
                    } else {
                        Action::Next
                    },
                    next_state: None,
                });
            }
            states.insert(
                "Start".to_string(),
                State {
                    name: "Start".to_string(),
                    rules,
                },
            );
        }

        Ok(TemplateIR {
            values,
            states,
            macros: self.macros.clone(),
        })
    }
}

impl StateRuleDef {
    fn lower(&self) -> Result<Rule, ScraperError> {
        let (line_action, record_action, next_state) = match &self.action {
            Some(a) => {
                let la = match a.line.unwrap_or(LineActionDef::Next) {
                    LineActionDef::Next => Action::Next,
                    LineActionDef::Continue => Action::Continue,
                };
                let ra = match a.record.unwrap_or(RecordActionDef::None) {
                    RecordActionDef::None => Action::Next,
                    RecordActionDef::Record => Action::Record,
                    RecordActionDef::Clear => Action::Clear,
                };
                (la, ra, a.next.clone())
            }
            None => (Action::Next, Action::Next, None),
        };

        Ok(Rule {
            regex: self.regex.clone(),
            line_action,
            record_action,
            next_state,
        })
    }
}

fn collect_placeholders(s: &str, out: &mut HashSet<String>) {
    // Matches ${name} placeholders.
    static PLACEHOLDER_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re = PLACEHOLDER_RE.get_or_init(|| regex::Regex::new(r"\$\{([A-Za-z0-9_]+)\}").unwrap());
    for cap in re.captures_iter(s) {
        if let Some(m) = cap.get(1) {
            out.insert(m.as_str().to_string());
        }
    }
}

fn collect_named_groups(s: &str, out: &mut HashSet<String>) {
    // Matches (?P<name>...) capture groups.
    static NAMED_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
    let re =
        NAMED_RE.get_or_init(|| regex::Regex::new(r"\(\?P<([A-Za-z_][A-Za-z0-9_]*)>").unwrap());
    for cap in re.captures_iter(s) {
        if let Some(m) = cap.get(1) {
            out.insert(m.as_str().to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::types::FieldType;
    use crate::engine::Template;

    #[test]
    fn modern_toml_explicit_int_type_emits_json_number() {
        let doc = r#"
version = 1

[fields]
speed = { type = "int" }

[[patterns]]
regex = '^speed=(?P<speed>[0-9,]+)$'
record = true
"#;

        let ir = load_toml_str(doc).unwrap();
        assert_eq!(
            ir.values.get("speed").unwrap().type_hint,
            Some(FieldType::Int)
        );

        let template = Template::from_ir(ir).unwrap();
        let results = template.parse("speed=1,234").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["speed"],
            serde_json::Value::Number(serde_json::Number::from(1234_i64))
        );
    }

    #[test]
    fn modern_toml_explicit_string_type_overrides_numeric_heuristics() {
        let doc = r#"
version = 1

[fields]
speed = { type = "string" }

[[patterns]]
regex = '^speed=(?P<speed>[0-9,]+)$'
record = true
"#;

        let ir = load_toml_str(doc).unwrap();
        let template = Template::from_ir(ir).unwrap();
        let results = template.parse("speed=1,234").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["speed"],
            serde_json::Value::String("1,234".to_string())
        );
    }

    #[test]
    fn modern_toml_unknown_field_type_fails_with_path() {
        let doc = r#"
version = 1

[fields]
speed = { type = "integer" }

[[patterns]]
regex = '^speed=(?P<speed>[0-9,]+)$'
record = true
"#;

        let err = load_toml_str(doc).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("fields.speed.type"), "{msg}");
    }

    #[test]
    fn modern_local_macro_overrides_builtin_in_compiled_regex() {
        let doc = r#"
version = 1

[macros]
ipv4 = "X"

[fields]
ip = { type = "string" }

[[states.Start]]
regex = '^IP {{ipv4}}$'
"#;

        let ir = load_toml_str(doc).unwrap();
        let template = Template::from_ir(ir).unwrap();
        let compiled = template.states["Start"][0].regex.as_str();
        assert!(compiled.contains("X"), "{compiled}");
        assert!(!compiled.contains("\\d{1,3}"), "{compiled}");
    }

    #[test]
    fn modern_placeholder_requires_field_pattern() {
        let doc = r#"
version = 1

[fields]
iface = { type = "string" }

[[patterns]]
regex = '^Interface ${iface}$'
record = true
"#;

        let err = load_toml_str(doc).unwrap_err();
        assert!(err.to_string().contains("fields.iface.pattern"));
    }

    #[test]
    fn modern_rejects_both_states_and_patterns() {
        let doc = r#"
version = 1

[fields]
ip = { type = "string" }

[[patterns]]
regex = '^IP (?P<ip>\\S+)$'
record = true

[[states.Start]]
regex = '^IP (?P<ip>\\S+)$'
"#;

        let err = load_toml_str(doc).unwrap_err();
        assert!(
            err.to_string()
                .contains("exactly one of 'states' or 'patterns'"),
            "{err}"
        );
    }

    #[test]
    fn modern_rejects_missing_states_and_patterns() {
        let doc = r#"
version = 1

[fields]
ip = { type = "string" }
"#;

        let err = load_toml_str(doc).unwrap_err();
        assert!(
            err.to_string().contains("either 'states' or 'patterns'"),
            "{err}"
        );
    }

    #[test]
    fn modern_rejects_unsupported_version() {
        let doc = r#"
version = 2

[fields]
ip = { type = "string" }

[[patterns]]
regex = '^IP (?P<ip>\\S+)$'
record = true
"#;

        let err = load_toml_str(doc).unwrap_err();
        assert!(
            err.to_string()
                .contains("Unsupported modern template version 2"),
            "{err}"
        );
    }

    #[test]
    fn modern_placeholder_parses_when_field_pattern_defined() {
        let doc = r#"
version = 1

[fields]
iface = { type = "string", pattern = "\\S+" }

[[patterns]]
regex = '^Interface ${iface}$'
record = true
"#;

        let ir = load_toml_str(doc).unwrap();
        let template = Template::from_ir(ir).unwrap();
        let results = template.parse("Interface Eth1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["iface"], "Eth1");
    }

    #[test]
    fn modern_named_capture_groups_work_without_placeholders() {
        let doc = r#"
version = 1

[fields]
hostname = { type = "string" }

[[patterns]]
regex = '^Host=(?P<hostname>\S+)$'
record = true
"#;

        let ir = load_toml_str(doc).unwrap();
        let template = Template::from_ir(ir).unwrap();
        let results = template.parse("Host=Router1").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0]["hostname"], "Router1");
    }
}

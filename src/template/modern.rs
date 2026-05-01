use crate::ScraperError;
use crate::engine::types::{Action, FieldType, Rule, State, TemplateIR, Value};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModernFormat {
    Yaml,
    Toml,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModernTemplateDoc {
    pub version: u32,

    /// Universal Ledger schemas this template claims to satisfy. Accepts a
    /// single string (`claims_schema: interface`) or a list
    /// (`claims_schema: [interface, interface_counters]`).
    ///
    /// When present, the validator restricts bare `common_schema:`
    /// references to the claimed schemas and enforces that every required
    /// key in each claimed schema is mapped by at least one field. When
    /// absent, bare references must resolve unambiguously across the entire
    /// registry — the legacy inference path.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claims_schema: Option<OneOrMany>,

    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub macros: HashMap<String, String>,

    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub fields: BTreeMap<String, FieldDef>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub states: Option<BTreeMap<String, Vec<StateRuleDef>>>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patterns: Option<Vec<PatternRuleDef>>,

    /// Metadata section - parsed separately by metadata module, ignored by template loader
    #[serde(default, skip_serializing)]
    pub metadata: Option<serde_json::Value>,
}

/// Accepts either a single string or a list of strings. Used for
/// `claims_schema:` in templates.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(untagged)]
pub enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    pub fn as_slice(&self) -> &[String] {
        match self {
            OneOrMany::One(s) => std::slice::from_ref(s),
            OneOrMany::Many(v) => v.as_slice(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldDef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<FieldTypeDef>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,

    #[serde(default, skip_serializing_if = "is_false")]
    pub filldown: bool,

    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,

    #[serde(default, skip_serializing_if = "is_false")]
    pub list: bool,

    #[serde(default, skip_serializing_if = "is_false")]
    pub identity: bool,

    #[serde(default, skip_serializing_if = "is_false")]
    pub ignore: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub common_schema: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<crate::engine::types::FieldConstraints>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FieldTypeDef {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StateRuleDef {
    pub regex: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<ActionDef>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionDef {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<LineActionDef>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record: Option<RecordActionDef>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LineActionDef {
    Next,
    Continue,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RecordActionDef {
    None,
    Record,
    Clear,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatternRuleDef {
    pub regex: String,

    #[serde(default, skip_serializing_if = "is_false")]
    pub record: bool,
}

pub fn load_str(format: ModernFormat, input: &str) -> Result<TemplateIR, ScraperError> {
    let doc: ModernTemplateDoc = match format {
        ModernFormat::Toml => {
            let de = toml::de::Deserializer::parse(input)
                .map_err(|e| ScraperError::Template(format!("TOML parse error: {e}")))?;
            serde_path_to_error::deserialize(de)
                .map_err(|e| ScraperError::Template(format!("TOML schema error: {e}")))?
        }
        ModernFormat::Yaml => {
            let de = serde_yaml_ng::Deserializer::from_str(input);
            serde_path_to_error::deserialize(de)
                .map_err(|e| ScraperError::Template(format!("YAML schema error: {e}")))?
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

pub fn to_yaml_string(doc: &ModernTemplateDoc) -> Result<String, ScraperError> {
    serde_yaml_ng::to_string(doc)
        .map_err(|e| ScraperError::Template(format!("YAML serialize error: {e}")))
}

pub fn to_toml_string(doc: &ModernTemplateDoc) -> Result<String, ScraperError> {
    toml::to_string_pretty(doc)
        .map_err(|e| ScraperError::Template(format!("TOML serialize error: {e}")))
}

impl ModernTemplateDoc {
    fn validate(&self) -> Result<(), ScraperError> {
        if self.version != 1 {
            return Err(ScraperError::Template(format!(
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
                    return Err(ScraperError::Template(
                        "Modern templates with explicit states must define a 'Start' state"
                            .to_string(),
                    ));
                }
            }
            (false, true) => {}
            (true, true) => {
                return Err(ScraperError::Template(
                    "Modern template must define exactly one of 'states' or 'patterns'".to_string(),
                ));
            }
            (false, false) => {
                return Err(ScraperError::Template(
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
                ScraperError::Template(format!(
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
                return Err(ScraperError::Template(format!(
                    "Rule references placeholder '${{{}}}' but 'fields.{}.pattern' is missing",
                    name, name
                )));
            }
        }

        for name in named_groups.iter() {
            if !self.fields.contains_key(name) {
                return Err(ScraperError::Template(format!(
                    "Rule contains named capture group '{name}' but 'fields.{name}' is not defined"
                )));
            }
        }

        self.validate_common_schema_references()?;

        Ok(())
    }

    fn validate_common_schema_references(&self) -> Result<(), ScraperError> {
        use crate::engine::common_schemas::{KeyType, ResolveError, builtin_registry};

        let registry = builtin_registry();
        let claimed_owned: Vec<String> = self
            .claims_schema
            .as_ref()
            .map(|c| c.as_slice().to_vec())
            .unwrap_or_default();
        let mut errors: Vec<String> = Vec::new();

        // Verify every claimed schema name is actually known.
        for schema_name in &claimed_owned {
            if registry.get(schema_name).is_none() {
                let known: Vec<&str> = registry.schema_names().collect();
                errors.push(format!(
                    "claims_schema lists '{}' but no such schema is registered (known: {})",
                    schema_name,
                    known.join(", ")
                ));
            }
        }

        // Track which keys per schema were satisfied; used to enforce
        // required-key coverage when claims_schema is declared.
        let mut covered: std::collections::BTreeMap<String, std::collections::BTreeSet<String>> =
            std::collections::BTreeMap::new();

        for (field_name, def) in &self.fields {
            let Some(reference) = def.common_schema.as_deref() else {
                continue;
            };
            match registry.resolve(reference, &claimed_owned) {
                Ok((schema, key_name, key_def)) => {
                    let field_ty = match def.r#type.unwrap_or(FieldTypeDef::String) {
                        FieldTypeDef::Int => KeyType::Int,
                        FieldTypeDef::String => KeyType::String,
                    };
                    if field_ty != key_def.ty {
                        errors.push(format!(
                            "field '{}' has type {:?} but '{}.{}' is declared as {:?} in the {} schema",
                            field_name, field_ty, schema.name, key_name, key_def.ty, schema.name
                        ));
                    }
                    covered
                        .entry(schema.name.clone())
                        .or_default()
                        .insert(key_name.to_string());
                }
                Err(ResolveError::UnknownSchema { schema, known }) => {
                    errors.push(format!(
                        "field '{}' references common_schema '{}' but no such schema is registered (known: {})",
                        field_name,
                        schema,
                        known.join(", ")
                    ));
                }
                Err(ResolveError::UnknownKey {
                    schema,
                    key,
                    suggestions,
                }) => {
                    let scope = match schema {
                        Some(s) => format!("schema '{}'", s),
                        None => "any built-in schema".to_string(),
                    };
                    let mut msg = format!(
                        "field '{}' references common_schema key '{}' which is not declared by {}",
                        field_name, key, scope
                    );
                    if !suggestions.is_empty() {
                        msg.push_str(&format!(". Did you mean: {}?", suggestions.join(", ")));
                    }
                    errors.push(msg);
                }
                Err(ResolveError::AmbiguousKey { key, schemas }) => {
                    errors.push(format!(
                        "field '{}' references common_schema key '{}' which is declared by multiple schemas ({}); qualify with 'schema.key' or add a top-level 'claims_schema:'",
                        field_name,
                        key,
                        schemas.join(", ")
                    ));
                }
            }
        }

        // Required-key enforcement: every claimed schema must have all of its
        // required keys mapped by some field. Skipped when claims_schema is
        // absent (legacy inference path).
        for schema_name in &claimed_owned {
            let Some(schema) = registry.get(schema_name) else {
                continue; // already reported as unknown above
            };
            let satisfied = covered
                .get(schema_name)
                .cloned()
                .unwrap_or_default();
            let missing: Vec<&str> = schema
                .required_keys()
                .filter(|k| !satisfied.contains(*k))
                .collect();
            if !missing.is_empty() {
                errors.push(format!(
                    "claims_schema '{}' but does not map required keys: {}",
                    schema_name,
                    missing.join(", ")
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ScraperError::Template(errors.join("\n")))
        }
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
                    identity: def.identity,
                    ignore: def.ignore,
                    common_schema: def.common_schema.clone(),
                    constraints: def.constraints.clone(),
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

fn is_false(v: &bool) -> bool {
    !*v
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
    use crate::engine::Template;
    use crate::engine::types::FieldType;

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
    fn modern_yaml_unknown_field_type_fails_with_path() {
        let doc = r#"
version: 1
fields:
  speed:
    type: integer
patterns:
  - regex: '^speed=(?P<speed>[0-9,]+)$'
    record: true
"#;

        let err = load_yaml_str(doc).unwrap_err();
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

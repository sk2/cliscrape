use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Next,
    Continue,
    Record,
    Clear,
    ClearAll,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    Int,
    String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub choices: Option<Vec<String>>,
    pub regex: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub name: String,
    pub regex: String,
    pub filldown: bool,
    pub required: bool,
    pub list: bool,
    pub identity: bool,
    pub ignore: bool,
    pub common_schema: Option<String>,
    pub constraints: Option<FieldConstraints>,
    pub type_hint: Option<FieldType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub regex: String,
    pub line_action: Action,
    pub record_action: Action,
    pub next_state: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub name: String,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateIR {
    pub values: HashMap<String, Value>,
    pub states: HashMap<String, State>,
    pub macros: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub regex: Regex,
    pub original_pattern: String,
    pub line_action: Action,
    pub record_action: Action,
    pub next_state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub states: HashMap<String, Vec<CompiledRule>>,
    pub values: HashMap<String, Value>,
}

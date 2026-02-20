use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Next,
    Continue,
    Record,
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldType {
    Int,
    String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub name: String,
    pub regex: String,
    pub filldown: bool,
    pub required: bool,
    pub list: bool,
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
    pub line_action: Action,
    pub record_action: Action,
    pub next_state: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub states: HashMap<String, Vec<CompiledRule>>,
    pub values: HashMap<String, Value>,
}

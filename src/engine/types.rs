#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Next,
    Continue,
    Record,
    Clear,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Value {
    pub name: String,
    pub regex: String,
    pub filldown: bool,
    pub required: bool,
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
pub struct Template {
    pub values: std::collections::HashMap<String, Value>,
    pub states: std::collections::HashMap<String, State>,
}

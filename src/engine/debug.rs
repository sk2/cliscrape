use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DebugReport {
    pub lines: Vec<String>,
    /// All successful rule matches, grouped by the input line they matched.
    pub matches_by_line: Vec<Vec<LineMatch>>,
    /// Records emitted during parsing, with the line index that triggered emission.
    pub records: Vec<EmittedRecord>,
}

impl DebugReport {
    pub fn new(lines: Vec<String>) -> Self {
        let matches_by_line = vec![Vec::new(); lines.len()];
        Self {
            lines,
            matches_by_line,
            records: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LineMatch {
    pub line_idx: usize,
    pub state_before: String,
    pub state_after: String,
    pub rule_idx: usize,
    pub line_action: String,
    pub record_action: String,
    pub next_state: Option<String>,
    pub captures: Vec<CaptureSpan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CaptureSpan {
    pub name: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub raw: String,
    pub typed: serde_json::Value,
    pub is_list: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmittedRecord {
    pub line_idx: usize,
    pub record: HashMap<String, serde_json::Value>,
}

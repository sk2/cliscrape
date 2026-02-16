use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub mod engine;

use crate::engine::{Template, Value, Action, Rule, State};

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Parsing error: {0}")]
    Parse(String),
}

pub struct FsmParser {
    template: Template,
}

impl FsmParser {
    pub fn new(template: Template) -> Self {
        Self { template }
    }

    pub fn from_file<P: AsRef<Path>>(_path: P) -> Result<Self, ScraperError> {
        // This will eventually call the TextFSM or YAML/TOML loader
        Err(ScraperError::Parse("Loader not implemented yet".to_string()))
    }

    pub fn parse(&self, _input: &str) -> Result<Vec<HashMap<String, String>>, ScraperError> {
        // Implementation of the FSM loop
        Ok(vec![])
    }
}

use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub mod engine;
pub mod template;

use crate::engine::Template;
use crate::template::loader::TextFsmLoader;

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

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ScraperError> {
        let content = std::fs::read_to_string(&path)?;
        let path_ref = path.as_ref();
        
        let ir = if path_ref.extension().and_then(|s| s.to_str()) == Some("textfsm") {
            TextFsmLoader::parse_str(&content)?
        } else {
            return Err(ScraperError::Parse("Unsupported template format".to_string()));
        };
        
        let template = Template::from_ir(ir)?;
        Ok(Self { template })
    }

    pub fn parse(&self, input: &str) -> Result<Vec<HashMap<String, String>>, ScraperError> {
        self.template.parse(input)
    }
}

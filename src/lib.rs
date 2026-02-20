use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

pub mod engine;
pub mod template;

use crate::engine::Template;
use crate::template::loader::TextFsmLoader;
use crate::template::modern;

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

        let ext = path_ref.extension().and_then(|s| s.to_str());
        let ext_display = ext.unwrap_or("<none>");
        let ir = match ext {
            Some("textfsm") => TextFsmLoader::parse_str(&content)?,
            Some("yaml") | Some("yml") => modern::load_yaml_str(&content)?,
            Some("toml") => modern::load_toml_str(&content)?,
            _ => {
                return Err(ScraperError::Parse(format!(
                    "Unsupported template extension '{ext_display}'. Supported: .textfsm, .yaml, .yml, .toml"
                )));
            }
        };

        let template = Template::from_ir(ir)?;
        Ok(Self { template })
    }

    pub fn parse(
        &self,
        input: &str,
    ) -> Result<Vec<HashMap<String, serde_json::Value>>, ScraperError> {
        self.template.parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn write_temp_template(ext: &str, content: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        let uniq = format!(
            "cliscrape-modern-{}-{}.{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            ext
        );
        path.push(uniq);
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn from_file_loads_modern_toml_by_extension() {
        let doc = r#"
version = 1

[fields]
speed = { type = "int" }

[[patterns]]
regex = '^speed=(?P<speed>[0-9,]+)$'
record = true
"#;

        let path = write_temp_template("toml", doc);
        let parser = FsmParser::from_file(&path).unwrap();
        let results = parser.parse("speed=1,234").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["speed"],
            serde_json::Value::Number(serde_json::Number::from(1234_i64))
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn from_file_loads_modern_yaml_by_extension() {
        let doc = r#"
version: 1
fields:
  speed:
    type: int
patterns:
  - regex: '^speed=(?P<speed>[0-9,]+)$'
    record: true
"#;

        let path = write_temp_template("yaml", doc);
        let parser = FsmParser::from_file(&path).unwrap();
        let results = parser.parse("speed=1,234").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["speed"],
            serde_json::Value::Number(serde_json::Number::from(1234_i64))
        );

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn from_file_loads_modern_yml_alias_by_extension() {
        let doc = r#"
version: 1
fields:
  speed:
    type: string
patterns:
  - regex: '^speed=(?P<speed>[0-9,]+)$'
    record: true
"#;

        let path = write_temp_template("yml", doc);
        let parser = FsmParser::from_file(&path).unwrap();
        let results = parser.parse("speed=1,234").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0]["speed"],
            serde_json::Value::String("1,234".to_string())
        );

        let _ = std::fs::remove_file(path);
    }
}

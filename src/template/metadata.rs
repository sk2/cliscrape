use crate::TemplateFormat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    pub description: String,
    pub compatibility: String,
    pub version: String,
    pub author: String,
    #[serde(default)]
    pub maintainer: Option<String>,
}

/// Extract metadata from template content.
///
/// This function is fault-tolerant: invalid or missing metadata returns
/// sensible defaults without errors. Metadata extraction failures never
/// prevent template usage.
pub fn extract_metadata(content: &str, format: TemplateFormat) -> TemplateMetadata {
    match format {
        TemplateFormat::Yaml | TemplateFormat::Toml => extract_from_modern(content, format),
        TemplateFormat::Textfsm => extract_from_textfsm_comments(content),
        TemplateFormat::Auto => default_metadata(),
    }
}

/// Extract metadata from modern template formats (YAML/TOML).
///
/// Looks for a top-level `metadata` key in the parsed document.
/// Returns defaults on any parsing failure.
fn extract_from_modern(content: &str, format: TemplateFormat) -> TemplateMetadata {
    match format {
        TemplateFormat::Yaml => {
            // Parse YAML and look for metadata section
            match serde_yaml_ng::from_str::<serde_yaml_ng::Value>(content) {
                Ok(doc) => {
                    if let Some(metadata_value) = doc.get("metadata") {
                        // Try to deserialize metadata section
                        match serde_yaml_ng::from_value::<TemplateMetadata>(metadata_value.clone()) {
                            Ok(metadata) => metadata,
                            Err(_) => default_metadata(),
                        }
                    } else {
                        default_metadata()
                    }
                }
                Err(_) => default_metadata(),
            }
        }
        TemplateFormat::Toml => {
            // Parse TOML and look for metadata section
            match toml::from_str::<toml::Value>(content) {
                Ok(doc) => {
                    if let Some(metadata_value) = doc.get("metadata") {
                        // Try to deserialize metadata section
                        match metadata_value.clone().try_into::<TemplateMetadata>() {
                            Ok(metadata) => metadata,
                            Err(_) => default_metadata(),
                        }
                    } else {
                        default_metadata()
                    }
                }
                Err(_) => default_metadata(),
            }
        }
        _ => default_metadata(),
    }
}

/// Extract metadata from TextFSM comment headers.
///
/// Processes lines starting with `#` at the beginning of the file,
/// parsing them as "Key: Value" pairs. Stops at first non-comment line.
fn extract_from_textfsm_comments(content: &str) -> TemplateMetadata {
    let mut metadata = default_metadata();

    for line in content.lines() {
        let trimmed = line.trim();

        // Stop at first non-comment line
        if !trimmed.starts_with('#') {
            break;
        }

        // Remove leading '#' and parse as "Key: Value"
        let comment = trimmed.trim_start_matches('#').trim();

        if let Some((key, value)) = comment.split_once(':') {
            let key = key.trim().to_lowercase();
            let value = value.trim().to_string();

            match key.as_str() {
                "description" => metadata.description = value,
                "compatibility" => metadata.compatibility = value,
                "version" => metadata.version = value,
                "author" => metadata.author = value,
                "maintainer" => metadata.maintainer = Some(value),
                _ => {} // Ignore unknown metadata keys
            }
        }
    }

    metadata
}

/// Return default metadata for templates without metadata.
fn default_metadata() -> TemplateMetadata {
    TemplateMetadata {
        description: "No description available".to_string(),
        compatibility: "Unknown".to_string(),
        version: "1.0.0".to_string(),
        author: "Unknown".to_string(),
        maintainer: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_yaml_metadata_extraction() {
        let template = r#"
version: 1
metadata:
  description: "Parse Cisco IOS show version output"
  compatibility: "Cisco IOS 12.x - 15.x"
  version: "2.1.0"
  author: "Network Team"
  maintainer: "ops@example.com"
fields:
  version:
    type: string
patterns:
  - regex: '^Cisco IOS.*Version (?P<version>\S+)'
    record: true
"#;

        let metadata = extract_metadata(template, TemplateFormat::Yaml);

        assert_eq!(metadata.description, "Parse Cisco IOS show version output");
        assert_eq!(metadata.compatibility, "Cisco IOS 12.x - 15.x");
        assert_eq!(metadata.version, "2.1.0");
        assert_eq!(metadata.author, "Network Team");
        assert_eq!(metadata.maintainer, Some("ops@example.com".to_string()));
    }

    #[test]
    fn test_toml_metadata_extraction() {
        let template = r#"
version = 1

[metadata]
description = "Parse interface statistics"
compatibility = "Juniper JunOS 18.x+"
version = "1.5.3"
author = "Juniper Admin"

[fields]
interface = { type = "string" }

[[patterns]]
regex = '^(?P<interface>\S+)'
record = true
"#;

        let metadata = extract_metadata(template, TemplateFormat::Toml);

        assert_eq!(metadata.description, "Parse interface statistics");
        assert_eq!(metadata.compatibility, "Juniper JunOS 18.x+");
        assert_eq!(metadata.version, "1.5.3");
        assert_eq!(metadata.author, "Juniper Admin");
        assert_eq!(metadata.maintainer, None);
    }

    #[test]
    fn test_textfsm_comment_extraction() {
        let template = r#"# Description: Parse BGP neighbors
# Compatibility: Arista EOS 4.x
# Version: 3.0.1
# Author: BGP Team
# Maintainer: network-ops@example.com

Value Filldown NEIGHBOR (\S+)
Value STATE (\w+)

Start
  ^Neighbor ${NEIGHBOR}
  ^State: ${STATE} -> Record
"#;

        let metadata = extract_metadata(template, TemplateFormat::Textfsm);

        assert_eq!(metadata.description, "Parse BGP neighbors");
        assert_eq!(metadata.compatibility, "Arista EOS 4.x");
        assert_eq!(metadata.version, "3.0.1");
        assert_eq!(metadata.author, "BGP Team");
        assert_eq!(metadata.maintainer, Some("network-ops@example.com".to_string()));
    }

    #[test]
    fn test_missing_metadata_returns_defaults() {
        let template = r#"
version: 1
fields:
  hostname:
    type: string
patterns:
  - regex: '^Host=(?P<hostname>\S+)$'
    record: true
"#;

        let metadata = extract_metadata(template, TemplateFormat::Yaml);

        assert_eq!(metadata.description, "No description available");
        assert_eq!(metadata.compatibility, "Unknown");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "Unknown");
        assert_eq!(metadata.maintainer, None);
    }

    #[test]
    fn test_invalid_yaml_returns_defaults() {
        let template = r#"
this is not: valid [yaml at all
{broken syntax
"#;

        let metadata = extract_metadata(template, TemplateFormat::Yaml);

        assert_eq!(metadata.description, "No description available");
        assert_eq!(metadata.compatibility, "Unknown");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "Unknown");
    }

    #[test]
    fn test_textfsm_no_comments_returns_defaults() {
        let template = r#"Value HOSTNAME (\S+)

Start
  ^Host: ${HOSTNAME} -> Record
"#;

        let metadata = extract_metadata(template, TemplateFormat::Textfsm);

        assert_eq!(metadata.description, "No description available");
        assert_eq!(metadata.compatibility, "Unknown");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "Unknown");
    }

    #[test]
    fn test_empty_content_returns_defaults() {
        let metadata = extract_metadata("", TemplateFormat::Yaml);

        assert_eq!(metadata.description, "No description available");
        assert_eq!(metadata.compatibility, "Unknown");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.author, "Unknown");
    }

    #[test]
    fn test_auto_format_returns_defaults() {
        let template = r#"version: 1"#;
        let metadata = extract_metadata(template, TemplateFormat::Auto);

        assert_eq!(metadata.description, "No description available");
        assert_eq!(metadata.compatibility, "Unknown");
    }

    #[test]
    fn test_textfsm_case_insensitive_keys() {
        let template = r#"# DESCRIPTION: Test template
# VERSION: 1.0.0
# Compatibility: All devices
# AUTHOR: Test User

Value X (\S+)
Start
  ^X ${X} -> Record
"#;

        let metadata = extract_metadata(template, TemplateFormat::Textfsm);

        assert_eq!(metadata.description, "Test template");
        assert_eq!(metadata.version, "1.0.0");
        assert_eq!(metadata.compatibility, "All devices");
        assert_eq!(metadata.author, "Test User");
    }

    #[test]
    fn test_partial_metadata_merges_with_defaults() {
        let template = r#"
version: 1
metadata:
  description: "Only has description"
fields:
  x:
    type: string
patterns:
  - regex: '^(?P<x>\S+)$'
    record: true
"#;

        // This should fail to deserialize because other required fields are missing
        // and fall back to defaults
        let metadata = extract_metadata(template, TemplateFormat::Yaml);

        // Since deserialization fails, we get full defaults
        assert_eq!(metadata.description, "No description available");
        assert_eq!(metadata.compatibility, "Unknown");
    }
}

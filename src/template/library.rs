//! Embedded template library
//!
//! This module provides compile-time embedding of template files from the `templates/` directory.
//!
//! # Behavior
//!
//! - **Release builds**: Templates are embedded at compile time into the binary
//! - **Debug builds**: Templates are loaded from filesystem (hot-reload support)
//! - **Directory**: `templates/` relative to project root
//!
//! # Supported formats
//!
//! - `.yaml` - Template metadata and configuration
//! - `.toml` - Template metadata and configuration
//! - `.textfsm` - TextFSM template definitions
//!
//! Test and documentation files (`.md`, `tests/*`) are excluded from embedding.

use rust_embed::RustEmbed;

/// Embedded template directory
///
/// Uses rust-embed to compile templates into the binary in release mode,
/// while allowing filesystem access in debug mode for hot-reload development.
#[derive(RustEmbed)]
#[folder = "templates/"]
#[include = "*.yaml"]
#[include = "*.toml"]
#[include = "*.textfsm"]
#[exclude = "*.md"]
#[exclude = "tests/*"]
pub struct EmbeddedTemplates;

/// List all embedded template names
///
/// Returns the filenames of all templates embedded in the binary.
///
/// # Examples
///
/// ```no_run
/// use cliscrape::template::library::list_embedded;
///
/// let templates = list_embedded();
/// for name in templates {
///     println!("Template: {}", name);
/// }
/// ```
pub fn list_embedded() -> Vec<String> {
    EmbeddedTemplates::iter()
        .map(|s| s.to_string())
        .collect()
}

/// Get an embedded template by name
///
/// Returns the embedded file data if the template exists.
///
/// # Arguments
///
/// * `name` - Template filename (e.g., "cisco_ios_show_version.yaml")
///
/// # Returns
///
/// - `Some(EmbeddedFile)` if the template exists
/// - `None` if the template is not found
///
/// # Examples
///
/// ```no_run
/// use cliscrape::template::library::get_embedded;
///
/// if let Some(template) = get_embedded("cisco_ios_show_version.yaml") {
///     let content = template.data;
///     println!("Template size: {} bytes", content.len());
/// }
/// ```
pub fn get_embedded(name: &str) -> Option<rust_embed::EmbeddedFile> {
    EmbeddedTemplates::get(name)
}

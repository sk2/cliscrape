//! Template resolver with security validation and XDG precedence
//!
//! This module provides secure template name resolution with the following precedence:
//!
//! 1. **User directory** (highest priority): `$XDG_DATA_HOME/cliscrape/templates/`
//!    - Defaults to `~/.local/share/cliscrape/templates/`
//! 2. **System directories** (medium priority): `$XDG_DATA_DIRS/cliscrape/templates/`
//!    - Defaults to `/usr/local/share/cliscrape/templates/:/usr/share/cliscrape/templates/`
//! 3. **Embedded templates** (lowest priority): Compiled into binary
//!
//! # Security
//!
//! Template names are validated BEFORE any filesystem operations to prevent path traversal attacks.
//! Only alphanumeric characters, underscores, hyphens, and dots are allowed.

use std::path::PathBuf;
use rust_embed::EmbeddedFile;
use xdg::BaseDirectories;

use super::library;

/// Template source location
pub enum TemplateSource {
    /// Template from XDG user directory
    UserFile(PathBuf),
    /// Template from embedded resources
    Embedded(EmbeddedFile),
}

impl std::fmt::Debug for TemplateSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateSource::UserFile(path) => f.debug_tuple("UserFile").field(path).finish(),
            TemplateSource::Embedded(_) => f.debug_tuple("Embedded").field(&"<embedded>").finish(),
        }
    }
}

/// Validate template name for security
///
/// Ensures template names cannot be used for path traversal attacks.
///
/// # Security validation rules
///
/// - Name must not be empty
/// - Name must not contain backslashes (`\`)
/// - Name must not contain parent directory references (`..`)
/// - Name must not start with `/` (absolute path indicator)
/// - Name must match allowlist pattern: `^[a-zA-Z0-9_\-\./]+$`
/// - Forward slashes (`/`) are allowed for subdirectory organization
///
/// # Arguments
///
/// * `name` - Template name to validate
///
/// # Returns
///
/// - `Ok(())` if the name is valid
/// - `Err(String)` with descriptive error message if validation fails
///
/// # Examples
///
/// ```
/// use cliscrape::template::resolver::validate_template_name;
///
/// // Valid names
/// assert!(validate_template_name("cisco_ios_show_version.yaml").is_ok());
/// assert!(validate_template_name("template-name.toml").is_ok());
/// assert!(validate_template_name("simple.textfsm").is_ok());
/// assert!(validate_template_name("modern/template.yaml").is_ok()); // subdirectories OK
///
/// // Invalid names (path traversal attempts)
/// assert!(validate_template_name("../etc/passwd").is_err());
/// assert!(validate_template_name("../../secret").is_err());
/// assert!(validate_template_name("/etc/passwd").is_err());
/// assert!(validate_template_name("path\\to\\file").is_err()); // backslashes not allowed
/// assert!(validate_template_name("").is_err());
/// ```
pub fn validate_template_name(name: &str) -> Result<(), String> {
    // Empty name check
    if name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }

    // Absolute path check
    if name.starts_with('/') || name.starts_with('\\') {
        return Err(format!(
            "Invalid template name '{}': absolute paths not allowed",
            name
        ));
    }

    // Path traversal check - parent directory references
    if name.contains("..") {
        return Err(format!(
            "Invalid template name '{}': parent directory references (..) not allowed",
            name
        ));
    }

    // Path separator check - reject backslashes but allow forward slashes for subdirectories
    if name.contains('\\') {
        return Err(format!(
            "Invalid template name '{}': backslash not allowed",
            name
        ));
    }

    // Allowlist pattern check - allow alphanumeric, underscore, hyphen, dot, and forward slash
    let valid_pattern = regex::Regex::new(r"^[a-zA-Z0-9_\-\./]+$").unwrap();
    if !valid_pattern.is_match(name) {
        return Err(format!(
            "Invalid template name '{}': only alphanumeric, underscore, hyphen, dot, and forward slash allowed",
            name
        ));
    }

    Ok(())
}

/// Template resolver with XDG directory support
///
/// Resolves template names to their source location following XDG precedence.
pub struct TemplateResolver {
    xdg: BaseDirectories,
}

impl TemplateResolver {
    /// Create a new template resolver
    ///
    /// Initializes XDG base directories with prefix "cliscrape".
    ///
    /// # Returns
    ///
    /// - `Ok(TemplateResolver)` if XDG directories can be initialized
    /// - `Err(xdg::BaseDirectoriesError)` if initialization fails
    pub fn new() -> Result<Self, xdg::BaseDirectoriesError> {
        let xdg = BaseDirectories::with_prefix("cliscrape");
        Ok(TemplateResolver { xdg })
    }

    /// Find a template in the XDG user directory
    ///
    /// Searches for the template in:
    /// - `$XDG_DATA_HOME/cliscrape/templates/{name}`
    /// - `$XDG_DATA_DIRS/cliscrape/templates/{name}`
    ///
    /// # Arguments
    ///
    /// * `name` - Template filename
    ///
    /// # Returns
    ///
    /// - `Some(PathBuf)` if the template exists in user/system directories
    /// - `None` if not found
    fn find_user_template(&self, name: &str) -> Option<PathBuf> {
        self.xdg.find_data_file(format!("templates/{}", name))
    }

    /// Resolve a template name to its source
    ///
    /// Resolution precedence:
    /// 1. User directory (XDG_DATA_HOME)
    /// 2. System directories (XDG_DATA_DIRS)
    /// 3. Embedded templates
    ///
    /// # Security
    ///
    /// Template name is validated BEFORE any filesystem operations to prevent
    /// time-of-check-time-of-use (TOCTOU) vulnerabilities.
    ///
    /// # Arguments
    ///
    /// * `template_name` - Name of the template to resolve
    ///
    /// # Returns
    ///
    /// - `Ok(TemplateSource::UserFile)` if found in XDG directories
    /// - `Ok(TemplateSource::Embedded)` if found in embedded resources
    /// - `Err(String)` if validation fails or template not found
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use cliscrape::template::resolver::TemplateResolver;
    ///
    /// let resolver = TemplateResolver::new().unwrap();
    /// match resolver.resolve("cisco_ios_show_version.yaml") {
    ///     Ok(source) => println!("Template found"),
    ///     Err(e) => eprintln!("Error: {}", e),
    /// }
    /// ```
    pub fn resolve(&self, template_name: &str) -> Result<TemplateSource, String> {
        // CRITICAL: Validate template name FIRST, before any filesystem operations
        // This prevents TOCTOU vulnerabilities and path traversal attacks
        validate_template_name(template_name)?;

        // Check XDG user directory (highest priority)
        if let Some(path) = self.find_user_template(template_name) {
            return Ok(TemplateSource::UserFile(path));
        }

        // Fallback to embedded templates (lowest priority)
        if let Some(embedded) = library::get_embedded(template_name) {
            return Ok(TemplateSource::Embedded(embedded));
        }

        // Template not found in any source
        Err(format!(
            "Template '{}' not found in user directories or embedded resources",
            template_name
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_template_name_valid() {
        // Valid names
        assert!(validate_template_name("cisco_ios_show_version.yaml").is_ok());
        assert!(validate_template_name("template-name.toml").is_ok());
        assert!(validate_template_name("simple.textfsm").is_ok());
        assert!(validate_template_name("123.yaml").is_ok());
        assert!(validate_template_name("a_b-c.d").is_ok());
        assert!(validate_template_name("modern/template.yaml").is_ok());
        assert!(validate_template_name("vendor/device/template.toml").is_ok());
    }

    #[test]
    fn test_validate_template_name_empty() {
        let result = validate_template_name("");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }

    #[test]
    fn test_validate_template_name_path_traversal() {
        // Parent directory references
        assert!(validate_template_name("../etc/passwd").is_err());
        assert!(validate_template_name("../../secret").is_err());
        assert!(validate_template_name("..").is_err());
        assert!(validate_template_name("foo..bar").is_err());
    }

    #[test]
    fn test_validate_template_name_absolute_path() {
        assert!(validate_template_name("/etc/passwd").is_err());
        assert!(validate_template_name("\\windows\\system32").is_err());
    }

    #[test]
    fn test_validate_template_name_path_separators() {
        // Forward slashes are allowed for subdirectories
        assert!(validate_template_name("path/to/file").is_ok());
        // Backslashes are not allowed
        assert!(validate_template_name("path\\to\\file").is_err());
    }

    #[test]
    fn test_validate_template_name_invalid_chars() {
        assert!(validate_template_name("template name.yaml").is_err()); // space
        assert!(validate_template_name("template$name.yaml").is_err()); // dollar sign
        assert!(validate_template_name("template;name.yaml").is_err()); // semicolon
        assert!(validate_template_name("template&name.yaml").is_err()); // ampersand
    }

    #[test]
    fn test_resolver_creation() {
        // Should create successfully
        let resolver = TemplateResolver::new();
        assert!(resolver.is_ok());
    }

    #[test]
    fn test_resolver_validates_name() {
        let resolver = TemplateResolver::new().unwrap();

        // Should reject invalid names
        let result = resolver.resolve("../etc/passwd");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("parent directory"));
    }
}

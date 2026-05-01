# Phase 6: Template Library Foundation - Research

**Researched:** 2026-02-22
**Domain:** Embedded template library with XDG-based user overrides
**Confidence:** HIGH

## Summary

This phase requires building an embedded template library system for cliscrape that allows users to reference templates by name without providing file paths. The implementation needs three components: (1) compile-time embedding of template files into the binary using rust-embed, (2) XDG Base Directory compliant user override system using the xdg crate, and (3) CLI interface extensions for template discovery and metadata display.

The research confirms this is a well-established pattern in Rust CLI tools. Template embedding via rust-embed is production-ready and widely used. The xdg crate provides standard-compliant directory access with proper precedence handling. Security validation for template names is straightforward using path component validation without filesystem canonicalization.

**Primary recommendation:** Use rust-embed 8.11.0 for embedded templates, xdg 3.0.0 for user directory management, extend clap CLI with new subcommands for template listing/viewing, follow ntc-templates naming convention (vendor_os_command format), and implement strict validation rejecting path separators and traversal sequences.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**Metadata fields (required):**
- description: Human-readable purpose of the template
- compatibility: Devices/OS versions this template works with
- version: Template version for tracking evolution
- author/maintainer: Who created/maintains this template

**Detail view contents (when viewing specific template):**
- Full metadata (description, compatibility, version, author)
- Template source code (actual YAML/TOML content)
- Location (whether embedded or user override from XDG)
- Fields extracted (list of captured variables this template produces)

### Claude's Discretion

The following areas are left to Claude's judgment during planning and implementation:

**Template naming & organization:**
- Naming convention (flat with underscores vs hierarchical paths)
- Handling device OS variations (separate templates per variant vs shared with compatibility metadata)
- Categories/tags for organization (explicit taxonomy vs searchable names/descriptions)
- Initial embedded library scope (small focused set vs comprehensive multi-vendor coverage)

**Metadata structure:**
- How compatibility information is expressed (structured data vs free-text vs both)
- Versioning scheme (semantic versioning vs simple incrementing vs date-based)
- Whether to include example input/output in template metadata

**CLI interface & discovery:**
- Template listing command structure (--list-templates with format flags vs separate commands)
- Filtering/search support (dedicated filter flags vs pattern matching vs rely on Unix pipes)
- Command for viewing template details (--show-template vs --template with --info modifier)

**Override behavior:**
- Override precedence mechanism (exact name match vs version-aware vs always-warn)
- Feedback level when user template overrides embedded one (silent vs info log vs warning)
- Validation of user templates (syntax only vs syntax + metadata warnings vs trust user)
- How to display overridden templates in listings (show only active vs show both vs note override)

### Deferred Ideas (OUT OF SCOPE)

None - discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

This phase must implement all Template Library requirements:

| ID | Description | Research Support |
|----|-------------|-----------------|
| LIB-01 | User can parse with embedded templates without providing file paths | rust-embed enables compile-time binary embedding; template resolution logic maps names to embedded files |
| LIB-02 | User can reference templates by name (e.g., `--template cisco_ios_show_version`) | Template name validation and lookup system; existing resolve_template_spec function provides foundation |
| LIB-03 | User can add custom templates to XDG user directory (~/.local/share/cliscrape/templates/) | xdg crate BaseDirectories provides place_data_file and find_data_file methods for XDG compliance |
| LIB-04 | User can view template metadata including version, description, and compatibility | Metadata extraction from template files; CLI subcommands for template info display using comfy-table for formatting |
| LIB-05 | User can override embedded templates with custom versions via XDG directory | XDG precedence system (user dirs before system dirs); lookup checks user directory before embedded resources |
| LIB-06 | User receives security validation errors for invalid template names (path traversal protection) | Input validation rejecting "..", "/", "\\", and absolute paths before any filesystem operations |

</phase_requirements>

## Standard Stack

### Core Dependencies

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| rust-embed | 8.11.0 | Compile-time file embedding | De facto standard for embedding static resources; 8.11.0 is latest stable; dual-mode (embedded in release, filesystem in dev) supports development workflow |
| xdg | 3.0.0 | XDG Base Directory compliance | Standard implementation of XDG spec; 3.0.0 is latest stable; provides BaseDirectories API for config/data/cache paths |
| clap | 4.5.58 | CLI argument parsing (already in project) | Already used for CLI; supports subcommands needed for list/show operations |
| comfy-table | 7.1 | Table formatting (already in project) | Already used for table output; needed for template listing display |
| serde | 1.0 | Metadata serialization/deserialization (already in project) | Already used for JSON/YAML/TOML; needed for metadata parsing |

### Supporting

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| serde_yaml_ng | 0.10.0 | YAML metadata frontmatter (already in project) | When templates include YAML metadata headers |
| toml | 1.0.3 | TOML metadata frontmatter (already in project) | When templates include TOML metadata headers |
| regex | 1.12.3 | Template name validation (already in project) | For validating template identifiers match allowed patterns |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| rust-embed | include_dir 0.7.4 | include_dir expands to large code (all files as Rust byte strings), slower compile with many files; rust-embed uses filesystem in dev mode for faster iteration |
| rust-embed | Standard library include_str!/include_bytes! | Manual per-file inclusion is not scalable for template library; no directory iteration support |
| xdg | directories 5.0.1 | directories crate is cross-platform but doesn't follow XDG spec on Linux; phase explicitly requires XDG compliance |
| xdg | Manual PathBuf construction | Easy to get precedence wrong, miss edge cases, non-portable across distros |

**Installation:**
```toml
# Add to Cargo.toml [dependencies]
rust-embed = "8.11.0"
xdg = "3.0.0"
# clap, comfy-table, serde, serde_yaml_ng, toml, regex already present
```

## Architecture Patterns

### Recommended Project Structure

```
src/
├── template/
│   ├── mod.rs                # Existing template module
│   ├── loader.rs             # Existing TextFSM loader
│   ├── modern.rs             # Existing modern format loader
│   ├── library.rs            # NEW: Embedded template library
│   ├── metadata.rs           # NEW: Template metadata extraction
│   └── resolver.rs           # NEW: Template resolution (embedded + XDG)
├── cli.rs                    # Extend with new subcommands
└── main.rs                   # Add template subcommand routing

templates/                     # NEW: Directory for embedded templates
├── cisco_ios_show_version.yaml
├── cisco_ios_show_interfaces.yaml
└── ...                        # Additional vendor templates
```

### Pattern 1: Embedded Template Library (rust-embed)

**What:** Compile-time embedding of template directory into binary with runtime access by filename.

**When to use:** For shipping default templates without requiring external files.

**Example:**
```rust
// Source: https://docs.rs/rust-embed/8.11.0/rust_embed/
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct TemplateLibrary;

impl TemplateLibrary {
    pub fn get_template(name: &str) -> Option<EmbeddedFile> {
        Self::get(name)
    }

    pub fn list_templates() -> impl Iterator<Item = String> {
        Self::iter().map(|s| s.into_owned())
    }
}
```

**Key features:**
- In release builds: files embedded as static data in binary
- In debug builds: files loaded from filesystem for hot-reload during development
- Supports compression via `compression` feature flag
- Memory-efficient: lazy loading, not all files in memory simultaneously

### Pattern 2: XDG Base Directory Resolution

**What:** Standard-compliant user/system directory resolution with proper precedence.

**When to use:** For allowing user customization of embedded resources while maintaining system-wide defaults.

**Example:**
```rust
// Source: https://docs.rs/xdg/3.0.0/xdg/struct.BaseDirectories.html
use xdg::BaseDirectories;

pub struct TemplateResolver {
    xdg_dirs: BaseDirectories,
}

impl TemplateResolver {
    pub fn new() -> Result<Self, xdg::BaseDirectoriesError> {
        let xdg_dirs = BaseDirectories::with_prefix("cliscrape")?;
        Ok(Self { xdg_dirs })
    }

    pub fn find_template(&self, name: &str) -> Option<PathBuf> {
        // XDG precedence: user data dir takes precedence over system dirs
        // Typically: ~/.local/share/cliscrape/templates/ before /usr/share/cliscrape/templates/
        self.xdg_dirs.find_data_file(format!("templates/{}", name))
    }

    pub fn place_user_template(&self, name: &str, content: &str) -> Result<PathBuf, std::io::Error> {
        let path = self.xdg_dirs.place_data_file(format!("templates/{}", name))?;
        std::fs::write(&path, content)?;
        Ok(path)
    }
}
```

**XDG directory locations:**
- User data: `$XDG_DATA_HOME/cliscrape/templates/` (defaults to `~/.local/share/cliscrape/templates/`)
- System data: `$XDG_DATA_DIRS/cliscrape/templates/` (defaults to `/usr/local/share/cliscrape/templates/:/usr/share/cliscrape/templates/`)

**Precedence rules:**
1. User directory (`~/.local/share/cliscrape/templates/`) - highest priority
2. System directories (`/usr/local/share`, `/usr/share`) - fallback
3. Embedded templates in binary - lowest priority

### Pattern 3: Template Resolution with Security Validation

**What:** Combined lookup checking XDG directories before falling back to embedded templates, with path traversal protection.

**When to use:** For all template name resolution in the application.

**Example:**
```rust
pub struct TemplateService {
    resolver: TemplateResolver,
}

impl TemplateService {
    pub fn resolve(&self, template_name: &str) -> Result<TemplateSource, ScraperError> {
        // Security validation FIRST - before any filesystem operations
        Self::validate_template_name(template_name)?;

        // Check XDG user directory first
        if let Some(user_path) = self.resolver.find_template(&format!("{}.yaml", template_name)) {
            return Ok(TemplateSource::UserFile(user_path));
        }

        // Fallback to embedded templates
        if let Some(embedded) = TemplateLibrary::get(&format!("{}.yaml", template_name)) {
            return Ok(TemplateSource::Embedded(embedded));
        }

        Err(ScraperError::Parse(format!("Template '{}' not found", template_name)))
    }

    fn validate_template_name(name: &str) -> Result<(), ScraperError> {
        // Source: https://www.stackhawk.com/blog/rust-path-traversal-guide-example-and-prevention/
        // Reject path traversal attempts and invalid characters
        if name.contains("..") || name.contains('/') || name.contains('\\') {
            return Err(ScraperError::Parse(
                format!("Invalid template name '{}': path traversal not allowed", name)
            ));
        }

        // Reject absolute paths
        if name.starts_with('/') || name.starts_with('\\') {
            return Err(ScraperError::Parse(
                format!("Invalid template name '{}': absolute paths not allowed", name)
            ));
        }

        // Validate characters (alphanumeric, underscore, hyphen, dot only)
        let valid_chars = name.chars().all(|c| {
            c.is_alphanumeric() || c == '_' || c == '-' || c == '.'
        });

        if !valid_chars {
            return Err(ScraperError::Parse(
                format!("Invalid template name '{}': only alphanumeric, underscore, hyphen, and dot allowed", name)
            ));
        }

        Ok(())
    }
}

pub enum TemplateSource {
    UserFile(PathBuf),
    Embedded(EmbeddedFile),
}
```

### Pattern 4: Template Metadata Extraction

**What:** Parse metadata from template files using YAML frontmatter or dedicated metadata sections.

**When to use:** For displaying template information and supporting template discovery.

**Example:**
```rust
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

impl TemplateMetadata {
    pub fn from_template_content(content: &str, format: TemplateFormat) -> Result<Self, ScraperError> {
        match format {
            TemplateFormat::Yaml | TemplateFormat::Toml => {
                // Modern templates: metadata is part of the document structure
                Self::extract_from_modern(content, format)
            }
            TemplateFormat::Textfsm => {
                // TextFSM: metadata in comment headers
                Self::extract_from_textfsm_comments(content)
            }
            _ => Err(ScraperError::Parse("Auto format requires explicit format".into()))
        }
    }

    fn extract_from_modern(content: &str, format: TemplateFormat) -> Result<Self, ScraperError> {
        // Parse the template document and extract metadata section
        // Modern templates can include a 'metadata' top-level key
        match format {
            TemplateFormat::Yaml => {
                let doc: serde_yaml_ng::Value = serde_yaml_ng::from_str(content)
                    .map_err(|e| ScraperError::Parse(format!("YAML parse error: {}", e)))?;

                if let Some(metadata) = doc.get("metadata") {
                    serde_yaml_ng::from_value(metadata.clone())
                        .map_err(|e| ScraperError::Parse(format!("Metadata parse error: {}", e)))
                } else {
                    Self::default_metadata()
                }
            }
            TemplateFormat::Toml => {
                let doc: toml::Value = toml::from_str(content)
                    .map_err(|e| ScraperError::Parse(format!("TOML parse error: {}", e)))?;

                if let Some(metadata) = doc.get("metadata") {
                    metadata.clone().try_into()
                        .map_err(|e| ScraperError::Parse(format!("Metadata parse error: {}", e)))
                } else {
                    Self::default_metadata()
                }
            }
            _ => Self::default_metadata()
        }
    }

    fn extract_from_textfsm_comments(content: &str) -> Result<Self, ScraperError> {
        // TextFSM templates can have metadata in header comments
        // Format: # Key: Value
        let mut metadata = Self::default_metadata()?;

        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with('#') {
                break; // Stop at first non-comment line
            }

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

        Ok(metadata)
    }

    fn default_metadata() -> Result<Self, ScraperError> {
        Ok(Self {
            description: "No description available".to_string(),
            compatibility: "Unknown".to_string(),
            version: "1.0.0".to_string(),
            author: "Unknown".to_string(),
            maintainer: None,
        })
    }
}
```

### Pattern 5: CLI Subcommand Extension

**What:** Add template discovery subcommands to existing clap CLI structure.

**When to use:** For exposing template library functionality to users.

**Example:**
```rust
// Source: https://oneuptime.com/blog/post/2026-02-03-rust-clap-cli-applications/view
use clap::{Parser, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a raw text file using a template
    Parse {
        // ... existing parse options
    },

    /// List available templates
    #[command(name = "list-templates", alias = "templates")]
    ListTemplates {
        /// Filter templates by pattern (supports wildcards)
        #[arg(short, long)]
        filter: Option<String>,

        /// Output format for template listing
        #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Show detailed information about a specific template
    #[command(name = "show-template")]
    ShowTemplate {
        /// Template name to display
        template: String,

        /// Show template source code
        #[arg(long)]
        source: bool,
    },

    // ... other existing commands (Debug, Convert)
}
```

### Anti-Patterns to Avoid

- **Don't use Path::canonicalize for validation:** Canonicalize touches the filesystem and can be slow or fail if path doesn't exist. For security validation of template names, use string validation before any filesystem operations.

- **Don't embed templates in source code:** Avoid `const TEMPLATE: &str = include_str!("...")` pattern for individual templates. Use rust-embed to handle the entire directory, providing iteration and dynamic lookup.

- **Don't bypass XDG precedence:** Never check embedded templates before user directories. Always respect XDG precedence (user > system > embedded) to allow customization.

- **Don't trust user input for paths:** Even with XDG directories, validate template names before constructing paths. Reject "..", "/", and other traversal attempts.

- **Don't parse templates at list time:** When listing templates, only parse metadata headers, not full template parsing. Full parsing is expensive and unnecessary for discovery.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| File embedding | Custom build script copying files to target dir | rust-embed crate | Handles dev/release modes, compression, proper error handling; custom solutions miss edge cases and add maintenance burden |
| XDG directory resolution | Manual ~/.local/share path construction | xdg crate | Respects XDG_DATA_HOME env var, handles system directories, proper precedence; manual construction is fragile and non-compliant |
| Path traversal validation | Custom regex or string checks | Established validation pattern (reject separators + ..) | Path traversal has subtle attack vectors; well-tested patterns are essential for security |
| Template name pattern matching | String contains/starts_with | glob crate (already in project) | Glob patterns support wildcards, bracket expressions; reimplementing is error-prone |
| Metadata serialization | Manual parsing of YAML/TOML frontmatter | serde + serde_yaml_ng/toml (already in project) | Serde ecosystem handles edge cases, validation, error reporting; custom parsers are fragile |

**Key insight:** Template library infrastructure is security-critical (path traversal) and standards-compliant (XDG). Use well-tested crates that handle edge cases rather than custom implementations.

## Common Pitfalls

### Pitfall 1: Path Traversal via Template Names

**What goes wrong:** User provides template name like `../../etc/passwd` or `templates/../../../sensitive.yaml`, bypassing intended template directories and reading arbitrary files.

**Why it happens:** Trusting user input when constructing file paths. Many implementations validate after joining paths instead of before.

**How to avoid:**
- Validate template name BEFORE any filesystem operations
- Reject names containing `..`, `/`, `\`, or starting with path separators
- Use allowlist validation (alphanumeric + underscore + hyphen + dot) rather than blocklist
- Never use Path::canonicalize for validation as it requires filesystem access

**Warning signs:**
- Template resolution errors mentioning unexpected file paths
- Security audit flags path construction with user input
- Tests don't cover malicious template name inputs

**Prevention pattern:**
```rust
// GOOD: Validate before constructing path
fn validate_name(name: &str) -> Result<(), Error> {
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(Error::InvalidName);
    }
    if !name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == '.') {
        return Err(Error::InvalidCharacters);
    }
    Ok(())
}

// BAD: Validate after constructing path
fn bad_validate(base: &Path, name: &str) -> Result<(), Error> {
    let path = base.join(name);
    let canonical = path.canonicalize()?; // Filesystem access, can fail
    if !canonical.starts_with(base) {
        return Err(Error::PathTraversal);
    }
    Ok(())
}
```

### Pitfall 2: XDG Precedence Reversal

**What goes wrong:** Embedded templates override user customizations, or system templates take precedence over user-specific versions.

**Why it happens:** Checking embedded templates before XDG directories, or using first-found logic in wrong order.

**How to avoid:**
- Always check XDG user directory first (`~/.local/share/cliscrape/templates/`)
- Then XDG system directories (`/usr/local/share`, `/usr/share`)
- Finally fall back to embedded templates
- Document precedence clearly in user-facing messages

**Warning signs:**
- User reports custom templates being ignored
- Templates work differently depending on installation method
- Unclear "template not found" errors when override should apply

**Correct precedence:**
```rust
// CORRECT: User > System > Embedded
fn resolve_template(name: &str) -> Option<TemplateSource> {
    // 1. User directory (highest priority)
    if let Some(path) = xdg_dirs.find_data_file(format!("templates/{}", name)) {
        return Some(TemplateSource::UserFile(path));
    }

    // 2. System directories (medium priority)
    // xdg find_data_file already checks all XDG_DATA_DIRS

    // 3. Embedded (lowest priority)
    if let Some(embedded) = EmbeddedTemplates::get(name) {
        return Some(TemplateSource::Embedded(embedded));
    }

    None
}
```

### Pitfall 3: Metadata Parse Failures Breaking Template Usage

**What goes wrong:** Template is functional but has invalid/missing metadata, causing template listing or info commands to fail or hide the template.

**Why it happens:** Treating metadata as required rather than optional, or failing to provide sensible defaults.

**How to avoid:**
- Make metadata extraction fault-tolerant with defaults
- Separate metadata parsing errors from template parsing errors
- Display templates even if metadata is incomplete
- Log metadata warnings without blocking template usage

**Warning signs:**
- Templates disappear from listings after minor metadata changes
- Valid templates rejected due to missing non-essential metadata fields
- Users can't use templates that parse and execute correctly

**Fault-tolerant approach:**
```rust
pub fn load_template_with_metadata(name: &str) -> Result<(Template, TemplateMetadata), Error> {
    let content = load_template_content(name)?;

    // Template parsing is critical - propagate errors
    let template = parse_template(&content)?;

    // Metadata parsing is best-effort - use defaults on failure
    let metadata = match extract_metadata(&content) {
        Ok(meta) => meta,
        Err(e) => {
            eprintln!("Warning: Failed to parse metadata for '{}': {}", name, e);
            TemplateMetadata::default_for(name)
        }
    };

    Ok((template, metadata))
}
```

### Pitfall 4: Large Template Library Increasing Binary Size

**What goes wrong:** Embedding many templates dramatically increases binary size, affecting distribution and memory footprint.

**Why it happens:** rust-embed includes all files in the embedded directory at compile time without optimization.

**How to avoid:**
- Use rust-embed's `compression` feature to compress embedded files
- Start with small, curated template library (5-10 core templates)
- Consider lazy-loading pattern where templates are fetched on first use
- Document binary size impact in comments and limit embedded scope

**Warning signs:**
- Binary size grows disproportionately with each added template
- CI/CD complains about artifact size
- Users report slow download/installation times

**Optimization approach:**
```toml
# Enable compression for embedded files
[dependencies]
rust-embed = { version = "8.11.0", features = ["compression"] }
```

```rust
// Start with focused library, not comprehensive coverage
#[derive(RustEmbed)]
#[folder = "templates/"]
// Include only core templates by default
#[include = "*.yaml"]
#[include = "*.toml"]
// Exclude test fixtures and documentation
#[exclude = "*.md"]
#[exclude = "tests/*"]
struct TemplateLibrary;
```

### Pitfall 5: Template Name Ambiguity Between Formats

**What goes wrong:** User has `cisco_ios_show_version.yaml` and `cisco_ios_show_version.toml`, causing unpredictable template resolution.

**Why it happens:** Template name doesn't include format extension, allowing multiple formats with same base name.

**How to avoid:**
- Include format extension in template name resolution
- Document that template names include extensions (`.yaml`, `.toml`, `.textfsm`)
- Add validation warning if multiple formats found for same base name
- Establish precedence order if supporting extension-less names (e.g., `.yaml` > `.toml` > `.textfsm`)

**Warning signs:**
- Template behavior changes depending on directory listing order
- Users report inconsistent results with same template name
- Tests fail intermittently due to template resolution races

**Disambiguation approach:**
```rust
fn resolve_template_with_format(name: &str) -> Result<TemplateSource, Error> {
    // If name includes extension, use it directly
    if name.ends_with(".yaml") || name.ends_with(".toml") || name.ends_with(".textfsm") {
        return resolve_exact(name);
    }

    // Otherwise, try formats in precedence order
    for ext in &[".yaml", ".toml", ".textfsm"] {
        let full_name = format!("{}{}", name, ext);
        if let Some(source) = try_resolve(&full_name) {
            return Ok(source);
        }
    }

    Err(Error::TemplateNotFound(name.to_string()))
}
```

## Code Examples

Verified patterns from research and crate documentation:

### Embedding Template Directory

```rust
// Source: https://docs.rs/rust-embed/8.11.0/rust_embed/
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "templates/"]
#[include = "*.yaml"]
#[include = "*.toml"]
#[include = "*.textfsm"]
struct EmbeddedTemplates;

// List all embedded template names
pub fn list_embedded() -> Vec<String> {
    EmbeddedTemplates::iter()
        .map(|name| name.into_owned())
        .collect()
}

// Get template content by name
pub fn get_embedded(name: &str) -> Option<EmbeddedFile> {
    EmbeddedTemplates::get(name)
}
```

### XDG Directory Resolution

```rust
// Source: https://docs.rs/xdg/3.0.0/xdg/
use xdg::BaseDirectories;

pub struct TemplateResolver {
    xdg: BaseDirectories,
}

impl TemplateResolver {
    pub fn new() -> Result<Self, xdg::BaseDirectoriesError> {
        let xdg = BaseDirectories::with_prefix("cliscrape")?;
        Ok(Self { xdg })
    }

    // Find template in XDG data directories
    // Searches: $XDG_DATA_HOME/cliscrape/templates/, then $XDG_DATA_DIRS/cliscrape/templates/
    pub fn find_user_template(&self, name: &str) -> Option<PathBuf> {
        self.xdg.find_data_file(format!("templates/{}", name))
    }

    // Create user template directory and place file
    pub fn save_user_template(&self, name: &str, content: &[u8]) -> Result<PathBuf, std::io::Error> {
        let path = self.xdg.place_data_file(format!("templates/{}", name))?;
        std::fs::write(&path, content)?;
        Ok(path)
    }

    // List all user template directories (returns in precedence order)
    pub fn list_user_template_dirs(&self) -> Vec<PathBuf> {
        self.xdg.get_data_dirs()
            .into_iter()
            .map(|base| base.join("templates"))
            .collect()
    }
}
```

### Secure Template Name Validation

```rust
// Source: https://www.stackhawk.com/blog/rust-path-traversal-guide-example-and-prevention/
// Pattern: Validate before any filesystem operations

use regex::Regex;
use std::sync::OnceLock;

pub fn validate_template_name(name: &str) -> Result<(), String> {
    // Reject empty names
    if name.is_empty() {
        return Err("Template name cannot be empty".to_string());
    }

    // Reject path separators (prevents directory traversal)
    if name.contains('/') || name.contains('\\') {
        return Err(format!(
            "Invalid template name '{}': path separators not allowed",
            name
        ));
    }

    // Reject path traversal sequences
    if name.contains("..") {
        return Err(format!(
            "Invalid template name '{}': parent directory references not allowed",
            name
        ));
    }

    // Reject absolute path indicators
    if name.starts_with('/') || name.starts_with('\\') {
        return Err(format!(
            "Invalid template name '{}': absolute paths not allowed",
            name
        ));
    }

    // Allowlist: alphanumeric, underscore, hyphen, dot only
    static VALID_NAME: OnceLock<Regex> = OnceLock::new();
    let pattern = VALID_NAME.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9_\-\.]+$").unwrap()
    });

    if !pattern.is_match(name) {
        return Err(format!(
            "Invalid template name '{}': only alphanumeric, underscore, hyphen, and dot allowed",
            name
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_names() {
        assert!(validate_template_name("cisco_ios_show_version").is_ok());
        assert!(validate_template_name("template-v1.2.yaml").is_ok());
        assert!(validate_template_name("simple.toml").is_ok());
    }

    #[test]
    fn test_path_traversal_rejected() {
        assert!(validate_template_name("../etc/passwd").is_err());
        assert!(validate_template_name("..\\windows\\system32").is_err());
        assert!(validate_template_name("normal/../traversal").is_err());
    }

    #[test]
    fn test_absolute_paths_rejected() {
        assert!(validate_template_name("/etc/passwd").is_err());
        assert!(validate_template_name("\\windows\\system32").is_err());
    }

    #[test]
    fn test_path_separators_rejected() {
        assert!(validate_template_name("subdir/template.yaml").is_err());
        assert!(validate_template_name("subdir\\template.yaml").is_err());
    }
}
```

### Template Listing with Metadata

```rust
use comfy_table::{Table, presets};

pub fn list_templates(service: &TemplateService, filter: Option<&str>) -> Result<String, Error> {
    let mut templates = service.discover_all()?;

    // Apply filter if provided
    if let Some(pattern) = filter {
        let glob_pattern = glob::Pattern::new(pattern)
            .map_err(|e| Error::InvalidPattern(e.to_string()))?;
        templates.retain(|(name, _)| glob_pattern.matches(name));
    }

    // Sort by name for consistent output
    templates.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Build table
    let mut table = Table::new();
    table.load_preset(presets::UTF8_FULL);
    table.set_header(vec!["Name", "Description", "Compatibility", "Version", "Source"]);

    for (name, info) in templates {
        table.add_row(vec![
            name,
            info.metadata.description,
            info.metadata.compatibility,
            info.metadata.version,
            match info.source {
                TemplateSource::UserFile(_) => "User Override",
                TemplateSource::Embedded(_) => "Embedded",
            },
        ]);
    }

    Ok(table.to_string())
}
```

### Template Detail Display

```rust
pub fn show_template_details(service: &TemplateService, name: &str, include_source: bool) -> Result<String, Error> {
    let (template, metadata, source) = service.load_with_metadata(name)?;

    let mut output = String::new();

    // Metadata section
    output.push_str(&format!("Template: {}\n", name));
    output.push_str(&format!("Description: {}\n", metadata.description));
    output.push_str(&format!("Compatibility: {}\n", metadata.compatibility));
    output.push_str(&format!("Version: {}\n", metadata.version));
    output.push_str(&format!("Author: {}\n", metadata.author));
    if let Some(maintainer) = &metadata.maintainer {
        output.push_str(&format!("Maintainer: {}\n", maintainer));
    }

    // Source location
    match source {
        TemplateSource::UserFile(path) => {
            output.push_str(&format!("Source: User Override ({})\n", path.display()));
        }
        TemplateSource::Embedded(_) => {
            output.push_str("Source: Embedded\n");
        }
    }

    // Fields extracted
    output.push_str("\nFields Extracted:\n");
    for field_name in template.list_fields() {
        output.push_str(&format!("  - {}\n", field_name));
    }

    // Optional source code
    if include_source {
        output.push_str("\nTemplate Source:\n");
        output.push_str("─".repeat(60).as_str());
        output.push('\n');
        output.push_str(&template.source_content());
        output.push('\n');
        output.push_str("─".repeat(60).as_str());
        output.push('\n');
    }

    Ok(output)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Build scripts copying files to target | rust-embed with dual dev/release modes | rust-embed 1.0 (2019) | Faster development iteration, single source of truth for embedded resources |
| Manual ~/.local/share construction | xdg crate BaseDirectories | xdg 2.0 (2019) | Standards compliance, proper precedence, env var support |
| String-based path validation | Allowlist validation before filesystem ops | OWASP recommendations (ongoing) | More secure, catches edge cases, no filesystem dependency |
| Individual include_str! per file | Directory-level embedding with iteration | rust-embed 1.0+ | Scalable to large template libraries, dynamic discovery |
| clap 2.x Args/SubCommand | clap 4.x derive API | clap 4.0 (2022) | Less boilerplate, better error messages, compile-time validation |

**Deprecated/outdated:**
- **include_flate macro**: Replaced by rust-embed's built-in compression feature
- **clap 2.x App builder pattern**: Use clap 4.x derive API for better ergonomics
- **directories crate on Linux**: Use xdg for Linux/Unix; directories is for cross-platform but doesn't follow XDG spec

## Template Naming Convention Research

### Industry Standard (ntc-templates)

Based on research, the network automation industry standard established by ntc-templates follows this convention:

**Format:** `{vendor}_{os}_{command_with_underscores}.{extension}`

**Examples:**
- `cisco_ios_show_version.textfsm`
- `cisco_ios_show_interfaces.textfsm`
- `juniper_junos_show_route.textfsm`
- `arista_eos_show_ip_bgp.textfsm`

**Component breakdown:**
- **vendor**: Network device vendor (cisco, juniper, arista, etc.)
- **os**: Operating system (ios, junos, eos, nxos, etc.)
- **command**: CLI command with spaces replaced by underscores (show_version, show_interfaces)
- **extension**: Format indicator (.textfsm, .yaml, .toml)

**Key principles:**
- Lowercase throughout
- Underscores separate all components
- No hyphens (underscores only)
- Extension indicates template format
- Command portion is descriptive of actual CLI command

**Benefits:**
- Self-documenting (name tells you vendor, OS, and command)
- Easily searchable and filterable
- Natural alphabetical grouping by vendor
- Compatible with filesystem restrictions
- Established convention in network automation community

### Recommendation for cliscrape

**Primary convention:** Follow ntc-templates standard for network device templates
- Format: `{vendor}_{os}_{command}.{yaml|toml|textfsm}`
- Example: `cisco_ios_show_version.yaml`

**Flexibility:** Allow users to create custom naming if needed
- Validation still applies (no path separators, no traversal)
- Users can choose descriptive names for non-network templates
- Embedded library follows standard convention for consistency

## Open Questions

### 1. Initial Template Library Scope

**What we know:**
- ntc-templates has 200+ templates covering major vendors
- Embedding large libraries increases binary size
- rust-embed compression feature mitigates size impact
- Users can add custom templates via XDG directories

**What's unclear:**
- How many templates to include in initial embedded library?
- Which vendor/device templates are highest priority?
- Should we start minimal (5-10 core templates) or comprehensive (50+)?

**Recommendation:**
- Start with small curated set (5-10 templates) covering common use cases
- Choose variety: Cisco IOS, Juniper JunOS, Arista EOS show version/interfaces
- Enable compression feature to minimize binary impact
- Document how users can contribute additional templates
- Can expand embedded library in future releases based on usage

### 2. Metadata Embedding Strategy

**What we know:**
- Modern templates (YAML/TOML) can include metadata in document structure
- TextFSM templates use comment headers for metadata
- Parsing full template just for metadata is expensive
- Metadata needed for template listing and discovery

**What's unclear:**
- Should metadata be separate sidecar files or embedded in templates?
- If embedded, how to extract efficiently without full template parse?
- How to handle templates missing metadata?

**Recommendation:**
- Use embedded metadata in template files (no separate sidecar files)
- Modern formats: add top-level `metadata` section in YAML/TOML
- TextFSM: parse only leading comment lines (stop at first Value definition)
- Provide sensible defaults for missing metadata fields
- Cache parsed metadata in memory after first access

### 3. Template Format Precedence

**What we know:**
- cliscrape supports .textfsm, .yaml, .toml formats
- Same template might exist in multiple formats (user migration path)
- Users might want different formats for different use cases

**What's unclear:**
- If `cisco_ios_show_version` exists in multiple formats, which takes precedence?
- Should template name include extension or be format-agnostic?
- How to communicate format precedence to users?

**Recommendation:**
- Require extension in template names for explicitness: `cisco_ios_show_version.yaml`
- No automatic format precedence; exact name match only
- If users want format-agnostic names, they specify `--template-format` flag
- Document that template names include extensions in user guide
- Reduces ambiguity and makes behavior predictable

## Sources

### Primary (HIGH confidence)

- [rust-embed crate documentation](https://docs.rs/rust-embed/8.11.0/) - Embedding static files
- [xdg crate documentation](https://docs.rs/xdg/3.0.0/) - XDG Base Directory implementation
- [clap crate documentation](https://docs.rs/clap/latest/clap/) - CLI argument parsing
- [Rust Path Traversal Guide: Example and Prevention](https://www.stackhawk.com/blog/rust-path-traversal-guide-example-and-prevention/) - Security validation
- [xdg crate on crates.io](https://crates.io/crates/xdg/3.0.0) - Version and feature information
- [rust-embed crate on crates.io](https://crates.io/crates/rust-embed/8.11.0) - Version and feature information

### Secondary (MEDIUM confidence)

- [ntc-templates GitHub repository](https://github.com/networktocode/ntc-templates) - Network template naming conventions
- [ntc-templates documentation](https://ntc-templates.readthedocs.io/en/latest/) - Template organization patterns
- [TextFSM CLI Table documentation](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_clitable.html) - Template naming conventions
- [Network Automation Template Organization](https://opsmill.com/blog/network-naming-conventions/) - Industry naming patterns
- [How to Build CLI Applications with Clap in Rust](https://oneuptime.com/blog/post/2026-02-03-rust-clap-cli-applications/view) - CLI patterns
- [Rust CLI with Clap Tutorial](https://codezup.com/rust-clap-cli-tutorial/) - Subcommand patterns
- [YAML Frontmatter conventions](https://gohugo.io/content-management/front-matter/) - Metadata best practices
- [include_dir vs rust-embed comparison](https://docs.rs/include_dir) - Embedding alternatives

### Tertiary (LOW confidence)

- Web search results on Rust CLI patterns - General guidance, needs verification with official docs
- Web search results on network automation - Industry trends, verify with specific project standards

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - All crates are well-established, latest stable versions verified via cargo
- Architecture: HIGH - Patterns verified with official crate documentation and security resources
- Pitfalls: HIGH - Based on documented security vulnerabilities and crate best practices
- Template naming: MEDIUM - ntc-templates is industry standard but specific to Python ecosystem; adapting to Rust context

**Research date:** 2026-02-22
**Valid until:** ~90 days (March 2026) - Stack is stable; rust-embed and xdg are mature crates with infrequent breaking changes

**Libraries at current stable versions:**
- rust-embed 8.11.0 (latest stable)
- xdg 3.0.0 (latest stable)
- All other dependencies already in project at current versions

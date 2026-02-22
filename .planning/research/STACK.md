# Technology Stack

**Project:** cliscrape v1.5 Template Ecosystem & Production Hardening
**Researched:** 2026-02-22

## Context

This stack analysis focuses ONLY on additions/changes needed for v1.5 milestone features:
- Template library management
- Template discovery mechanism
- Validation testing suite
- Production logging
- Documentation generation

**Existing stack (DO NOT change):**
- Core: Rust with pest/pest_derive for TextFSM parsing
- TUI: ratatui + crossterm
- CLI: clap with derive feature
- Serialization: serde + serde_json + serde_yaml_ng + toml
- Testing: criterion (benchmarks) + assert_cmd (CLI e2e)

## Recommended Stack

### Template Library Management
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| rust-embed | 8.11.0 | Embed templates in binary | Industry standard for static asset embedding. Compile-time embedding in release mode, filesystem loading in dev mode. Supports metadata-only mode for listing available templates. Zero runtime overhead. Latest version (Jan 14 2026) adds INSTA_PENDING_DIR for hermetic builds. |
| directories-next | 2.0.0 | XDG-compliant user directories | Mid-level library for cross-platform config/cache/data directories. Provides application-specific paths (`~/.config/cliscrape/templates/`, `~/.local/share/cliscrape/templates/`). More ergonomic than low-level dirs-next for project-specific paths. |

### Template Discovery
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| glob | 0.3 | Template file discovery | Already in Cargo.toml. Pattern-based file discovery for user templates. Well-established, minimal dependencies. |
| serde | 1.0 | Template metadata | Already in Cargo.toml. Deserialize embedded template metadata (name, vendor, command, description) from bundled index. |

### Validation Testing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| insta | 1.46.0 | Snapshot testing | Most popular Rust snapshot testing framework. Batteries-included with YAML/JSON/TOML serialization. VS Code extension for reviewing snapshots. Supports redaction for timestamps/IDs. Perfect for verifying parser output against golden files. Latest version (Jan 3 2026) adds Bazel compatibility. |
| assert_cmd | 2.0 | CLI integration tests | Already in dev-dependencies. E2e CLI testing for template discovery and parsing workflows. |

### Production Logging
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tracing | 0.27 | Structured instrumentation | Current standard for Rust observability. Unified API for logging and span-based tracing. Ecosystem-wide adoption (tokio, axum, ripgrep). Supports async contexts. Superior to env_logger for production use. |
| tracing-subscriber | 0.3.20 | Subscriber implementation | Companion to tracing. Provides EnvFilter (env_logger-compatible syntax), JSON formatting for production, pretty formatting for dev. Runtime log level control via RUST_LOG env var. |

### Documentation
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| mdbook | 0.4.x | User guide | Official Rust documentation tool. Creates clean, navigable books from Markdown. Used by rust-lang for official books. Rich plugin ecosystem (admonish for callouts, alerts for GitHub-style notices, mermaid for diagrams). Requires Rust 1.88+. |
| cargo-doc | (built-in) | API reference | Standard Rust doc generator. Already used for library API docs. Zero additional dependencies. Complements mdbook for reference vs guide documentation. |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Template Embedding | rust-embed | include_dir | include_dir increases compile time significantly (5s compile, 730MB RAM, 72MB binary for large directories). rust-embed is optimized for this use case with metadata-only mode. |
| Directories | directories-next | dirs-next | dirs-next is low-level (16 functions). directories-next provides application-specific project/cache paths ergonomically. Use dirs-next for simple cases, directories-next for project-specific paths. |
| Logging | tracing | env_logger | env_logger is simpler but lacks structured logging, async support, span context. Tracing is production-standard for CLI tools in 2026. Supersedes env_logger. |
| Snapshot Testing | insta | goldenfile, goldie, goldentests | Insta has largest community, VS Code integration, best DX (cargo insta test), most flexible serialization options. goldie is simpler but less feature-rich. |
| User Docs | mdbook | cargo-doc only | cargo-doc is reference-focused. mdbook provides guide/tutorial format needed for template authoring, CLI usage examples, troubleshooting. Both serve complementary purposes. |

## Anti-Recommendations

**DO NOT ADD:**

| Technology | Why Avoid |
|-----------|-----------|
| env_logger | Superseded by tracing-subscriber for production. Lacks structured output, async context, span tracking. Use tracing instead. |
| slog | More complex API than tracing. Less ecosystem adoption. Tracing is current standard as of 2026. |
| log crate | Lower-level facade. Use tracing which provides compatible bridge if needed via tracing-log. |
| git2 | Template discovery from git repos deferred to v2.0. Adds 3MB+ to binary size. Filesystem + rust-embed sufficient for v1.5. |
| Custom template registry | Use filesystem + rust-embed for v1.5. Remote registry deferred to future milestone. |
| include_dir | Less optimized than rust-embed for asset embedding. Performance issues with large directories. |

## Installation

### Production Dependencies

```bash
# Template library management
cargo add rust-embed@8.11.0
cargo add directories-next@2.0.0

# Production logging
cargo add tracing@0.27
cargo add tracing-subscriber@0.3.20 --features env-filter,json
```

### Development Dependencies

```bash
# Validation testing
cargo add --dev insta@1.46.0 --features yaml,json
```

### Documentation Tools

```bash
# User guide (install globally, not in Cargo.toml)
cargo install mdbook --version ^0.4

# Optional mdbook plugins
cargo install mdbook-admonish  # Material Design callouts
cargo install mdbook-mermaid   # Diagram support
```

## Integration Points with Existing Codebase

### Template Discovery Flow

```rust
use rust_embed::RustEmbed;
use directories_next::ProjectDirs;

#[derive(RustEmbed)]
#[folder = "templates/"]
struct BuiltInTemplates;

// Priority: CLI arg > user dir > embedded
fn discover_template(name: &str) -> Result<String> {
    // 1. Check explicit --template path (already implemented)
    if let Some(path) = cli_template_path {
        return std::fs::read_to_string(path);
    }

    // 2. Search user directory ~/.local/share/cliscrape/templates/{vendor}/{command}.yaml
    if let Some(proj_dirs) = ProjectDirs::from("com", "cliscrape", "cliscrape") {
        let user_template = proj_dirs.data_dir()
            .join("templates")
            .join(format!("{}.yaml", name));
        if user_template.exists() {
            return std::fs::read_to_string(user_template);
        }
    }

    // 3. Fall back to embedded BuiltInTemplates::get("cisco/show_version.yaml")
    BuiltInTemplates::get(&format!("{}.yaml", name))
        .map(|f| String::from_utf8_lossy(&f.data).to_string())
        .ok_or_else(|| anyhow!("Template not found: {}", name))
}
```

### Logging Integration

```rust
use tracing::{info, warn, error, debug, instrument};
use tracing_subscriber::EnvFilter;

// Initialize once in main()
fn init_logging() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("cliscrape=info"))
        )
        .with_target(false)
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .init();
}

// Use throughout codebase
#[instrument(skip(input), fields(template_len = template.len(), input_lines = input.lines().count()))]
fn parse_template(template: &str, input: &str) -> Result<Vec<Record>> {
    debug!("Loading template");
    info!("Parsing input");

    let records = engine.parse(input)?;

    info!(record_count = records.len(), "Parsing complete");
    Ok(records)
}

// For production: export RUST_LOG=cliscrape=debug
// For JSON output: use .json() formatter instead of .fmt()
```

### Snapshot Testing Pattern

```rust
use insta::assert_yaml_snapshot;

#[test]
fn test_cisco_show_version() {
    let template = include_str!("../templates/cisco/show_version.yaml");
    let input = include_str!("../fixtures/cisco_ios_show_version.txt");

    let result = parse(template, input).unwrap();

    // First run: cargo insta test --review
    // Accept snapshots: cargo insta test --accept
    // Update snapshots: cargo insta test --accept
    assert_yaml_snapshot!(result);
}

#[test]
fn test_juniper_show_interfaces_with_redaction() {
    let template = include_str!("../templates/juniper/show_interfaces.yaml");
    let input = include_str!("../fixtures/juniper_show_interfaces.txt");

    let result = parse(template, input).unwrap();

    // Redact timestamps and dynamic values
    insta::with_settings!({
        filters => vec![
            (r"\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}", "[TIMESTAMP]"),
            (r"([0-9a-f]{2}:){5}[0-9a-f]{2}", "[MAC_ADDRESS]"),
        ]
    }, {
        assert_yaml_snapshot!(result);
    });
}
```

### Template Metadata Index

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct TemplateIndex {
    templates: Vec<TemplateMetadata>,
}

#[derive(Serialize, Deserialize)]
struct TemplateMetadata {
    name: String,
    vendor: String,
    command: String,
    description: String,
    path: String,
}

// Embed index.yaml alongside templates
#[derive(RustEmbed)]
#[folder = "templates/"]
struct BuiltInTemplates;

fn list_templates() -> Result<Vec<TemplateMetadata>> {
    let index_data = BuiltInTemplates::get("index.yaml")
        .ok_or_else(|| anyhow!("Template index not found"))?;

    let index: TemplateIndex = serde_yaml_ng::from_slice(&index_data.data)?;
    Ok(index.templates)
}
```

## Feature Flags

No new feature flags required for core functionality. Optional:

```toml
[features]
# Enable embedded templates in dev mode (default: filesystem)
debug-embed = ["rust-embed/debug-embed"]

# JSON logging format (default: pretty)
json-logs = ["tracing-subscriber/json"]
```

## Binary Size Impact

| Addition | Estimated Size |
|----------|---------------|
| rust-embed (code) | ~10KB |
| Embedded templates (20 files × 2KB avg) | ~40KB |
| directories-next | ~15KB |
| tracing + tracing-subscriber | ~150KB |
| insta (dev-only) | 0KB |
| mdbook (external tool) | 0KB |
| **Total** | **~215KB** |

**Current binary:** ~4MB (v1.0)
**Projected binary:** ~4.2MB (v1.5)
**Impact:** +5% size increase, acceptable for production features.

## Documentation Structure

```
docs/
├── book.toml              # mdbook configuration
├── src/
│   ├── SUMMARY.md         # Book structure
│   ├── introduction.md    # Getting started
│   ├── installation.md    # Install guide
│   ├── cli/
│   │   ├── usage.md       # CLI commands
│   │   └── examples.md    # Common workflows
│   ├── templates/
│   │   ├── authoring.md   # Template syntax guide
│   │   ├── discovery.md   # Template discovery mechanism
│   │   └── library.md     # Built-in template reference
│   ├── tui/
│   │   ├── live-lab.md    # Live Lab usage
│   │   └── state-tracer.md # State Tracer usage
│   └── troubleshooting.md # Common issues
└── theme/                 # Custom CSS/styling
```

Build: `mdbook build docs/`
Serve: `mdbook serve docs/` (live reload at http://localhost:3000)

## Version Verification Status

| Crate | Version | Verification | Confidence |
|-------|---------|--------------|------------|
| rust-embed | 8.11.0 | WebSearch (docs.rs, Jan 14 2026) | HIGH |
| directories-next | 2.0.0 | WebSearch (docs.rs, 2026) | HIGH |
| insta | 1.46.0 | WebSearch (GitHub release, Jan 3 2026) | HIGH |
| tracing | 0.27 | WebSearch (crates.io, 2026) | MEDIUM |
| tracing-subscriber | 0.3.20 | WebSearch (docs.rs, 2026) | HIGH |
| mdbook | 0.4.x | WebSearch (docs, requires Rust 1.88+) | MEDIUM |

**Note:** tracing 0.27 referenced in documentation but should be verified during installation (may be 0.1.x series, verify actual version). tracing-subscriber 0.3.20 confirmed as latest stable.

## Migration Notes

### From env_logger to tracing

If any existing code uses env_logger:

```rust
// Old (env_logger)
env_logger::init();
log::info!("Message");

// New (tracing)
tracing_subscriber::fmt().init();
tracing::info!("Message");
```

Tracing is compatible with log crate via tracing-log bridge, so existing log macros will work automatically.

### Snapshot Testing Workflow

```bash
# Initial setup
cargo add --dev insta --features yaml,json
cargo install cargo-insta

# Write test with assert_yaml_snapshot!
# Run tests (will create .snap.new files)
cargo insta test

# Review snapshots
cargo insta review

# Accept all snapshots
cargo insta accept
```

Snapshots stored in `src/snapshots/` by default. Commit `.snap` files to git, ignore `.snap.new` files.

## Sources

### Template Embedding
- [rust-embed 8.11.0 documentation](https://docs.rs/crate/rust-embed/latest)
- [rust-embed crates.io](https://crates.io/crates/rust-embed)
- [include_dir documentation](https://docs.rs/include_dir/latest/include_dir/)
- [Rust embedded resources discussion](https://users.rust-lang.org/t/packaging-assets-in-a-library-crate/4133)

### Directory Management
- [directories-next documentation](https://docs.rs/directories-next)
- [dirs-next vs directories-next comparison](https://users.rust-lang.org/t/directories-next-dirs-next-the-new-home-for-directories-crates/42788)
- [directories-next on lib.rs](https://lib.rs/crates/directories-next)

### Snapshot Testing
- [Insta snapshot testing](https://insta.rs/)
- [Insta GitHub releases](https://github.com/mitsuhiko/insta/releases)
- [Using Insta for Rust snapshot testing](https://blog.logrocket.com/using-insta-rust-snapshot-testing/)
- [Complete Guide to Rust Testing](https://blog.blackwell-systems.com/posts/rust-testing-comprehensive-guide/)
- [Snapshot testing guide](https://rustprojectprimer.com/testing/snapshot.html)

### Production Logging
- [Structured JSON logs with tracing](https://oneuptime.com/blog/post/2026-01-25-structured-json-logs-tracing-rust/view)
- [Rust structured logs with OpenTelemetry](https://oneuptime.com/blog/post/2026-01-07-rust-tracing-structured-logs/view)
- [tracing documentation](https://docs.rs/tracing)
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/)
- [Comparing logging and tracing in Rust](https://blog.logrocket.com/comparing-logging-tracing-rust/)
- [Getting started with Tracing (Tokio)](https://tokio.rs/tokio/topics/tracing)
- [env_logger to tracing migration](https://fdeantoni.medium.com/from-env-logger-to-tokio-tracing-and-opentelemetry-adb247c0d40f)

### Documentation
- [mdBook documentation](https://rust-lang.github.io/mdBook/)
- [mdBook preprocessors guide](https://rust-lang.github.io/mdBook/for_developers/preprocessors.html)
- [mdBook third-party plugins](https://github.com/rust-lang/mdBook/wiki/Third-party-plugins)
- [cargo doc documentation](https://doc.rust-lang.org/cargo/commands/cargo-doc.html)
- [mdBook vs cargo doc discussion](https://users.rust-lang.org/t/cargo-doc-instead-of-mdbook/103772)

### Testing Best Practices
- [Mastering Testing in Rust](https://codezup.com/mastering-testing-in-rust-best-practices-techniques/)
- [Testing CLI applications](https://rust-cli.github.io/book/tutorial/testing.html)
- [Rust testing organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

---

*Stack research for: cliscrape v1.5 Template Ecosystem & Production Hardening*
*Researched: 2026-02-22*
*Focus: NEW capabilities only (template library, discovery, validation, logging, docs)*

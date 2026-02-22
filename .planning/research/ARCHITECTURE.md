# Architecture Research

**Domain:** CLI Parser with Template Library and Production Features
**Researched:** 2026-02-22
**Confidence:** HIGH

## Integration Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                          CLI Layer (main.rs)                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │ Parse Command │  │Debug Command │  │Convert Command│              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         │                  │                  │                       │
├─────────┴──────────────────┴──────────────────┴───────────────────────┤
│                    Template Discovery Layer (NEW)                    │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Template Resolver: path → PathBuf | name → library lookup  │    │
│  │  • Embedded Library Index (include_dir!)                     │    │
│  │  • XDG User Templates ($XDG_DATA_HOME/cliscrape/templates)  │    │
│  │  • CWD Fallback (existing behavior)                          │    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                    Template Loading Layer (EXISTING)                 │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐              │
│  │TextFsmLoader │  │ YAMLLoader   │  │  TOMLLoader  │              │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘              │
│         └──────────────────┴──────────────────┘                      │
│                             ↓                                        │
│                        TemplateIR                                    │
├─────────────────────────────────────────────────────────────────────┤
│                         Engine Layer (EXISTING)                      │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  Template::from_ir() → Compiled FSM with macros resolved    │    │
│  └─────────────────────────────────────────────────────────────┘    │
│  ┌─────────────────────────────────────────────────────────────┐    │
│  │  FsmParser::parse() → Vec<HashMap<String, serde_json::Value>>│    │
│  └─────────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────────┤
│                    Observability Layer (NEW)                         │
│  ┌──────────────────┐  ┌──────────────────┐                         │
│  │ tracing::info!() │  │ Structured Logs  │                         │
│  │ (spans for ops)  │  │ (JSON formatter) │                         │
│  └──────────────────┘  └──────────────────┘                         │
└─────────────────────────────────────────────────────────────────────┘

      Supporting Systems (Independent, New Components)
      ┌─────────────────────┐  ┌─────────────────────┐
      │ Validation Suite    │  │ Documentation Gen   │
      │ (templates/ tests)  │  │ (template catalog)  │
      └─────────────────────┘  └─────────────────────┘
```

## Component Responsibilities

| Component | Responsibility | Integration Point | New/Modified |
|-----------|----------------|-------------------|--------------|
| **Template Resolver** | Resolve template spec (path/name) to PathBuf | `resolve_template_spec()` in main.rs | **MODIFIED** - extend existing function |
| **Embedded Library** | Bundle pre-built templates into binary | Compile-time `include_dir!()` macro | **NEW** - static data |
| **Library Index** | Map template names to files (vendor_os_command) | Read by Template Resolver | **NEW** - metadata structure |
| **XDG Config Discovery** | Find user-installed templates | `dirs` crate for XDG paths | **NEW** - search path extension |
| **Template Loaders** | Parse template formats into TemplateIR | Existing `template::loader` module | **EXISTING** - no changes |
| **FsmParser** | Parse CLI output using compiled template | Existing `lib.rs` API | **EXISTING** - no changes |
| **Validation Suite** | Automated tests for library templates | Separate test harness | **NEW** - independent test suite |
| **Tracing Integration** | Structured logging for operations | `main.rs` spans, library log events | **NEW** - instrumentation layer |
| **Error Formatter** | Format errors per `--error-format` | Existing `print_error()` | **EXISTING** - extend for new error types |

## Recommended Project Structure

```
cliscrape/
├── src/
│   ├── main.rs                      # MODIFIED: add tracing, extend resolve_template_spec
│   ├── lib.rs                       # EXISTING: no changes
│   ├── cli.rs                       # MODIFIED: add --template-library flag
│   ├── engine/                      # EXISTING: no changes
│   ├── template/
│   │   ├── loader.rs                # EXISTING: no changes
│   │   ├── modern.rs                # EXISTING: no changes
│   │   ├── convert.rs               # EXISTING: no changes
│   │   ├── library.rs               # NEW: embedded library + index parsing
│   │   └── discovery.rs             # NEW: template resolution logic
│   ├── transcript/                  # EXISTING: no changes
│   ├── tui/                         # EXISTING: no changes (verify separately)
│   └── output.rs                    # EXISTING: no changes
├── templates/                       # EXTENDED: add comprehensive library
│   ├── index                        # NEW: ntc-templates-style index file
│   ├── cisco_ios/
│   │   ├── show_version.textfsm
│   │   ├── show_interfaces.yaml
│   │   └── show_ip_route.yaml
│   ├── cisco_nxos/
│   ├── juniper_junos/
│   └── arista_eos/
├── tests/
│   ├── validation/                  # NEW: validation test suite
│   │   ├── test_library_templates.rs
│   │   ├── fixtures/                # Real device outputs
│   │   │   ├── cisco_ios_show_version.txt
│   │   │   └── ...
│   │   └── snapshots/               # Expected parsed output
│   └── integration/                 # EXISTING: e2e CLI tests
└── Cargo.toml                       # MODIFIED: add deps (include_dir, dirs, tracing)
```

### Structure Rationale

- **template/library.rs:** Encapsulates embedded library logic, keeping main.rs clean
- **template/discovery.rs:** Separates resolution logic from CLI parsing (testable in isolation)
- **templates/index:** Follows ntc-templates convention (vendor_os_command mapping)
- **tests/validation/:** Separate from unit tests, focuses on template correctness vs device outputs
- **Validation fixtures:** Real device outputs ensure templates work in production

## Architectural Patterns

### Pattern 1: Compile-Time Template Embedding

**What:** Bundle templates directory into binary at compile-time using `include_dir!()` macro

**When to use:** Always for shipped library templates (guarantees availability without network/filesystem)

**Trade-offs:**
- **Pro:** Zero runtime dependencies, fast lookup, guaranteed availability
- **Pro:** No installation step, works immediately after `cargo install`
- **Con:** Binary size increases (~100KB for 50 templates, acceptable)
- **Con:** Updates require recompile (mitigated by XDG user templates)

**Example:**
```rust
// src/template/library.rs
use include_dir::{include_dir, Dir};

static TEMPLATE_LIBRARY: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

pub fn get_embedded_template(name: &str) -> Option<&'static str> {
    TEMPLATE_LIBRARY
        .get_file(&format!("{}.textfsm", name))
        .and_then(|f| f.contents_utf8())
}
```

### Pattern 2: Layered Template Discovery (XDG-Aware)

**What:** Multi-source template resolution with priority: CLI path → XDG user → embedded library → CWD

**When to use:** When users need to override shipped templates or add custom ones

**Trade-offs:**
- **Pro:** Follows Linux/macOS conventions (`~/.local/share/cliscrape/templates/`)
- **Pro:** Users can shadow shipped templates without modifying binary
- **Con:** More complex resolution logic (must check multiple locations)
- **Con:** Platform-specific behavior (XDG on Linux, `~/Library` on macOS)

**Example:**
```rust
// src/template/discovery.rs
use dirs::data_local_dir;
use std::path::{Path, PathBuf};

pub fn resolve_template_path(spec: &str, format_filter: TemplateFormat) -> anyhow::Result<TemplateSource> {
    // 1. If spec is existing path, use directly
    if Path::new(spec).exists() {
        return Ok(TemplateSource::Path(PathBuf::from(spec)));
    }

    // 2. Check XDG user templates
    if let Some(xdg_path) = resolve_xdg_template(spec, format_filter) {
        return Ok(TemplateSource::Path(xdg_path));
    }

    // 3. Check embedded library
    if let Some(embedded) = library::lookup_template(spec) {
        return Ok(TemplateSource::Embedded(embedded));
    }

    // 4. Fallback: CWD identifier search (existing behavior)
    resolve_cwd_identifier(spec, format_filter)
}

fn resolve_xdg_template(name: &str, format: TemplateFormat) -> Option<PathBuf> {
    let data_dir = data_local_dir()?;
    let template_dir = data_dir.join("cliscrape/templates");

    for ext in format.extensions() {
        let path = template_dir.join(format!("{}.{}", name, ext));
        if path.exists() {
            return Some(path);
        }
    }
    None
}
```

### Pattern 3: Index-Based Template Lookup

**What:** Central `templates/index` file mapping template names to files (ntc-templates pattern)

**When to use:** When you have many templates and need structured lookup

**Trade-offs:**
- **Pro:** Explicit mapping, supports vendor/os/command hierarchy
- **Pro:** Enables metadata (description, version, author)
- **Con:** Extra maintenance (must update index when adding templates)
- **Con:** Stale index if files added without updating

**Example:**
```rust
// templates/index format (similar to ntc-templates)
// vendor, os, command, template_file
cisco, ios, show_version, cisco_ios_show_version.textfsm
cisco, ios, show_interfaces, cisco_ios_show_interfaces.yaml
cisco, nxos, show_version, cisco_nxos_show_version.textfsm
juniper, junos, show_version, juniper_junos_show_version.yaml

// src/template/library.rs
pub struct TemplateIndex {
    entries: Vec<IndexEntry>,
}

#[derive(Debug)]
pub struct IndexEntry {
    pub vendor: String,
    pub os: String,
    pub command: String,
    pub file: String,
}

impl TemplateIndex {
    pub fn from_embedded() -> Result<Self> {
        let index_content = TEMPLATE_LIBRARY.get_file("index")
            .and_then(|f| f.contents_utf8())
            .ok_or_else(|| anyhow::anyhow!("Missing index file in embedded templates"))?;

        let entries = index_content
            .lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| {
                let parts: Vec<_> = l.split(',').map(|s| s.trim()).collect();
                IndexEntry {
                    vendor: parts[0].to_string(),
                    os: parts[1].to_string(),
                    command: parts[2].to_string(),
                    file: parts[3].to_string(),
                }
            })
            .collect();

        Ok(Self { entries })
    }

    pub fn lookup(&self, name: &str) -> Option<&IndexEntry> {
        // Support both "cisco_ios_show_version" and hierarchical lookup
        self.entries.iter().find(|e| {
            let canonical = format!("{}_{}__{}", e.vendor, e.os, e.command.replace(' ', '_'));
            canonical == name
        })
    }
}
```

### Pattern 4: Tracing-Based Structured Logging

**What:** Use `tracing` crate with spans for operation context, JSON output for production

**When to use:** For CLI tools that need both human-readable dev logs and machine-parseable prod logs

**Trade-offs:**
- **Pro:** Unified API for logs and distributed tracing (future-proof for APM)
- **Pro:** Automatic context propagation via spans (no manual threading of request IDs)
- **Pro:** `RUST_LOG` env var for filtering (standard Rust convention)
- **Con:** More complex than simple `eprintln!()` debugging
- **Con:** JSON logs harder to read without `jq` or log aggregator

**Example:**
```rust
// Cargo.toml additions
// [dependencies]
// tracing = "0.1"
// tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

// src/main.rs
use tracing::{info, warn, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() {
    // Initialize tracing with JSON formatting if CLISCRAPE_LOG_JSON=1
    let use_json = std::env::var("CLISCRAPE_LOG_JSON").is_ok();

    if use_json {
        tracing_subscriber::registry()
            .with(EnvFilter::from_env("RUST_LOG"))
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(EnvFilter::from_env("RUST_LOG"))
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    // ... rest of main
}

#[instrument(skip(parser))] // Don't log large parser in span
fn run_command(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Parse { template, .. } => {
            let _parse_span = tracing::info_span!("parse_command", template = %template).entered();

            info!("Resolving template");
            let template_path = resolve_template_spec(&template, template_format)?;

            info!(path = %template_path.display(), "Loading template");
            let (parser, warnings) = FsmParser::from_file_with_warnings(&template_path)?;

            for warning in &warnings {
                warn!(kind = %warning.kind, message = %warning.message, "Template warning");
            }

            // ... parsing logic

            info!(records = all_results.len(), sources = input_sources.len(), "Parse complete");
        }
    }
}
```

## Data Flow

### Template Resolution Flow (NEW + EXISTING)

```
User Input: cliscrape parse -t <spec> input.txt
    ↓
resolve_template_spec(spec, format_filter)
    ↓
┌─────────────────────────────────────┐
│ 1. Is spec an existing file path?   │ → YES → Return PathBuf
│    Path::new(spec).exists()          │
└─────────────────┬───────────────────┘
                  NO
                  ↓
┌─────────────────────────────────────┐
│ 2. XDG user template?                │ → YES → Return PathBuf
│    $XDG_DATA_HOME/cliscrape/templates│
└─────────────────┬───────────────────┘
                  NO
                  ↓
┌─────────────────────────────────────┐
│ 3. Embedded library template?       │ → YES → Write to temp file, return PathBuf
│    TEMPLATE_LIBRARY.get_file()       │        (or extend loader to accept &str)
└─────────────────┬───────────────────┘
                  NO
                  ↓
┌─────────────────────────────────────┐
│ 4. CWD identifier search?            │ → YES → Return PathBuf
│    Look for <spec>.{textfsm,yaml}    │
└─────────────────┬───────────────────┘
                  NO
                  ↓
            Error: template not found
```

### Validation Test Flow (NEW)

```
cargo test --test validation

For each template in templates/:
    ↓
    1. Load template via FsmParser::from_file()
    ↓
    2. Load corresponding fixture (tests/validation/fixtures/<name>.txt)
    ↓
    3. Parse fixture with template
    ↓
    4. Compare output to snapshot (tests/validation/snapshots/<name>.json)
    ↓
    5. Assert match (or update snapshot if INSTA_UPDATE=1)
```

### Logging Flow (NEW)

```
Operation Start (e.g., parse command)
    ↓
Create span: info_span!("parse_command", template = <name>)
    ↓
    ├→ info!("Resolving template")
    ├→ info!("Loading template", path = <path>)
    ├→ warn!("Template warning", kind = <kind>) [if warnings]
    ├→ info!("Parsing input", source = <source>)
    └→ info!("Parse complete", records = <count>, duration = <ms>)
    ↓
Span end (automatic timing)
    ↓
JSON output (if CLISCRAPE_LOG_JSON=1):
{
  "timestamp": "2026-02-22T13:45:00Z",
  "level": "INFO",
  "target": "cliscrape",
  "span": {"name": "parse_command", "template": "cisco_ios_show_version"},
  "fields": {"message": "Parse complete", "records": 5, "duration_ms": 12}
}
```

## Integration Points

### External Dependencies (New Crates)

| Crate | Version | Purpose | Integration Notes |
|-------|---------|---------|-------------------|
| `include_dir` | 0.7+ | Embed templates at compile-time | Use `include_dir!("$CARGO_MANIFEST_DIR/templates")` |
| `dirs` | 5.0+ | XDG directory resolution | `dirs::data_local_dir()` for user templates |
| `tracing` | 0.1 | Structured logging | Replace `eprintln!()` in main.rs |
| `tracing-subscriber` | 0.3 | Log formatting (JSON/text) | Initialize in main(), use `EnvFilter` |

### Internal Module Boundaries

| Boundary | Communication | Modification Required |
|----------|---------------|----------------------|
| **main.rs ↔ template::discovery** | `resolve_template_spec()` returns `TemplateSource` enum | **NEW** - create discovery module |
| **template::discovery ↔ template::library** | `lookup_template(name)` returns `Option<&'static str>` | **NEW** - create library module |
| **main.rs ↔ lib.rs** | `FsmParser::from_file()` (unchanged) | **EXISTING** - no changes |
| **template::loader ↔ engine** | `TemplateIR → Template::from_ir()` (unchanged) | **EXISTING** - no changes |

### New API Surface

```rust
// src/template/discovery.rs (NEW)
pub enum TemplateSource {
    Path(PathBuf),           // File on disk
    Embedded(&'static str),  // From include_dir!()
}

pub fn resolve_template_spec(spec: &str, format: TemplateFormat) -> anyhow::Result<TemplateSource>;

// src/template/library.rs (NEW)
pub struct TemplateLibrary {
    index: TemplateIndex,
}

impl TemplateLibrary {
    pub fn new() -> Result<Self>;
    pub fn lookup(&self, name: &str) -> Option<&'static str>;
    pub fn list_templates(&self) -> Vec<&IndexEntry>;
}

// Modification to lib.rs (EXTEND existing)
impl FsmParser {
    // Add new constructor for embedded templates
    pub fn from_str_with_format(content: &str, format: TemplateFormat) -> Result<Self, ScraperError> {
        let ir = match format {
            TemplateFormat::Textfsm => TextFsmLoader::parse_str(content)?,
            TemplateFormat::Yaml => modern::load_yaml_str(content)?,
            TemplateFormat::Toml => modern::load_toml_str(content)?,
            _ => anyhow::bail!("Auto format requires file path"),
        };
        let template = Template::from_ir(ir)?;
        Ok(Self { template })
    }
}
```

## Build Order and Dependencies

### Phase 1: Foundation (Template Library Infrastructure)
**Goal:** Enable embedded templates without changing CLI behavior

1. **Add dependencies to Cargo.toml**
   - `include_dir = "0.7"`
   - `dirs = "5.0"`
   - Not blocking anything

2. **Create template library structure**
   - Create `templates/index` file
   - Organize templates into `templates/cisco_ios/`, etc.
   - Depends on: nothing (can be done in parallel)

3. **Implement `src/template/library.rs`**
   - Parse index file
   - Wrap `include_dir!()` macro
   - Provide `lookup_template()` API
   - Depends on: step 1 (deps), step 2 (template structure)
   - Testable independently (unit tests for index parsing)

4. **Implement `src/template/discovery.rs`**
   - Extract existing `resolve_template_spec()` from main.rs
   - Add XDG search path
   - Add embedded library lookup
   - Depends on: step 3 (library API)
   - Testable independently (mock filesystem)

5. **Integrate into `main.rs`**
   - Replace inline `resolve_template_spec()` with `discovery::resolve_template_spec()`
   - Add `FsmParser::from_str_with_format()` call for embedded templates
   - Depends on: step 4 (discovery API)
   - Testable via existing CLI e2e tests

**Deliverable:** Users can run `cliscrape parse -t cisco_ios_show_version` without local files

### Phase 2: Validation Suite
**Goal:** Ensure library templates work with real device outputs

6. **Create validation test infrastructure**
   - Create `tests/validation/` directory
   - Add `insta` crate for snapshot testing
   - Create `test_library_templates.rs` harness
   - Depends on: step 5 (library templates accessible)
   - Independent of other features

7. **Add device output fixtures**
   - Collect real device outputs for each template
   - Store in `tests/validation/fixtures/`
   - Create expected output snapshots
   - Depends on: step 6 (test infrastructure)
   - Can be populated incrementally

**Deliverable:** `cargo test --test validation` passes for all library templates

### Phase 3: Production Logging
**Goal:** Add observability without changing functionality

8. **Add tracing dependencies**
   - `tracing = "0.1"`
   - `tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }`
   - Not blocking anything

9. **Instrument main operations**
   - Add `tracing_subscriber::init()` to main()
   - Add spans to `parse`, `debug`, `convert` commands
   - Add structured log events for key operations
   - Depends on: step 8 (deps)
   - Non-breaking (logs only if `RUST_LOG` set)

10. **Add CLI flag for JSON logging**
    - Add `--log-format json` flag (or env var `CLISCRAPE_LOG_JSON`)
    - Configure `tracing_subscriber` formatter
    - Depends on: step 9 (tracing integrated)
    - Testable via CLI tests checking JSON output

**Deliverable:** `RUST_LOG=info cliscrape parse ...` produces structured logs

### Phase 4: Documentation
**Goal:** Document new features and template library

11. **Template catalog documentation**
    - Generate list of available templates from index
    - Document template naming convention
    - Provide examples of template usage
    - Depends on: step 5 (library working)
    - Can be manual or auto-generated

12. **User guide updates**
    - Document template discovery mechanism
    - Document XDG user template installation
    - Document logging configuration
    - Depends on: steps 5, 10 (features complete)
    - Independent task

**Deliverable:** README.md and docs/ updated with v1.5 features

### Dependency Graph

```
[1. Deps]     [2. Template Structure]
    ↓               ↓
    └─────┬─────────┘
          ↓
    [3. library.rs]
          ↓
    [4. discovery.rs]
          ↓
    [5. main.rs integration]
          ↓
    [6. Validation infrastructure]
          ↓
    [7. Fixtures] (parallel with 8-12)

[8. Tracing deps]
    ↓
[9. Instrument main.rs]
    ↓
[10. Log format flag]

[11. Template docs] (depends on 5)
[12. User guide] (depends on 5, 10)
```

### Critical Path

The fastest path to a working v1.5:

1. Deps → 2. Template structure → 3. library.rs → 4. discovery.rs → 5. main.rs integration

Then in parallel:
- Validation: 6 → 7
- Logging: 8 → 9 → 10
- Docs: 11, 12

Estimated build order time: 3-5 days (assumes templates already authored)

## Anti-Patterns

### Anti-Pattern 1: Downloading Templates at Runtime

**What people do:** Fetch templates from GitHub/CDN on first use, cache locally

**Why it's wrong:**
- Network dependency makes tool fragile (offline use broken)
- Security risk (MITM attacks, compromised CDN)
- Slower startup (latency on first use)
- Violates "works immediately after install" principle

**Do this instead:** Embed templates at compile-time with `include_dir!()`, let users override via XDG if needed

### Anti-Pattern 2: Single-Source Template Resolution

**What people do:** Only check embedded library OR only check filesystem

**Why it's wrong:**
- Embedded-only: Users can't customize or add templates
- Filesystem-only: Requires installation step, breaks without templates/
- Missing user expectations (Linux users expect XDG, macOS users expect ~/Library)

**Do this instead:** Layered resolution (explicit path → XDG → embedded → CWD) with clear precedence

### Anti-Pattern 3: Ignoring Logging in Library Code

**What people do:** Only add logging to main.rs, keep library print-free

**Why it's wrong:**
- Actually correct for libraries! But CLI binary should instrument library calls
- Missing: span context showing which operation triggered library code
- Users have no visibility into what's happening during parsing

**Do this instead:** Keep library print-free (correct), but wrap library calls in tracing spans from main.rs

### Anti-Pattern 4: Tightly Coupling Validation to Unit Tests

**What people do:** Put validation tests in `src/engine/mod.rs` or `src/template/loader.rs`

**Why it's wrong:**
- Mixes concerns (engine correctness vs template correctness)
- Validation fixtures are large (real device outputs), bloat unit test runtime
- Hard to update snapshots independently from code changes
- Validation failures don't indicate code bugs, just template issues

**Do this instead:** Separate `tests/validation/` with fixtures and snapshots, use `insta` for snapshot testing

### Anti-Pattern 5: Hardcoding Template Paths in Code

**What people do:** `include_str!("../templates/cisco_ios_show_version.textfsm")` in multiple places

**Why it's wrong:**
- DRY violation (path repeated everywhere)
- No central index (can't list available templates)
- Adding templates requires code changes
- Can't support metadata (description, version, author)

**Do this instead:** Central `templates/index` file with lookup API, single source of truth

## Scaling Considerations

| Scale | Architecture Adjustments |
|-------|--------------------------|
| **50-100 templates** | Current architecture sufficient. Embedded binary grows ~100-200KB (acceptable). Index file parsed once at startup (~1ms). |
| **100-500 templates** | Consider lazy loading: don't parse all templates at startup, only when requested. Keep index in memory, load template content on demand from `TEMPLATE_LIBRARY`. Binary size ~500KB-1MB (still acceptable). |
| **500+ templates** | Split library into vendor-specific crates (`cliscrape-cisco`, `cliscrape-juniper`). Users install only what they need. Still embed templates (don't move to runtime download). Binary size could reach 5MB+ with all vendors. |

### Scaling Priorities

1. **First bottleneck:** Index lookup performance (linear scan)
   - **Fix:** Switch from `Vec<IndexEntry>` to `HashMap<String, IndexEntry>` when >100 templates
   - **Impact:** O(n) → O(1) lookup, negligible memory increase

2. **Second bottleneck:** Binary size with 500+ templates
   - **Fix:** Feature flags in Cargo.toml (`--features cisco,juniper`) to conditionally include vendors
   - **Impact:** Users compile only what they need, reduces binary size 50-80%

3. **Third bottleneck:** Validation test runtime with >200 templates
   - **Fix:** Parallelize validation tests with `cargo-nextest`, group by vendor
   - **Impact:** Test time grows sublinearly with template count

## Production Deployment Considerations

### Configuration Management

Templates will be discovered in this order (first match wins):

1. **Explicit CLI path:** `--template /path/to/template.yaml` (highest priority)
2. **XDG user templates:** `$XDG_DATA_HOME/cliscrape/templates/` (Linux: `~/.local/share`, macOS: `~/Library/Application Support`)
3. **Embedded library:** Compiled into binary via `include_dir!()`
4. **CWD identifier:** `./template-name.{textfsm,yaml,toml}` (existing fallback, lowest priority)

Users can install custom templates:

```bash
mkdir -p ~/.local/share/cliscrape/templates
cp my_custom_template.yaml ~/.local/share/cliscrape/templates/
cliscrape parse -t my_custom_template input.txt  # Picks up user template
```

### Logging Configuration

Recommended production settings:

```bash
# Development: human-readable logs
export RUST_LOG=cliscrape=debug,warn
cliscrape parse -t template input.txt

# Production: JSON logs to stdout, parse results to stdout
export RUST_LOG=cliscrape=info,warn
export CLISCRAPE_LOG_JSON=1
cliscrape parse -t template input.txt 2>logs.jsonl >results.json

# Filter logs with jq
cat logs.jsonl | jq 'select(.level == "ERROR")'
```

Log events emitted:

- `template_resolved`: Which template source was used (path/xdg/embedded/cwd)
- `template_loaded`: Template parsing complete, warnings if any
- `input_parsed`: Per-input-file parse complete with record count
- `parse_complete`: Overall operation summary with timing

### Error Handling Strategy

New error types needed:

```rust
#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Template '{0}' not found in any search path")]
    TemplateNotFound(String),

    #[error("Ambiguous template '{0}': found in multiple locations: {1:?}")]
    AmbiguousTemplate(String, Vec<String>),

    #[error("Library index is corrupted: {0}")]
    CorruptedIndex(String),
}
```

Errors are formatted per existing `--error-format` flag (human/json), maintaining consistency with v1.0 behavior.

## Sources

**Rust CLI Configuration and Discovery:**
- [XDG Base Directory Specification - Rust](https://whitequark.github.io/rust-xdg/xdg/struct.BaseDirectories.html)
- [Configuration - Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/configuration.html)
- [GitHub - rust-cli/confy](https://github.com/rust-cli/confy)
- [Handle XDG Directories - Ratatui](https://ratatui.rs/recipes/apps/config-directories/)

**Compile-Time Resource Embedding:**
- [include_dir - Rust](https://docs.rs/include_dir/latest/include_dir/)
- [include_dir - crates.io](https://crates.io/crates/include_dir)
- [Bundle Resource Files into a Rust Application](http://www.legendu.net/misc/blog/bundle-resource-files-into-a-rust-application/)
- [rust-embed - crates.io](https://crates.io/crates/rust-embed)

**Structured Logging with Tracing:**
- [How to Structure Logs Properly in Rust with tracing and OpenTelemetry](https://oneuptime.com/blog/post/2026-01-07-rust-tracing-structured-logs/view)
- [How to Create Structured JSON Logs with tracing in Rust](https://oneuptime.com/blog/post/2026-01-25-structured-json-logs-tracing-rust/view)
- [Logging and Distributed Tracing in Rust Microservices - Calmops](https://calmops.com/programming/rust/logging-and-distributed-tracing-in-rust-microservices/)
- [tracing - Rust](https://docs.rs/tracing)
- [GitHub - tokio-rs/tracing](https://github.com/tokio-rs/tracing)

**Network Template Library Patterns:**
- [GitHub - networktocode/ntc-templates](https://github.com/networktocode/ntc-templates)
- [Leveraging NTC-Templates for Network Automation](https://networktocode.com/blog/leveraging-ntc-templates-for-network-automation-2025-08-08/)
- [ntc-templates/ntc_templates/templates/index at master](https://github.com/networktocode/ntc-templates/blob/master/ntc_templates/templates/index)
- [Getting Started - NTC Templates Documentation](https://ntc-templates.readthedocs.io/en/latest/user/lib_getting_started/)

**Network Device Parser Validation:**
- [Network Test Automation with NetBox + pyATS + Genie](https://netboxlabs.com/blog/network-test-automation-netbox-pyats-genie/)
- [Parsing Strategies - PyATS Genie Parsers](https://networktocode.com/blog/parsing-strategies-pyats-genie/)
- [Network Validation with pyATS - BlueAlly](https://www.blueally.com/network-validation-with-pyats/)
- [Testing Device Configuration Templates - ipSpace.net blog](https://blog.ipspace.net/2024/05/netlab-integration-tests/)

**pyATS Genie Parser Architecture:**
- [index - Genie - Cisco DevNet](https://developer.cisco.com/docs/genie-docs/)
- [Write a parser - pyATS Development Guide](https://pubhub.devnetcloud.com/media/pyats-development-guide/docs/writeparser/writeparser.html)
- [GitHub - CiscoTestAutomation/genieparser](https://github.com/CiscoTestAutomation/genieparser)

---
*Architecture research for: cliscrape v1.5 Template Ecosystem & Production Hardening*
*Researched: 2026-02-22*

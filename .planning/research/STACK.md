# Technology Stack Research

**Project:** cliscrape - High-performance CLI scraping/parsing tool
**Milestone:** v0.1 Alpha (Phases 1-4)
**Researched:** 2026-02-17
**Overall Confidence:** HIGH

## Executive Summary

This research identifies the specific Rust libraries, versions, and integration patterns needed for v0.1 Alpha (FSM engine, TextFSM parsing, TUI debugger, and modern YAML/TOML frontends). All recommendations are verified against current stable versions as of February 2026, with rationale based on performance requirements, ecosystem maturity, and integration complexity.

**Core principle:** Prefer mature, actively maintained libraries with proven performance in production environments. Avoid bleeding-edge dependencies that could destabilize the Alpha release.

---

## Core Technology

### Language & Edition

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| Rust | 1.82+ (Edition 2024) | Systems language | Memory safety without GC, zero-cost abstractions, strong type system. Edition 2024 brings async closure support and improved RPIT lifetime capture. Installed version: 1.93.1 ✓ |

**Rationale:** Rust 1.82+ ensures access to Edition 2024 features (stabilized Feb 2025), including async closures which may benefit future phases. Current installation (1.93.1) exceeds minimum requirement.

---

## Phase 1: Core FSM Engine & IR

### Regular Expression Engine

| Library | Version | Purpose | Phase | Why Recommended |
|---------|---------|---------|-------|-----------------|
| **regex** | **1.12.3** | Primary regex engine | 1, 2, 3, 4 | **RECOMMENDED for all patterns.** Guarantees linear-time matching O(m*n), uses finite automata, zero catastrophic backtracking risk. Built-in SIMD acceleration. Industry standard (powers ripgrep, fd, bat). |
| fancy-regex | 0.17.0 | Advanced regex features | 2 (optional) | **AVOID unless TextFSM requires backreferences/lookaround.** Uses backtracking VM with exponential worst-case. Only use if TextFSM compatibility demands it. Delegates simple patterns to `regex` internally. |

**Decision:** Start with `regex` only. Add `fancy-regex` in Phase 2 only if TextFSM parsing reveals patterns requiring backreferences/lookaround. TextFSM typically uses simpler patterns, so `regex` should suffice.

**Performance notes:**
- `regex` provides `RegexSet` for matching multiple patterns simultaneously (O(m*n) where m = pattern count)
- Pre-compile all regexes at template load time (compilation costs microseconds-milliseconds)
- Use `std::sync::LazyLock` or `once_cell` to avoid recompilation in loops

**Sources:**
- [regex crate docs](https://docs.rs/regex) - Version 1.12.3 features
- [fancy-regex comparison](https://github.com/rust-lang/regex/discussions/960) - Performance benchmarks showing 3x+ difference
- [RegexSet documentation](https://docs.rs/regex/latest/regex/struct.RegexSet.html) - Multi-pattern optimization

### Memory Optimization

| Pattern | Library | Purpose | When to Use |
|---------|---------|---------|-------------|
| Zero-copy strings | `std::borrow::Cow` | Avoid allocations during parsing | **Always for captured values.** Only clone when buffer is modified. Cow<'a, str> delays allocation until mutation (to_mut()). |

**Design pattern:** Values in FSM state should be `Cow<'a, str>` referencing original input. Record buffer clones only when `Record` action fires.

**Sources:**
- [Cow optimization guide (2026)](https://oneuptime.com/blog/post/2026-01-25-rust-cow-clone-on-write/view) - Best practices for read-heavy workloads

### Thread Safety (Deferred to Post-v0.1)

| Library | Version | Purpose | Notes |
|---------|---------|---------|-------|
| parking_lot | 0.12.5 | Fast locks (RwLock, Mutex) | **NOT NEEDED for v0.1.** Single-threaded execution sufficient for Alpha. Add in Phase 5 (connectivity) when parallelizing across devices. 50x faster than std::sync in benchmarks. |

**Rationale:** v0.1 focuses on single-file parsing. Thread safety becomes critical in Phase 5 (SSH/Telnet connectivity, parallel device polling). Defer `parking_lot` until then.

**Sources:**
- [parking_lot performance](https://github.com/Amanieu/parking_lot) - Up to 50x faster than std::sync::RwLock

### FSM Implementation

| Approach | Library | Recommendation |
|----------|---------|----------------|
| **Hand-rolled FSM** | None (manual) | **RECOMMENDED.** Custom struct-based FSM gives full control over memory layout, state transitions, and debugging. FSM libraries (rust-fsm, finny.rs) add macro complexity without matching cliscrape's line-by-line execution model. |
| FSM library | rust-fsm, finny.rs | **AVOID.** These are macro-based DSLs for event-driven FSMs. cliscrape needs imperative line-by-line processing with custom action execution (Record, Clear, Continue). |

**Design:**
```rust
struct Engine {
    states: Vec<State>,           // All states from template
    current: StateId,             // Current state index
    buffer: HashMap<String, Cow<'a, str>>,  // Named captures
    results: Vec<HashMap<String, String>>,  // Recorded outputs
}

struct State {
    rules: Vec<Rule>,
}

struct Rule {
    regex: Regex,                 // Pre-compiled
    next_state: Option<StateId>,
    actions: Vec<Action>,         // Record, Clear, Continue, Next
}
```

**Sources:**
- [rust-fsm analysis](https://github.com/eugene-babichenko/rust-fsm) - Macro-based DSL, not suitable for imperative parsing
- [State machine patterns in Rust](https://hoverbear.org/blog/rust-state-machine-pattern/) - Idiomatic struct-based approaches

---

## Phase 2: TextFSM Compatibility Frontend

### Parsing TextFSM Grammar

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| **nom** | **8.0.0** | Parser combinators | **RECOMMENDED.** Fastest parser combinator library (3x faster than pest in benchmarks). Zero-copy parsing, linear performance, works on stable Rust. Ideal for TextFSM's simple line-based format. |
| pest | 2.8.6 | PEG parser generator | **AVOID unless nom proves insufficient.** Requires separate grammar file, slower than nom, trades performance for grammar clarity. TextFSM is simple enough for combinators. |

**Rationale:** TextFSM format is line-oriented with simple structure:
```
Value INTERFACE (\S+)
Value STATUS (up|down)

Start
  ^Interface ${INTERFACE} is ${STATUS} -> Record
```

nom's combinator approach handles this efficiently without external grammar files. Use `nom::character::complete` for line parsing, `nom::bytes::complete::tag` for keywords.

**Performance:** nom guarantees zero-copy where possible, avoiding string allocations during parsing. Benchmarks show 3x+ advantage over pest for text-based grammars.

**Sources:**
- [nom vs pest comparison](https://blog.wesleyac.com/posts/rust-parsing) - nom 3x faster for text parsing
- [nom 8.0.0 documentation](https://docs.rs/nom) - Current API and best practices
- [Parser benchmarks](https://github.com/rust-bakery/parser_benchmarks) - Performance comparisons

---

## Phase 3: TUI Debugger (MVP)

### Terminal UI Framework

| Library | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| **ratatui** | **0.30.0** | TUI framework | **RECOMMENDED.** Actively maintained fork of tui-rs (archived 2023). Immediate-mode rendering, no imposed architecture. Rich widget library, responsive layouts. Powers production tools like bottom, gitui. |
| crossterm | 0.29.0 | Terminal backend | **REQUIRED by ratatui.** Cross-platform terminal manipulation (Win/Mac/Linux). Handles raw mode, events, cursor. ratatui's default backend. |

**Integration pattern:**
```rust
use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

// Immediate-mode loop: redraw entire UI each frame from current FSM state
loop {
    terminal.draw(|f| {
        // Build UI from FSM state
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(f.area());

        // Render input, state, values, trace
        f.render_widget(input_widget, chunks[0]);
        f.render_widget(state_widget, chunks[1]);
    })?;

    // Handle events
    if let Event::Key(key) = event::read()? {
        match key.code {
            KeyCode::Char('n') => engine.step_line(),
            KeyCode::Char('q') => break,
            _ => {}
        }
    }
}
```

**Best practices (2026):**
- Implement `Drop` trait to restore terminal on panic (prevents ruined terminal)
- Use alternate screen buffer (`EnterAlternateScreen`) for clean entry/exit
- Layout nesting for responsive multi-pane design (see DESIGN.md's 4-pane layout)

**Sources:**
- [ratatui 0.30.0 docs](https://docs.rs/ratatui) - Current API
- [ratatui best practices (2026)](https://ratatui.rs/) - Official guide with Drop pattern, alternate screen
- [TUI comparison: Ratatui vs BubbleTea](https://www.glukhov.org/post/2026/02/tui-frameworks-bubbletea-go-vs-ratatui-rust/) - Pick ratatui for control and performance

---

## Phase 4: Modern YAML/TOML Frontends

### Serialization Framework

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| **serde** | **1.0.228** | Serialization framework | **REQUIRED.** Universal serialization with derive macros. Enables `#[derive(Serialize, Deserialize)]` on IR structs. |

**Configuration:**
```toml
serde = { version = "1.0.228", features = ["derive"] }
```

### Format-Specific Parsers

| Library | Version | Purpose | Status | Why Recommended |
|---------|---------|---------|--------|-----------------|
| **serde_yml** | **0.0.12** | YAML parsing | **RECOMMENDED** | Actively maintained successor to serde_yaml (deprecated March 2024). Despite low version number (0.0.12), this is the community-adopted replacement. Note: docs.rs build issues, but crate is functional. |
| serde_yaml | 0.9.34+deprecated | YAML parsing (legacy) | **AVOID** | Officially unmaintained as of March 2024. Use serde_yml instead. |
| **toml** | **1.0.2** | TOML parsing | **RECOMMENDED** | Official TOML parser, implements TOML 1.1.0 spec. Stable, well-maintained, used by Cargo itself. |

**Rationale:**
- **YAML:** serde_yml is the clear successor despite version instability. Alternative serde_yaml-ng exists but has less adoption.
- **TOML:** toml crate is the standard, used by Cargo. No viable alternatives needed.

**Template deserialization pattern:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct Template {
    meta: Meta,
    values: HashMap<String, ValueDef>,
    states: HashMap<String, Vec<RuleDef>>,
}

// YAML loading
let yaml_str = std::fs::read_to_string("template.yml")?;
let template: Template = serde_yml::from_str(&yaml_str)?;

// TOML loading
let toml_str = std::fs::read_to_string("template.toml")?;
let template: Template = toml::from_str(&toml_str)?;
```

**Sources:**
- [serde_yaml deprecation discussion](https://users.rust-lang.org/t/serde-yaml-deprecation-alternatives/108868) - Community consensus on serde_yml
- [serde_yml GitHub](https://github.com/sebastienrousseau/serde_yml) - Official successor repo
- [toml crate docs](https://docs.rs/toml) - Stable API, TOML 1.1.0 compliance

---

## Cross-Cutting: CLI, Errors, Testing

### Command-Line Parsing

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| **clap** | **4.5.58** | CLI argument parsing | **RECOMMENDED.** Industry standard (powers ripgrep, fd, bat). Derive macro simplifies definition. Rich help generation, subcommands, validation. |

**Configuration:**
```toml
clap = { version = "4.5.58", features = ["derive"] }
```

**Usage:**
```rust
use clap::Parser;

#[derive(Parser)]
#[command(name = "cliscrape")]
#[command(about = "High-performance CLI scraping tool")]
struct Cli {
    /// TextFSM template file
    #[arg(short, long)]
    template: PathBuf,

    /// Input file to parse
    #[arg(short, long)]
    input: PathBuf,

    /// Enable TUI debugger
    #[arg(long)]
    debug: bool,
}
```

**Sources:**
- [clap 4.5.58 docs](https://docs.rs/clap) - Current derive API
- [Building CLI tools with Clap (2026)](https://dasroot.net/posts/2026/01/building-cli-tools-clap-rust/) - Best practices

### Error Handling

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **anyhow** | **1.0.101** | Application errors | **Main binary, examples.** Propagate errors with `?`, add context with `.context()`, simple `Result<T>` return type. |
| **thiserror** | **2.0.18** | Library errors | **lib.rs modules.** Define typed errors for public API. Users can match specific error variants. |

**Pattern:**
- **lib.rs (engine, parser modules):** Use `thiserror` to define `EngineError`, `ParseError` enums with `#[error]` attributes. Consumers can match and handle specific cases.
- **main.rs, CLI code:** Use `anyhow` for quick error propagation. Log/display errors to users without type-level matching.

```rust
// lib.rs - thiserror
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidTransition { from: String, to: String },

    #[error("Regex compilation failed: {0}")]
    RegexError(#[from] regex::Error),
}

// main.rs - anyhow
use anyhow::{Context, Result};

fn main() -> Result<()> {
    let template = load_template(&args.template)
        .context("Failed to load template")?;
    Ok(())
}
```

**Sources:**
- [thiserror vs anyhow guide (2026)](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view) - When to use each
- [Error handling best practices](https://momori.dev/posts/rust-error-handling-thiserror-anyhow/) - Library vs application patterns

### Testing & Benchmarking

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| **criterion** | **0.8.2** | Benchmarking | **Required for performance validation.** Statistical analysis, works on stable Rust, regression detection. Use for FSM throughput, regex compilation cost. |
| **proptest** | **1.10.0** | Property testing | **Recommended for FSM logic.** Generate random inputs, verify invariants (e.g., "FSM never panics on any input"). Hypothesis-style testing. |

**Benchmarking setup:**
```toml
[dev-dependencies]
criterion = { version = "0.8.2", features = ["html_reports"] }
proptest = "1.10.0"

[[bench]]
name = "engine_bench"
harness = false
```

**Example criterion benchmark:**
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_fsm_parsing(c: &mut Criterion) {
    let template = load_template("tests/templates/cisco_ios.fsm");
    let input = std::fs::read_to_string("tests/data/show_ip_int_brief.txt").unwrap();

    c.bench_function("parse cisco show ip int brief", |b| {
        b.iter(|| {
            template.parse(black_box(&input))
        });
    });
}

criterion_group!(benches, bench_fsm_parsing);
criterion_main!(benches);
```

**Sources:**
- [criterion 0.8.2 docs](https://docs.rs/criterion) - Current API
- [Criterion benchmarking guide](https://bheisler.github.io/criterion.rs/book/getting_started.html) - Best practices
- [proptest guide (2025)](https://generalistprogrammer.com/tutorials/proptest-rust-crate-guide) - Property testing patterns

### Optional: CLI Output Formatting

| Library | Version | Purpose | Why Recommended |
|---------|---------|---------|-----------------|
| **colored** | **3.1.1** | Colored terminal output | **Optional for v0.1.** Simpler API than termcolor, respects CLICOLOR env vars. Use for error/success messages. |

**Note:** Not critical for v0.1. Add if time permits for better UX in error messages.

**Sources:**
- [colored vs termcolor (2026)](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html) - Recommendation against termcolor due to deprecated Windows APIs

---

## Development Tools

### Profiling (Post-v0.1)

| Tool | Purpose | When to Use |
|------|---------|-------------|
| cargo-flamegraph | CPU profiling | **Phase 5+.** Visualize hot paths in FSM execution. Not needed for Alpha. |
| DHAT (dhat-rs) | Memory profiling | **Phase 5+.** Find allocation hotspots. Defer until performance tuning phase. |

**Rationale:** v0.1 focuses on correctness and feature completeness. Performance profiling becomes critical in Phase 6 (massive file optimization).

**Sources:**
- [Rust profiling guide (2026)](https://oneuptime.com/blog/post/2026-02-03-rust-profiling/view) - flamegraph, perf, samply workflows
- [Memory profiling tools](https://www.polarsignals.com/blog/posts/2023/12/20/rust-memory-profiling) - DHAT, heaptrack, bytehound

---

## Installation Commands

### Core Dependencies (Phases 1-4)

```toml
[dependencies]
# Phase 1: Core engine
regex = "1.12.3"
anyhow = "1.0.101"
thiserror = "2.0.18"

# Phase 2: TextFSM parsing
nom = "8.0.0"

# Phase 3: TUI debugger
ratatui = "0.30.0"
crossterm = "0.29.0"

# Phase 4: Modern frontends
serde = { version = "1.0.228", features = ["derive"] }
serde_yml = "0.0.12"
toml = "1.0.2"

# CLI
clap = { version = "4.5.58", features = ["derive"] }

[dev-dependencies]
criterion = { version = "0.8.2", features = ["html_reports"] }
proptest = "1.10.0"

[[bench]]
name = "engine"
harness = false
```

### Optional Dependencies

```toml
# Optional: Better error messages
colored = "3.1.1"

# Conditional: Only if TextFSM requires advanced regex features
fancy-regex = "0.17.0"  # Add only if needed in Phase 2
```

### Rust Edition

Ensure `Cargo.toml` specifies:
```toml
[package]
edition = "2024"
rust-version = "1.82"
```

---

## Alternatives Considered

| Category | Recommended | Alternative | When to Use Alternative |
|----------|-------------|-------------|-------------------------|
| Regex | regex 1.12.3 | fancy-regex 0.17.0 | Only if TextFSM templates require backreferences/lookaround (unlikely). Test with regex first. |
| Parser | nom 8.0.0 | pest 2.8.6 | If TextFSM grammar becomes complex enough to justify separate grammar file. nom should suffice. |
| TUI | ratatui 0.30.0 | cursive, tui-realm | If component-based architecture needed. ratatui's immediate mode fits FSM state rendering. |
| YAML | serde_yml 0.0.12 | serde_yaml-ng | If serde_yml proves unstable. Current community consensus favors serde_yml. |
| Locks | std::sync | parking_lot 0.12.5 | Defer to Phase 5. Use parking_lot when adding parallelism for 50x performance gain. |

---

## What NOT to Use

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| serde_yaml 0.9.34 | Officially unmaintained since March 2024 | serde_yml 0.0.12 |
| tui-rs | Archived in 2023, no longer maintained | ratatui 0.30.0 (active fork) |
| libtest bench (#[bench]) | Requires nightly Rust, unstable API | criterion 0.8.2 (stable) |
| termcolor | Targets deprecated Windows Console APIs | colored 3.1.1 or handle ANSI directly |
| FSM libraries (rust-fsm, finny.rs) | Macro-based event-driven models don't fit line-by-line parsing | Hand-rolled struct-based FSM |
| fancy-regex by default | Backtracking can cause exponential blowup | regex 1.12.3 (linear guarantees) |

---

## Version Compatibility Matrix

| Package | Version | Compatible With | Notes |
|---------|---------|-----------------|-------|
| ratatui | 0.30.0 | crossterm 0.28+, 0.29+ | Crossterm is default backend |
| serde | 1.0.228 | serde_yml 0.0.12, toml 1.0.2 | Enable `derive` feature |
| nom | 8.0.0 | Rust 1.82+ | Major version bump from 7.x, check migration if needed |
| criterion | 0.8.2 | Rust 1.82+ | HTML reports require `html_reports` feature |
| clap | 4.5.58 | Rust 1.82+ | Use `derive` feature for proc macros |

**Note:** All versions verified against Rust 1.93.1 (installed) and Rust 1.82+ (minimum). No known incompatibilities.

---

## Integration Patterns

### FSM Engine + Regex

**Pre-compile all regexes at template load:**
```rust
use regex::Regex;
use std::sync::LazyLock;

// For runtime templates
struct CompiledRule {
    regex: Regex,  // Pre-compiled
    actions: Vec<Action>,
}

impl Template {
    fn compile(&self) -> Result<CompiledTemplate, EngineError> {
        let states = self.states.iter().map(|(name, rules)| {
            let compiled_rules = rules.iter().map(|rule| {
                let regex = Regex::new(&rule.pattern)?;
                Ok(CompiledRule { regex, actions: rule.actions.clone() })
            }).collect::<Result<Vec<_>, _>>()?;

            Ok((name.clone(), compiled_rules))
        }).collect()?;

        Ok(CompiledTemplate { states })
    }
}
```

**Use RegexSet for multi-rule matching:**
```rust
use regex::RegexSet;

// If state has many rules, use RegexSet for fast dispatch
let patterns: Vec<String> = state.rules.iter().map(|r| r.pattern.clone()).collect();
let set = RegexSet::new(&patterns)?;

for line in input.lines() {
    let matches = set.matches(line);
    for idx in matches.iter() {
        // Execute rule at idx
    }
}
```

### TUI + FSM State

**Immediate-mode rendering from engine state:**
```rust
impl Engine {
    fn render_tui(&self, f: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(40), // Input stream
                Constraint::Percentage(30), // Current state/values
                Constraint::Percentage(30), // Match trace
            ])
            .split(f.area());

        // Input stream with current line highlighted
        let input_widget = Paragraph::new(self.format_input())
            .block(Block::default().borders(Borders::ALL).title("Input"));
        f.render_widget(input_widget, layout[0]);

        // Current state and buffer
        let state_widget = Paragraph::new(self.format_state())
            .block(Block::default().borders(Borders::ALL).title("FSM State"));
        f.render_widget(state_widget, layout[1]);

        // Match trace
        let trace_widget = Paragraph::new(self.format_trace())
            .block(Block::default().borders(Borders::ALL).title("Match Trace"));
        f.render_widget(trace_widget, layout[2]);
    }
}
```

### Serde + Template IR

**Unified IR for multiple formats:**
```rust
#[derive(Serialize, Deserialize, Debug)]
pub struct TemplateIR {
    pub meta: Option<Meta>,
    pub values: HashMap<String, ValueDef>,
    pub states: HashMap<String, Vec<RuleDef>>,
}

impl TemplateIR {
    pub fn from_yaml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_yml::from_str(&content)?)
    }

    pub fn from_toml(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    pub fn from_textfsm(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        textfsm::parse(&content)  // Uses nom
    }
}
```

---

## Phase-Specific Dependency Mapping

### Phase 1: Core Engine & IR
**Critical:**
- regex 1.12.3 (pattern matching)
- anyhow 1.0.101 (error handling)
- thiserror 2.0.18 (typed errors)

**Testing:**
- criterion 0.8.2 (benchmark FSM throughput)
- proptest 1.10.0 (fuzz FSM state transitions)

### Phase 2: TextFSM Frontend
**Critical:**
- nom 8.0.0 (parse TextFSM format)

**Conditional:**
- fancy-regex 0.17.0 (only if TextFSM uses backreferences)

### Phase 3: TUI Debugger
**Critical:**
- ratatui 0.30.0 (TUI framework)
- crossterm 0.29.0 (terminal backend)

### Phase 4: Modern Frontends
**Critical:**
- serde 1.0.228 (serialization framework)
- serde_yml 0.0.12 (YAML parsing)
- toml 1.0.2 (TOML parsing)

### All Phases: CLI
**Critical:**
- clap 4.5.58 (argument parsing)

---

## Confidence Assessment

| Category | Confidence | Rationale |
|----------|-----------|-----------|
| Core regex (regex) | **HIGH** | Verified version 1.12.3 from docs.rs, proven in production tools |
| TUI stack (ratatui + crossterm) | **HIGH** | Verified versions, active maintenance, 2026 best practices documented |
| Serialization (serde + formats) | **MEDIUM** | serde_yml is low version (0.0.12) but community-adopted. toml is stable. |
| Parser (nom) | **HIGH** | Version 8.0.0 verified, performance benchmarks confirm superiority |
| Error handling (anyhow/thiserror) | **HIGH** | Verified versions, 2026 best practices align with recommendations |
| Testing (criterion/proptest) | **HIGH** | Current versions verified, stable APIs |
| Deferred (parking_lot) | **MEDIUM** | Version verified but not needed for v0.1, confidence based on future need |

**Overall:** HIGH confidence for Phase 1-4 dependencies. All versions are current as of Feb 2026, with clear rationale and integration patterns. Only concern is serde_yml's low version number, mitigated by community consensus and functional stability.

---

## Open Questions / Future Research

1. **TextFSM regex complexity:** Will TextFSM templates require fancy-regex features? **Resolution:** Test with regex first in Phase 2, add fancy-regex only if needed.

2. **serde_yml stability:** Version 0.0.12 is low, docs.rs build failed. **Mitigation:** Community consensus is strong, consider serde_yaml-ng as fallback if issues arise.

3. **Performance profiling tools:** When to add flamegraph/DHAT? **Decision:** Defer to Phase 6 (massive file optimization), not needed for v0.1 Alpha.

4. **Colored output:** Include colored 3.1.1 for v0.1? **Decision:** Optional, add if time permits for better CLI UX.

---

## Sources

### Primary (HIGH Confidence)
- [regex 1.12.3 docs](https://docs.rs/regex) - Official documentation
- [ratatui 0.30.0 docs](https://docs.rs/ratatui) - Official documentation
- [nom 8.0.0 docs](https://docs.rs/nom) - Official documentation
- [serde 1.0.228 docs](https://docs.rs/serde) - Official documentation
- [clap 4.5.58 docs](https://docs.rs/clap) - Official documentation
- [Rust Edition 2024 guide](https://doc.rust-lang.org/edition-guide/rust-2024/index.html) - Official Rust project

### Secondary (MEDIUM Confidence)
- [serde_yaml deprecation discussion](https://users.rust-lang.org/t/serde-yaml-deprecation-alternatives/108868) - Community consensus on serde_yml
- [nom vs pest benchmarks](https://blog.wesleyac.com/posts/rust-parsing) - Performance comparisons
- [Ratatui best practices (2026)](https://ratatui.rs/) - Official guide
- [Error handling guide (2026)](https://oneuptime.com/blog/post/2026-01-25-error-types-thiserror-anyhow-rust/view) - Best practices

### Tertiary (Context)
- [Rust CLI color recommendations](https://rust-cli-recommendations.sunshowers.io/managing-colors-in-rust.html) - termcolor deprecation notes
- [Profiling guide (2026)](https://oneuptime.com/blog/post/2026-02-03-rust-profiling/view) - Tool landscape
- [Cow optimization (2026)](https://oneuptime.com/blog/post/2026-01-25-rust-cow-clone-on-write/view) - Memory patterns

---

*Stack research for: cliscrape v0.1 Alpha*
*Researched: 2026-02-17*
*Rust version: 1.93.1 (verified compatible)*

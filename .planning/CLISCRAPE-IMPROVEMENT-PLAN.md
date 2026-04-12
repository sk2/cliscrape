# cliscrape Improvement Plan: From Parser to Platform

**Version:** 1.2 (2026-04-12) — reality-check sync against current code, tests, CI, and beads
**Scope:** tracked improvements across feature innovation and operational readiness
**Target:** Post-v2.0 milestone — transforms cliscrape from a capable parser into a production-grade, distributable, well-tested platform

---

## 1. Executive Summary

cliscrape is a Rust-based parser for semi-structured network CLI output. It converts raw device output into structured JSON via an FSM engine, with a TUI debugger (Live Lab), constraint validation, semantic diffing, grammar induction, and best-effort oracle/generation support.

**Current state:** v2.0 milestone at 80% (Phase 16 of 20 complete, Phase 17 still planned). Core engine is mature. But significant gaps exist in:
- **User friction:** template selection is manual, no auto-detection
- **Ecosystem reach:** no Python bindings, limited template library
- **Operational readiness:** basic GitHub Actions test CI exists, but clippy/fmt/audit/coverage gates and release automation are not in place
- **Developer experience:** no dev tooling, documented tech debt unresolved

This plan addresses these gaps through self-contained improvements organized into dependency chains and leaf tasks, executable across 3 waves.

> **Revision note (v1.2):** `br-dquy` is closed with diff and comprehensive constraint tests. Fuzzing is tracked as `br-ydmz`. The streaming iterator API is still proposed work, not implemented code.

---

## 2. Project Context

### 2.1 Architecture Overview

```
┌──────────────────────────────────────────────────────┐
│                     CLI (clap)                        │
│  parse │ debug │ diff │ infer │ generate │ session*   │
├──────────────────────────────────────────────────────┤
│                   Library API (lib.rs)                │
│  FsmParser::from_file() → parse() → Vec<BTreeMap>    │
│  parse_iter() / streaming API is proposed by br-z4wc, not implemented yet     │
├──────────┬─────────────┬────────────┬────────────────┤
│ Engine   │  Template   │ Transcript │     TUI        │
│ fsm.rs   │  loader.rs  │ ios_prompt │   app.rs       │
│ types.rs │  modern.rs  │  .rs       │   ui.rs        │
│ records  │  library.rs │            │   editor.rs    │
│ validate │  resolver   │            │   trace.rs     │
│ debug.rs │  metadata   │            │   browser.rs   │
│ diff.rs  │  convert.rs │            │   worker.rs    │
│ infer.rs │             │            │   watch.rs     │
│ coverage │             │            │   picker.rs    │
│ macros   │             │            │   event.rs     │
└──────────┴─────────────┴────────────┴────────────────┘
         * = new subcommand proposed in this plan
```

### 2.2 Key Data Structures

```rust
// Template fields (engine/types.rs)
pub enum FieldType { Int, String }  // Note: no IP/Bool variants — handled at parse time

pub struct Value {
    pub name: String,
    pub regex: String,
    pub type_hint: Option<FieldType>,
    pub list: bool,
    pub required: bool,
    pub constraints: Option<FieldConstraints>,
    pub common_schema: Option<String>,
    // + filldown, identity, ignore
}

pub struct FieldConstraints {
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub choices: Option<Vec<String>>,
    pub regex: Option<String>,
}

// Parse output — current API
pub struct FsmParser { template: Template }
// parse() returns Vec<BTreeMap<String, serde_json::Value>>  — materialized
// results_with_warnings() returns parsed records plus TemplateWarning values
// parse_iter() is not present yet; add it as part of br-z4wc if streaming lands.

// Debug infrastructure
pub struct DebugReport {
    pub lines: Vec<String>,
    pub matches_by_line: Vec<Vec<LineMatch>>,  // KEY: empty vec = unmatched line
    pub records: Vec<EmittedRecord>,
    pub trace: Vec<TraceEvent>,
}
```

### 2.3 Current Template Metadata

Templates have `metadata.description` and `metadata.compatibility` but **no explicit `vendor` or `command` fields**. Template identity is derived from filename convention: `{vendor}_{os}_{command}.yaml` (e.g., `cisco_ios_show_version.yaml`).

### 2.4 Existing Prompt Detection

`transcript/ios_prompt.rs` already implements `preprocess_ios_transcript()`:
- Segments multi-command transcripts by detecting prompt patterns
- Prompt regex: `^[A-Za-z0-9_.:-]+(?:\([^\r\n)]*\))?[#>](?:[ \t]+(?P<cmd>.*))?$`
- Handles: command echo stripping, config mode prompts, confidence thresholds (2+ matching bases)
- **Gap:** Returns `Vec<String>` — discards command names captured by the regex
- **Gap:** Only IOS prompts — no Juniper `user@host>`, no Nokia `A:host#`

### 2.5 Verified Technical Debt

| Source | Item | Status |
|--------|------|--------|
| v1.0 audit | loader.rs dead code (parse_definition, parse_state_block, etc.) | Specific stale helper names no longer appear; rerun clippy before closing br-8a9o |
| v1.0 audit | tui/mod.rs FsChanged.which never read | Resolved in code by matching on `which` |
| v1.0 audit | tui/app.rs jump_to_line, toggle_watch unused | Resolved in current code; symbols no longer present |
| v1.0 audit | cli.rs "CSV/Table placeholder" comments (both are implemented) | Resolved; comments now describe real CSV/Table output |
| v1.5 deferred | Interactive converter smoke test (Phase 3) | Unresolved |
| v1.5 deferred | Live Lab TUI interactive verification (Phase 4) | Unresolved |
| v1.5 deferred | Phase 11 plan 03 catalog/doc-validation artifacts | Unresolved |
| Cargo.toml | version = "0.1.0", release metadata missing | Unresolved; edition 2024 is valid on current Rust |
| CI | Basic test workflow exists; clippy, fmt, audit, coverage gates missing | Unresolved |
| Tests | diff.rs dedicated tests | Resolved by br-dquy; keep expanding through regressions |
| Tests | TUI rendering/unit coverage | Partial: AppState tests exist; TestBackend/render coverage still missing |
| Tests | FSM engine lacks fuzzing for malformed/hostile inputs | Unresolved |
| Tests | Constraint validation edge coverage | Resolved by br-dquy with comprehensive constraint tests |

---

## 3. Improvement Categories

### 3.1 Feature Innovation (Round 1 — 16 items)

These add new user-facing capabilities.

### 3.2 Operational Readiness (Round 2 — 15 items)

These improve quality, testing, packaging, and developer experience.

---

## 4. Dependency Graph

```
CHAIN 1: Quality Infrastructure
  br-8a9o Dead Code Cleanup
    └→ br-67nr CI Quality Gates (clippy, fmt, audit, coverage)
        ├→ br-o12g Dev Workflow Tooling (justfile)
        ├→ br-xqva Cross-Platform CI (macOS + Windows)
        └→ br-5dyn Performance Regression CI

CHAIN 2: Parsing Features
  br-dpm7 Multi-Vendor Prompt Detection
    ├→ br-wkr3 Template Auto-Discovery
    │    ├→ br-sm6v Multi-Command Session Parsing
    │    └→ br-zfqv Python Bindings (PyO3)
    └→ br-sm6v Multi-Command Session Parsing

CHAIN 3: Error Handling
  br-t76f Error Recovery / Partial Parse
    ├→ br-efyb Batch Processing
    └→ br-981w Compliance Reporting
  br-4e86 Error Context Enrichment
    └→ br-jvyb Structured Error Messages
        └→ br-izvb Inline Test Fixtures

CHAIN 4: Streaming
  br-z4wc Streaming / Pipe Mode
    └→ br-efyb Batch Processing

ADDITIONAL TRACKED ITEMS (mix of independent leaves and newly filed gaps):
  br-ydmz  Fuzz Testing the FSM Engine (v1.1)
  br-48hc  JSON Schema Generation
  br-dquy  Diff + Constraint Test Suites (closed)
  cliscrape-o6e  Cargo.toml + Release Automation
  cliscrape-0ev  Cisco IOS show version serial extraction regression
  cliscrape-1s8  Common-schema Universal Ledger mappings
  cliscrape-nuy  Strict FSM-Oracle verification for selected templates
  cliscrape-ka1  Evidence-backed benchmark baselines and performance claims
  br-9jbq  Field Transformation Pipeline
  br-y0yl  Template Composition / Inheritance
  br-71zd  Template Ecosystem: Core Commands
  br-do0g  Template Validation Warnings
  br-dzyd  API Documentation with Examples
  br-8wxa  Stress Test Suite
  br-n5jy  TUI Unit Test Suite
  br-fket  Output Field Selection
  br-tilt  Configuration File Support
  br-2d02  Shell Completions
  br-g5mz  TUI Debug Export
  br-t9zw  Memory Profiling Benchmarks
```

---

## 5. Implementation Waves

### Wave 1: Foundations (parallel where dependencies allow)

**Quality foundations (start immediately):**

| Bead | Priority | Effort | Description |
|------|----------|--------|-------------|
| br-8a9o | P1 | 2-4h | Dead code cleanup — unblock CI gates |
| br-dquy | P1 | Done | Diff engine + constraint validation test suites (closed) |
| cliscrape-o6e | P2 | 3-5h | Fix Cargo.toml metadata + release workflow |
| cliscrape-0ev | P1 | 1-3h | Fix Cisco IOS show version serial extraction |
| br-dpm7 | P1 | 4-6h | Multi-vendor prompt detection (IOS, NX-OS, EOS, JUNOS) |
| br-t76f | P1 | 6-10h | Error recovery / partial parse mode |
| br-48hc | P1 | 3-5h | JSON Schema generation from templates |
| br-ydmz | P1 | 4-6h | Fuzz testing FSM engine (cargo-fuzz) — v1.1 |

**Feature foundations (parallel with above):**

| Bead | Priority | Effort | Description |
|------|----------|--------|-------------|
| br-4e86 | P2 | 4-6h | Error context enrichment (template path, FSM state in errors) |
| br-z4wc | P2 | 4-6h | Streaming / pipe mode (`--stream` for stdin) |
| br-9jbq | P2 | 6-10h | Field transformation pipeline (MAC normalization, etc.) |
| br-jvyb* | P2 | 4-8h | Structured error messages with fix suggestions |
| br-n5jy | P2 | 8-16h | TUI unit test suite (3,194 LOC currently untested) |
| br-71zd | P2 | 10-20h | Template ecosystem: show ip route, show bgp, show lldp |
| br-do0g | P2 | 4-6h | Template validation warnings (unused fields, unreachable states) |
| br-y0yl | P2 | 6-10h | Template composition / inheritance |
| br-dzyd | P2 | 3-5h | API documentation with inline examples |
| br-8wxa | P2 | 6-10h | Stress test suite (10M+ lines) |

*br-jvyb depends on br-4e86 but both can start in Wave 1 since br-4e86 is a quick foundation.

**Nice-to-have (parallel, no deps):**

| Bead | Priority | Effort | Description |
|------|----------|--------|-------------|
| br-fket | P3 | 2-3h | Output field selection (`--fields hostname,version`) |
| br-tilt | P3 | 3-5h | Configuration file support (~/.config/cliscrape/) |
| br-2d02 | P3 | 1-2h | Shell completions (bash/zsh/fish via clap_complete) |
| br-g5mz | P3 | 3-5h | TUI debug export (Ctrl+E → JSON) |
| br-t9zw | P3 | 4-6h | Memory profiling benchmarks |

### Wave 2: Features (after Wave 1 dependencies resolve)

| Bead | Priority | Depends On | Effort | Description |
|------|----------|------------|--------|-------------|
| br-67nr | P1 | br-8a9o | 4-8h | CI quality gates (clippy, fmt, audit, coverage) |
| br-wkr3 | P1 | br-dpm7 | 8-12h | Template auto-discovery from device output |
| br-efyb | P2 | br-z4wc, br-t76f | 6-10h | Batch processing with parallel execution |
| br-981w | P2 | br-t76f | 6-10h | Compliance reporting mode |
| br-izvb | P2 | br-jvyb | 4-6h | Inline test fixtures in templates |

### Wave 3: Ecosystem (after Wave 2)

| Bead | Priority | Depends On | Effort | Description |
|------|----------|------------|--------|-------------|
| br-sm6v | P1 | br-dpm7, br-wkr3 | 8-12h | Multi-command session parsing |
| br-zfqv | P1 | br-wkr3 | 12-20h | Python bindings via PyO3 |
| br-o12g | P2 | br-67nr | 2-4h | Developer workflow tooling (justfile) |
| br-xqva | P3 | br-67nr | 3-5h | Cross-platform CI (macOS + Windows) |
| br-5dyn | P3 | br-67nr | 4-6h | Performance regression CI |

---

## 6. Detailed Specifications

### 6.1 CHAIN 1: Quality Infrastructure

#### 6.1.1 Dead Code Cleanup (br-8a9o) — P1

**Goal:** Resolve all documented technical debt to unblock CI quality gates.

**Current actions after 2026-04-12 reality check:**
1. Rerun `cargo clippy --all-targets -- -D warnings` and update br-8a9o from actual diagnostics.
2. Keep deferred v1.5 items explicit: interactive converter smoke test, Live Lab TUI verification, and Phase 11 catalog/doc-validation reconciliation.
3. Do not keep chasing already-resolved symbols: `jump_to_line`, `toggle_watch`, and the CSV/Table placeholder comments are no longer present in the current source.

**Acceptance:** `cargo clippy` produces zero `dead_code` warnings. All existing tests pass unchanged. No functional changes.

#### 6.1.2 CI Quality Gates (br-67nr) — P1

**Goal:** Add clippy, fmt, audit, coverage to CI. Zero quality gate violations.

**CI additions to `.github/workflows/ci.yml`:**
```yaml
lint:
  runs-on: ubuntu-latest
  steps:
    - uses: dtolnay/rust-toolchain@stable
      with: { components: clippy, rustfmt }
    - run: cargo clippy --all-targets -- -D warnings
    - run: cargo fmt --all -- --check

security:
  runs-on: ubuntu-latest
  steps:
    - run: cargo install cargo-audit --locked
    - run: cargo audit

coverage:
  runs-on: ubuntu-latest
  steps:
    - run: cargo install cargo-tarpaulin --locked
    - run: cargo tarpaulin --out xml --skip-clean
    - uses: codecov/codecov-action@v4
```

**Config files:**
- `.rustfmt.toml`: `max_width = 100`, `edition = "2024"`
- `clippy.toml`: `cognitive-complexity-threshold = 30` (FSM logic is inherently complex)

**Clippy cleanup strategy:** Fix critical paths (engine/, template/) first. Allow `.expect()` with descriptive messages in test code. Do NOT attempt to fix all 180+ `.unwrap()` at once.

**Acceptance:** All CI jobs pass green. Coverage baseline established > 60%.

#### 6.1.3 Developer Workflow Tooling (br-o12g) — P2

**Goal:** `justfile` with standard dev commands. `just --list` is self-documenting.

**Commands:** `just test`, `just lint`, `just fmt`, `just bench`, `just bench-check`, `just coverage`, `just audit`, `just release`, `just check-all`

#### 6.1.4 Cross-Platform CI (br-xqva) — P3

**Goal:** Test on macOS and Windows alongside Linux. Platform-specific concerns: file watching (notify crate), path handling (template resolver), terminal rendering (ratatui).

#### 6.1.5 Performance Regression CI (br-5dyn) — P3

**Goal:** Track Criterion baselines, warn on >5% regression in PRs.

---

### 6.2 CHAIN 2: Parsing Features

#### 6.2.1 Multi-Vendor Prompt Detection (br-dpm7) — P1

**Goal:** Extend prompt detection from IOS-only to all major vendors.

**New patterns:**
```rust
pub enum VendorHint { CiscoIos, CiscoNxos, AristaEos, JuniperJunos, NokiaSros, Unknown }

// Juniper: user@hostname> / user@hostname#
// Regex: ^(?P<user>[a-z0-9_]+)@(?P<host>[A-Za-z0-9_.-]+)[>#]

// Nokia: A:hostname# / *A:hostname#
// Regex: ^[*]?[A-Z]:(?P<host>[A-Za-z0-9_.-]+)[#>]

// IOS/NX-OS/EOS share the base pattern (differentiated by context, not prompt format)
```

**Implementation:** Refactor `ios_prompt.rs` into `transcript/prompt/mod.rs` with vendor-specific submodules. Try patterns in order of specificity: Nokia > Juniper > IOS-like.

**Acceptance:** Each vendor prompt pattern has 5+ test cases. Existing `ios_prompt.rs` tests pass unchanged.

#### 6.2.2 Template Auto-Discovery (br-wkr3) — P1

**Goal:** `cliscrape parse input.txt` (no `--template`) automatically selects the correct template.

**Detection pipeline (3 stages, cascading confidence):**
1. **Prompt detection** (~95%+ confidence) — scan first/last 5 lines for vendor prompts
2. **Command echo detection** (~85%+) — match command string against template metadata
3. **Structure fingerprinting** (~70%+) — grammar induction statistical comparison as tiebreaker

**CLI changes:**
- `--template` becomes `Option<String>` (currently required `String` in `cli.rs:32`)
- `--detect` flag: show ranked candidates without parsing
- `--detect-threshold 0.8`: adjust confidence requirement

**Template identity:** Parse vendor/command from filename convention (`cisco_ios_show_version` → vendor=`cisco_ios`, command=`show version`). Optional `vendor`/`command` metadata fields override.

**Critical constraint:** Detection < 10ms overhead. False positives worse than false negatives — conservative thresholds.

**Acceptance:** Auto-selects correct template for all 5 vendor fixtures. Confidence ≥ 0.8 for standard fixtures. Confidence < 0.6 for non-CLI text (no false positives).

#### 6.2.3 Multi-Command Session Parsing (br-sm6v) — P1

**Goal:** Parse entire SSH session transcripts with multiple commands.

**Key insight (verified):** `preprocess_ios_transcript()` already segments transcripts but **discards command names** captured by the prompt regex. The `cmd` capture group exists but is thrown away during block construction.

**Design:**
```rust
pub struct SessionBlock {
    pub command: Option<String>,
    pub output: String,
    pub line_range: (usize, usize),
    pub prompt_hostname: String,
}

pub fn session_tokenize(raw: &str) -> Vec<SessionBlock>;
// preprocess_ios_transcript() calls session_tokenize() internally for backward compat
```

**New CLI subcommand:** `cliscrape session transcript.log`
- `--commands 'show version,show interfaces'` — filter to specific commands
- `--format ndjson` — one JSON object per command
- `--split-dir output/` — write per-command files

**Edge cases:** `--More--` pagination stripping (vendor-aware), error responses (`% Invalid input`), config mode sections, banner/MOTD, nested output (`show tech`).

#### 6.2.4 Python Bindings via PyO3 (br-zfqv) — P1

**Goal:** `pip install cliscrape` — Rust-backed TextFSM-style parsing from Python with evidence-backed performance claims.

**API:**
```python
import cliscrape

records = cliscrape.parse(text, template='cisco_ios_show_version')  # list[dict]
result = cliscrape.auto_parse(text)  # ParseResult with .records, .confidence
templates = cliscrape.list_templates()  # list[TemplateInfo]
```

**Build:** PyO3 + maturin. Wheels for Linux x86_64, macOS arm64/x86_64, Windows x86_64. Embedded templates in wheel. Type stubs (.pyi) for IDE support. GIL released during parsing.

**Acceptance:** benchmarked against TextFSM on representative fixtures with recorded evidence. Type stubs pass mypy --strict. Output format matches TextFSM (list of dicts) for drop-in replacement.

---

### 6.3 CHAIN 3: Error Handling

#### 6.3.1 Error Recovery / Partial Parse (br-t76f) — P1

**Goal:** `--partial` mode emits records + quality metadata instead of silent failure.

**Key insight (verified):** `DebugReport.matches_by_line` already tracks per-line match status. Lines with empty `Vec<LineMatch>` are unmatched. Implementation reuses this infrastructure rather than building new tracking.

**Output format with `--partial`:**
```json
{
  "records": [...],
  "_metadata": {
    "parse_quality": 0.95,
    "total_lines": 200,
    "matched_lines": 190,
    "unmatched": [
      {"line": 47, "content": "...", "state": "PARSE_BODY", "nearest_rule": "rule 3"}
    ]
  }
}
```

**Exit codes:** 0 = success, 1 = below `--min-quality` threshold, 2 = no records at all.

**Nearest-rule suggestion:** Levenshtein distance between unmatched line and all rule patterns in the current FSM state. Only computed for unmatched lines (rare), so no performance impact.

#### 6.3.2 Error Context Enrichment (br-4e86) — P2

**Goal:** Errors include template name, FSM state, rule index, and pattern.

**Current:** `"Parsing error at line 47: ..."` — no context.
**After:** `"Parsing error at line 47 in template 'cisco_ios_show_version', state 'PARSE_BODY', rule 3 (pattern: '^Interface\\s+...'): ..."`

Foundation for br-jvyb (Structured Error Messages with Fix Suggestions).

#### 6.3.3 Structured Error Messages (br-jvyb) — P2

**Goal:** Actionable errors with Levenshtein-based fix suggestions.

Example: `"Line 47: no rule matched in state PARSE_BODY. Closest: rule 3 (edit distance: 2). Did you mean to add a rule for this line format?"`

#### 6.3.4 Compliance Reporting (br-981w) — P2

**Goal:** `cliscrape compliance input.txt --template ... --policy policy.yaml` — structured compliance reports (SOX, PCI, HIPAA).

Builds on Phase 16 constraints. Adds: severity levels, remediation guidance, multi-format output (JSON, table, CSV).

---

### 6.4 CHAIN 4: Streaming & Batch

#### 6.4.1 Streaming / Pipe Mode (br-z4wc) — P2

**Goal:** `ssh router 'show interfaces' | cliscrape parse --template ... --stream` — add true zero-buffer NDJSON streaming via a new `parse_iter()` iterator API.

**Architectural note (v1.2):** Current code materializes records into `Vec<BTreeMap<_, _>>`; `parse_iter()` and `--stream` do not exist yet. The intended design is to add `parse_iter()`, wire `--stream` directly to stdout, and keep `parse()` as the backward-compatible materialized API.

#### 6.4.2 Batch Processing (br-efyb) — P2

**Goal:** Parallel multi-file processing with combined output.

**Note (verified):** CLI already has `--input-glob` flag. What's missing: Rayon parallel execution, per-file metadata in output, NDJSON streaming via the proposed `parse_iter()`, and progress indicator. Optional Parquet output (`--format parquet`) deferred to a future `cliscrape-polars` feature crate (see Section 11).

---

### 6.5 Independent Features

#### 6.5.1 JSON Schema Generation (br-48hc) — P1

**Goal:** `cliscrape schema cisco_ios_show_version` → JSON Schema Draft 2020-12.

**Type mapping (verified against actual FieldType enum):**
- `type_hint = Some(Int)` → `{"type": "integer"}`
- `type_hint = Some(String)` or `None` → `{"type": "string"}`
- `list = true` → wrap in `{"type": "array", "items": ...}`
- `required = true` → include in `required` array
- `constraints.min` → `minimum`, `.max` → `maximum`, `.choices` → `enum`, `.regex` → `pattern`

#### 6.5.2 Field Transformation Pipeline (br-9jbq) — P2

**Goal:** Post-parse transforms in templates.

```yaml
fields:
  mac_address:
    transform: normalize_mac    # xx:xx:xx:xx:xx:xx lowercase
  interface:
    transform: expand_interface # Gi0/1 → GigabitEthernet0/1
  uptime:
    transform: duration_seconds # '5 days, 3:22:01' → 439321
```

Built-ins: `normalize_mac`, `expand_interface` (vendor-aware), `duration_seconds`, `strip_whitespace`, `lowercase`, `uppercase`. Custom: `{regex: '...', replace: '...'}`.

#### 6.5.3 Template Composition / Inheritance (br-y0yl) — P2

**Goal:** Templates `extend: cisco_ios_base` to inherit common patterns. Single inheritance only.

#### 6.5.4 Inline Test Fixtures (br-izvb) — P2

**Goal:** Templates include `tests:` section. `cliscrape test-template <template.yaml>` runs them. Enables TDD for template authoring.

#### 6.5.5 Template Validation Warnings (br-do0g) — P2

**Goal:** Detect unused fields, unreachable states, conflicting patterns. Surface as warnings during template loading. `--strict-template` flag treats warnings as errors.

#### 6.5.6 Output Field Selection (br-fket) — P3

**Goal:** `--fields hostname,version` and `--exclude-fields uptime`.

#### 6.5.7 Configuration File (br-tilt) — P3

**Goal:** `~/.config/cliscrape/config.toml` for persistent defaults.

#### 6.5.8 Shell Completions (br-2d02) — P3

**Goal:** `cliscrape completions bash/zsh/fish` via `clap_complete`. Dynamic template name completion.

---

### 6.6 Testing & Performance

#### 6.6.1 Diff Engine + Constraint Test Suites (br-dquy) — P1 — Closed

**Status:** Closed. Dedicated diff engine tests and comprehensive constraint validation tests are present in `tests/diff_engine.rs` and `tests/constraint_validation_comprehensive.rs`.

**Diff tests:** Identical → empty diff. Single field change. Added/removed records. Identity field matching. Reordered records. Empty inputs.

**Constraint tests:** Boundary values (exactly at min/max). Empty choices list. Invalid regex. Combined min+max. Multiple constraints per field. List field validation.

#### 6.6.2 TUI Unit Test Suite (br-n5jy) — P2

**Goal:** Expand TUI state/rendering coverage. Current code has AppState unit tests; it still lacks broad `ratatui::backend::TestBackend` rendering coverage.

Focus: AppState transitions, EditorState operations, TemplateBrowser filtering, trace navigation.

#### 6.6.3 Stress Test Suite (br-8wxa) — P2

**Goal:** 10M+ line inputs, 100K+ records, 100 constraints. Verify memory stays bounded, parsing completes in < 60s.

#### 6.6.4 API Documentation (br-dzyd) — P2

**Goal:** `/// # Examples` on all public API methods. `cargo test --doc` passes.

#### 6.6.5 Fuzz Testing the FSM Engine (br-ydmz) — P1 (New in v1.1)

**Goal:** `cargo-fuzz` target for the core FSM engine to detect and prevent panics on malformed or adversarial input.

**Rationale (GPT Pro revision):** Network CLI output is notoriously hostile — unexpected unicode, malformed escape sequences, interleaved syslog messages, terminal pagination artifacts, binary garbage from serial console corruption. Volume testing (br-8wxa) verifies scale but doesn't catch edge-case panics. Fuzzing is the only reliable way to verify the FSM won't `unwrap()` into a panic or enter an infinite loop on adversarial input.

**Implementation:**
- `fuzz/fuzz_targets/fuzz_parse.rs` — feed arbitrary `&[u8]` through `FsmParser::parse()`
- `fuzz/fuzz_targets/fuzz_template_load.rs` — feed arbitrary bytes as YAML template content
- `fuzz/fuzz_targets/fuzz_prompt_detect.rs` — feed arbitrary strings to prompt detection
- Use `libFuzzer` via `cargo-fuzz` (standard Rust fuzzing infrastructure)
- Corpus: seed with existing test fixtures for guided coverage
- CI: nightly fuzz run (10 minutes), crash artifacts committed to `fuzz/artifacts/`

**Synergy with clippy cleanup (br-67nr):** Fuzz testing will immediately expose panics from the 180+ `.unwrap()` calls. Fix these alongside the clippy cleanup for maximum efficiency.

**Acceptance:**
- Zero panics on 10-minute fuzz run with empty corpus
- Zero panics on 1-hour fuzz run with seeded corpus (existing fixtures)
- All crash artifacts triaged and fixed before release
- Fuzz targets compile in CI (nightly job)

#### 6.6.6 Memory Profiling (br-t9zw) — P3

**Goal:** Track peak RSS and allocations per parse call. Verify O(records) not O(input_lines).

#### 6.6.7 TUI Debug Export (br-g5mz) — P3

**Goal:** `Ctrl+E` exports `DebugReport` as JSON. Already has `Serialize` derive.

---

### 6.7 Packaging & Distribution

#### 6.7.1 Cargo.toml + Release Automation (cliscrape-o6e) — P2

**Fixes:**
```toml
[package]
version = "<next-release>"  # currently "0.1.0"; reconcile with report revision and tag scheme
edition = "2024"            # current code already uses Rust 2024
description = "High-performance parser for network CLI output"
license = "MIT OR Apache-2.0"
repository = "https://github.com/<user>/cliscrape"
keywords = ["network", "parsing", "cli", "textfsm", "automation"]
categories = ["command-line-utilities", "parsing", "network-programming"]
```

**Release workflow:** GitHub Actions or documented local release script on tag `v*`. Matrix: linux-x86_64, linux-aarch64, macos-x86_64, macos-aarch64, windows-x86_64. SHA256 checksums. Depends on `br-67nr`.

#### 6.7.2 Template Ecosystem Expansion (br-71zd) — P2

**New templates:**
1. `cisco_ios_show_ip_route.yaml` — routing table (most queried command)
2. `cisco_ios_show_bgp_summary.yaml` — BGP neighbor state
3. `multi_vendor_show_lldp_neighbors.yaml` — topology discovery
4. `cisco_ios_show_inventory.yaml` — hardware inventory

Each with: YAML modern format, constraints, test fixtures, snapshot tests, common schema mappings.

---

## 7. Execution Summary

| Wave | Beads | Critical Path | Estimated Effort |
|------|-------|---------------|------------------|
| Wave 1 | P1/P2 leaf work | No single critical path | ~90-160h total |
| Wave 2 | 5 | br-8a9o → br-67nr, br-dpm7 → br-wkr3 | ~30-50h |
| Wave 3 | 5 | br-wkr3 → br-sm6v/br-zfqv, br-67nr → br-o12g/br-xqva/br-5dyn | ~30-50h |
| **Total** | **tracked set** | | **~150-260h before newly filed gaps** |

### Recommended Start Order (maximum parallelism)

**Immediate (P1 leaves, highest impact per hour):**
1. br-8a9o Dead Code Cleanup (2-4h) — unblocks CI gates
2. br-dpm7 Multi-Vendor Prompts (4-6h) — unblocks auto-discovery chain
3. br-t76f Error Recovery (6-10h) — unblocks batch + compliance
4. br-dquy Diff+Constraint Tests — closed; keep as regression coverage
5. cliscrape-o6e Cargo.toml/release metadata (3-5h) — unblocks distribution; depends on br-67nr for release gating
6. br-48hc JSON Schema (3-5h) — independent, high value
7. br-ydmz Fuzz Testing (4-6h) — exposes panics alongside clippy cleanup (v1.1)

**High-value P2 leaves (start alongside P1):**
8. br-4e86 Error Context Enrichment (4-6h) — unblocks structured errors
9. br-n5jy TUI Unit Tests (8-16h) — fills biggest test gap
10. br-z4wc Streaming Mode (4-6h) — unblocks batch processing

---

## 8. Success Criteria

After all tracked improvements:

1. **Zero-config parsing:** `cliscrape parse input.txt` auto-detects vendor and template
2. **Python ecosystem:** `pip install cliscrape` available with documented benchmark evidence
3. **Session-native:** Paste entire SSH transcript, get all commands parsed
4. **Production-grade:** Partial parsing, compliance reports, quality metrics
5. **Well-tested:** Diff engine, constraints, TUI all have dedicated test suites; CI has quality gates
6. **Distributable:** Pre-built binaries, proper Cargo.toml, release automation
7. **Developer-friendly:** justfile, completions, config file, API docs
8. **Template-rich:** 9+ templates covering version, interfaces, routing, BGP, LLDP, inventory

---

## 9. Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| PyO3 build complexity across platforms | Medium | High | Start with Linux+macOS; use maturin's mature cross-compilation |
| Auto-discovery false positives | Medium | Medium | Conservative thresholds; `--detect` preview mode; explicit `--template` always wins |
| Clippy cleanup cascade (180+ unwraps) | Low | Medium | Staged: fix engine/template first, allow .expect() in tests |
| Template fingerprinting inaccuracy | Low | Low | Fingerprinting is tiebreaker only; prompt+echo detection handle 90%+ of cases |
| Backward compat: `--template` becoming optional | Low | Medium | Default behavior unchanged when `--template` provided; new behavior only when omitted |

---

## 10. Bead Reference Table

| ID | Title | Priority | Deps | Round |
|----|-------|----------|------|-------|
| br-8a9o | Dead Code Cleanup + Tech Debt | P1 | — | R2 |
| br-67nr | CI Quality Gates | P1 | br-8a9o | R2 |
| br-dquy | Diff + Constraint Test Suites | P1 | — | R2, closed |
| cliscrape-o6e | Cargo.toml + Release Automation | P2 | br-67nr | R2 |
| cliscrape-0ev | Cisco IOS show version serial extraction | P1 | — | R2 |
| cliscrape-1s8 | Common-schema Universal Ledger mappings | P1 | — | R1 |
| cliscrape-nuy | Strict FSM-Oracle verification | P2 | cliscrape-0ev | R2 |
| cliscrape-ka1 | Evidence-backed benchmark baselines | P2 | — | R2 |
| br-dpm7 | Multi-Vendor Prompt Detection | P1 | — | R1 |
| br-wkr3 | Template Auto-Discovery | P1 | br-dpm7 | R1 |
| br-sm6v | Multi-Command Session Parsing | P1 | br-dpm7, br-wkr3 | R1 |
| br-zfqv | Python Bindings (PyO3) | P1 | br-wkr3 | R1 |
| br-t76f | Error Recovery / Partial Parse | P1 | — | R1 |
| br-48hc | JSON Schema Generation | P1 | — | R1 |
| br-ydmz | Fuzz Testing FSM Engine | P1 | — | v1.1 |
| br-4e86 | Error Context Enrichment | P2 | — | R2 |
| br-jvyb | Structured Error Messages | P2 | br-4e86 | R1 |
| br-izvb | Inline Test Fixtures | P2 | br-jvyb | R1 |
| br-z4wc | Streaming / Pipe Mode | P2 | — | R1 |
| br-efyb | Batch Processing | P2 | br-z4wc, br-t76f | R1 |
| br-981w | Compliance Reporting | P2 | br-t76f | R1 |
| br-9jbq | Field Transformation Pipeline | P2 | — | R1 |
| br-y0yl | Template Composition | P2 | — | R1 |
| br-o12g | Dev Workflow Tooling (justfile) | P2 | br-67nr | R2 |
| br-n5jy | TUI Unit Test Suite | P2 | — | R2 |
| br-71zd | Template Ecosystem: Core Commands | P2 | — | R2 |
| br-do0g | Template Validation Warnings | P2 | — | R2 |
| br-dzyd | API Documentation | P2 | — | R2 |
| br-8wxa | Stress Test Suite | P2 | — | R2 |
| br-fket | Output Field Selection | P3 | — | R1 |
| br-tilt | Configuration File | P3 | — | R1 |
| br-2d02 | Shell Completions | P3 | — | R1 |
| br-g5mz | TUI Debug Export | P3 | — | R2 |
| br-t9zw | Memory Profiling | P3 | — | R2 |
| br-5dyn | Performance Regression CI | P3 | br-67nr | R2 |
| br-xqva | Cross-Platform CI | P3 | br-67nr | R2 |

### v1.1 — Deferred to future plan cycle

| ID | Title | Priority | Rationale for deferral |
|----|-------|----------|----------------------|
| — | Arrow/Polars DataFrame Integration | P3 | Heavy dependency, niche use case. Better as separate `cliscrape-polars` crate after core stabilizes. |
| — | Template LSP (Language Server) | P3 | High effort (~40h+), effectively a sub-project. Template validation warnings (br-do0g) cover the static analysis use case for now. Revisit for v3.0. |

---

## 11. GPT Pro Revision Assessment (v1.1)

### Wholeheartedly agree: Fuzz Testing the FSM Engine

This is the single highest-value addition from the review. Network CLI output is genuinely hostile territory — serial console corruption, interleaved syslog, terminal escape sequences, malformed UTF-8 from misconfigured devices. The 180+ `.unwrap()` calls in the codebase make this especially urgent: any one of them is a potential panic on adversarial input.

Slotting `cargo-fuzz` into Wave 1 alongside the clippy cleanup (br-67nr) creates a powerful feedback loop: fuzzing discovers the panics, clippy cleanup fixes the root causes. Tracked as br-ydmz, P1, independent leaf.

### Somewhat agree: Iterator-Based Streaming Engine

The core insight is correct — `parse()` returning a materialized `Vec` is an O(N) memory commitment that limits streaming use cases. However, **replacing** the existing API signature would be a breaking change that ripples through every consumer (CLI, TUI, Python bindings, tests).

**Plan:** `parse()` should stay as-is for backward compatibility. Add `parse_iter()` as the new streaming primitive and wire `--stream` to stdout as part of br-z4wc. The FSM's `RecordBuffer` already emits records one at a time during line processing, so the iterator adapter is architecturally natural, but it is not present in current code yet.

### Somewhat disagree: Arrow/Polars DataFrame Integration

The idea is sound for analytics-heavy users, but premature for this plan cycle:
- **Polars is a heavy dependency** — adds significant compile time and binary size
- **The primary user base is network engineers**, not data scientists. They pipe JSON into jq, Python, or Ansible.
- **NDJSON output already enables Polars/DuckDB ingestion** — `polars.read_ndjson()` or `duckdb.read_json()` work out of the box
- **Better as a separate crate** — `cliscrape-polars` as an optional feature crate after the core API stabilizes

Deferred to future plan cycle. Noted in Section 10 reference table.

### Disagree: Template LSP

The vision is appealing — live regex validation and state reachability in VSCode is genuinely useful for template authors. But:
- **This is a sub-project**, not a bead. A proper LSP implementation is ~40-60h of work with its own test suite, protocol compliance, and editor extension packaging.
- **Template Validation Warnings (br-do0g) covers 80% of the value** — unused fields, unreachable states, conflicting patterns are surfaced during template loading. The `--strict-template` flag catches authoring mistakes at parse time.
- **The TUI (Live Lab) is the runtime equivalent** — interactive debugging with trace stepping, template editing, and hot-reload.
- **User base size doesn't justify it yet** — LSP makes sense when there are hundreds of template authors. Currently the embedded library has 5 templates.

Deferred to v3.0 planning when template ecosystem is larger and author community exists.

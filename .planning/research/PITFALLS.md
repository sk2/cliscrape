# Domain Pitfalls: CLI Scraping & Parsing

**Domain:** Network CLI Scraping / Rust TUI
**Researched:** 2024-10-24
**Confidence:** HIGH

## Critical Pitfalls

Mistakes that cause rewrites, major performance bottlenecks, or catastrophic parsing failures.

### 1. Fragile Regex & "Firmware Drift"
**What goes wrong:** Regular expressions are written too strictly (e.g., matching exact space counts) or fail to account for minor variation in vendor output (Cisco IOS vs. XE, or version 15.x vs 17.x).
**Why it happens:** Development is often done against a single "golden" output capture rather than diverse real-world samples.
**Consequences:** Templates break silently or return partial data when the network device is upgraded.
**Prevention:** Use flexible whitespace matches (`\s+` instead of ` `), anchor patterns to stable headers, and implement "fuzzy" matching for non-critical fields.
**Detection:** Implement a "parse coverage" metric; if a line in the input isn't matched by any rule, flag it as a warning in the TUI.

### 2. Unbounded Memory in State-Tracing
**What goes wrong:** The TUI debugger stores every state transition and captured group for "live-editing" and "time-travel," leading to OOM (Out of Memory) when parsing 50MB+ BGP table outputs.
**Why it happens:** Storing the entire history of the state machine transitions in a `Vec<State>` without limits.
**Consequences:** Tool crashes on large outputs, precisely when the debugger is most needed for complex parsing.
**Prevention:** Implement a circular buffer for traces or a "streaming" mode where only the last N lines of state are kept unless explicitly "pinned" by the user.
**Detection:** Monitor heap usage during large file imports in the TUI.

### 3. Blocking the UI Thread with Network I/O
**What goes wrong:** Integrating the scraper core directly with the TUI without a robust async/threading boundary.
**Why it happens:** Rust's `tokio` or `std::mpsc` channels are misconfigured, or a synchronous SSH library is used in the same thread as the Ratatui render loop.
**Consequences:** UI freezes during network latency spikes or SSH handshakes, creating a poor "hanging" experience.
**Prevention:** Use a dedicated worker thread/task for parsing and I/O. Use non-blocking channels to send `UIEvent` updates to the renderer.
**Detection:** Visible "stutter" in TUI animations or input responsiveness when network operations are active.

## Moderate Pitfalls

Mistakes that cause delays, technical debt, or confusing user experiences.

### 1. The "Norway Problem" (YAML/TOML Type Coercion)
**What goes wrong:** Hybrid YAML/TOML formats interpret "unquoted" values incorrectly. For example, `NO` as `false`, `08` as an octal error, or a serial number like `1e10` as a float.
**Why it happens:** Standard YAML/TOML parsers apply implicit typing to scalars.
**Consequences:** Structured data contains booleans or floats where strings were expected, breaking downstream integrations.
**Prevention:** Use a schema-aware parser or force string types for all template values unless explicitly typed in the format.
**Detection:** Validation errors in the YAML/TOML loader or unexpected types in the TUI's variable inspector.

### 2. Paging and Buffer Fragmentation
**What goes wrong:** The scraper fails when it encounters `--More--` prompts or when a single logical line is split across two TCP packets/buffers.
**Why it happens:** Parsing logic assumes the entire output is available as a single contiguous string in memory.
**Consequences:** Partial data records or "garbage" text (the paging prompt itself) being parsed as data.
**Prevention:** Implement a "pre-processor" or "cleaner" layer that handles paging prompts and re-assembles fragments before passing to the TextFSM engine.
**Detection:** Regex failures that only occur on large outputs (which trigger paging) but pass on small snippets.

## Minor Pitfalls

Mistakes that cause annoyance or minor maintenance overhead.

### 1. Inefficient Regex Re-compilation
**What goes wrong:** Compiling `Regex::new()` inside a loop for every line of CLI output.
**Why it happens:** Forgetting that regex compilation is expensive in Rust.
**Consequences:** Significant performance degradation (10x-100x slower) on large files.
**Prevention:** Use `once_cell` or `lazy_static` to compile regexes once, or compile them once when the template is loaded.
**Detection:** Profiling with `flamegraph` shows `regex::compiler` taking significant CPU time.

### 2. Missing "Record" Actions in State Machine
**What goes wrong:** Users define values and patterns but forget to trigger the `Record` action in TextFSM.
**Why it happens:** TextFSM state machine logic can be non-intuitive (implicit vs. explicit recording).
**Consequences:** The parser runs through the whole file but returns an empty list of results.
**Prevention:** The TUI debugger should visually highlight when a `Record` action is triggered (e.g., a flash or a counter).
**Detection:** TUI "Results" pane remains empty while the "Matches" pane shows hits.

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| **Core Parser** | Regex re-compilation / performance | Use `lazy_static` or `RegexSet`. Profile with large datasets early. |
| **TUI Debugger** | UI Thread Blocking | Architecture must use `tokio` channels or `crossbeam` for I/O separation. |
| **Hybrid Formats** | Type Ambiguity | Implement strict string coercion for unquoted values in YAML/TOML. |
| **State Tracing** | Memory Bloat | Implement a max-size window for state history in the TUI. |

## Sources

- [TextFSM ReadTheDocs - Common Errors](https://textfsm.readthedocs.io/en/latest/#common-errors)
- [Rust Regex Performance Best Practices](https://docs.rs/regex/latest/regex/#performance)
- [Ratatui Documentation - Async & Threading](https://ratatui.rs/concepts/async/)
- [The "Norway Problem" in YAML](https://hitchdev.com/strictyaml/why/implicit-typing-is-evil/)

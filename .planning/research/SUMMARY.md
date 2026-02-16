# Research Summary: cliscrape

## Executive Summary

`cliscrape` is a high-performance network CLI scraping and parsing tool built in Rust. It aims to bridge the gap between legacy TextFSM-based automation and modern development workflows by providing a dedicated TUI debugger and a single-binary, memory-efficient runtime. The tool prioritizes full compatibility with the existing `ntc-templates` ecosystem while introducing a more flexible internal state machine IR that supports modern configuration formats like YAML and TOML.

The recommended approach focuses on building a robust, modular core that separates the parsing engine from the input formats and transport layers. By leveraging Rust's async ecosystem (`tokio`, `russh`) and modern TUI frameworks (`ratatui`), `cliscrape` can provide real-time feedback during template development—a major pain point in the industry. Key risks include handling vendor-specific "firmware drift" and ensuring the TUI remains responsive during intensive network I/O or large-scale parsing.

## Key Findings

### From STACK.md
- **Core:** Rust (1.82+) with `tokio` for safe, high-concurrency async I/O.
- **Connectivity:** `russh` chosen for pure-Rust, async-native SSH connectivity.
- **Parsing:** `textfsm-rust` for 99%+ compatibility with the legacy ecosystem; `serde` for JSON/YAML/TOML serialization.
- **TUI:** `ratatui` for the visual debugger, with `tracing` and `tui-logger` for live state instrumentation.

### From FEATURES.md
- **Table Stakes:** Multi-vendor SSH support, TextFSM compatibility, and robust prompt management are non-negotiable.
- **Differentiators:** The TUI Debugger with live regex matching and state tracing is the primary value proposition.
- **Anti-Features:** Explicitly avoiding configuration management (Read-only focus) and web dashboards to maintain performance and simplicity.

### From ARCHITECTURE.md
- **Modular Parser Engine:** Implementation of an internal State Machine IR that serves as a common target for TextFSM, YAML, and TOML frontends.
- **Event-Driven Tracing:** The engine emits `TraceEvents` to decouple execution from the UI, allowing the TUI to visualize matches without blocking.
- **Hot-Reloading:** Fast Rust execution enables instant re-parsing of templates upon edit, providing a modern developer experience.

### From PITFALLS.md
- **Critical Pitfalls:** Fragile regex (firmware drift), memory bloat in state-tracing, and UI thread blocking during network operations.
- **Moderate Pitfalls:** The "Norway Problem" (type coercion in YAML) and handling paging/buffer fragmentation during SSH sessions.
- **Mitigation:** Use of circular buffers for traces, dedicated worker threads for I/O, and strict string coercion for template values.

## Implications for Roadmap

### Suggested Phase Structure

1.  **Phase 1: Foundation (Core Engine & IR)**
    - *Rationale:* Build the execution logic first without being tied to a specific DSL.
    - *Delivers:* Internal state machine IR and parsing engine.
    - *Pitfalls:* Implement regex caching immediately to avoid performance debt.
2.  **Phase 2: Legacy Compatibility (TextFSM Frontend)**
    - *Rationale:* Critical for immediate utility with existing template libraries.
    - *Delivers:* `.textfsm` parser and verification against `ntc-templates`.
    - *Research Flag:* Standard patterns (skip research).
3.  **Phase 3: CLI Utility (Local Mode)**
    - *Rationale:* Provides a "Unix-style" tool for local file parsing.
    - *Delivers:* File ingestion, JSON/YAML/CSV output, and basic error reporting.
4.  **Phase 4: TUI Debugger (The Killer Feature)**
    - *Rationale:* Implements the main differentiator for template developers.
    - *Delivers:* Live regex matching, state visualization, and hot-reloading templates.
    - *Research Flag:* Needs `/gsd:research-phase` for Ratatui concurrency and event handling.
5.  **Phase 5: Live Connectivity (SSH Scraper)**
    - *Rationale:* Transition from local files to live device scraping.
    - *Delivers:* `russh` integration and automated prompt management.
    - *Research Flag:* Needs `/gsd:research-phase` for multi-vendor prompt/paging handling.
6.  **Phase 6: Modern Formats (YAML/TOML Templates)**
    - *Rationale:* Future-proofs the tool and addresses the limitations of the TextFSM DSL.
    - *Delivers:* Modern structured configuration for command-to-template mapping.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | Leverages stable, industry-standard Rust crates. |
| Features | HIGH | Clear alignment with industry pain points (TextFSM debugging). |
| Architecture | HIGH | Modular IR-based design prevents legacy lock-in. |
| Pitfalls | HIGH | Covers critical areas of network automation and UI responsiveness. |

### Gaps to Address
- **Prompt Management:** Detailed handling for diverse vendor prompts (Juniper, Arista) needs validation during implementation.
- **Large Dataset Performance:** Behavior of the TUI state-trace with very large (>50MB) files needs practical stress testing.

## Sources
- [TextFSM-Rust GitHub & Crates.io](https://crates.io/crates/textfsm-rust)
- [Russh Documentation](https://docs.rs/russh/latest/russh/)
- [Ratatui Ecosystem & Concepts](https://ratatui.rs/concepts/architecture/)
- [Netmiko & TextFSM Documentation](https://github.com/ktbyers/netmiko)
- [Scrapli Performance Comparison](https://github.com/scrapli/scrapli)
- [The "Norway Problem" in YAML](https://hitchdev.com/strictyaml/why/implicit-typing-is-evil/)

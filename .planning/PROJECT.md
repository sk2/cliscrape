# CLI Parser

## Concept

A Rust parsing engine for network device output. It transforms semi-structured CLI text into structured data (JSON/YAML) using a TextFSM-compatible state machine. Designed as a modern, ergonomic alternative to legacy TextFSM workflows, it supports legacy templates and a small embedded template library while full external template-library coverage remains tracked follow-up work.

### Core Value

Enable rapid, reliable extraction of network state. The engine is built for predictable parsing, structured diagnostics, and policy-aware validation; release performance claims should be tied to current benchmark output.

## Features

- **High-Performance Engine**: Optimized Rust implementation with benchmarks in `benches/`; release claims should be tied to current benchmark output.
- **Library Compatibility**: Legacy TextFSM support plus a small embedded template library. Full `ntc-templates` coverage remains a compatibility target, not the current implementation.
- **Ergonomic Template Format**: Introduces a clean YAML/TOML based template syntax for faster development and better maintainability.
- **Interactive TUI Debugger**: A built-in terminal interface for template testing, trace inspection, editing, and hot-reload debugging against device output.
- **Owned JSON Output**: Captures are converted into owned JSON records. Zero-copy parsing remains a possible future optimization.

## Use Cases

- **High-Frequency Monitoring**: Parse `show` commands in polling cycles for telemetry and health checks.
- **Inventory Discovery**: Automatically extract device details, interface states, and hardware versions for asset management.
- **Pre-deployment Validation**: Verify the current state of a network before pushing configuration changes to ensure safety.
- **Automated Troubleshooting**: Rapidly analyze complex outputs like routing tables or BGP summaries during incident response.

## Technical Depth

The engine is built on a custom regex-based state machine implemented in Rust. It utilizes pre-compiled patterns and a non-backtracking execution model to ensure deterministic performance even with complex multi-state templates.

- **Stack**: Rust, `regex` crate for optimized matching, `ratatui` for the TUI debugger.
- **Parallelism**: Thread-safe design allowing for concurrent parsing of multiple device outputs across all available CPU cores.
- **Validation**: Required fields, coverage thresholds, and field constraints can warn or fail depending on parse options.

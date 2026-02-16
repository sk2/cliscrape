# cliscrape

## What This Is

`cliscrape` is a high-performance CLI scraping and parsing tool for network devices, written in Rust. It provides a modern, ergonomic, and blazingly fast alternative to legacy tools like `TextFSM`, while maintaining first-class compatibility with existing templates.

## Why We're Building This

The networking industry relies heavily on CLI-based management, and parsing this unstructured data is a major bottleneck in automation pipelines. `cliscrape` aims to:
- Solve the **throughput bottleneck** of Python-based parsers.
- Replace "regex soup" with **modern, ergonomic, and structured** (YAML/TOML) template formats.
- Provide a **TUI-based "Dry Run" experience** for both live-editing templates and tracing FSM state transitions.

## Core Value

The one thing that must work perfectly: **Extremely fast, reliable parsing of semi-structured CLI output into structured data, regardless of whether the template is legacy TextFSM or the new ergonomic format.**

## Current Milestone: v0.1 Alpha

**Goal:** Build a complete, high-performance CLI parsing tool with TextFSM compatibility, modern YAML/TOML templates, and an interactive TUI debugger.

**Target capabilities:**
- High-throughput FSM engine for parsing CLI output
- Full TextFSM grammar support for backward compatibility
- Modern YAML/TOML template format as ergonomic alternative
- Interactive TUI for template development and debugging
- Vendor-agnostic design (Cisco, Juniper, Arista, etc.)

## Requirements

### Validated

- ✓ **Rust Project Scaffolding** — Basic project structure and CLI setup initialized.
- ✓ **Vision & Documentation** — Initial README and DESIGN documents created.

### Active

- [ ] **High-Throughput FSM Engine** — A core Rust-based state machine capable of processing thousands of command outputs per second.
- [ ] **TextFSM Compatibility Layer** — Support for the full TextFSM grammar, including `Filldown`, `Required`, and all actions (`Next`, `Continue`, `Record`, `Clear`).
- [ ] **Modern Ergonomic Format** — A hybrid YAML/TOML format using named blocks, shared regex patterns, and typed captures.
- [ ] **TUI "Dry Run" Environment** — An interactive debugger with live-editing and step-through state tracing.
- [ ] **Vendor-Agnostic Core** — A generic engine that handles Cisco, Juniper, Arista, and others equally well.

### Out of Scope

- **Massive File Optimization (v0.1)** — While fast, the primary focus is throughput across many small outputs rather than gigabyte-scale single files.
- **Direct Device Interaction (v0.1)** — SSH/Telnet connectivity is deferred. Other tools like `netmiko` can handle device connections; `cliscrape` focuses on parsing excellence. May be added as optional transport layer in future versions.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| **Language: Rust** | Need for high performance, memory safety, and concurrency without a GIL. | — Finalized |
| **Primary Goal: Throughput** | Optimize for high-volume automation fleets rather than massive single logs. | — Finalized |
| **Format: Hybrid YAML** | Combine the power of FSM logic with the readability of structured YAML. | — Finalized |
| **TUI Mode: Dual-Purpose** | Support both "Live Lab" (editing) and "State Tracer" (debugging) workflows. | — Finalized |

---
*Last updated: 2026-02-17 after v0.1 milestone initialization*

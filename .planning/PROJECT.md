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

## Current Milestone: v0.5 Beta

**Goal:** Transform cliscrape into a complete network automation solution with direct device connectivity and a comprehensive template ecosystem.

**Target capabilities:**
- SSH/Telnet connectivity layer (single commands, interactive sessions, batch operations)
- ntc-templates compatibility validation and testing
- Template conversion pipeline (TextFSM → modern YAML)
- Built-in template library with discovery mechanism
- Device credential management and authentication

## Requirements

### Validated

**v0.1 Alpha (Complete):**
- ✓ **Rust Project Scaffolding** — Basic project structure and CLI setup initialized.
- ✓ **Vision & Documentation** — Initial README and DESIGN documents created.
- ✓ **High-Throughput FSM Engine** — Core Rust-based state machine processing ~4.1M lines/sec.
- ✓ **TextFSM Compatibility Layer** — Full grammar support with Filldown, Required, all actions.
- ✓ **Modern Ergonomic Format** — YAML/TOML templates with typed captures and macro support.
- ✓ **TUI "Dry Run" Environment** — Interactive Live Lab and State Tracer for debugging.
- ✓ **Vendor-Agnostic Core** — Generic engine handling Cisco, Juniper, Arista, etc.

### Active

**v0.5 Beta (Current):**
- [ ] **SSH/Telnet Connectivity** — Direct device connection with command execution and session management.
- [ ] **Interactive Session Management** — Multi-command sessions with context tracking and prompt handling.
- [ ] **Batch Device Operations** — Parallel execution across device fleets with result aggregation.
- [ ] **ntc-templates Compatibility** — Validation suite ensuring existing templates work unchanged.
- [ ] **Template Conversion Tools** — Pipeline to convert TextFSM templates to modern YAML format.
- [ ] **Template Library & Discovery** — Built-in templates referenced by name without local files.

### Out of Scope

- **Massive File Optimization** — Primary focus remains throughput across many small outputs rather than gigabyte-scale single files.
- **GUI/Web Interface** — CLI-first tool; graphical interfaces deferred to future versions.
- **Configuration Management** — Focuses on read-only command execution; device configuration changes out of scope.
- **SNMP/NETCONF/REST** — SSH/CLI-focused; other protocols deferred to maintain scope.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| **Language: Rust** | Need for high performance, memory safety, and concurrency without a GIL. | — Finalized |
| **Primary Goal: Throughput** | Optimize for high-volume automation fleets rather than massive single logs. | — Finalized |
| **Format: Hybrid YAML** | Combine the power of FSM logic with the readability of structured YAML. | — Finalized |
| **TUI Mode: Dual-Purpose** | Support both "Live Lab" (editing) and "State Tracer" (debugging) workflows. | — Finalized |

---
*Last updated: 2026-02-21 after v0.5 milestone initialization*

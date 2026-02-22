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

## Current State

**Shipped:** v1.0 MVP (2026-02-22)
**Status:** Production-ready CLI parser with TUI debugger
**Codebase:** 6,970 lines of Rust
**Test Coverage:** 77 tests passing (51 engine + 9 TUI + 6 CLI e2e + 3 modern + 8 TextFSM compat)

## Requirements

### Validated

**v1.0 MVP (Shipped 2026-02-22):**
- ✓ **CORE-01:** High-throughput Rust-based FSM engine (~4.1M lines/sec) — v1.0
- ✓ **CORE-02:** Full TextFSM grammar (Filldown, Required, all actions) — v1.0
- ✓ **CORE-03:** Shared regex pattern library (macros: ipv4, mac_address, interface_name) — v1.0
- ✓ **FORM-01:** 100% .textfsm compatibility — v1.0
- ✓ **FORM-02:** Modern YAML/TOML ergonomic format — v1.0
- ✓ **FORM-03:** Typed captures (int, string) with automatic conversion — v1.0
- ✓ **TUI-01:** Live Lab split-pane view with real-time regex matching — v1.0
- ✓ **TUI-02:** State Tracer for FSM transitions line-by-line — v1.0
- ✓ **TUI-03:** Trace history buffer for debugging complex templates — v1.0
- ✓ **CLI-01:** Standalone CLI for local files or piped input — v1.0
- ✓ **CLI-02:** Basic Cisco IOS/NX-OS prompt support — v1.0

### Active

**v2.0 (Planned):**
- [ ] **SSH/Telnet Connectivity** — Direct device connection with command execution
- [ ] **Interactive Session Management** — Multi-command sessions with prompt handling
- [ ] **Batch Device Operations** — Parallel execution across device fleets
- [ ] **ntc-templates Validation Suite** — Automated compatibility testing
- [ ] **Template Library & Discovery** — Built-in templates by device/command

### Out of Scope

- **Massive File Optimization** — Primary focus remains throughput across many small outputs rather than gigabyte-scale single files.
- **GUI/Web Interface** — CLI-first tool; graphical interfaces deferred to future versions.
- **Configuration Management** — Focuses on read-only command execution; device configuration changes out of scope.
- **SNMP/NETCONF/REST** — SSH/CLI-focused; other protocols deferred to maintain scope.

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| **Language: Rust** | Need for high performance, memory safety, and concurrency without a GIL. | ✓ Good — Achieved 4.1M lines/sec (v1.0) |
| **Primary Goal: Throughput** | Optimize for high-volume automation fleets rather than massive single logs. | ✓ Good — 40x faster than Python TextFSM |
| **Format: Hybrid YAML** | Combine the power of FSM logic with the readability of structured YAML. | ✓ Good — Modern templates working alongside TextFSM |
| **TUI Mode: Dual-Purpose** | Support both "Live Lab" (editing) and "State Tracer" (debugging) workflows. | ✓ Good — Both modes implemented (v1.0) |
| **Pest for TextFSM Grammar** | Use parser generator for strict TextFSM DSL compliance. | ✓ Good — 100% compatibility achieved (v1.0) |
| **Emit-time Type Conversion** | Convert captured strings to typed JSON at record emission vs parse-time. | ✓ Good — Allows heuristics with explicit override (v1.0) |
| **Warning-Returning Loader API** | Library returns warnings without printing to stderr. | ✓ Good — Clean library API, print-free (v1.0) |
| **TTY-Aware Format Auto** | `format=auto` resolves to table (TTY) or JSON (non-TTY). | ✓ Good — Unix-style CLI contract (v1.0) |
| **Full Variable Snapshots in Trace** | Store complete variable state per trace event vs deltas. | — Pending — Monitor memory with real-world templates |

## Constraints

- **Vendor-Agnostic:** No vendor-specific logic in the engine; all device differences handled via templates
- **CLI-First:** Focus on terminal workflows; GUI/web interfaces out of scope
- **Read-Only:** Parse command outputs only; no configuration management or device changes

---
*Last updated: 2026-02-22 after v1.0 MVP milestone*

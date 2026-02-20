# Project State - cliscrape

## Project Reference

**Core Value:** Extremely fast, reliable parsing of semi-structured CLI output into structured data, regardless of whether the template is legacy TextFSM or the new ergonomic format.

**Current Focus:** Phase 4: TUI Debugger Foundation (Live Lab)

## Current Position

**Active Phase:** Phase 4: TUI Debugger Foundation (Live Lab)
**Active Plan:** 04-02 complete (2/5)
**Status:** 🟢 In progress

**Progress:**
`[█████████████████░░░] 84%` (04-02 complete; carryover: interactive converter smoke pending)

## Performance Metrics

- **Parsing Throughput:** ~4.1M lines/sec (Target: >100k)
- **TextFSM Compatibility:** 100% (Core TextFSM attributes and EOF logic implemented)
- **Code Coverage:** ~60%

## Accumulated Context

### Key Decisions
- **Language:** Rust for performance and safety.
- **TUI Framework:** Ratatui for the visual debugger.
- **SSH Library:** Russh for async-native connectivity (v2).
- **Output Formats:** JSON, CSV, and Pretty-Table (02-03).
- **Parsing Grammar:** Pest for legacy TextFSM templates (02-01).
- **Emit-time Type Conversion:** Convert captured strings to typed `serde_json::Value` at record emission; explicit per-field hint wins, numeric-only heuristics otherwise, failure preserves raw string.
- **Modern Template Typing Default:** Modern templates default fields to explicit `string` typing (type_hint=String) unless declared otherwise, to avoid heuristic surprises.
- **CLI Template Format Override:** `cliscrape parse --template-format auto|textfsm|yaml|toml` forces loader selection (useful when file extensions are ambiguous).
- **Macro Expansion Semantics:** Macro expansion is recursive with max depth 10 and explicit cycle detection; template-local macros shadow builtins.
- **Template Migration Tooling:** `cliscrape convert` converts `.textfsm` to strict-schema modern YAML/TOML via a shared `ModernTemplateDoc` + serialization helpers.
- **Debug Trace Actions:** DebugReport stores action/record semantics as strings to stay serde-friendly without changing engine Action types.
- **EOF Record Attribution:** DebugReport uses `line_idx = lines.len()` as an explicit EOF sentinel for implicit record emission.

### Todos
- [x] Create engine module structure
- [x] Implement regex macro library
- [x] Implement FSM execution engine
- [x] Implement record management and validation
- [x] Implement TextFSM DSL Parser
- [x] Implement CLI & Input Stream Handling
- [x] Add Serialization & Refinements
- [x] Implement Modern Ergonomic Templates (YAML/TOML)
- [x] Add Automatic Type Conversion
- [ ] Run interactive converter smoke test (Phase 3 verification)
- [x] Phase 4: Implement TUI scaffolding + `cliscrape debug` wiring (04-02)

### Blockers
- None

## Session Continuity

**Last Session:** 2026-02-20T23:34:03Z
**Stopped at:** Completed 04-tui-debugger-foundation-(live-lab) 04-02-PLAN.md
**Resume file:** None

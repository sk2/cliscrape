# Project State - cliscrape

## Project Reference

**Core Value:** Extremely fast, reliable parsing of semi-structured CLI output into structured data, regardless of whether the template is legacy TextFSM or the new ergonomic format.

**Current Focus:** Milestone v0.5 Beta - Connectivity & Templates

## Current Position

**Active Phase:** 02-legacy-compatibility-cli (Legacy Compatibility & CLI)
**Active Plan:** 02-05 (Parse clap contract)
**Status:** In progress

**Progress:**
`[█████████░] 96%` (24/25 plans complete; latest: 02-05)

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
- **Phase-2 Parse CLI Contract:** `cliscrape parse` exposes multi-input flags (`inputs...`, `--input`, `--stdin`, `--input-glob`), defaults `--format` to `auto`, and provides global `--error-format human|json`.
- **Macro Expansion Semantics:** Macro expansion is recursive with max depth 10 and explicit cycle detection; template-local macros shadow builtins.
- **Template Migration Tooling:** `cliscrape convert` converts `.textfsm` to strict-schema modern YAML/TOML via a shared `ModernTemplateDoc` + serialization helpers.
- **Debug Trace Actions:** DebugReport stores action/record semantics as strings to stay serde-friendly without changing engine Action types.
- **EOF Record Attribution:** DebugReport uses `line_idx = lines.len()` as an explicit EOF sentinel for implicit record emission.
- **Live Reload Architecture:** Debounced notify watcher (watch parent dirs) emits `Message::FsChanged`; background parse worker coalesces requests and retains last-good results while showing current parse errors.
- **Live Lab Keymap:** Tab toggles Matches/Records view; `[ ]` / `h l` / left-right cycles match/record selection.
- **Highlight Safety:** Capture span rendering uses byte offsets with UTF-8 boundary checks; invalid spans are skipped and surfaced as warnings.
- **Trace Variable Snapshots:** Full variable snapshots per trace event (not delta encoding) for initial simplicity; optimize only if profiling shows memory issues with real-world templates.
- **Default Stepping Mode:** LineByLine for most granular visibility (users can switch to coarser StateByState or ActionByAction modes).
- **Default Filter State:** Show all event types (visibility-first approach; users can hide events if needed).
- **Timeline Pane Layout:** Vertical split with timeline (top) and variables (bottom) in right pane; timeline gets more space for scrolling through long traces.
- **Variable Change Highlighting:** Yellow color + bold for changed variables; old->new values shown inline; watched variables marked with star and sorted first.
- **StateTracer View Mode:** Tab cycles through Matches -> Records -> StateTracer; [ ] navigation maps to trace stepping in StateTracer mode.
- **State Tracer Keybindings:** PgUp/PgDn for step backward/forward; Ctrl+N/P for jump to next/previous Record; m for toggle stepping mode (Line/State/Action); F1-F4 for toggle filter categories.

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
- [x] Phase 4: Add live reload watcher + background parsing with last-good retention (04-03)
- [x] Phase 4: Add match shading + capture highlights + records toggle (04-04)
- [x] Phase 4: Add picker + inline template editor workflow (04-05)
- [ ] Run Live Lab TUI interactive verification for 04-05 (deferred to milestone)

### Blockers
- None

## Session Continuity

**Last Session:** 2026-02-22T00:00:00Z
**Stopped at:** Session resumed, proceeding to execute Phase 2 (plans 02-06 through 02-09)
**Resume file:** None

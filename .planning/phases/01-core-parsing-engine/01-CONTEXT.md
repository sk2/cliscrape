# Phase 1 Context: Core Parsing Engine

This phase establishes the foundational parsing engine: a deterministic finite state machine (FSM) that can apply state-scoped regex rules to an input stream and emit structured records.

## Goal

- High-throughput, deterministic parsing of semi-structured CLI output using an FSM model.
- Support a small built-in regex macro library (e.g., `{{ipv4}}`, `{{mac_address}}`) to make templates readable.

## Non-Goals (Deferred)

- External user-defined macro files (e.g., `macros.yaml`).
- Template migration tooling.
- Automatic type coercion / typed outputs.
- Debugger / trace UI.

## Locked Decisions (for this phase)

### Regex Macros (simple start)

- Built-in only: macros are provided by the engine.
- No macro nesting in this phase (keep compilation simple).
- Local shadowing: if a template defines a value with the same name as a macro, the template value wins.

### FSM Execution Behavior

- Unmatched lines: if no rules match a line in the current state, discard the line and remain in the same state.
- Invalid transitions: error if a rule transitions to a non-existent state.
- Verbosity: default engine stays quiet (data output or hard errors only).

### Record Semantics

- Required values: if a `Record` action triggers but a value marked `Required` is empty, silently discard the record.
- Data is stored and returned as strings only (no typing).

## Done Looks Like

- Benchmark shows parsing throughput above the project target.
- Unit tests demonstrate deterministic state transitions.
- Macro substitution works for `{{ipv4}}` and `{{mac_address}}` in compiled regex patterns.

---
*Created: 2024-05-22*  
*Moved into phase directory: 2026-02-21*

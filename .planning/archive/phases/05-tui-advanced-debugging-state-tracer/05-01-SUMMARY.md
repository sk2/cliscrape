---
phase: 05-tui-advanced-debugging-state-tracer
plan: 01
subsystem: debug-infrastructure
tags: [debug, trace, temporal, fsm-state]
dependency_graph:
  requires: [phase-04-tui-debugger-foundation]
  provides: [temporal-trace-events, variable-snapshots, event-type-filtering]
  affects: [src/engine/debug.rs, src/engine/fsm.rs, src/engine/records.rs]
tech_stack:
  added: []
  patterns: [temporal-snapshot, event-classification]
key_files:
  created: []
  modified:
    - src/engine/debug.rs
    - src/engine/fsm.rs
    - src/engine/records.rs
decisions:
  - Use full variable snapshots per event for initial simplicity (not delta encoding)
  - event_type classification enables filtering by stepping mode
  - variables HashMap matches engine's typed capture format via convert_scalar
  - EOF records use line_idx = lines.len() as explicit sentinel
metrics:
  duration: 196s
  tasks_completed: 3
  tests_added: 5
  completed_date: 2026-02-21
---

# Phase 05 Plan 01: Temporal Trace Infrastructure Summary

**One-liner:** Full FSM state trace with temporal variable snapshots and event type classification for time-travel debugging

## What Was Built

Extended Phase 4's debug infrastructure with comprehensive temporal trace capabilities. DebugReport now contains a complete timeline of FSM execution with state transitions, variable values at each line, and classified event types to support different stepping modes in the future TUI debugger.

### Temporal Trace Model (Task 1)

Added `TraceEvent` struct to capture FSM state at each line:
- `line_idx`: Position in input
- `state_before/after`: FSM state transitions
- `variables`: Full snapshot of RecordBuffer as typed JSON values
- `event_type`: Classification for filtering

Defined `TraceEventType` enum with four event types:
- `LineProcessed`: Default for lines that matched but didn't change state or emit records
- `StateChange`: FSM transitioned to different state
- `RecordEmitted`: Record action executed
- `RecordCleared`: Clear action executed

### FSM Instrumentation (Task 2)

Instrumented the FSM execution loop in `Template::debug_parse`:
- Capture `state_before` at start of each line's processing
- After rule match and action execution, determine event type from state changes and actions
- Snapshot variables via new `RecordBuffer.current_values()` helper
- Push `TraceEvent` to `debug_report.trace`

Added `RecordBuffer.current_values()` method to convert raw string buffer to typed JSON snapshot matching the emit format.

Handled EOF record emission by recording trace event with `line_idx = lines.len()` and `event_type: RecordEmitted`.

### Test Coverage (Task 3)

Added five comprehensive tests:
1. `trace_records_line_by_line_events`: Verifies sequential line_idx progression
2. `trace_captures_state_transitions`: Validates StateChange event type and state_before/after
3. `trace_records_variable_snapshots`: Confirms variables HashMap contains captured fields
4. `trace_marks_record_emission`: Checks RecordEmitted events match EmittedRecord line_idx
5. `trace_handles_eof_record`: Validates EOF sentinel handling

## Deviations from Plan

None - plan executed exactly as written.

## Test Results

All tests pass (51 total, including 5 new trace tests):
```
running 51 tests
test result: ok. 51 passed; 0 failed; 0 ignored
```

Phase 4 debug_parse behavior unchanged (LineMatch and EmittedRecord still work as before).

## Files Modified

### src/engine/debug.rs
- Added `TraceEvent` and `TraceEventType` definitions
- Added `trace: Vec<TraceEvent>` field to DebugReport
- Updated `DebugReport::new()` to initialize trace as empty Vec

### src/engine/fsm.rs
- Added trace event recording in FSM loop after each rule match
- Implemented event type determination logic
- Added EOF trace event recording
- Added 5 trace event tests

### src/engine/records.rs
- Added `current_values()` method to snapshot buffer as typed JSON

## Architecture Notes

**Variable Snapshots:** Full snapshots per event (not deltas) for simplicity. Profile with real-world templates before optimizing.

**Event Type Precedence:** RecordEmitted > RecordCleared > StateChange > LineProcessed. This ordering matches semantic importance.

**Type Conversion:** Variables snapshot uses same `convert_scalar` logic as record emission, ensuring trace values match emitted records.

**EOF Handling:** Consistent with Phase 4 decision - EOF records use `line_idx = lines.len()` as explicit sentinel rather than special marker.

## Integration Points

**Phase 4 Compatibility:** Trace events stack with existing LineMatch/EmittedRecord infrastructure. All Phase 4 tests still pass.

**Future Phase 5 Plans:** Trace data provides foundation for:
- 05-02: Time-travel stepping (line-by-line, state-by-state, action-by-action)
- 05-03: Variable change inspection over time
- 05-04: FSM decision flow visualization

## Commits

| Task | Commit  | Description                                    |
| ---- | ------- | ---------------------------------------------- |
| 1    | 6e43dde | Define TraceEvent model in debug.rs            |
| 2    | 50cc542 | Instrument FSM loop to record trace events     |
| 3    | 242997e | Add trace event tests                          |

## Verification

Build: Clean compilation with no new warnings
Tests: All 51 tests pass (46 existing + 5 new trace tests)
Coverage: Trace event recording verified across line processing, state transitions, variable snapshots, Record/Clear actions, and EOF handling

## Self-Check: PASSED

All claimed files exist:
- FOUND: src/engine/debug.rs
- FOUND: src/engine/fsm.rs
- FOUND: src/engine/records.rs

All claimed commits exist:
- FOUND: 6e43dde (Task 1: TraceEvent model)
- FOUND: 50cc542 (Task 2: FSM instrumentation)
- FOUND: 242997e (Task 3: Trace tests)

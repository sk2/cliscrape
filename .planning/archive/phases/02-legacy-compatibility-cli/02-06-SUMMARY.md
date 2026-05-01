---
phase: 02-legacy-compatibility-cli
plan: 06
subsystem: engine/template-loader
tags: [textfsm-compatibility, action-semantics, regression-testing]
dependency_graph:
  requires: [02-05]
  provides: [textfsm-action-semantics, clear-clearall-distinction, error-action]
  affects: [template-parsing, record-management, fsm-execution]
tech_stack:
  added: []
  patterns: [fixture-backed-testing, action-lowering]
key_files:
  created:
    - tests/textfsm_compat.rs
    - tests/fixtures/textfsm/error_action.textfsm
    - tests/fixtures/textfsm/clear_vs_clearall.textfsm
  modified:
    - src/engine/types.rs
    - src/engine/records.rs
    - src/engine/fsm.rs
    - src/template/loader.rs
    - src/template/textfsm.pest
decisions:
  - "Action::ClearAll added as distinct from Action::Clear to match TextFSM semantics"
  - "Action::Error triggers immediate parsing abort with ScraperError::Parse"
  - "Clear preserves Filldown values; ClearAll clears everything including Filldown"
  - "Fixture-backed regression tests ensure action semantics remain stable"
metrics:
  duration: 98s
  tasks_completed: 2
  commits: 2
  files_modified: 5
  files_created: 3
  tests_added: 2
  completed: 2026-02-22T06:46:06Z
---

# Phase 02 Plan 06: TextFSM Action Semantics (Clear/Clearall/Error) Summary

**One-liner:** Implemented distinct Clear vs Clearall behavior and fail-fast Error action with fixture-backed regression tests to align runtime behavior with ntc-templates expectations.

## Objective Achievement

Successfully closed TextFSM action-semantics gaps by implementing distinct `Clear` vs `Clearall` behavior and supporting `-> Error` as a fail-fast action. All runtime action behavior now matches TextFSM/ntc-templates expectations, with regression tests locking down semantics.

## Tasks Completed

### Task 1: Add ClearAll + Error actions end-to-end (grammar -> loader -> runtime)
**Status:** ✓ Complete
**Commit:** c2d252d

**Implementation:**
- Extended `src/engine/types.rs` with `Action::ClearAll` and `Action::Error` variants
- Updated `src/template/textfsm.pest` grammar to include `Error` and `Clearall` tokens in record_action rule
- Updated `src/template/loader.rs` to map:
  - `Clear` -> `Action::Clear`
  - `Clearall` -> `Action::ClearAll`
  - `Error` -> `Action::Error`
- Updated `src/engine/records.rs` with `clear_non_filldown()` method that:
  - `Clear` clears only non-Filldown values (preserves Filldown state)
  - `ClearAll` clears everything including Filldown values
- Updated `src/engine/fsm.rs` so `Action::Error` aborts parsing with `ScraperError::Parse`

**Key Changes:**
- Grammar now parses all three action variants distinctly
- Loader correctly maps TextFSM action strings to internal Action enum
- Runtime executes Clear vs ClearAll semantics correctly
- Error action triggers fail-fast behavior with no partial output

**Files Modified:**
- src/engine/types.rs (added ClearAll and Error variants)
- src/engine/records.rs (added clear_non_filldown method)
- src/engine/fsm.rs (added Error action handling with early return)
- src/template/loader.rs (added Clearall and Error mapping)
- src/template/textfsm.pest (added Error to record_action rule)

**Verification:** `cargo test -q` passes; existing tests continue to work with extended action model.

### Task 2: Add fixture-backed regression tests for Clear vs Clearall and Error
**Status:** ✓ Complete
**Commit:** 5b39676

**Implementation:**
- Created `tests/fixtures/textfsm/error_action.textfsm`:
  - Template with `-> Error` action triggered by "BOOM" line
  - Test asserts parsing returns `Err(...)` and no rows are produced
  - Verifies fail-fast behavior with no partial output

- Created `tests/fixtures/textfsm/clear_vs_clearall.textfsm`:
  - Uses `Value Filldown CHASSIS` and non-filldown `Value SLOT`
  - Triggers `-> Clear` and verifies Filldown persists across subsequent `Record`
  - Triggers `-> Clearall` and verifies Filldown does NOT persist
  - Demonstrates Clear preserves R1 chassis across records, ClearAll clears it

- Created `tests/textfsm_compat.rs` with two deterministic library-level tests:
  - `error_action_aborts_and_discards_rows()`: Verifies Error action fails fast
  - `clear_preserves_filldown_and_clearall_clears_it()`: Verifies Clear vs Clearall semantics

**Test Results:**
```
running 2 tests
test clear_preserves_filldown_and_clearall_clears_it ... ok
test error_action_aborts_and_discards_rows ... ok
```

**Files Created:**
- tests/textfsm_compat.rs (2 regression tests)
- tests/fixtures/textfsm/error_action.textfsm (Error action fixture)
- tests/fixtures/textfsm/clear_vs_clearall.textfsm (Clear vs Clearall fixture)

**Verification:** Tests are deterministic, use pure library parse API (no stdout/stderr), and fail against pre-gap implementation.

## Deviations from Plan

None - plan executed exactly as written. Both tasks completed successfully with clear separation between implementation (Task 1) and regression testing (Task 2).

## Success Criteria

- [x] Verifier gaps for TextFSM actions (Clear vs Clearall, Error) are closed
- [x] Behavior is locked down via fixtures so future changes regress loudly
- [x] `cargo test -q` passes
- [x] `tests/textfsm_compat.rs` contains explicit assertions for Clear/Clearall and Error semantics

## Verification Results

1. `cargo test -q` passes ✓
2. `tests/textfsm_compat.rs` contains explicit assertions for Clear/Clearall and Error semantics ✓
3. Error action triggers fail-fast with no partial output ✓
4. Clear preserves Filldown values across records ✓
5. ClearAll clears everything including Filldown values ✓

## Impact

**Compatibility:** Full TextFSM action semantics now supported, closing critical gap in `.textfsm` template compatibility.

**Reliability:** Fixture-backed regression tests ensure action semantics remain stable across future refactoring.

**Developer Experience:** Clear error messages when Error action is triggered; predictable Clear/ClearAll behavior matches ntc-templates expectations.

## Technical Notes

**Action Execution Order:**
1. Line action (Continue/Next)
2. Record action (Record/Clear/ClearAll/Error)
3. State transition

**Clear Semantics:**
- `Action::Clear` → `RecordBuffer::clear_non_filldown()` → preserves Filldown values
- Implemented via `reset_after_emit()` logic that retains Filldown values in next buffer

**ClearAll Semantics:**
- `Action::ClearAll` → `RecordBuffer::clear_all()` → clears entire buffer
- Wipes all state including Filldown values

**Error Semantics:**
- `Action::Error` → immediate `return Err(ScraperError::Parse(...))`
- No partial records emitted; parsing stops on first Error action
- Error message includes line number for debugging

**Grammar Ordering:**
The pest grammar uses atomic matching with longest-first priority, so `Clearall` must be listed before `Clear` in the record_action rule to avoid mis-parsing "Clearall" as "Clear" + garbage.

## Self-Check: PASSED

**Created files verified:**
```
FOUND: tests/textfsm_compat.rs
FOUND: tests/fixtures/textfsm/error_action.textfsm
FOUND: tests/fixtures/textfsm/clear_vs_clearall.textfsm
```

**Commits verified:**
```
FOUND: c2d252d (Task 1: feat commit)
FOUND: 5b39676 (Task 2: test commit)
```

**Test execution verified:**
```
cargo test --test textfsm_compat
running 2 tests
test clear_preserves_filldown_and_clearall_clears_it ... ok
test error_action_aborts_and_discards_rows ... ok
```

All artifacts exist and are functional.

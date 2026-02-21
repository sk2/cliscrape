---
phase: 05-tui-advanced-debugging-state-tracer
plan: 02
subsystem: tui/trace-navigation
tags: [tui, debugger, time-travel, navigation, state-tracking]

dependency_graph:
  requires: [05-01]
  provides: [trace-navigation-api, stepping-modes, filter-state, watch-list]
  affects: [tui/app, tui/trace]

tech_stack:
  added: []
  patterns:
    - "Mode-aware trace navigation (LineByLine, StateByState, ActionByAction)"
    - "Jump shortcuts for common navigation patterns (next/prev record, jump to line)"
    - "Watch list for pinning variables"
    - "Filter state for event type visibility control"

key_files:
  created:
    - path: src/tui/trace.rs
      lines: 40
      purpose: "Trace navigation types (SteppingMode, FilterState)"
  modified:
    - path: src/tui/app.rs
      changes: "Added trace navigation state and methods"
    - path: src/tui/mod.rs
      changes: "Exported trace module"

decisions:
  - what: "Default stepping mode to LineByLine"
    why: "Most granular view gives users full visibility; they can switch to coarser modes if needed"
    alternatives: ["StateByState as default", "ActionByAction as default"]
  - what: "Default filter state shows all event types"
    why: "Visibility-first approach; users can hide events if needed"
  - what: "Use full iteration for find_prev_event instead of reverse indexing"
    why: "Simpler, clearer logic; performance is fine for typical trace sizes"

metrics:
  tasks_completed: 3
  commits: 3
  duration_seconds: 151
  files_created: 1
  files_modified: 2
  completed_at: "2026-02-21T03:39:05Z"

requirements:
  fulfilled: [TUI-02, TUI-03]
---

# Phase 05 Plan 02: Trace Navigation State and Logic Summary

Trace navigation infrastructure with stepping modes, filtering, and jump shortcuts for time-travel debugging.

## Overview

Added complete trace navigation API to AppState, enabling users to step through FSM execution at different granularities (line-by-line, state-by-state, action-by-action) and jump to specific events. This provides the foundation for time-travel debugging UI in subsequent plans.

## Tasks Completed

### Task 1: Create trace.rs module with stepping and filtering types
**Status:** ✅ Complete
**Commit:** 011a45d

Created new `src/tui/trace.rs` module with:
- `SteppingMode` enum with three navigation granularities (LineByLine, StateByState, ActionByAction)
- `FilterState` struct with event type visibility flags (line events, state changes, record actions, clear actions)
- `FilterState::matches()` helper for filtering logic
- Default filter state shows all event types (visibility-first approach)

**Verification:** `cargo build` succeeded

### Task 2: Add trace navigation state to AppState
**Status:** ✅ Complete
**Commit:** 25ce1b3

Extended AppState with trace navigation fields:
- `trace_index: usize` - Current position in trace (primary navigation pointer)
- `stepping_mode: SteppingMode` - Current stepping granularity (default: LineByLine)
- `filter_state: FilterState` - Event type visibility controls (default: show all)
- `watch_list: HashSet<String>` - Pinned variable names

Added `sync_cursor_to_trace()` helper to keep cursor aligned with trace index after navigation.

**Verification:** `cargo build` succeeded; existing TUI code still compiles

### Task 3: Implement stepping, filtering, and jump methods
**Status:** ✅ Complete
**Commit:** 2b967f3

Implemented comprehensive trace navigation API:

**Stepping methods:**
- `step_forward()` / `step_backward()` - Navigate according to current stepping mode
- `find_next_event()` / `find_prev_event()` - Mode-aware event finding
  - LineByLine: Every trace event
  - StateByState: Only state transitions (state_before != state_after)
  - ActionByAction: Only Record/Clear actions

**Jump methods:**
- `jump_to_next_record()` / `jump_to_previous_record()` - Navigate to record emission events
- `jump_to_line(line_idx)` - Jump to first trace event at specific line

**Filter/mode toggles:**
- `toggle_stepping_mode()` - Cycle through three stepping modes
- `toggle_filter_line_events()` / `toggle_filter_state_changes()` / `toggle_filter_record_actions()` / `toggle_filter_clear_actions()` - Toggle event type visibility
- `toggle_watch(var_name)` - Add/remove variables from watch list

All navigation methods call `sync_cursor_to_trace()` to maintain cursor alignment.

**Verification:** `cargo build` and `cargo clippy` passed

## Deviations from Plan

None - plan executed exactly as written.

## Technical Implementation Notes

### Mode-Aware Navigation Logic

The stepping logic uses iterator-based searching with mode-specific predicates:

- **LineByLine:** Simple index increment/decrement (fastest)
- **StateByState:** Scan for events where `state_before != state_after`
- **ActionByAction:** Scan for `RecordEmitted` or `RecordCleared` event types

### Cursor Sync Pattern

Every navigation method follows the pattern:
1. Find target trace index
2. Update `self.trace_index`
3. Call `sync_cursor_to_trace()` to update cursor position

This ensures the input line cursor always reflects the current trace position.

### Watch List Design

The watch list uses `HashSet<String>` for O(1) toggle operations. Variables are identified by name (string). This supports the future UI feature where users can pin specific variables to always show at the top of the variable inspector.

## Success Criteria

- [x] AppState tracks trace_index, stepping_mode, filter_state, watch_list
- [x] step_forward/backward navigate according to stepping mode
- [x] jump_to_next_record/previous_record/line work correctly
- [x] toggle_stepping_mode cycles through three modes
- [x] Filter toggles update filter_state
- [x] sync_cursor_to_trace() keeps cursor aligned with trace index

## Next Steps

Plan 05-03 will add UI panels to render the trace timeline, variable inspector with watch list support, and state transition view. Plan 05-04 will wire up keybindings to these navigation methods.

## Self-Check

Verifying all claimed files and commits exist:

```bash
# Check created files
[ -f "src/tui/trace.rs" ] && echo "FOUND: src/tui/trace.rs" || echo "MISSING: src/tui/trace.rs"

# Check commits
git log --oneline --all | grep -q "011a45d" && echo "FOUND: 011a45d" || echo "MISSING: 011a45d"
git log --oneline --all | grep -q "25ce1b3" && echo "FOUND: 25ce1b3" || echo "MISSING: 25ce1b3"
git log --oneline --all | grep -q "2b967f3" && echo "FOUND: 2b967f3" || echo "MISSING: 2b967f3"
```

**Result:**
```
FOUND: src/tui/trace.rs
FOUND: 011a45d
FOUND: 25ce1b3
FOUND: 2b967f3
```

## Self-Check: PASSED

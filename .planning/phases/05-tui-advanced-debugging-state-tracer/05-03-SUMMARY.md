---
phase: 05-tui-advanced-debugging-state-tracer
plan: 03
subsystem: tui/state-tracer-ui
tags: [tui, debugger, timeline, variables, visualization]

dependency_graph:
  requires: [05-02]
  provides: [timeline-pane, variables-pane, state-tracer-view-mode]
  affects: [tui/ui, tui/app]

tech_stack:
  added: []
  patterns:
    - "Timeline pane showing FSM state transitions with line numbers"
    - "Variables pane with change highlighting and watch indicators"
    - "ViewMode::StateTracer for time-travel debugging UI"
    - "Tab cycling through three view modes (Matches/Records/StateTracer)"

key_files:
  created: []
  modified:
    - path: src/tui/ui.rs
      changes: "Added render_timeline_pane, render_variables_pane, integrated StateTracer mode into layout"
    - path: src/tui/app.rs
      changes: "Added ViewMode::StateTracer, reset trace_index on parse, map navigation to trace stepping"

decisions:
  - what: "Use vertical split for timeline (top) + variables (bottom) in right pane"
    why: "Matches existing layout pattern; timeline benefits from vertical scrolling space"
    alternatives: ["Three-column layout", "Separate tab for each pane"]
  - what: "Map [ ] navigation keys to trace stepping in StateTracer mode"
    why: "Reuses existing navigation pattern; intuitive for users already familiar with match/record navigation"
  - what: "Display line numbers with @L notation (e.g., 'HEADER @L15')"
    why: "User decision from 05-CONTEXT.md; helps correlate FSM decisions with input lines"
  - what: "Sort watched variables first in variables pane"
    why: "Pin important variables at top; user decision from 05-CONTEXT.md watch feature"
  - what: "Reset trace_index to 0 on new parse"
    why: "Start at beginning of new trace; prevents index-out-of-bounds errors"

metrics:
  tasks_completed: 3
  commits: 3
  duration_seconds: 251
  files_created: 0
  files_modified: 2
  completed_at: "2026-02-21T04:52:57Z"

requirements:
  fulfilled: [TUI-02, TUI-03]
---

# Phase 05 Plan 03: Timeline & Variable Inspection UI Summary

Timeline visualization and variable inspection panes for time-travel debugging with change highlighting and watch list support.

## Overview

Built the visual UI components for the State Tracer: a timeline pane showing FSM state transitions over time, and a variables pane displaying current variable state with change highlighting. Users can now switch to StateTracer mode via Tab key and see the execution trace visually with line numbers, state transitions, and variable evolution.

## Tasks Completed

### Task 1: Add timeline pane rendering with state sequence display
**Status:** ✅ Complete
**Commit:** bd89efd

Added `render_timeline_pane` function to ui.rs:
- Displays trace events as a scrollable list with state transitions
- Shows line numbers with @L notation (e.g., "Start @L1", "Header -> Body @L15")
- Highlights current trace event with background color and bold styling
- Applies FilterState to hide/show event types based on user preferences
- Shows stepping mode in title bar (LineByLine/StateByState/ActionByAction)

**Verification:** `cargo build` succeeded

### Task 2: Add variable inspection pane with change highlighting
**Status:** ✅ Complete
**Commit:** c27f066

Added `render_variables_pane` function to ui.rs:
- Displays current variable state at trace_index position
- Highlights changed variables (vs previous trace event) in yellow with bold
- Shows old->new value transitions for changed variables
- Marks watched variables with star indicator (★)
- Sorts watched variables first, then alphabetical
- Shows line number in pane title (e.g., "Variables @L15")

**Verification:** `cargo build` succeeded

### Task 3: Wire timeline/variables panes into layout and add helper text
**Status:** ✅ Complete
**Commit:** 0e14613

Integrated timeline and variables panes into TUI layout:
- Added `ViewMode::StateTracer` enum variant
- Extended `toggle_view_mode()` to cycle: Matches -> Records -> StateTracer
- Modified main UI layout to show timeline (top) + variables (bottom) when in StateTracer mode
- Reset `trace_index` to 0 on new parse (prevents out-of-bounds errors)
- Mapped [ ] navigation keys to `step_backward()`/`step_forward()` in StateTracer mode
- Added help text in status pane for StateTracer keybindings
- Handled ViewMode::StateTracer in all match statements (sync_selections, render functions)

**Verification:** `cargo build` succeeded; UI layout cleanly integrates new panes

## Deviations from Plan

None - plan executed exactly as written.

## Technical Implementation Notes

### Timeline Pane Rendering

The timeline uses ratatui's `List` widget with custom styling:
- Filters trace events based on `FilterState` before rendering
- Current event gets `bg(Color::Rgb(50, 50, 50))` + `fg(Color::White)` + `Modifier::BOLD`
- State display shows both before/after states for transitions: "Start -> Header @L5"
- Event type displayed as debug format: `LineProcessed`, `RecordEmitted`, etc.

### Variables Pane Change Detection

Change highlighting compares current trace event variables with previous event:
- New variables: highlighted yellow with "(new)" annotation
- Changed variables: highlighted yellow with "old -> new" value display
- Unchanged variables: normal styling
- Watch indicator (★) appears before watched variable names

### ViewMode Integration

StateTracer mode is now a first-class view mode alongside Matches and Records:
- Tab key cycles through all three modes
- Each mode has its own right-pane layout (timeline+variables vs matches+details vs records+details)
- Navigation keys adapt to current mode: [ ] steps through trace in StateTracer mode

### Layout Decision

Chose vertical split (timeline top, variables bottom) for right pane because:
- Timeline can have hundreds of events → needs vertical scrolling space
- Variables pane typically shows 5-20 variables → less vertical space needed
- Matches existing two-row layout pattern from Matches/Records modes

## Success Criteria

- [x] Timeline pane renders state sequence with @L line numbers
- [x] Current trace event highlighted with background color
- [x] Variable pane shows current variable values at trace_index
- [x] Changed variables highlighted with yellow color + old->new display
- [x] Watched variables marked with star and sorted first
- [x] Timeline and variable panes integrated into TUI layout
- [x] Help text documents State Tracer keybindings

## Next Steps

Plan 05-04 will add keybindings to wire up all the navigation methods (PgUp/PgDn for stepping, Ctrl+N/P for jump to record, m for mode toggle, f1-f4 for filters, w for watch). The UI components are ready; now we just need to connect the keyboard inputs to the existing navigation API.

## Self-Check

Verifying all claimed files and commits exist:

```bash
# Check modified files
[ -f "src/tui/ui.rs" ] && echo "FOUND: src/tui/ui.rs" || echo "MISSING: src/tui/ui.rs"
[ -f "src/tui/app.rs" ] && echo "FOUND: src/tui/app.rs" || echo "MISSING: src/tui/app.rs"

# Check commits
git log --oneline --all | grep -q "bd89efd" && echo "FOUND: bd89efd" || echo "MISSING: bd89efd"
git log --oneline --all | grep -q "c27f066" && echo "FOUND: c27f066" || echo "MISSING: c27f066"
git log --oneline --all | grep -q "0e14613" && echo "FOUND: 0e14613" || echo "MISSING: 0e14613"
```

**Result:**
```
FOUND: src/tui/ui.rs
FOUND: src/tui/app.rs
FOUND: bd89efd
FOUND: c27f066
FOUND: 0e14613
```

## Self-Check: PASSED

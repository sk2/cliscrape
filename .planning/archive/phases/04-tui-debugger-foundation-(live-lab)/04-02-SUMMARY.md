---
phase: 04-tui-debugger-foundation-(live-lab)
plan: 02
subsystem: ui
tags: [rust, ratatui, crossterm, tui, debugger]

# Dependency graph
requires:
  - phase: 04-tui-debugger-foundation-(live-lab)
    provides: Phase 04-01 baseline + debug report model from engine
provides:
  - cliscrape debug CLI accepts optional template/input paths
  - Ratatui+crossterm 3-pane TUI scaffold with line cursor navigation
  - Debug subcommand dispatch that loads template+input and renders per-line DebugReport
affects: [04-03, 04-04, 04-05, 05-tui-debugger-live-lab]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Immediate-mode Ratatui draw loop driven by key actions
    - AppState owns cursor_line_idx + optional DebugReport; UI is pure draw(frame, &AppState)

key-files:
  created: [src/tui/app.rs, src/tui/event.rs, src/tui/mod.rs, src/tui/ui.rs]
  modified: [Cargo.toml, src/cli.rs, src/main.rs]

key-decisions:
  - "Keep right panes in lockstep with cursor_line_idx by indexing DebugReport.matches_by_line[cursor]."

patterns-established:
  - "tui/ split: app state in app.rs, key mapping in event.rs, rendering in ui.rs"

# Metrics
duration: 5 min
completed: 2026-02-20
---

# Phase 04 Plan 02: TUI Debugger Scaffolding Summary

**3-pane Ratatui debugger UI wired to `cliscrape debug`, with line-cursor navigation driving per-line DebugReport rendering.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-20T23:28:30Z
- **Completed:** 2026-02-20T23:34:03Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments
- Added `cliscrape debug` flags for optional `--template/-t` and `--input/-i`
- Implemented Ratatui+crossterm terminal init/restore and a minimal action-driven event loop
- Rendered a 3-pane layout where the left cursor line drives matches/details panes in lockstep

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend CLI debug command to accept template + input paths** - `efeb9bb` (feat)
2. **Task 2: Add Ratatui app skeleton with message-driven event loop** - `aa87bf7` (feat)
3. **Task 3: Wire Commands::Debug to launch the TUI and render first debug parse** - `b685b24` (feat)

**Plan metadata:** (added after this summary)

## Files Created/Modified
- `src/cli.rs` - Adds optional debug args `--template/-t` and `--input/-i`
- `src/main.rs` - Dispatches `Commands::Debug` into `tui::run_debugger(template, input)`
- `src/tui/app.rs` - `AppState` + cursor selection model + DebugReport storage
- `src/tui/event.rs` - Key mapping (arrows + vim keys) into cursor/quit actions
- `src/tui/ui.rs` - 3-pane `draw(frame, &AppState)` implementation
- `src/tui/mod.rs` - Crossterm terminal setup/teardown + main loop + `run_debugger` loader
- `Cargo.toml` - Enables Ratatui `crossterm` backend feature

## Decisions Made
None - followed plan as specified.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added `CLISCRAPE_TUI_EXIT_AFTER_MS` as a smoke-run escape hatch**
- **Found during:** Task 3 (verification in non-TTY automation)
- **Issue:** Plan verification runs in a non-interactive environment; the TUI needs a deterministic way to exit for automated smoke checks
- **Fix:** Added `CLISCRAPE_TUI_EXIT_AFTER_MS` to exit the loop after N ms when set
- **Files modified:** `src/tui/mod.rs`
- **Verification:** `script -q /dev/null bash -lc 'CLISCRAPE_TUI_EXIT_AFTER_MS=250 cargo run -- debug --template ... --input ...'` exits cleanly
- **Committed in:** `aa87bf7`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Verification-only hook; normal interactive behavior unchanged.

## Issues Encountered
- Extending `Commands::Debug` required updating the match arm in `src/main.rs` to compile (resolved immediately)

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- TUI scaffolding and debug wiring are in place; ready to iterate on richer panes, highlighting, and live reload in the remaining Phase 04 plans.

---
*Phase: 04-tui-debugger-foundation-(live-lab)*
*Completed: 2026-02-20*

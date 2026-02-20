---
phase: 04-tui-debugger-foundation-(live-lab)
plan: 03
subsystem: ui
tags: [ratatui, crossterm, notify, notify-debouncer-mini, live-reload, threads]

# Dependency graph
requires:
  - phase: 04-tui-debugger-foundation-(live-lab)
    provides: "TUI scaffolding and `cliscrape debug` wiring (04-02)"
provides:
  - "Debounced filesystem watching for template/input changes"
  - "Background parse worker producing ParseDone/ParseError messages"
  - "Last-good retention with an in-TUI status/error panel"
affects: ["Phase 04 live lab UX", "inline editor", "match highlighting panes"]

# Tech tracking
tech-stack:
  added: [notify, notify-debouncer-mini]
  patterns:
    - "Immediate-mode Ratatui loop driven by a Message channel"
    - "Coalesced background work (drain requests -> parse latest)"

key-files:
  created:
    - src/tui/watch.rs
    - src/tui/worker.rs
  modified:
    - Cargo.toml
    - src/tui/mod.rs
    - src/tui/app.rs
    - src/tui/event.rs
    - src/tui/ui.rs

key-decisions:
  - "Use notify + notify-debouncer-mini watching parent dirs to handle replace-on-save"
  - "Keep last-good DebugReport rendered on ParseError; show new error separately"

patterns-established:
  - "Message::FsChanged triggers ParseRequest; worker emits ParseDone/ParseError"

# Metrics
duration: 5 min
completed: 2026-02-20
---

# Phase 4 Plan 3: Live Reload Summary

**Debounced file watching plus a background parse worker so template/input edits auto-reparse without freezing the TUI, while keeping last-good results visible on errors.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-21T10:08:22+10:30
- **Completed:** 2026-02-20T23:44:05Z
- **Tasks:** 3
- **Files modified:** 7

## Accomplishments

- Added `notify` + `notify-debouncer-mini` dependencies for cross-platform debounced watch
- Implemented `watch::start_watcher()` that watches parent dirs and emits `Message::FsChanged`
- Moved parsing into a coalescing worker thread and added a status/error panel with last-good retention

## Task Commits

Each task was committed atomically:

1. **Task 1: Add notify dependencies for debounced filesystem watching** - `ff498ba` (chore)
2. **Task 2: Implement debounced file watch producing FsChanged messages** - `5aa4e44` (feat)
3. **Task 3: Move parsing into a worker thread and add error panel + last-good retention** - `8f7bd7f` (feat)

**Plan metadata:** (see docs commit that adds this SUMMARY/STATE update)

## Files Created/Modified

- `Cargo.toml` - Adds `notify` + `notify-debouncer-mini`
- `src/tui/watch.rs` - Debounced watcher -> `Message::FsChanged`
- `src/tui/worker.rs` - Background parser -> `Message::ParseDone` / `Message::ParseError`
- `src/tui/mod.rs` - Message enum + wiring for watcher/worker and live reparse loop
- `src/tui/app.rs` - `last_good`, `current_error`, and `ParseStatus`
- `src/tui/event.rs` - Key handling mapped into `Message`
- `src/tui/ui.rs` - Status/error panel; renders `last_good` matches/details

## Decisions Made

- Used a single `std::sync::mpsc` message channel for watcher + worker results to keep the Ratatui loop simple and non-blocking.
- Coalesced parse requests in the worker (drain pending requests, parse only the latest) to avoid wasted work during save storms.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

- `cliscrape debug` cannot be smoke-run in this non-TTY execution environment (alternate screen/raw mode fails with OS error 6); verified via `cargo build` instead.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Live reload loop exists; ready to build richer UI affordances (inline editor, per-line capture highlighting, and deeper trace navigation).

---
*Phase: 04-tui-debugger-foundation-(live-lab)*
*Completed: 2026-02-20*

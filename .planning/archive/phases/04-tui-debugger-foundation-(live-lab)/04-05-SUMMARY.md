---
phase: 04-tui-debugger-foundation-(live-lab)
plan: 05
subsystem: ui
tags: [tui, ratatui, crossterm, editor, picker, live-lab]

requires:
  - phase: 04-tui-debugger-foundation-(live-lab)
    provides: "Live Lab visualization + records toggle (04-04)"
provides:
  - "Inline template editor mode with Ctrl+S save + immediate parse refresh"
  - "In-TUI picker startup flow for missing --template/--input paths"
affects: [phase-05, tui-ux, workflow]

tech-stack:
  added: []
  patterns:
    - "Mode-based TUI workflow: Picker -> Browse -> EditTemplate"
    - "Atomic-ish save: write temp file then rename"

key-files:
  created: [src/tui/editor.rs, src/tui/picker.rs]
  modified: [src/tui/app.rs, src/tui/ui.rs, src/tui/event.rs, src/tui/mod.rs]

key-decisions:
  - "Keybindings: e enters editor, Esc exits, Ctrl+S saves"
  - "Picker supports directory browsing + manual path entry (i)"

patterns-established:
  - "Editor buffer operates on Vec<String> with UTF-8 safe cursor movement and edit ops"

duration: 15 min
completed: 2026-02-21
---

# Phase 4 Plan 05: Picker + Inline Editor Summary

**`cliscrape debug` now supports an in-TUI file picker (when flags are missing) plus inline template edits with save-triggered reparse.**

## Performance

- **Duration:** 15 min
- **Started:** 2026-02-21T00:14:15Z
- **Completed:** 2026-02-21T00:29:01Z
- **Tasks:** 2 (+ 1 deferred human verification checkpoint)
- **Files modified:** 7

## Accomplishments

- Added a minimal inline template editor (insert/delete/newline, cursor movement, scroll) with atomic save and explicit parse refresh
- Added a startup picker flow so running `cliscrape debug` without paths can select template + input inside the TUI

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement minimal inline template editor mode** - `e84389e` (feat)
2. **Task 2: Add in-TUI picker for missing template/input paths** - `97e54ff` (feat)

**Plan metadata:** docs commit created after SUMMARY/STATE updates

## Files Created/Modified

- `src/tui/editor.rs` - Inline editor buffer + save-to-file logic
- `src/tui/picker.rs` - Minimal directory list picker with manual path entry
- `src/tui/app.rs` - Mode state machine + picker/editor integration
- `src/tui/ui.rs` - Picker and editor rendering integration
- `src/tui/event.rs` - Key handling for mode switching + editor/picker actions
- `src/tui/mod.rs` - Module wiring for editor/picker and message flow

## Decisions Made

- Chose a minimal bespoke editor (Vec<String>) rather than adding a textarea widget dependency for Ratatui 0.30 compatibility.
- Implemented save as write-temp + rename to better match editor save semantics and reduce partial-write reads by the watcher.

## Deviations from Plan

Human verification checkpoint was deferred per user instruction (treat as milestone-level verification).

### Deferred Human Verification (run later)

1. Run: `cargo run -- debug`.
2. Use the in-TUI picker to choose:
   - template: `templates/modern/ios_show_interfaces.yaml`
   - input: `examples/output.txt`
3. In Browse mode:
   - Navigate lines with both arrows and `j/k`.
   - Confirm matched lines are shaded and capture spans are highlighted for the selected match.
   - Toggle Records view (Tab) and confirm records list + details update.
4. Enter editor mode (`e`), change a regex so it matches fewer/more lines, save (`Ctrl+S`):
   - Confirm highlights update within ~300ms.
5. Introduce an invalid regex/template syntax, save:
   - Confirm last-good highlights remain.
   - Confirm error panel shows the new error.
6. Fix the error, save:
   - Confirm error clears and results update.
7. Quit (`q`) and confirm terminal is restored (no raw-mode issues).

## Issues Encountered

- `cargo build` warns about an unused field in `Message::FsChanged` (dead_code); build succeeds.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Phase 4 complete; ready for Phase 5 planning (deep step-through FSM tracing + timeline).
- Carryover verification: Phase 3 interactive converter smoke test still pending.

---
*Phase: 04-tui-debugger-foundation-(live-lab)*
*Completed: 2026-02-21*

---
phase: 04-tui-debugger-foundation-(live-lab)
plan: 01
subsystem: ui
tags: [rust, fsm, debug-trace, serde, serde_json, regex]

requires:
  - phase: 01-core-parsing-engine
    provides: FSM parse loop + record buffer semantics
  - phase: 03-modern-ergonomic-templates
    provides: typed capture conversion via convert_scalar + TemplateIR lowering
provides:
  - DebugReport trace model for per-line match inspection (captures + records)
  - Template::debug_parse built from the real FSM parse loop
  - FsmParser::debug_parse public wrapper for TUI/debugger consumers
affects: [04-tui-debugger-foundation-(live-lab), 05-tui-advanced-debugging]

tech-stack:
  added: []
  patterns:
    - Single FSM parse loop with optional debug instrumentation

key-files:
  created:
    - src/engine/debug.rs
  modified:
    - src/engine/mod.rs
    - src/engine/fsm.rs
    - src/lib.rs
    - tests/convert_cli_defaults.rs

key-decisions:
  - "Store debug actions as strings so DebugReport stays serde-serializable without changing engine Action types"
  - "Represent EOF-emitted records with line_idx = lines.len() sentinel"

patterns-established:
  - "Debug traces are produced by instrumenting the core FSM loop (no duplicate parse logic)"

duration: 14 min
completed: 2026-02-20
---

# Phase 04 Plan 01: Engine Debug Parse Report Summary

**UI-friendly per-line FSM match trace (Continue stacking) with capture byte spans and emitted record attribution.**

## Performance

- **Duration:** 14 min
- **Started:** 2026-02-20T23:09:27Z
- **Completed:** 2026-02-20T23:23:36Z
- **Tasks:** 3
- **Files modified:** 5

## Accomplishments

- Added a serializable `DebugReport` model with per-line `LineMatch` events, `CaptureSpan` byte ranges, and `EmittedRecord` snapshots.
- Implemented `Template::debug_parse` by instrumenting the real FSM parse loop (preserves Continue/state/record semantics).
- Exposed `FsmParser::debug_parse` and added regression tests for Continue stacking, spans, and record emission parity with `parse()`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Define DebugReport trace model for TUI consumption** - `c36302e` (feat)
2. **Task 2: Implement Template::debug_parse using the real FSM loop** - `9c63fa3` (feat)
3. **Task 3: Expose debug parsing through FsmParser and add regression tests** - `077946d` (feat)

**Plan metadata:** (docs commit created after SUMMARY/STATE updates)

## Files Created/Modified

- `src/engine/debug.rs` - DebugReport/LineMatch/CaptureSpan/EmittedRecord trace model
- `src/engine/fsm.rs` - Shared parse loop + Template::debug_parse instrumentation + regression tests
- `src/engine/mod.rs` - Expose `engine::debug` module
- `src/lib.rs` - `FsmParser::debug_parse` wrapper + debug type re-exports
- `tests/convert_cli_defaults.rs` - Make CLI conversion test robust to existing output file

## Decisions Made

- Kept debug model serde-friendly by storing actions as strings rather than requiring `Action: Serialize`.
- Marked EOF record emission with `line_idx = lines.len()` so UIs can treat it as an explicit sentinel.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed flaky convert CLI test when output file already exists**

- **Found during:** Task 2 (Implement Template::debug_parse using the real FSM loop)
- **Issue:** `tests/convert_cli_defaults.rs` failed if `target/tmp_converted.yaml` existed from a prior run
- **Fix:** Delete the output path before running the conversion command
- **Files modified:** `tests/convert_cli_defaults.rs`
- **Verification:** `cargo test`
- **Committed in:** `9c63fa3`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required to keep `cargo test` reliable; no scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `.planning/phases/04-tui-debugger-foundation-(live-lab)/04-02-PLAN.md` (TUI scaffolding + `cliscrape debug` wiring).

---
*Phase: 04-tui-debugger-foundation-(live-lab)*
*Completed: 2026-02-20*

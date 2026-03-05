---
phase: 10-production-logging
plan: "02"
subsystem: infra
tags: [tracing, rust, rust_log, json-logs, observability]

# Dependency graph
requires:
  - phase: 10-production-logging/10-01
    provides: tracing subscriber init + `-v/--log-format` CLI controls
provides:
  - Command-level spans and lifecycle events for parse/list/show
  - Template resolver events (embedded vs user path)
  - Engine parse summary events at parse boundaries (counts + duration)
affects: [10-production-logging/10-03, production-debugging]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Target-filterable `tracing` events routed to stderr (stdout reserved for command output)
    - Boundary-only engine instrumentation (no hot-loop tracing)

key-files:
  created: []
  modified:
    - src/main.rs
    - src/template/resolver.rs
    - src/engine/fsm.rs
    - src/output.rs

key-decisions:
  - "Emit engine parse summaries at debug level under target cliscrape::engine to avoid default noise"

patterns-established:
  - "Command spans use stable event ids (event=...) for downstream JSON log parsing"

# Metrics
duration: 13 min
completed: 2026-03-05
---

# Phase 10 Plan 02: Command + Library Boundary Events Summary

**High-level structured tracing events for template resolution + parse lifecycle, with RUST_LOG target filtering and stdout preserved for command output.**

## Performance

- **Duration:** 13 min
- **Started:** 2026-03-05T03:17:40Z
- **Completed:** 2026-03-05T03:31:16Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Added command-level spans + lifecycle events for `parse`, `list-templates`, and `show-template` while keeping stdout outputs unchanged.
- Routed loader/transcript/parse warnings through `tracing::warn!` (stderr subscriber) instead of ad-hoc `eprintln!`.
- Instrumented library boundaries: template resolver emits `template_resolved` (embedded vs user) and engine emits `parse_summary` only at parse boundaries.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add command-level spans and structured events in the CLI** - `b1edbfd` (feat)
2. **Task 2: Add library-side high-level events (no hot-loop logging)** - `6408ee4` (feat)

**Plan metadata:** (docs commit added after execution)

## Files Created/Modified

- `src/main.rs` - command spans/events; warnings routed through tracing; embedded template id resolution.
- `src/template/resolver.rs` - template resolution events with source kind and optional path.
- `src/engine/fsm.rs` - parse boundary summary event (line/record/warning counts + elapsed).
- `src/output.rs` - JSON output stays valid for empty result sets.

## Decisions Made

- Emit engine parse summaries as `debug` under `cliscrape::engine` so default `warn` output stays quiet while `RUST_LOG` can enable them.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Allow embedded templates to resolve by identifier without extension**

- **Found during:** Task 1 verification
- **Issue:** `--template cisco_ios_show_version` failed to resolve embedded templates unless the extension was included.
- **Fix:** Try embedded/XDG resolution using the same extension set as CWD identifier search; error on ambiguity.
- **Files modified:** `src/main.rs`
- **Verification:** Plan verification command resolves and runs successfully.
- **Committed in:** `b1edbfd`

**2. [Rule 1 - Bug] JSON output was invalid for empty result sets**

- **Found during:** Task 1 verification
- **Issue:** `--format json` produced an empty string when there were 0 records, breaking JSON pipelines.
- **Fix:** Emit `[]` for empty results in JSON/auto modes.
- **Files modified:** `src/output.rs`
- **Verification:** `python3 -m json.tool /tmp/out.json` succeeds for empty-input runs.
- **Committed in:** `b1edbfd`

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes were required for correctness and for the plan's verification steps; no scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Instrumentation is in place and filterable; ready for `10-03-PLAN.md` (bench overhead verification).

---
*Phase: 10-production-logging*
*Completed: 2026-03-05*

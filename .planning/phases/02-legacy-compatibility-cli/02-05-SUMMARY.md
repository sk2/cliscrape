---
phase: 02-legacy-compatibility-cli
plan: 05
subsystem: cli
tags: [clap, assert_cmd, rust]

# Dependency graph
requires:
  - phase: 02-legacy-compatibility-cli
    provides: Existing `cliscrape` binary with `parse` subcommand
provides:
  - Phase-2 `cliscrape parse` clap surface (multi-input flags, format=auto default, error-format, quiet)
  - Help regression test locking `parse --help` contract
affects: [phase-02, cli-surface, error-format, parse-io]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Help-output regression tests for CLI contract stability

key-files:
  created:
    - tests/cli_parse_help.rs
  modified:
    - src/cli.rs
    - src/main.rs
    - src/output.rs

key-decisions:
  - "Expose `OutputFormat::Auto` in clap and default `--format` to auto; current serializer maps auto -> JSON until tty-aware selection is implemented."

patterns-established:
  - "CLI contract locked via `cliscrape parse --help` assertions"

# Metrics
duration: 5 min
completed: 2026-02-21
---

# Phase 02 Plan 05: Parse Clap Contract Summary

**Phase-2 `cliscrape parse` clap contract: multi-input flags, `--format=auto` default, global `--error-format`, and `--quiet`, backed by a help regression test.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-21T22:06:05Z
- **Completed:** 2026-02-21T22:11:53Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments
- Updated `cliscrape parse` clap surface to match Phase-2 contract (inputs + format/error/status flags)
- Added `OutputFormat::Auto` and defaulted `--format/-f` to `auto`
- Locked the contract with a `parse --help` regression test

## Task Commits

Each task was committed atomically:

1. **Task 1: Add Phase-2 `parse` flags + defaults to clap** - `17304f6` (feat)
2. **Task 2: Add `parse --help` regression test for the Phase-2 contract** - `da75fc9` (test)

**Plan metadata:** (docs commit after SUMMARY/STATE update)

## Files Created/Modified
- `src/cli.rs` - Phase-2 `parse` flags/defaults; adds global `--error-format`
- `src/main.rs` - Compiles against new parse args; implements basic multi-input reading and guards `--input-glob` as unimplemented
- `src/output.rs` - Handles `OutputFormat::Auto` (currently treated as JSON)
- `tests/cli_parse_help.rs` - Regression test asserting required flags/defaults appear in `parse --help`

## Decisions Made
None - followed plan as specified (with minimal wiring changes to keep the binary compiling).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated runtime wiring to compile with new clap surface**
- **Found during:** Task 1 (Add Phase-2 `parse` flags + defaults to clap)
- **Issue:** `src/main.rs` and `src/output.rs` referenced the old parse args and lacked `OutputFormat::Auto`, causing compilation failures.
- **Fix:** Updated parse argument destructuring + input handling, and added `OutputFormat::Auto` handling in serializer.
- **Files modified:** `src/main.rs`, `src/output.rs`
- **Verification:** `cargo test -q`
- **Committed in:** `17304f6` (part of Task 1 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to keep the crate compiling while exposing the Phase-2 CLI contract.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- `cliscrape parse` help output now documents the Phase-2 flag surface; ready to implement the actual Phase-2 behavior in `src/main.rs` (template id resolution, glob expansion, tty-aware auto format, structured error output).

---
*Phase: 02-legacy-compatibility-cli*
*Completed: 2026-02-21*

---
phase: 03-modern-ergonomic-templates
plan: 06
subsystem: testing
tags: [rust, regex, macros, yaml, toml, serde_path_to_error]

# Dependency graph
requires:
  - phase: 03-modern-ergonomic-templates
    provides: Modern template schema loader + TemplateIR lowering (03-03)
provides:
  - Focused modern schema invariant/wiring tests (states xor patterns, capture styles, macro precedence, path errors)
  - Recursive, deterministic macro expansion with cycle detection and depth limit
affects: [modern-templates, template-loader, macro-expansion, regex-compilation]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - Token-order macro expansion with memoization, cycle detection, and max depth

key-files:
  created: []
  modified:
    - src/template/modern.rs
    - src/engine/macros.rs
    - src/engine/fsm.rs

key-decisions:
  - "Macro expansion is recursive with MAX_DEPTH=10; cycles error with an explicit chain"
  - "Unknown macros remain unexpanded (preserve prior behavior); errors are surfaced at Template compilation with state context"

patterns-established:
  - "Schema loader tests assert serde_path_to_error paths for both TOML and YAML"

# Metrics
duration: 8 min
completed: 2026-02-20
---

# Phase 3 Plan 6: Modern Wiring + Recursive Macros Summary

**Recursive macro expansion (cycle-safe) plus targeted modern schema wiring/invariant tests to de-risk the modern template layer.**

## Performance

- **Duration:** 8 min
- **Started:** 2026-02-20T11:48:12Z
- **Completed:** 2026-02-20T11:57:06Z
- **Tasks:** 2
- **Files modified:** 3

## Accomplishments
- Added a YAML path-aware schema error test to ensure `serde_path_to_error` paths are preserved across both modern formats.
- Implemented recursive macro expansion with memoization, deterministic token-order replacement, depth limiting, and cycle detection.
- Propagated macro expansion failures through Template compilation with state context for actionable errors.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add targeted tests for modern schema invariants + wiring** - `f4991b2` (test)
2. **Task 2: Upgrade macro expansion to recursive with cycle detection** - `784e4d0` (feat)

**Plan metadata:** [pending] (docs: complete plan)

## Files Created/Modified
- `src/template/modern.rs` - Adds a YAML schema error-path assertion to keep loader diagnostics precise.
- `src/engine/macros.rs` - Replaces single-pass HashMap iteration with recursive, deterministic expansion + cycle errors.
- `src/engine/fsm.rs` - Surfaces macro expansion failures during rule compilation with state context.

## Decisions Made
- Recursive macro semantics are enforced in the engine (depth-limited + cycle-safe) rather than constraining templates to non-nested macros.
- Modern schema parse errors must include field paths for both TOML and YAML inputs.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Modern loader invariants and macro expansion semantics are locked in with tests.
- Ready for `.planning/phases/03-modern-ergonomic-templates/03-04-PLAN.md`.

---
*Phase: 03-modern-ergonomic-templates*
*Completed: 2026-02-20*

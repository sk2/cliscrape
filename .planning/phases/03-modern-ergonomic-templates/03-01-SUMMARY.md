---
phase: 03-modern-ergonomic-templates
plan: 01
subsystem: engine
tags: [rust, serde_json, textfsm, type-conversion]

requires:
  - phase: 01-core-parsing-engine
    provides: FSM engine + RecordBuffer-based JSON emission
provides:
  - Value type hints (FieldType + Value.type_hint)
  - Emit-time scalar conversion to typed serde_json::Value (explicit-first + numeric heuristics)
  - Record emission converts scalars and lists without changing regex capture behavior
affects: [03-modern-ergonomic-templates, FORM-03]

tech-stack:
  added: []
  patterns:
    - "Emit-time conversion pipeline: string capture -> typed serde_json::Value"

key-files:
  created:
    - src/engine/convert.rs
    - tests/template_macro_expansion.rs
  modified:
    - src/engine/types.rs
    - src/engine/records.rs
    - src/engine/mod.rs
    - src/engine/fsm.rs
    - src/template/loader.rs

key-decisions:
  - "Explicit per-field type_hint wins; FieldType::String disables heuristics"
  - "On conversion failure, preserve the original captured value as a JSON string"

patterns-established:
  - "Conversion policy isolated in src/engine/convert.rs; never during regex capture"

duration: 7 min
completed: 2026-02-20
---

# Phase 03 Plan 01: Typed Record Emission Summary

**Record emission converts captured strings into typed JSON (ints) using per-field hints with a numeric-only heuristic fallback.**

## Performance

- **Duration:** 7 min
- **Started:** 2026-02-20T07:39:18Z
- **Completed:** 2026-02-20T07:46:18Z
- **Tasks:** 3
- **Files modified:** 8
- **Files modified:** 7

## Accomplishments
- Added `FieldType` + `Value.type_hint` so fields can opt into typed output without changing capture behavior
- Implemented `engine::convert::convert_scalar` (explicit-first, lenient int parsing, safe string fallback)
- Applied conversion at `RecordBuffer::emit` for both scalar and list fields; added unit tests proving typed + heuristic behaviors

## Task Commits

Each task was committed atomically:

1. **Task 1: Add field type metadata + conversion utility** - `5ada1c4` (feat)
2. **Task 2: Apply typed conversion at record emission** - `e3f0f75` (feat)
3. **Task 3: Add focused tests for typed and heuristic conversion** - `658f68d` (test)

## Files Created/Modified
- `src/engine/convert.rs` - String-to-typed `serde_json::Value` conversion (explicit hint + numeric heuristic)
- `src/engine/records.rs` - Apply conversion at emit-time for scalar and list fields; conversion tests
- `src/engine/types.rs` - Add `FieldType` and `Value.type_hint`
- `src/engine/mod.rs` - Export `convert` module
- `src/template/loader.rs` - Legacy loader initializes `type_hint: None`

## Decisions Made
- Explicit `type_hint` is authoritative; `FieldType::String` preserves raw strings and disables numeric heuristics.
- Conversion never drops records: when parsing fails, the original captured value is emitted as a JSON string.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated legacy Value initializers for new type_hint field**

- **Found during:** Task 1
- **Issue:** Existing loaders/tests constructing `Value { ... }` failed to compile after adding `Value.type_hint`
- **Fix:** Added `type_hint: None` where required
- **Files modified:** `src/template/loader.rs`, `src/engine/fsm.rs`, `tests/template_macro_expansion.rs`
- **Verification:** `cargo test`
- **Committed in:** `5ada1c4`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Required for compilation; no scope creep.

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
Ready for `03-02-PLAN.md` (IOS prompt/echo handling + transcript segmentation).

---
*Phase: 03-modern-ergonomic-templates*
*Completed: 2026-02-20*

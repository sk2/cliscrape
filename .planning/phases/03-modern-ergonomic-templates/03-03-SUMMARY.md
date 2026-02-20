---
phase: 03-modern-ergonomic-templates
plan: 03
subsystem: template
tags: [rust, serde, toml, yaml, serde_path_to_error, templateir]

requires:
  - phase: 03-modern-ergonomic-templates
    provides: Typed record emission with explicit-per-field type hints
provides:
  - Strict modern template schema for YAML/TOML lowered into TemplateIR
  - Extension-based loading of modern templates in FsmParser
affects: [03-modern-ergonomic-templates, templates, cli]

tech-stack:
  added: [toml, serde_yaml_ng, serde_path_to_error]
  patterns:
    - Schema-to-TemplateIR compiler with post-deserialize invariant validation
    - Strict serde schema via deny_unknown_fields + path-aware errors

key-files:
  created: [src/template/modern.rs]
  modified: [Cargo.toml, src/template/mod.rs, src/lib.rs]

key-decisions:
  - "Modern template fields default to explicit string typing (type_hint=String) unless a type is declared"
  - "Named capture groups must be declared in fields to be emitted/typed"

patterns-established:
  - "Modern loader validates invariants (version, states vs patterns) before lowering"
  - "Template-local macros are lowered into TemplateIR.macros and override builtins"

duration: 5 min
completed: 2026-02-20
---

# Phase 3 Plan 03: Modern YAML/TOML Schema Summary

**Strict YAML/TOML modern templates compiled into TemplateIR with explicit field typing, macro lowering, and extension-based loading via FsmParser.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-20T11:40:38Z
- **Completed:** 2026-02-20T11:46:21Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Implemented a strict modern template document schema (YAML/TOML) with path-aware validation errors and invariant checks.
- Lowered modern templates into engine `TemplateIR`, including template-local macros and explicit `Value.type_hint` typing.
- Added extension-based loading in `FsmParser::from_file` for `.yaml`/`.yml`/`.toml` alongside `.textfsm`.

## Task Commits

Each task was committed atomically:

1. **Task 1: Add strict modern YAML/TOML schema + loader** - `47c30d1` (feat)
2. **Task 2: Load modern templates by extension in FsmParser** - `c35854d` (feat)

## Files Created/Modified

- `src/template/modern.rs` - Modern schema structs, strict validation, and lowering into `TemplateIR` with typed field hints and local macros.
- `Cargo.toml` - Adds `toml`, `serde_yaml_ng`, and `serde_path_to_error` for modern template parsing and path-aware errors.
- `src/template/mod.rs` - Exposes `template::modern` module.
- `src/lib.rs` - Loads templates by extension (`.textfsm`/`.yaml`/`.yml`/`.toml`) and adds tests for modern loading.

## Decisions Made

- Default modern field typing to `string` (lowered as `Value.type_hint = Some(FieldType::String)`) unless explicitly declared, to avoid heuristic surprises.
- Require all named capture groups used in modern rules to be declared in `fields` so emission/typing is deterministic.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Aligned `FsmParser::parse` return type with typed engine output**

- **Found during:** Task 2 (extension-based loader + tests)
- **Issue:** `engine::Template::parse` returns typed `serde_json::Value`, but `FsmParser::parse` still declared `String` values, breaking compilation.
- **Fix:** Updated `FsmParser::parse` signature to return `serde_json::Value` records.
- **Files modified:** `src/lib.rs`
- **Verification:** `cargo test`
- **Committed in:** `c35854d`

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to compile and expose the already-implemented typed output; no scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `03-04-PLAN.md` (CLI format override + starter templates + e2e tests).

---
*Phase: 03-modern-ergonomic-templates*
*Completed: 2026-02-20*

---
phase: 03-modern-ergonomic-templates
plan: 04
subsystem: cli
tags: [rust, clap, assert_cmd, yaml, toml, textfsm]

requires:
  - phase: 03-modern-ergonomic-templates
    provides: Modern YAML/TOML template schema loader and extension-based inference
provides:
  - CLI `--template-format` override for explicit loader selection
  - In-repo starter modern templates (YAML state machine, TOML patterns)
  - End-to-end tests proving typed JSON output from modern templates
affects: [templates, cli, testing, docs]

tech-stack:
  added: [assert_cmd]
  patterns:
    - Library-level `TemplateFormat` override passed from CLI to template loader

key-files:
  created:
    - templates/modern/ios_show_interfaces.yaml
    - templates/modern/simple_hostname.toml
    - tests/modern_templates.rs
  modified:
    - src/cli.rs
    - src/main.rs
    - src/lib.rs
    - src/template/modern.rs
    - Cargo.toml

key-decisions:
  - "Keep clap parsing in the binary crate; expose a plain `TemplateFormat` in the library for reuse"

patterns-established:
  - "CLI integration tests parse JSON output rather than string-matching formatting"

duration: 5 min
completed: 2026-02-20
---

# Phase 3 Plan 4: Modern CLI UX + Starter Templates Summary

**`cliscrape parse` supports modern YAML/TOML templates with an explicit `--template-format` override, plus a small starter template pack covered by integration tests.**

## Performance

- **Duration:** 5 min
- **Started:** 2026-02-20T11:48:28Z
- **Completed:** 2026-02-20T11:54:23Z
- **Tasks:** 3
- **Files modified:** 8

## Accomplishments

- Added `--template-format auto|textfsm|yaml|toml` to force loader selection when extensions are ambiguous
- Shipped two small modern starter templates (YAML state-machine + typed int, TOML patterns-only)
- Added end-to-end tests validating template loading and typed JSON emission (including CLI override)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add optional CLI template format override** - `2b404db` (feat)
2. **Task 2: Add a small starter pack of modern templates** - `706cbb4` (feat)
3. **Task 3: Add end-to-end tests for modern templates + typed output** - `52fcac2` (test)

**Plan metadata:** TBD (docs: complete plan)

## Files Created/Modified

- templates/modern/ios_show_interfaces.yaml - Minimal YAML state-machine example with typed `int` capture
- templates/modern/simple_hostname.toml - Minimal TOML patterns-only example
- tests/modern_templates.rs - Integration tests for loader + typed JSON, and CLI override coverage
- src/cli.rs - `--template-format` flag wiring
- src/main.rs - Pass override through to the library loader
- src/lib.rs - `FsmParser::from_file_with_format` and `TemplateFormat`

## Decisions Made

- Keep clap-specific enums in `src/cli.rs` and map into a clap-free `cliscrape::TemplateFormat` so the library API stays reusable.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed a TOML literal-string regex escape in an existing modern template unit test**

- **Found during:** Task 1 verification (`cargo test`)
- **Issue:** Test used `\\S+` inside a TOML literal string, producing a pattern that matched a literal `\S` instead of `\S`-class whitespace negation
- **Fix:** Switched to a single backslash in the TOML literal string so the compiled regex matches as intended
- **Files modified:** src/template/modern.rs
- **Verification:** `cargo test`
- **Committed in:** 2b404db

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Required to keep the test suite green; no scope creep.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for `.planning/phases/03-modern-ergonomic-templates/03-05-PLAN.md`.

---
*Phase: 03-modern-ergonomic-templates*
*Completed: 2026-02-20*

---
phase: 02-legacy-compatibility-cli
plan: 07
subsystem: template-loader
tags: [textfsm, validation, safety, warnings, eof, comments]
dependencies:
  requires: [02-06]
  provides: [strict-validation, warning-api, eof-semantics, comment-support]
  affects: [template-loader, engine-fsm, lib-api]
tech_stack:
  added: [TemplateWarning]
  patterns: [warn-and-skip, strict-token-validation, reserved-state-handling]
key_files:
  created:
    - tests/fixtures/textfsm/explicit_eof_empty.textfsm
    - tests/fixtures/textfsm/explicit_eof_rules.textfsm
    - tests/fixtures/textfsm/warn_skip_constructs.textfsm
    - tests/fixtures/textfsm/comment_lines_ignored.textfsm
  modified:
    - src/lib.rs
    - src/engine/macros.rs
    - src/engine/fsm.rs
    - src/template/loader.rs
    - src/template/textfsm.pest
    - tests/textfsm_compat.rs
decisions:
  - Unknown macros now error at template load (not preserved verbatim)
  - Undefined placeholders detected after expansion via regex scan
  - EOF state: empty suppresses implicit record; with rules executes once
  - Grammar accepts any identifier as flag/action; validation in loader
  - Comments allowed at file and state-block levels (not inline within rules)
metrics:
  duration_seconds: 766
  tasks_completed: 4
  files_modified: 6
  files_created: 4
  commits: 4
  test_coverage: All new features tested
completed: 2026-02-21T23:01:33Z
---

# Phase 02 Plan 07: TextFSM Safety Gaps Summary

**One-liner:** Strict undefined token validation, EOF reserved state semantics, warning-returning loader API, and comment-line support close Phase-2 TextFSM safety gaps.

## Tasks Completed

### Task 1: Enforce strict undefined macro + placeholder validation ✓
**Commit:** 65b0c9c

- Modified `src/engine/macros.rs` to error on unknown `{{macro}}` expansion (previously preserved verbatim)
- Added validation in `src/engine/fsm.rs` to detect leftover `${...}` or `{{...}}` tokens after expansion
- Added tests verifying undefined `${MISSING}` placeholder errors at template load
- Added tests verifying undefined `{{missing_macro}}` errors at template load

**Verification:** `cargo test -q undefined_` passes; templates with undefined tokens fail load.

---

### Task 2: Implement reserved `EOF` state semantics ✓
**Commit:** 9ab229e

- Implemented EOF state handling in `src/engine/fsm.rs`:
  - Explicit EOF state with zero rules: suppresses implicit EOF record emission
  - Explicit EOF state with rules: executes those rules once at end-of-input (EOF as empty pseudo-line)
  - No EOF state defined: retains existing implicit EOF record behavior
- Updated grammar to allow empty state blocks (required for `EOF` with no rules)
- Created fixtures:
  - `explicit_eof_empty.textfsm`: demonstrates suppression
  - `explicit_eof_rules.textfsm`: demonstrates EOF rules execute once
- Added tests verifying EOF behavior matches TextFSM

**Verification:** `cargo test -q explicit_eof` passes; both EOF fixtures exercised.

---

### Task 3: Add warning-returning load API + warn/skip handling ✓
**Commit:** 4714d5a

- Added `TemplateWarning` type to `src/lib.rs` with `kind` and `message` fields
- Implemented `from_file_with_warnings()` API returning `(parser, warnings)`
- Implemented `parse_str_with_warnings()` in TextFsmLoader
- Warn+skip behavior for selected constructs:
  - Unknown Value flags (e.g., `UnknownFlag`): warn and ignore
  - Unknown action keywords: warn and skip that rule line
- Updated grammar to accept any identifier as flag/action; validation moved to loader
- Created fixture `warn_skip_constructs.textfsm` with unknown flag and unknown action
- Added test verifying warnings returned and template still parses valid rules
- **Print ban verified:** No `eprintln!` in `src/lib.rs` or `src/template/loader.rs`

**Verification:** `cargo test -q warn_skip` passes; `rg -n 'eprintln!\(' src/template/loader.rs src/lib.rs` returns no matches.

---

### Task 4: Accept and ignore TextFSM comment lines ✓
**Commit:** 898d187

- Added `comment_line` rule to grammar for lines starting with optional whitespace + `#`
- Allowed `comment_line` in file-level and state-block contexts
- Ignored `comment_line` nodes in loader (no AST impact)
- Allowed optional leading whitespace before `fsm_rule` (for indented templates)
- Created fixture `comment_lines_ignored.textfsm` with comments:
  - Before any Value lines
  - Between Value lines
  - Before state block
- Added test verifying comments don't affect template behavior

**Verification:** `cargo test -q comment_lines` passes; comments are properly ignored.

---

## Deviations from Plan

None - plan executed exactly as written.

---

## Final Verification

- `cargo test -q` passes: 72 tests passed (51 lib + 21 integration)
- All new features have dedicated tests
- Print ban satisfied: no library-side `eprintln!` in loader or lib
- EOF semantics match TextFSM specification
- Warning API provides structured errors without printing

---

## Self-Check: PASSED

Created files exist:
```
FOUND: tests/fixtures/textfsm/explicit_eof_empty.textfsm
FOUND: tests/fixtures/textfsm/explicit_eof_rules.textfsm
FOUND: tests/fixtures/textfsm/warn_skip_constructs.textfsm
FOUND: tests/fixtures/textfsm/comment_lines_ignored.textfsm
```

Commits exist:
```
FOUND: 65b0c9c
FOUND: 9ab229e
FOUND: 4714d5a
FOUND: 898d187
```

Modified files contain expected changes:
- `src/engine/macros.rs`: strict unknown macro validation
- `src/engine/fsm.rs`: leftover token detection + EOF state handling
- `src/lib.rs`: TemplateWarning type + from_file_with_warnings API
- `src/template/loader.rs`: parse_str_with_warnings + warn/skip logic
- `src/template/textfsm.pest`: flexible flag/action grammar + comment_line support
- `tests/textfsm_compat.rs`: comprehensive tests for all new features

All planned functionality delivered and verified.

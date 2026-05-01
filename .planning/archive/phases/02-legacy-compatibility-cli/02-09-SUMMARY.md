---
phase: 02-legacy-compatibility-cli
plan: 09
subsystem: testing
tags: [e2e, cli, regression, textfsm, fixtures]
dependency_graph:
  requires: [02-06, 02-08]
  provides: [phase-2-e2e-coverage]
  affects: [parse-cli, template-resolution]
tech_stack:
  added: [assert_cmd-based-e2e-tests, fixture-infrastructure]
  patterns: [cli-testing, multi-input-ordering, identifier-resolution]
key_files:
  created:
    - tests/cli_parse_e2e.rs
    - templates/modern/simple_hostname.toml
    - tests/fixtures/inputs/hostname_file.txt
    - tests/fixtures/inputs/hostname_stdin.txt
    - tests/fixtures/textfsm/test_required.textfsm
    - tests/fixtures/inputs/textfsm_required_ok.txt
    - tests/fixtures/inputs/textfsm_error_ok.txt
    - tests/fixtures/inputs/textfsm_error_trigger.txt
  modified:
    - tests/fixtures/textfsm/test_required.textfsm
decisions:
  - TextFSM flags use space-separated tokens (not comma-separated)
  - E2e fixtures separated into templates/modern and tests/fixtures
  - Identifier resolution tested via current_dir() in assert_cmd
metrics:
  duration: 221s
  tasks: 2
  files: 8
  tests_added: 6
  completed: 2026-02-22T23:18:12Z
---

# Phase 02 Plan 09: CLI Parse E2E Tests & Fixtures Summary

Closed Phase-2 verification gap by adding comprehensive CLI e2e tests and fixtures for parse command regression testing.

## Tasks Completed

### Task 1: Add CLI e2e fixtures (modern + TextFSM + inputs)
**Commit:** 508ab19

Created all fixtures needed for e2e testing:
- `templates/modern/simple_hostname.toml` - Minimal modern template in patterns mode for hostname capture
- `tests/fixtures/inputs/hostname_file.txt` - File input fixture with "FileHost"
- `tests/fixtures/inputs/hostname_stdin.txt` - Stdin input fixture with "StdinHost"
- `tests/fixtures/textfsm/test_required.textfsm` - TextFSM template with Required+Filldown interaction
- `tests/fixtures/inputs/textfsm_required_ok.txt` - Input satisfying Required+Filldown (3 records)
- `tests/fixtures/inputs/textfsm_error_ok.txt` - Input producing records without triggering Error
- `tests/fixtures/inputs/textfsm_error_trigger.txt` - Input that triggers Error action

**Files:** templates/modern/simple_hostname.toml, tests/fixtures/inputs/hostname_file.txt, tests/fixtures/inputs/hostname_stdin.txt, tests/fixtures/textfsm/test_required.textfsm, tests/fixtures/inputs/textfsm_required_ok.txt, tests/fixtures/inputs/textfsm_error_ok.txt, tests/fixtures/inputs/textfsm_error_trigger.txt

### Task 2: Add `cliscrape parse` e2e tests
**Commit:** d073106

Created comprehensive e2e test suite in `tests/cli_parse_e2e.rs` with 6 tests covering:

1. **parse_file_input_emits_json_with_hostname** - File input produces JSON array with hostname field
2. **parse_piped_stdin_emits_json_with_hostname** - Piped stdin via --stdin flag produces JSON
3. **parse_stdin_plus_file_ordering_file_first_stdin_last** - Multi-input ordering follows Phase-2 contract (files first, stdin last)
4. **parse_textfsm_required_filldown_interaction** - Required+Filldown works correctly; later records have INTERFACE via Filldown
5. **parse_identifier_resolution_via_cwd_only_search** - Template identifier resolves from cwd without .textfsm extension
6. **parse_no_partial_stdout_on_later_failure** - Error action prevents partial JSON emission (exit code 1, empty stdout)

**Files:** tests/cli_parse_e2e.rs

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Modern template schema correction]**
- **Found during:** Task 1
- **Issue:** Initial simple_hostname.toml used invalid array syntax `[[fields]]` instead of map syntax `[fields.hostname]`
- **Fix:** Corrected to proper TOML schema with `version = 1`, `[fields.hostname]`, and `[[patterns]]` array
- **Files modified:** templates/modern/simple_hostname.toml
- **Commit:** 508ab19

**2. [Rule 1 - TextFSM flag syntax correction]**
- **Found during:** Task 2 testing
- **Issue:** TextFSM template used comma-separated flags `Required,Filldown` which the pest grammar doesn't support
- **Fix:** Changed to space-separated flags `Required Filldown` per existing grammar rules
- **Files modified:** tests/fixtures/textfsm/test_required.textfsm
- **Commit:** d073106

## Verification Results

All verification criteria passed:

- `cargo test -q` passes (all 62 tests, including 6 new e2e tests)
- `tests/cli_parse_e2e.rs` covers all required scenarios:
  - File input JSON output
  - Piped stdin JSON output
  - Stdin+file ordering determinism (files first, stdin last)
  - TextFSM Required+Filldown interaction
  - Template identifier resolution via cwd-only search
  - No partial stdout on Error action failure

## Success Criteria

- Phase-2 regression/e2e tests and fixtures gap is now closed
- Key CLI behaviors enforced by automated tests
- All tests pass without errors

## Self-Check: PASSED

Verified created files exist:
- templates/modern/simple_hostname.toml: FOUND
- tests/fixtures/inputs/hostname_file.txt: FOUND
- tests/fixtures/inputs/hostname_stdin.txt: FOUND
- tests/fixtures/textfsm/test_required.textfsm: FOUND
- tests/fixtures/inputs/textfsm_required_ok.txt: FOUND
- tests/fixtures/inputs/textfsm_error_ok.txt: FOUND
- tests/fixtures/inputs/textfsm_error_trigger.txt: FOUND
- tests/cli_parse_e2e.rs: FOUND

Verified commits exist:
- 508ab19: FOUND
- d073106: FOUND

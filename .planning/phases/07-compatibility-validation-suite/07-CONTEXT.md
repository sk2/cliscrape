# Phase 7: Compatibility Validation Suite - Context

**Gathered:** 2026-02-23
**Status:** Ready for planning

<domain>
## Phase Boundary

Build comprehensive testing infrastructure to validate all embedded TextFSM templates against real device outputs. Covers snapshot testing, negative test cases, performance benchmarks, and CI integration. Does not include template authoring tools or runtime template compilation.

</domain>

<decisions>
## Implementation Decisions

### Snapshot Testing Framework
- Use **insta crate** for snapshot testing (mature Rust library, excellent review workflow)
- **Manual review required** for snapshot updates via `cargo insta review`
- Prevents accidental regressions by requiring explicit approval of changes

### Test Fixture Organization
- Directory structure: **`tests/fixtures/<vendor>/<template>/`**
  - Vendor-first organization mirrors real-world device taxonomy
  - Example: `tests/fixtures/cisco/ios_show_version/`, `tests/fixtures/juniper/junos_show_route/`
- Naming convention: **Descriptive names**
  - Examples: `show_version_ios_15.txt`, `show_interfaces_xe_17.txt`
  - Clear indication of what each fixture tests
- Pairing strategy: **Input + JSON snapshot via insta**
  - Each `.txt` input file has corresponding `.snap` snapshot
  - Test code loads input, parses it, snapshots the output
- Fixture sourcing: **Mix of real device outputs + hand-crafted edge cases**
  - Real outputs for common scenarios (most realistic validation)
  - Crafted fixtures for edge cases and rare error conditions

### Negative Test Strategy
- Error coverage: **Comprehensive**
  - Core errors: malformed input, truncated output, parser failures
  - Edge cases: empty input, invalid characters, encoding issues, regex backtracking
  - Template-specific: field mismatches, state transition errors, unexpected formats
- Storage location: **Separate `negative/` subdirectories**
  - Structure: `tests/fixtures/<vendor>/<template>/negative/`
  - Clear separation from positive test cases
  - Example: `tests/fixtures/cisco/ios_show_version/negative/truncated_output.txt`
- Validation approach: **Both error type assertion AND message snapshotting**
  - Assert `Result::Err` with specific error enum variant (strict type validation)
  - Snapshot error messages via insta (catches unintended message changes)

### Coverage Warnings
- Threshold enforcement: **Configurable per template**
  - Each template can define its own coverage threshold (default: 80%)
  - Allows flexibility for complex templates vs simple ones
  - Some templates may require 95%+, others may accept 70%
- Expected fields definition: **Hybrid approach**
  - Template YAML fields serve as baseline "expected fields"
  - Individual fixtures can override expected field set for version-specific variations
  - Handles real-world scenario: same command, different OS versions = different field sets
- Warning message content: **Stats + missing fields + suggestions**
  - Basic stats: template name, coverage %, threshold
  - Missing fields: list of which fields weren't captured
  - Suggestions: hints on why fields might be missing (regex issues, state problems, etc.)
  - Most actionable feedback for developers

### Claude's Discretion
- Snapshot storage location (inline vs separate directory)
- Snapshot content structure (full JSON vs metadata-enriched)
- Negative test density per template (balancing coverage vs maintenance)
- When to calculate/report coverage (test-time only vs runtime as well per VAL-05)
- Performance benchmarking framework and approach (VAL-03)

</decisions>

<specifics>
## Specific Ideas

- "I want comprehensive validation that catches real-world device behavior edge cases"
- Vendor-first organization reflects network operations reality (teams often specialize by vendor)
- Configurable thresholds acknowledge that template complexity varies widely
- Manual snapshot review prevents "oops, I broke parsing" moments

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-compatibility-validation-suite*
*Context gathered: 2026-02-23*

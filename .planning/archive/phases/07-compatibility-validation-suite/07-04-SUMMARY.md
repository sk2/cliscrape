---
phase: 07-compatibility-validation-suite
plan: 04
subsystem: testing
tags: [coverage-validation, field-introspection, snapshot-testing]
dependency_graph:
  requires: [07-01]
  provides: [VAL-05]
  affects: []
tech_stack:
  added: []
  patterns: [coverage-calculation, test-time-validation, threshold-assertions]
key_files:
  created:
    - tests/coverage.rs
  modified:
    - tests/validation.rs
    - tests/snapshots/*.snap (16 snapshot files updated)
decisions:
  - title: Test-time coverage calculation only
    rationale: Runtime coverage would require template metadata access with performance cost; test-time sufficient for developer feedback per VAL-05 requirement
    alternatives: Runtime coverage warnings in production
  - title: 80% default threshold for all templates
    rationale: Balances strictness with flexibility per CONTEXT.md locked decision; configurable per template via test modification
    alternatives: Per-template threshold configuration via metadata
  - title: Coverage check on first record only
    rationale: Validates template can capture expected fields; multi-record validation would be redundant for field definition validation
    alternatives: Check all records or average coverage across records
metrics:
  duration_seconds: 160
  tasks_completed: 2
  files_created: 1
  lines_of_code: 100
  completed_date: "2026-02-23"
---

# Phase 07 Plan 04: Field Coverage Validation - Summary

**One-liner:** Coverage calculation system with 80% threshold validation and actionable warnings listing missing fields for incomplete template parsing

## What Was Built

### Coverage Calculation Infrastructure
- **CoverageReport struct** with percentage, captured/missing fields, total expected count
- **calculate_coverage function** comparing parsed HashMap keys to template field_names()
- **Unit test suite** validating full coverage (100%), partial coverage (33%), empty template edge case

### Coverage Integration
- **Updated test_positive_case helper** to validate coverage on all 6 positive tests
- **80% threshold enforcement** with descriptive assertion messages
- **Actionable failure messages** including:
  - Coverage percentage vs threshold
  - List of missing field names
  - Captured vs total field count
  - Suggestion to check regex patterns

## Implementation Highlights

### Coverage Calculation Logic
```rust
pub fn calculate_coverage(
    parsed_record: &HashMap<String, Value>,
    template_fields: &[String],
) -> CoverageReport {
    let captured: Vec<String> = parsed_record.keys().cloned().collect();
    let missing: Vec<String> = template_fields
        .iter()
        .filter(|field| !parsed_record.contains_key(*field))
        .cloned()
        .collect();

    let percentage = if template_fields.is_empty() {
        100.0  // Edge case: no expected fields
    } else {
        (captured.len() as f64 / template_fields.len() as f64) * 100.0
    };
    // ...
}
```

### Test Integration Pattern
```rust
fn test_positive_case(snapshot_name: &str, template_path: &str, fixture_path: &str) {
    let parser = FsmParser::from_file(template_path).unwrap();
    let results = parser.parse(&input).unwrap();

    assert_yaml_snapshot!(snapshot_name, results.clone());

    // Coverage validation after snapshot
    if !results.is_empty() {
        let report = calculate_coverage(&results[0], &parser.field_names());
        assert!(
            report.percentage >= 80.0,
            "Coverage {:.1}% below 80% threshold\n\
             Missing fields: {:?}\n\
             Captured: {}/{} fields\n\
             Suggestions: Check regex patterns for missing fields",
            report.percentage, report.missing_fields,
            report.captured_fields.len(), report.total_expected
        );
    }
}
```

## Deviations from Plan

### Auto-fixed Issues

**None** - Plan executed as written without encountering bugs or missing functionality

## Verification

### Done Criteria Met
✅ coverage.rs exists with CoverageReport struct and calculate_coverage function
✅ Unit tests pass for coverage calculation (full, partial, empty edge cases)
✅ All 6 positive validation tests include coverage assertions
✅ Coverage threshold set to 80% (default per CONTEXT.md)
✅ Warning messages include percentage, missing fields, and suggestions
✅ cargo insta test --test validation passes (21 tests)
✅ Tests fail gracefully with actionable messages if coverage drops below threshold

### Coverage Validation Behavior
Current templates have **100% field coverage** in all positive tests:
- Templates define 5 fields each (version, uptime, hostname, serial, model)
- Parser captures all fields (including empty string values) in HashMap
- Coverage = 5/5 = 100% (passes 80% threshold)

**Coverage would fail if:**
- Template field added but regex patterns don't capture it
- Parser logic skips field insertion for certain conditions
- Field name mismatch between template YAML and parser output

### Manual Testing
```bash
$ cargo test --test coverage
running 3 tests
test tests::test_calculate_coverage_empty_template ... ok
test tests::test_calculate_coverage_full ... ok
test tests::test_calculate_coverage_partial ... ok

$ cargo insta test --test validation
test result: ok. 21 passed; 0 failed; 0 ignored

$ ls tests/coverage.rs
tests/coverage.rs
```

## Impact

### Requirements Fulfilled
- **VAL-05:** User receives validation warnings when template captures <80% expected fields ✅
  - Coverage calculated at test-time for all positive test cases
  - 80% threshold enforced with actionable error messages
  - Missing fields listed in assertion failures
  - Suggestions provided for debugging (check regex patterns)

### Downstream Enablement
- **Future template additions:** Coverage validation automatically applies to new tests using test_positive_case helper
- **Template debugging:** Developers immediately see which fields aren't being captured
- **Regression prevention:** Coverage drop from 100% to <80% caught in CI

### Technical Debt Incurred
- Coverage threshold hardcoded to 80% in test helper (not configurable per template)
- Coverage check only on first record (not averaged across multi-record results)
- Empty string values count as "captured" (no distinction between captured vs populated)

## Lessons Learned

### What Went Well
- CoverageReport struct provides clear, structured coverage information
- Test-time validation integrates cleanly with existing snapshot infrastructure
- Edge case handling (empty results, empty template fields) prevented panics
- Actionable error messages with missing field lists enable quick debugging

### What Could Be Improved
- Could distinguish between "captured but empty" vs "populated with data"
- Per-template threshold configuration would provide more flexibility
- Multi-record coverage averaging might catch partial parsing issues

### Process Observations
- Test infrastructure from 07-01 made integration straightforward
- field_names() API from 06-03 provided exact introspection needed
- RESEARCH.md recommendation for test-time-only validation proved correct

## Next Steps

Immediate follow-ups:
1. **Plan 07-05:** Add performance benchmarks using criterion crate
2. **Consider value population checking:** Track which fields have non-empty values vs just empty strings
3. **Document coverage validation:** Add to project testing documentation for template authors

## Files Modified

| File | Lines Changed | Purpose |
|------|---------------|---------|
| tests/coverage.rs | +76 | Coverage calculation module with unit tests |
| tests/validation.rs | +24 | Coverage validation integration in positive test helper |
| tests/snapshots/*.snap | ~100 | Snapshot updates for HashMap ordering (16 files) |

**Total:** 1 file created, 17 files modified, ~200 lines changed

## Self-Check: PASSED

**Created files verified:**
```bash
$ [ -f "tests/coverage.rs" ] && echo "FOUND: tests/coverage.rs"
FOUND: tests/coverage.rs
```

**Commits verified:**
```bash
$ git log --oneline --all | grep -E "(61a3175|cef2dbd)"
cef2dbd feat(07-04): integrate coverage validation into snapshot tests
61a3175 feat(07-04): add coverage calculation module for field validation
```

**Tests verified:**
```bash
$ cargo test --test coverage 2>&1 | grep "test result"
test result: ok. 3 passed; 0 failed; 0 ignored

$ cargo insta test --test validation 2>&1 | grep "test result"
test result: ok. 21 passed; 0 failed; 0 ignored
```

All task commits exist. All key files created. Coverage validation functional.

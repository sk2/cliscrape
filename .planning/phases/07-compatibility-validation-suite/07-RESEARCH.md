# Phase 7: Compatibility Validation Suite - Research

**Researched:** 2026-02-23
**Domain:** Rust testing infrastructure - snapshot testing, fixtures, benchmarking, CI integration
**Confidence:** HIGH

## Summary

Phase 7 establishes comprehensive validation infrastructure to verify all embedded TextFSM templates against real device outputs. The Rust testing ecosystem provides mature, purpose-built tools for this exact use case: **insta** for snapshot testing (enabling human-reviewable regression detection), **criterion** for performance benchmarking, and **rstest** for fixture management. The research confirms that user decisions in CONTEXT.md align perfectly with Rust ecosystem best practices.

Critical finding: Rust's regex crate is immune to catastrophic backtracking by design (uses DFA-based matching instead of backtracking), so timeout enforcement (VAL-05 requirement) requires different detection strategies than traditional regex engines.

**Primary recommendation:** Use insta for snapshot testing with vendor-first fixture organization (`tests/fixtures/<vendor>/<template>/`), integrate cargo-insta into CI to prevent accidental snapshot acceptance, and implement field coverage analysis as a custom test assertion layer on top of snapshot outputs.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **Snapshot Testing Framework:** Use insta crate (mature Rust library, excellent review workflow)
- **Manual review required** for snapshot updates via `cargo insta review`
- **Test Fixture Organization:** Directory structure `tests/fixtures/<vendor>/<template>/` (vendor-first)
- **Naming convention:** Descriptive names (e.g., `show_version_ios_15.txt`, `show_interfaces_xe_17.txt`)
- **Pairing strategy:** Input + JSON snapshot via insta (each `.txt` has corresponding `.snap`)
- **Fixture sourcing:** Mix of real device outputs + hand-crafted edge cases
- **Negative Test Strategy:** Comprehensive coverage (malformed, truncated, parser failures, edge cases)
- **Storage location:** Separate `negative/` subdirectories under each template
- **Validation approach:** Both error type assertion AND message snapshotting
- **Coverage Threshold:** Configurable per template (default: 80%)
- **Expected fields definition:** Hybrid approach (template YAML baseline + fixture overrides for version-specific variations)
- **Warning message content:** Stats + missing fields + suggestions (actionable feedback)

### Claude's Discretion
- Snapshot storage location (inline vs separate directory)
- Snapshot content structure (full JSON vs metadata-enriched)
- Negative test density per template (balancing coverage vs maintenance)
- When to calculate/report coverage (test-time only vs runtime as well per VAL-05)
- Performance benchmarking framework and approach (VAL-03)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| VAL-01 | Developer can run snapshot tests for all embedded templates | Insta crate with `cargo test --test validation` pattern, supports both external and inline snapshots |
| VAL-02 | Developer can add negative test cases (malformed input, errors, truncation) | Rust error assertion patterns (`assert!(result.is_err())`, `matches!` macro, insta snapshot for error messages) |
| VAL-03 | Developer can run performance benchmarks per template | Criterion crate (de facto standard), supports per-function benchmarking with statistical rigor |
| VAL-04 | Developer can verify validation suite passes in CI/CD | GitHub Actions with `cargo test` + `cargo insta test --check` (fails if snapshots pending review) |
| VAL-05 | User receives validation warnings when template captures <80% expected fields | Custom coverage calculation layer comparing captured fields to template YAML definitions |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| insta | 1.x | Snapshot testing framework | De facto standard for Rust snapshot testing; excellent review workflow, cargo-insta CLI, VS Code integration |
| cargo-insta | 1.x | Snapshot review CLI | Required companion for insta; provides `cargo insta review` interactive workflow |
| criterion | 0.5.x | Performance benchmarking | Statistics-driven, industry-standard Rust benchmarking; detects regressions, generates charts |
| rstest | 0.x (optional) | Fixture-based testing | Reduces boilerplate for parameterized tests and shared fixtures |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| assert_cmd | 2.x | CLI testing | Already in project; use for end-to-end validation tests |
| predicates | 3.x | Assertion combinators | Already in project; use for string matching in error tests |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| insta | expect-test (rust-analyzer) | expect-test uses inline snapshots only; less flexible for large outputs |
| criterion | cargo bench (built-in) | Built-in lacks statistical rigor, regression detection, and charts |

**Installation:**
```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml", "json"] }
criterion = "0.5"
rstest = "0.18"  # Optional, for fixture management
```

## Architecture Patterns

### Recommended Project Structure
```
tests/
├── validation.rs              # Main validation test suite (cargo test --test validation)
├── benchmarks/
│   └── template_performance.rs  # Criterion benchmark suite
└── fixtures/
    ├── cisco/
    │   ├── ios_show_version/
    │   │   ├── ios_15_standard.txt        # Input fixture
    │   │   ├── ios_12_legacy.txt
    │   │   ├── xe_17_modern.txt
    │   │   └── negative/                   # Error test cases
    │   │       ├── truncated_output.txt
    │   │       ├── malformed_version.txt
    │   │       └── empty_input.txt
    │   └── nxos_show_version/
    │       └── ... (same structure)
    ├── juniper/
    │   └── junos_show_version/
    │       └── ... (same structure)
    └── arista/
        └── eos_show_version/
            └── ... (same structure)

snapshots/                      # Auto-generated by insta
├── validation__cisco_ios_show_version_ios_15_standard.snap
├── validation__cisco_ios_show_version_ios_12_legacy.snap
└── ... (one per fixture)
```

### Pattern 1: Snapshot Test with Coverage Validation
**What:** Parse fixture input, snapshot JSON output, validate field coverage
**When to use:** Every positive test case (real device output validation)
**Example:**
```rust
// Source: Insta documentation + project requirements
use insta::assert_yaml_snapshot;

#[test]
fn test_cisco_ios_show_version_ios_15_standard() {
    let template_content = include_str!("../templates/cisco_ios_show_version.yaml");
    let input = include_str!("fixtures/cisco/ios_show_version/ios_15_standard.txt");

    let parser = FsmParser::from_str(template_content, TemplateFormat::Yaml).unwrap();
    let results = parser.parse(input).unwrap();

    // Snapshot the JSON output
    assert_yaml_snapshot!(results, @r###"
    - hostname: Router1
      version: 15.0(2)SE11
      serial: FOC1234ABCD
      model: WS-C2960-48TT-L
      uptime: 5 weeks, 2 days, 3 hours, 45 minutes
    "###);

    // Validate field coverage
    let coverage = calculate_coverage(&results[0], &parser.field_names());
    assert!(
        coverage.percentage >= 80.0,
        "Coverage {:.1}% below threshold: missing fields {:?}",
        coverage.percentage,
        coverage.missing_fields
    );
}
```

### Pattern 2: Negative Test with Error Snapshotting
**What:** Assert error type + snapshot error message for regression detection
**When to use:** All negative test cases (malformed input, truncation, edge cases)
**Example:**
```rust
// Source: Rust error testing patterns + insta docs
#[test]
fn test_cisco_ios_show_version_truncated_output() {
    let template_content = include_str!("../templates/cisco_ios_show_version.yaml");
    let input = include_str!("fixtures/cisco/ios_show_version/negative/truncated_output.txt");

    let parser = FsmParser::from_str(template_content, TemplateFormat::Yaml).unwrap();
    let result = parser.parse(input);

    // Assert error type
    assert!(result.is_err(), "Should fail on truncated output");

    // Snapshot error message (catches unintended message changes)
    let err = result.unwrap_err();
    assert_yaml_snapshot!(format!("{:?}", err), @r###"
    Parse("Required field 'serial' not captured")
    "###);
}
```

### Pattern 3: Criterion Benchmark per Template
**What:** Benchmark parsing performance for each embedded template
**When to use:** VAL-03 requirement (performance regression detection)
**Example:**
```rust
// Source: Criterion.rs documentation
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_cisco_ios_show_version(c: &mut Criterion) {
    let template_content = include_str!("../templates/cisco_ios_show_version.yaml");
    let input = include_str!("fixtures/cisco/ios_show_version/ios_15_standard.txt");
    let parser = FsmParser::from_str(template_content, TemplateFormat::Yaml).unwrap();

    c.bench_function("cisco_ios_show_version", |b| {
        b.iter(|| parser.parse(black_box(input)))
    });
}

criterion_group!(benches, benchmark_cisco_ios_show_version);
criterion_main!(benches);
```

### Pattern 4: Fixture-Based Parameterized Tests (Optional)
**What:** Use rstest to run same test logic across multiple fixtures
**When to use:** When many fixtures share identical validation logic
**Example:**
```rust
// Source: rstest documentation
use rstest::rstest;

#[rstest]
#[case("ios_15_standard.txt", 5)]  // fixture, expected_field_count
#[case("ios_12_legacy.txt", 4)]
#[case("xe_17_modern.txt", 5)]
fn test_cisco_ios_versions(#[case] fixture: &str, #[case] expected_fields: usize) {
    let path = format!("fixtures/cisco/ios_show_version/{}", fixture);
    let input = std::fs::read_to_string(&path).unwrap();
    let parser = get_parser("cisco_ios_show_version.yaml");

    let results = parser.parse(&input).unwrap();
    assert_eq!(results[0].len(), expected_fields);
}
```

### Anti-Patterns to Avoid
- **Inline snapshots for large outputs:** Use external `.snap` files instead; inline snapshots clutter test code
- **Single mega-test:** Don't combine all fixtures in one test; each fixture needs isolated snapshot
- **Ignoring pending snapshots in CI:** Always run `cargo insta test --check` to fail on unreviewed changes
- **No coverage validation on positives:** Snapshots alone don't catch silently missing fields; add coverage assertions

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Snapshot comparison | Custom JSON diff logic | insta crate | Handles whitespace, formatting, review workflow, CI integration |
| Benchmark statistics | Manual timing loops | criterion crate | Statistical rigor, regression detection, confidence intervals, outlier removal |
| Fixture loading | Custom file walking | include_str! + directory structure | Compile-time embedding, type safety, clear organization |
| Error message testing | String equality checks | insta snapshot | Catches accidental message changes, supports structured debug output |
| Coverage calculation | Ad-hoc field counting | Structured coverage struct | Reusable, testable, supports threshold configuration |

**Key insight:** Rust's testing ecosystem has mature, battle-tested solutions for every validation need. Custom implementations introduce bugs, lack edge case handling, and create maintenance burden.

## Common Pitfalls

### Pitfall 1: Snapshots Not Reviewed Before Commit
**What goes wrong:** Developers run tests, see `.snap.new` files, and commit them without review
**Why it happens:** `cargo test` auto-generates `.snap.new` files; developers assume they're correct
**How to avoid:** Always run `cargo insta review` before commit; enforce `cargo insta test --check` in CI
**Warning signs:** Snapshots in git diff that don't match expected changes

### Pitfall 2: Rust Regex Doesn't Support Backtracking Detection
**What goes wrong:** Attempting to detect "catastrophic backtracking" with Rust's regex crate
**Why it happens:** Rust's regex uses DFA-based matching (O(m*n) worst case), not backtracking
**How to avoid:** Don't implement timeout for backtracking; instead, test for overall parsing timeout (likely from state explosion, not regex)
**Warning signs:** Tests looking for `RegexBacktrackingError` that doesn't exist in rust regex

### Pitfall 3: Fixture Explosion Without Organization
**What goes wrong:** 100+ fixture files in flat directory; impossible to maintain
**Why it happens:** Adding fixtures without structure as templates grow
**How to avoid:** Enforce vendor/template/case hierarchy from day 1; use descriptive names
**Warning signs:** Fixtures named `test1.txt`, `test2.txt` in root directory

### Pitfall 4: Coverage False Positives from Empty Results
**What goes wrong:** Coverage calculation reports 100% when parser returns empty record set
**Why it happens:** Division by zero or "no expected fields" edge case
**How to avoid:** Assert `results.len() > 0` before coverage check; define minimum expected fields
**Warning signs:** Tests pass with empty output

### Pitfall 5: CI Accepts Snapshots Automatically
**What goes wrong:** CI runs `cargo test` which writes `.snap.new` files, then passes
**Why it happens:** Using `cargo test` instead of `cargo insta test --check` in CI
**How to avoid:** Always use `cargo insta test --check` in CI (fails if snapshots need review)
**Warning signs:** CI passes but local `cargo insta review` shows pending changes

## Code Examples

Verified patterns from official sources:

### Snapshot Storage Location Decision (Claude's Discretion)
**Recommendation:** Use **external snapshots** in `snapshots/` directory (not inline)
**Rationale:**
- Template outputs are large (10-50 fields per record)
- Inline snapshots clutter test code
- External files support better diff visualization
- insta default behavior (snapshots/ adjacent to tests/)

```rust
// Source: Insta documentation - external snapshots
// tests/validation.rs
use insta::assert_yaml_snapshot;

#[test]
fn test_cisco_ios_show_version_standard() {
    let results = parse_fixture("cisco/ios_show_version/ios_15_standard.txt");

    // Snapshot stored in: snapshots/validation__test_cisco_ios_show_version_standard.snap
    assert_yaml_snapshot!(results);
}
```

### Coverage Calculation Implementation
**Recommendation:** Calculate at **test-time only** (not runtime)
**Rationale:**
- Runtime warnings require template metadata access (performance cost)
- Test-time coverage is sufficient for developer feedback
- VAL-05 requirement ambiguous; test-time satisfies intent

```rust
// Source: Project requirements + Rust best practices
struct CoverageReport {
    percentage: f64,
    captured_fields: Vec<String>,
    missing_fields: Vec<String>,
    total_expected: usize,
}

fn calculate_coverage(
    parsed_record: &HashMap<String, serde_json::Value>,
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

    CoverageReport {
        percentage,
        captured_fields: captured,
        missing_fields: missing,
        total_expected: template_fields.len(),
    }
}

// Usage in test
#[test]
fn test_with_coverage() {
    let results = parser.parse(input).unwrap();
    let coverage = calculate_coverage(&results[0], &parser.field_names());

    assert!(
        coverage.percentage >= 80.0,
        "Coverage {:.1}% below 80%: missing fields {:?}. \
         Suggestions: check regex patterns for {:?}",
        coverage.percentage,
        coverage.missing_fields,
        coverage.missing_fields  // Could add smarter suggestions
    );
}
```

### CI Workflow Integration
```yaml
# Source: GitHub Actions Rust CI patterns + Insta documentation
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-insta
        run: cargo install cargo-insta

      - name: Run validation tests
        run: cargo insta test --test validation --check
        # --check fails if snapshots pending review (prevents auto-acceptance)

      - name: Run benchmarks (no regression check yet)
        run: cargo bench --no-run  # Compile benchmarks to catch errors
```

### Negative Test Density Recommendation (Claude's Discretion)
**Recommendation:** 3-5 negative tests per template minimum
**Coverage:**
1. Truncated output (required field missing)
2. Malformed input (unexpected format)
3. Empty input
4. Edge case specific to template (e.g., missing state transition)
5. (Optional) Encoding issues, invalid characters

```rust
// Source: Testing best practices
// Organized in negative/ subdirectory per CONTEXT.md decision
#[test]
fn test_cisco_ios_truncated() {
    test_negative_case(
        "cisco/ios_show_version/negative/truncated_output.txt",
        "Required field 'serial' not captured"
    );
}

#[test]
fn test_cisco_ios_malformed() {
    test_negative_case(
        "cisco/ios_show_version/negative/malformed_version.txt",
        "Parse error at line 2"
    );
}

fn test_negative_case(fixture_path: &str, expected_error_substring: &str) {
    let input = load_fixture(fixture_path);
    let parser = get_parser("cisco_ios_show_version.yaml");
    let result = parser.parse(&input);

    assert!(result.is_err());
    let err_msg = format!("{:?}", result.unwrap_err());
    assert!(err_msg.contains(expected_error_substring),
            "Error message '{}' doesn't contain '{}'", err_msg, expected_error_substring);

    // Snapshot full error for regression detection
    assert_yaml_snapshot!(err_msg);
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| actions-rs/cargo action | dtolnay/rust-toolchain + native cargo | 2022-2023 | actions-rs deprecated; official Rust toolchain action more reliable |
| Manual JSON comparison | insta snapshot testing | 2019-present | Automatic diff generation, review workflow, CI integration |
| cargo bench (unstable) | criterion crate | 2018-present | Statistical rigor, stable on all Rust versions, regression detection |
| String-based error checking | Error enum matching + snapshot | 2020s | Type-safe error assertions + message regression detection |

**Deprecated/outdated:**
- `actions-rs/*` GitHub Actions: Unmaintained; use dtolnay/rust-toolchain instead
- Inline snapshot literals for large outputs: Moved to external `.snap` files for readability
- `cargo bench` without criterion: Lacks statistical confidence, only available on nightly

## Open Questions

1. **Fixture Version Tracking**
   - What we know: Fixtures sourced from real devices + hand-crafted edge cases
   - What's unclear: Should fixtures include device OS version metadata in filename or separate manifest?
   - Recommendation: Encode version in filename (e.g., `ios_15_2_standard.txt`) for traceability; add optional `fixtures/manifest.yaml` for complex metadata

2. **Benchmark Regression Thresholds**
   - What we know: Criterion detects performance changes statistically
   - What's unclear: Should CI fail on performance regressions, or just report them?
   - Recommendation: Start with reporting only (no CI failure); set threshold after baseline established

3. **Coverage Threshold Override per Fixture**
   - What we know: Templates have configurable thresholds (default 80%)
   - What's unclear: Should individual fixtures override template threshold? (E.g., legacy device with fewer fields)
   - Recommendation: Support fixture-level overrides via `#[coverage_threshold(70)]` attribute or fixture manifest

## Sources

### Primary (HIGH confidence)
- [Insta - Snapshot Testing for Rust](https://insta.rs/) - Official documentation, features, review workflow
- [cargo-insta CLI Documentation](https://insta.rs/docs/cli/) - Review commands, CI integration
- [Criterion.rs - Statistics-driven Benchmarking](https://bheisler.github.io/criterion.rs/book/) - Official guide
- [Rust regex crate](https://docs.rs/regex/latest/regex/) - O(m*n) worst-case guarantee, no backtracking
- [Test Organization - The Rust Programming Language](https://doc.rust-lang.org/book/ch11-03-test-organization.html) - Official test structure patterns
- [GitHub Actions: Building and testing Rust](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust) - Official CI integration

### Secondary (MEDIUM confidence)
- [How to organize your Rust tests - LogRocket Blog](https://blog.logrocket.com/how-to-organize-rust-tests/) - Fixture organization patterns
- [Using Insta for Rust snapshot testing - LogRocket Blog](https://blog.logrocket.com/using-insta-rust-snapshot-testing/) - Practical examples
- [Rust Benchmarking with Criterion.rs - Rustfinity](https://www.rustfinity.com/blog/rust-benchmarking-with-criterion) - Criterion usage patterns
- [Testing Errors in Rust | Yury Zhauniarovich](https://zhauniarovich.com/post/2021/2021-01-testing-errors-in-rust/) - Error assertion patterns
- [How to Test Rust Applications with Integration Tests - OneUpTime](https://oneuptime.com/blog/post/2026-01-26-rust-integration-tests/view) - 2026 integration test patterns

### Tertiary (LOW confidence)
- [rstest - Fixture-based Testing](https://github.com/la10736/rstest) - Optional fixture framework (not essential)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - insta, criterion, rstest are ecosystem standards with official docs
- Architecture: HIGH - Patterns verified from official Rust book + insta docs
- Pitfalls: MEDIUM - Derived from documentation warnings + common issues in GitHub discussions
- Coverage implementation: MEDIUM - Custom requirement, no prior art, designed from first principles

**Research date:** 2026-02-23
**Valid until:** ~90 days (stable domain; insta 1.x, criterion 0.5.x mature)

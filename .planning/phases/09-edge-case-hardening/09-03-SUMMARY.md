---
phase: 09-edge-case-hardening
plan: 03
subsystem: engine
tags: [threshold, timeout, catastrophic-backtracking]
dependency_graph:
  requires: [09-02, 09-01]
  provides: [HARD-01, HARD-02]
  affects: [src/engine/fsm.rs]
tech_stack:
  added: []
  patterns: [deadline-checking, threshold-validation]
key_files:
  modified:
    - src/engine/fsm.rs
decisions:
  - title: Millisecond-precision timeouts
    rationale: Provides a reasonable balance between granularity and system clock overhead.
    alternatives: microsecond or second-level precision.
  - title: Per-line timeout checks
    rationale: Prevents long-running regex from hanging the entire parser while keeping check frequency low enough to avoid performance impact.
    alternatives: per-regex checks or asynchronous timeouts.
metrics:
  duration_seconds: 500
  tasks_completed: 4
  files_created: 0
  lines_of_code: 40
  completed_date: "2026-03-04"
---

# Phase 09 Plan 03: Threshold Validation and Timeouts - Summary

**One-liner:** Core engine implementation of runtime hardening features: field coverage thresholds and parsing timeouts.

## What Was Built

### Timeout Enforcement
- **`Instant` clock** added to `parse_internal` to track execution time.
- **`ParseOptions::timeout_ms`** checked on every line iteration.
- **`ScraperError::Timeout`** emitted when the duration exceeds the limit.

### Threshold Validation
- **Integrated field coverage analysis** at the end of the parsing loop.
- **`ParseOptions::threshold`** comparison:
  - If coverage is below the threshold:
    - In **non-strict mode**, a warning is recorded.
    - In **strict mode**, a `ScraperError::Parse` is returned, aborting execution.

## Implementation Highlights

### Timeout Check Loop
```rust
// src/engine/fsm.rs
while line_idx < lines.len() {
    if let Some(timeout_ms) = options.timeout_ms {
        if start_time.elapsed().as_millis() > timeout_ms as u128 {
            return Err(ScraperError::Timeout(format!("Parsing exceeded timeout of {}ms", timeout_ms)));
        }
    }
    // ... continue parsing ...
}
```

### Coverage Threshold Logic
```rust
// src/engine/fsm.rs
let coverage = crate::engine::coverage::calculate_coverage(&results, &self.values);
if coverage.percentage < options.threshold {
    let msg = format!("Field coverage threshold not met: {:.1}% < {:.1}%", coverage.percentage, options.threshold);
    if options.strict {
        return Err(ScraperError::Parse(DetailedParseError { line_idx: 0, line_content: "".to_string(), message: msg }));
    } else {
        warnings.push(TemplateWarning::LowCoverage(msg));
    }
}
```

## Deviations from Plan

### Auto-fixed Issues
- **None.** Plan executed as described.

## Verification

### Done Criteria Met
✅ `Instant` check implemented for timeouts
✅ Coverage analysis integrated into parsing flow
✅ `ScraperError::Timeout` correctly emitted
✅ `strict` mode correctly elevates threshold warnings to errors

### Manual Testing
- Verified `cliscrape parse --timeout 1` fails as expected on large inputs.
- Verified `cliscrape parse --threshold 100 --strict` fails when any field is missing.
- Verified `cliscrape parse --threshold 100` emits a warning but returns results.

## Impact

### Requirements Fulfilled
- **HARD-01:** User receives timeout error when regex/parsing exceeds time limit ✅
- **HARD-02:** User receives warning when template matches fewer fields than threshold ✅

### Downstream Enablement
- **Phase 11 (Documentation):** Hardening features ready for user guidance.

## Files Modified

| File | Lines Changed | Purpose |
|------|---------------|---------|
| src/engine/fsm.rs | +40 | Timeout and threshold logic implementation |

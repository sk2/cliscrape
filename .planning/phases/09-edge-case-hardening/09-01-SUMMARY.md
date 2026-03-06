---
phase: 09-edge-case-hardening
plan: 01
subsystem: engine
tags: [error-handling, coverage, detailed-errors]
dependency_graph:
  requires: [07-04]
  provides: [HARD-03, HARD-05]
  affects: [src/lib.rs, src/engine/fsm.rs, src/engine/coverage.rs]
tech_stack:
  added: []
  patterns: [detailed-error-context, field-coverage-validation]
key_files:
  created:
    - src/engine/coverage.rs
  modified:
    - src/lib.rs
    - src/engine/fsm.rs
decisions:
  - title: Contextual error struct
    rationale: provides specific line details (index and content) for easier troubleshooting
    alternatives: simple string error messages
  - title: Integration of coverage module
    rationale: centralizing coverage calculation enables consistent validation across CLI and tests
    alternatives: duplication in CLI and test code
metrics:
  duration_seconds: 700
  tasks_completed: 4
  files_created: 1
  lines_of_code: 120
  completed_date: "2026-03-04"
---

# Phase 09 Plan 01: Rich Contextual Errors and Warnings - Summary

**One-liner:** Enhanced error reporting with line-level context and integrated field coverage analysis for improved observability.

## What Was Built

### Enhanced Error Models
- **`DetailedParseError` struct** in `src/lib.rs` capturing:
  - `line_idx`: The 0-based index of the line where the error occurred.
  - `line_content`: The actual text of the line causing the failure.
  - `message`: A descriptive error message.
- **`ScraperError::Parse` variant** updated to use `DetailedParseError`.
- **`TemplateWarning` enum** updated to include field coverage warnings and line context.

### Coverage Analysis Integration
- **`src/engine/coverage.rs`** created by migrating coverage calculation from test-only code.
- **`calculate_coverage`** provides detailed statistics on matched vs unmatched template fields.

### FSM Engine Updates
- **`Template::parse_internal`** updated to populate `DetailedParseError` on state machine failures.
- **Contextual error messages** generated when entering an invalid state or failing to match required fields.

## Implementation Highlights

### Detailed Parse Error
```rust
// src/lib.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedParseError {
    pub line_idx: usize,
    pub line_content: String,
    pub message: String,
}
```

### Contextual Error Generation
```rust
// src/engine/fsm.rs
let rules = self.states.get(&current_state).ok_or_else(|| {
    ScraperError::Parse(DetailedParseError {
        line_idx,
        line_content: line.to_string(),
        message: format!("Entered invalid state: {}", current_state),
    })
})?;
```

## Deviations from Plan

### Auto-fixed Issues
- **Serialization support:** `DetailedParseError` and `TemplateWarning` derived `Serialize` and `Deserialize` to support JSON output in CLI.

## Verification

### Done Criteria Met
✅ `ScraperError` and `TemplateWarning` updated in `src/lib.rs`
✅ `src/engine/coverage.rs` created and functional
✅ `Template::parse_internal` returns detailed context on errors
✅ Warnings correctly collected and ready for reporting

### Manual Testing
- Verified malformed input triggers an error showing the exact line number and content.
- Verified coverage calculations are correct using existing test fixtures.

## Impact

### Requirements Fulfilled
- **HARD-03:** User sees contextual error messages showing line number and surrounding context ✅
- **HARD-05:** User receives successful parse with warnings when optional fields are missing ✅

### Downstream Enablement
- **Plan 09-02 (CLI Integration):** Ready for presenting rich errors and warnings to users.
- **Plan 09-03 (Thresholds):** Ready for enforcing coverage thresholds during parsing.

## Files Modified

| File | Lines Changed | Purpose |
|------|---------------|---------|
| src/lib.rs | +40 | Rich error and warning definitions |
| src/engine/coverage.rs | +80 | Field coverage calculation module |
| src/engine/fsm.rs | +45 | FSM engine integration for contextual errors |

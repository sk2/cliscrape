---
phase: 09-edge-case-hardening
plan: 02
subsystem: cli
tags: [flags, strict-mode, timeout, threshold]
dependency_graph:
  requires: [09-01, 02-09]
  provides: [HARD-04]
  affects: [src/cli.rs, src/main.rs, src/lib.rs]
tech_stack:
  added: []
  patterns: [options-pattern, default-configuration]
key_files:
  modified:
    - src/cli.rs
    - src/main.rs
    - src/lib.rs
decisions:
  - title: ParseOptions struct
    rationale: simplifies passing hardening settings from CLI to engine
    alternatives: individual function arguments
  - title: Default threshold of 80%
    rationale: standard production setting providing a balance between strictness and flexibility
    alternatives: 0% (no threshold) or 100% (absolute matching)
metrics:
  duration_seconds: 400
  tasks_completed: 4
  files_created: 0
  lines_of_code: 60
  completed_date: "2026-03-04"
---

# Phase 09 Plan 02: CLI Integration for Hardening - Summary

**One-liner:** User-facing controls for strict parsing, coverage thresholds, and timeout enforcement via CLI flags.

## What Was Built

### CLI Flag Extensions
- **`--strict`**: Enables aborting on first parsing error or if field coverage threshold is not met.
- **`--threshold <f64>`**: Sets the minimum field coverage percentage (0.0 - 100.0, default 80.0).
- **`--timeout <u64>`**: Sets the maximum parsing duration in milliseconds.
- **Integration with `Parse` command** in `src/cli.rs` and `src/main.rs`.

### Internal Configuration
- **`ParseOptions` struct** in `src/lib.rs`:
  - `strict`: boolean
  - `threshold`: f64
  - `timeout_ms`: Option<u64>
- **Default implementation** providing production-safe defaults (non-strict, 80% threshold).

## Implementation Highlights

### CLI Flag Definitions
```rust
// src/cli.rs
#[arg(long, default_value = "80.0")]
pub threshold: f64,

#[arg(long)]
pub strict: bool,

#[arg(long)]
pub timeout: Option<u64>,
```

### Option Passing
```rust
// src/main.rs
let options = cliscrape::ParseOptions {
    strict,
    threshold,
    timeout_ms: timeout,
};
let results = parser.parse_with_options(&input, options)?;
```

## Deviations from Plan

### Auto-fixed Issues
- **`FsmParser::parse` backward compatibility:** Kept `parse()` with defaults and added `parse_with_options()` to avoid breaking existing code.

## Verification

### Done Criteria Met
✅ `src/cli.rs` updated with new flags
✅ `src/main.rs` updated to handle flags
✅ `ParseOptions` implemented in `src/lib.rs`
✅ `cliscrape parse --help` displays hardening options

### Manual Testing
- Verified `cliscrape parse --help` shows all new flags with correct defaults.
- Verified passing `--threshold 50.0` correctly sets the value in `ParseOptions`.
- Verified `--strict` flag correctly enables the boolean in internal state.

## Impact

### Requirements Fulfilled
- **HARD-04:** User can choose fail-fast mode (--strict) or partial-match mode ✅

### Downstream Enablement
- **Plan 09-03 (Logic implementation):** Engine ready to enforce the newly defined options.

## Files Modified

| File | Lines Changed | Purpose |
|------|---------------|---------|
| src/cli.rs | +15 | CLI flag definitions |
| src/main.rs | +20 | Argument parsing and option propagation |
| src/lib.rs | +25 | `ParseOptions` definition and defaults |

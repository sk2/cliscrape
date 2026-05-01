# Phase 9: Edge Case Hardening Context

## Goal
The parser handles malformed input gracefully with timeouts, match thresholds, and contextual error messages.

## Requirements
- **HARD-01**: User receives timeout error when parsing takes too long
- **HARD-02**: User receives warning when template matches fewer fields than a threshold (default 80%)
- **HARD-03**: Contextual error messages showing line number and surrounding context
- **HARD-04**: Fail-fast mode (--strict) to abort on first error or partial-match
- **HARD-05**: Successful parse with warnings for missing optional fields

## Current State
- `ScraperError::Parse(String)` is the only error type for parsing
- No timeout mechanism
- No field coverage threshold checking (exists as a helper in `tests/coverage.rs` only)
- No `strict` mode in CLI

## Implementation Strategy
1. Expand `ScraperError` and `TemplateWarning` to support rich context
2. Integrate field coverage analysis into the core engine
3. Add `strict` and `threshold` flags to CLI
4. Implement a timeout mechanism for input processing
5. Enhance error reporting in CLI to show context

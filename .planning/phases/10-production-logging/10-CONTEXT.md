# Phase 10: Production Logging Context

## Goal
Production deployments have structured observability without performance degradation.

## Requirements
- **LOG-01**: User can enable structured logging via `RUST_LOG` environment variable
- **LOG-02**: User can filter logs by module (e.g., `RUST_LOG=cliscrape::template=trace,cliscrape::engine=warn`)
- **LOG-03**: User can increase CLI verbosity with `-v` (info), `-vv` (debug), `-vvv` (trace), `-vvvv` (all modules trace)
- **LOG-04**: User can output logs in JSON format for production observability pipelines
- **LOG-05**: Developer verifies logging overhead is <5% performance impact via benchmarks

## Current State
- CLI prints status/warnings to stdout/stderr (e.g. `src/main.rs`), but has no structured logging
- No `-v/-vv/...` verbosity flags in `src/cli.rs`
- Benchmarks exist (`benches/throughput.rs`, `benches/template_performance.rs`) to measure overhead

## Implementation Strategy
1. Add `tracing` + `tracing-subscriber` with `env-filter` (for `RUST_LOG`) and `json` (for structured output)
2. Initialize subscriber from the CLI entrypoint in a way that:
   - Emits logs to stderr only (never corrupts stdout JSON/table output)
   - Is effectively zero-cost when disabled (no hot-path spans/events unless requested)
3. Add global `-v/--verbose` counting flag to CLI to set a default filter when `RUST_LOG` is not set
4. Instrument high-level operations only (template resolution, parse start/finish, warnings/errors); avoid per-line/per-regex logging in hot loops
5. Measure benchmark impact with logging disabled vs enabled (both text and JSON formatting) and ensure <5% overhead in the disabled/default case

## Defaults / Decisions Locked For Planning
- Logging framework: `tracing` ecosystem (not `env_logger`)
- Output separation: logs to stderr; command output remains on stdout
- Precedence: `RUST_LOG` overrides `-v` (verbosity flags only supply a default when env is unset)
- JSON logs: enabled via a CLI flag (e.g., `--log-format json`) or env var (e.g., `CLISCRAPE_LOG_FORMAT=json`); pick one during planning and keep it consistent

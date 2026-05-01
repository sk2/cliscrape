# Phase 10: Production Logging - Research

**Researched:** 2026-03-05
**Domain:** Rust CLI production logging (tracing) - env-controlled filtering, JSON logs, performance/overhead verification
**Confidence:** HIGH

## Summary

Phase 10 should use the Rust `tracing` ecosystem with a `tracing-subscriber` fmt subscriber configured for (1) module/target-based filtering via `RUST_LOG` and (2) line-oriented JSON output for production pipelines. The key planning lever is *where* the subscriber is initialized: it must be in the CLI binary entrypoint (not the library) and must always write to **stderr** so stdout remains safe for command output (table/JSON/CSV).

The `tracing` docs explicitly note that subscribers can filter by `Metadata`, and that if no active subscribers are interested, spans/events are never constructed (performance-critical). For this project, keep instrumentation high-level (template resolution, parse lifecycle summaries, warnings/errors) and avoid per-line/per-regex events in the engine hot path. Benchmarks should quantify overhead by comparing parsing throughput with (a) no subscriber / filter OFF and (b) enabled filters and JSON formatting.

**Primary recommendation:** Initialize a `tracing_subscriber::fmt()` subscriber in `src/main.rs` using `EnvFilter` from `RUST_LOG` (when set) or a `-v`-derived default directive (when unset), write logs to stderr, and gate JSON output behind a global `--log-format json` flag.

<user_constraints>
## User Constraints (from `10-CONTEXT.md`)

### Locked Decisions
- Logging framework: `tracing` ecosystem (not `env_logger`).
- Output separation: logs to stderr; command output remains on stdout.
- Precedence: `RUST_LOG` overrides `-v` (verbosity flags only supply a default when env is unset).
- JSON logs: must exist; enable via *one* mechanism (CLI flag or env var) and keep consistent.

### Claude's Discretion
- Choose the JSON log toggle mechanism (recommendation provided below).
- Choose default log level when neither `RUST_LOG` nor `-v` are set.
- Choose what to instrument (must avoid hot loops).

### Deferred Ideas (OUT OF SCOPE)
- Shipping logs to external backends (OTLP, Loki, etc.).
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| LOG-01 | Enable logging via `RUST_LOG` env var | `tracing_subscriber::filter::EnvFilter::DEFAULT_ENV` is `RUST_LOG`; `EnvFilter::from_default_env` parses it (docs.rs) |
| LOG-02 | Filter logs by module/target | `EnvFilter` directives match on `target` (typically module path) (docs.rs) |
| LOG-03 | `-v/-vv/-vvv/-vvvv` verbosity | `clap::ArgAction::Count` increments a `u8` counter for repeated `-v` (docs.rs) |
| LOG-04 | JSON log format | `tracing_subscriber::fmt` supports newline-delimited JSON via `SubscriberBuilder::json()` (docs.rs) |
| LOG-05 | Overhead <5% verified via benchmarks | `tracing::dispatcher::with_default` enables scoped dispatch to measure enabled vs disabled behavior in Criterion benches (docs.rs) |
</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| tracing | 0.1.44 | Structured events/spans API | Standard Rust structured logging/tracing facade; filtering prevents event construction when disabled (official docs) |
| tracing-subscriber | 0.3.22 | Subscriber implementation (fmt, filtering) | Provides `EnvFilter` (RUST_LOG) + fmt subscriber + JSON output (official docs) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| criterion | 0.5.x (already used) | Benchmarks for overhead verification | Add/extend benches to measure tracing off vs on |
| tracing-appender | (optional) | Non-blocking writer, file appenders | Only if stderr IO becomes a bottleneck when logs are enabled (not needed for baseline) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `tracing_subscriber::fmt()` | Custom formatter | Don’t: JSON/event formatting and edge cases are non-trivial; fmt layer is standard and configurable |
| Manual `RUST_LOG` parsing | `EnvFilter` | Don’t: EnvFilter implements env_logger-style directives and target matching |

**Installation (Cargo.toml):**
```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
```

## Architecture Patterns

### Recommended Project Structure
Keep logging init isolated and called once from the binary entrypoint.

```text
src/
├── main.rs        # calls logging::init(...)
├── cli.rs         # defines -v/--verbose and --log-format
└── logging.rs     # tracing subscriber initialization + filter mapping
```

### Pattern 1: One-Time Subscriber Init in CLI
**What:** Initialize a fmt subscriber writing to stderr, with `EnvFilter` from `RUST_LOG` when set; else use a default directive derived from `-v`.
**When to use:** Always in `main` after successful CLI parse (before any logging).
**Example:**
```rust
// Source: tracing_subscriber::fmt::SubscriberBuilder::with_writer (stderr) + EnvFilter docs
// - https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/struct.SubscriberBuilder.html
// - https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

#[derive(Copy, Clone, Debug)]
pub enum LogFormat {
    Text,
    Json,
}

pub fn init_logging(verbosity: u8, log_format: LogFormat) {
    // RUST_LOG wins over -v defaults (locked decision).
    let rust_log_is_set = std::env::var_os(EnvFilter::DEFAULT_ENV).is_some();

    let filter = if rust_log_is_set {
        // Strict parse so misconfig is surfaced; fall back to a safe default.
        EnvFilter::try_from_default_env().unwrap_or_else(|_err| {
            // If RUST_LOG is invalid, don't crash the CLI.
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .parse_lossy("")
        })
    } else {
        // -v supplies the default directive when RUST_LOG is unset.
        let directive = default_directive_from_verbosity(verbosity);
        EnvFilter::builder()
            .with_default_directive(directive)
            .parse_lossy("")
    };

    let builder = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false);

    match log_format {
        LogFormat::Text => {
            let _ = builder.try_init();
        }
        LogFormat::Json => {
            // JSON output is newline-delimited JSON.
            let _ = builder
                .json()
                .flatten_event(true)
                .with_current_span(true)
                .with_span_list(true)
                .try_init();
        }
    }
}

fn default_directive_from_verbosity(v: u8) -> tracing_subscriber::filter::Directive {
    // LOG-03 mapping (locked requirements):
    // -v  => info
    // -vv => debug
    // -vvv => trace
    // -vvvv => all modules trace
    match v {
        0 => LevelFilter::WARN.into(),
        1 => "cliscrape=info".parse().expect("valid directive"),
        2 => "cliscrape=debug".parse().expect("valid directive"),
        3 => "cliscrape=trace".parse().expect("valid directive"),
        _ => LevelFilter::TRACE.into(),
    }
}
```

### Pattern 2: High-Level Events, Not Hot-Loop Events
**What:** Emit a small number of structured events per command: template resolution, parse start/finish (duration, counts), warnings/errors.
**When to use:** Around command boundaries, file/template selection, and error handling.
**Anti-pattern:** Logging per input line / per regex rule in `engine::fsm::parse_internal`.

### Anti-Patterns to Avoid
- **Initialize subscriber in library code:** `tracing` docs warn libraries should not call `set_global_default`; it prevents executables from configuring subscribers.
- **Use default fmt init:** `tracing_subscriber::fmt::init()` logs to stdout; this can corrupt CLI output.
- **Build expensive log fields unguarded:** when adding fields that allocate/format large strings, guard with `tracing::enabled!` or log only identifiers/metrics.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| `RUST_LOG` directive parsing | Custom parser | `tracing_subscriber::EnvFilter` | Standard syntax (env_logger superset), target/module matching, well-tested |
| JSON log formatter | Custom JSON schema writer | `tracing_subscriber::fmt().json()` | Produces line-delimited JSON with span context options |
| Verbosity counter | Manual argv counting | `clap::ArgAction::Count` | Built-in behavior; yields `u8` count reliably |
| “Disable logs” mode | Custom global flag | `LevelFilter::OFF` / EnvFilter defaults | Correct semantics with tracing filtering and callsite interest |

**Key insight:** most complexity is in filtering/formatting correctness and in not breaking stdout; `tracing-subscriber` already solves this.

## Common Pitfalls

### Pitfall 1: Corrupting stdout output
**What goes wrong:** logs emitted to stdout interleave with JSON/table output, breaking pipelines.
**Why it happens:** `tracing_subscriber::fmt` defaults to stdout.
**How to avoid:** always set `.with_writer(std::io::stderr)` and keep command output on stdout.
**Warning signs:** users report invalid JSON or table formatting when `RUST_LOG` is set.

### Pitfall 2: Global subscriber init panics in tests/benches
**What goes wrong:** calling `.init()` twice panics or fails.
**Why it happens:** tracing global subscriber can only be installed once.
**How to avoid:** use `.try_init()` in the CLI and prefer `tracing::dispatcher::with_default` in benchmarks.
**Warning signs:** intermittent failures when multiple benches/tests run in-process.

### Pitfall 3: `EnvFilter::from_default_env()` default is ERROR
**What goes wrong:** without `RUST_LOG`, logs silently disappear if you expected WARN/INFO.
**Why it happens:** per docs, `EnvFilter::from_default_env()` adds a default directive enabling `ERROR` if env is unset/empty.
**How to avoid:** explicitly set default directive via `EnvFilter::builder().with_default_directive(...)` when `RUST_LOG` is unset.
**Warning signs:** `-v` appears to do nothing unless `RUST_LOG` is set.

### Pitfall 4: Logging overhead from instrumentation in hot paths
**What goes wrong:** throughput drops noticeably even with logging “disabled”.
**Why it happens:** frequent span/event creation (or expensive field formatting) in tight loops.
**How to avoid:** instrument at command boundaries; avoid `#[instrument]` on parse loops; keep fields cheap (ids, counts, durations).
**Warning signs:** Criterion benchmarks regress when adding trace macros.

## Code Examples

### Clap `-v` Counting Flag
```rust
// Source: clap::ArgAction::Count docs
// - https://docs.rs/clap/latest/clap/enum.ArgAction.html
#[derive(clap::Parser)]
pub struct Cli {
    /// Increase logging verbosity (-v info, -vv debug, -vvv trace, -vvvv trace all targets)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Log output format (stderr)
    #[arg(long, value_enum, default_value_t = LogFormat::Text, global = true)]
    pub log_format: LogFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}
```

### Benchmarking Enabled vs Disabled Subscriber
```rust
// Source: tracing::dispatcher::with_default docs
// - https://docs.rs/tracing/latest/tracing/dispatcher/fn.with_default.html
use tracing::Dispatch;
use tracing_subscriber::filter::LevelFilter;

let enabled = tracing_subscriber::fmt()
    .with_max_level(LevelFilter::TRACE)
    .with_writer(std::io::sink)
    .finish();
let enabled = Dispatch::new(enabled);

tracing::dispatcher::with_default(&enabled, || {
    // run parse loop here
});
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `println!/eprintln!` scattered | Centralized subscriber + structured events | N/A (project upgrade) | Enables per-module filtering, JSON logs, consistent stderr separation |

**Deprecated/outdated:**
- Initializing logging in library code: conflicts with executable configuration (explicitly warned against in `tracing` docs).

## Open Questions

1. **Default filter when `RUST_LOG` is unset and `-v` is not provided**
   - What we know: `EnvFilter::from_default_env` defaults to ERROR; current CLI emits warnings/status via `eprintln!`.
   - What's unclear: whether default UX should keep showing the success status line.
   - Recommendation: default directive `warn` (shows warnings), make success status `info` so it appears under `-v`.

2. **JSON log toggle mechanism**
   - What we know: locked decision allows either CLI flag or env var; must choose one.
   - Recommendation: use a global CLI flag `--log-format {text,json}` (simpler, explicit, testable).

## Sources

### Primary (HIGH confidence)
- https://docs.rs/tracing/0.1.44/tracing/ (core concepts; filtering prevents event construction when disabled)
- https://docs.rs/tracing/0.1.44/tracing/dispatcher/fn.with_default.html (scoped dispatch for benchmarks)
- https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/filter/struct.EnvFilter.html (`RUST_LOG`, target/module directive syntax)
- https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/filter/struct.Builder.html (default directives via builder)
- https://docs.rs/tracing-subscriber/0.3.22/tracing_subscriber/fmt/struct.SubscriberBuilder.html (stderr writer, JSON formatter)
- https://docs.rs/clap/4.5.60/clap/enum.ArgAction.html (`ArgAction::Count` semantics)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - versions and capabilities verified via docs.rs.
- Architecture: HIGH - patterns are standard tracing-subscriber usage + locked project decisions.
- Pitfalls: HIGH - directly derived from subscriber defaults/constraints and global-init semantics.

**Research date:** 2026-03-05
**Valid until:** 2026-04-04

# Phase 02: Legacy Compatibility & CLI - Research

**Researched:** 2026-02-21
**Domain:** Rust CLI ergonomics + TextFSM (ntc-templates) compatibility
**Confidence:** MEDIUM (CLI patterns HIGH, TextFSM semantics HIGH; repo gap analysis MEDIUM)

This phase is already partially implemented in this repo (TextFSM loader + `cliscrape parse`). The research below is prescriptive guidance to meet the updated Phase 2 context requirements (TTY-aware defaults, multi-input via globs, structured JSON errors, template identifier resolution) while tightening TextFSM compatibility against the upstream TextFSM specification and real-world ntc-templates usage.

Key upstream semantics are defined in the official TextFSM wiki (reserved states `Start`/`EOF`/`End`, `Error` action, value flags, action grammar, comment rules). Source: https://github.com/google/textfsm/wiki/TextFSM

## Standard Stack

### Core (already in repo)

| Library | Version (repo) | Purpose | Why standard |
|---|---:|---|---|
| `clap` | 4.5.x | CLI flags/subcommands | De facto Rust CLI parser; supports derive + `try_parse` for controlled error handling (https://docs.rs/clap/latest/clap/trait.Parser.html) |
| `anyhow` | 1.0.x | App-layer error context | Ergonomic context chaining for CLI boundary (https://docs.rs/anyhow/latest/anyhow/) |
| `thiserror` | 2.0.x | Library error types | Clean typed errors for engine/template layer |
| `pest` + `pest_derive` | 2.7 | TextFSM grammar parsing | Stable PEG parser generator for Rust |
| `regex` | 1.12.x | Runtime regex | Rust regex engine used in parsing engine |
| `serde` + `serde_json` | 1.0.x | Output + structured errors | JSON output + JSON error mode |
| `csv` | 1.3.x | CSV output | Robust quoting/newlines; standard writer (https://docs.rs/csv/latest/csv/) |
| `comfy-table` | 7.1+ | Pretty tables | Widely used table renderer (https://docs.rs/comfy-table/latest/comfy_table/) |
| `toml` | 0.8/1.0 | Modern template format | TOML support for modern templates |
| `serde_yaml_ng` | 0.10 | Modern template format | YAML support with serde |
| `serde_path_to_error` | 0.1 | Better schema errors | Precise field path for YAML/TOML schema failures (https://docs.rs/serde_path_to_error/latest/serde_path_to_error/) |

### Supporting (add for Phase 2 requirements)

| Library | Recommended version | Purpose | When to Use |
|---|---:|---|---|
| `glob` | 0.3.x | Expand `--input-glob` patterns | Cross-platform Unix-style globbing, including `**` (https://docs.rs/glob/latest/glob/) |
| `strip-ansi-escapes` | 0.2.x | Transcript cleanup | Strip ANSI escape sequences before prompt/echo heuristics (https://docs.rs/strip-ansi-escapes/latest/strip_ansi_escapes/) |

### SOTA notes (important defaults)

- Use `std::io::IsTerminal` for TTY detection (stable since Rust 1.70; no `atty` dependency needed). Source: https://doc.rust-lang.org/std/io/trait.IsTerminal.html

## Architecture Patterns

### Pattern 1: CLI as an adapter layer (engine stays pure)

**What:** Keep `cliscrape` engine/template loaders free of CLI concepts (TTY detection, stdin/file resolution, JSON error envelopes). The binary (`src/main.rs`) adapts inputs/outputs and translates errors.

**When to use:** Always. It enables reuse from library API and keeps testability high.

**Concrete module responsibilities (fits this repo):**

- `src/cli.rs`: clap structs/enums only.
- `src/main.rs`: orchestration + exit code contract + stderr/stdout routing.
- `src/transcript/*`: transcript normalization (strip ANSI, prompt/echo cleanup) returning blocks + warnings.
- `src/output.rs`: pure serialization (records -> String/bytes), no I/O.
- `src/template/*` + `src/engine/*`: parsing semantics; should expose warnings separately from hard errors.

### Pattern 2: Deterministic resolution pipelines

**What:** Resolve template + inputs into explicit, ordered lists before doing any parsing. Fail fast on ambiguity or empty resolution.

**Why:** Meets: "ambiguity is an error" and "fail fast; no partial output".

**Template resolution rule (path or identifier, no search roots):**

1. If `--template` value exists on disk as given (file path), use it.
2. Else treat it as an identifier and look in the current working directory for exactly one of:
   - `<id>.textfsm`
   - `<id>.yaml` / `<id>.yml`
   - `<id>.toml`
3. If `--template-format` is not `auto`, restrict candidates to that extension set.
4. If 0 matches: error (tell user to pass a path).
5. If >1 matches: error (ambiguity); user must specify `--template-format` or a full path.

This satisfies "accept path or identifier/name" without introducing configurable template roots/search paths.

### Pattern 3: Input specs as a unified list

**What:** Model inputs as a list of `InputSpec` (files + optional stdin sentinel), where glob patterns are expanded to file paths.

**Required behavior:**

- Accept stdin and explicit inputs together.
- Support multiple inputs.
- Glob expansion is explicit via `--input-glob` (do not silently expand positional args).
- Deterministic ordering: sort expanded paths (bytewise path sort is fine).

### Pattern 4: Output defaults based on stdout TTY

**What:** Default `--format` is `auto` and selected at runtime:

- stdout is a TTY => table
- stdout is not a TTY => JSON

Use `std::io::stdout().is_terminal()` (https://doc.rust-lang.org/std/io/trait.IsTerminal.html).

### Pattern 5: Exit codes and error envelopes

**What:** Enforce Phase 2 exit contract:

- `0` success
- `1` any failure (including clap arg errors)

**Implementation detail:** use `Cli::try_parse()` (clap) and handle `clap::Error` yourself instead of `Cli::parse()`, which calls `Error::exit()` on error (https://docs.rs/clap/latest/clap/trait.Parser.html).

**Structured error mode:** add a flag like `--error-format human|json` and emit JSON errors to stderr when requested (still exit `1`).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| TTY detection | Custom `isatty` FFI / `atty` clone | `std::io::IsTerminal` | Stable std API; platform heuristics documented (https://doc.rust-lang.org/std/io/trait.IsTerminal.html) |
| Glob expansion | Manual directory walking + fnmatch | `glob` crate | Correct cross-platform semantics; supports `**` (https://docs.rs/glob/latest/glob/) |
| CSV quoting/newlines | Manual join with commas | `csv::Writer` | Handles quoting, embedded newlines, UTF-8 vs bytes (https://docs.rs/csv/latest/csv/) |
| Table rendering | Manual padding/alignment | `comfy-table` | Handles wrapping/width/layout (https://docs.rs/comfy-table/latest/comfy_table/) |
| YAML/TOML schema errors | Unstructured `serde_*` errors | `serde_path_to_error` | Includes precise failing field path (https://docs.rs/serde_path_to_error/latest/serde_path_to_error/) |
| ANSI escape stripping | Regex replace of `\x1b` sequences | `strip-ansi-escapes` | Correct handling of terminal escape sequences (https://docs.rs/strip-ansi-escapes/latest/strip_ansi_escapes/) |

## Common Pitfalls

### Pitfall 1: TextFSM comments break the Pest grammar

**What goes wrong:** Real TextFSM templates (e.g. ntc-templates) contain comment lines starting with optional whitespace + `#`. TextFSM defines comment detection as regex `^\s*#` (official wiki). If the grammar doesn't accept/ignore comments, many templates fail to load.

**Source:** https://github.com/google/textfsm/wiki/TextFSM ("A line is considered a comment...")

**How to avoid:** Ensure the Pest grammar treats comments as ignorable tokens anywhere newlines are accepted (both between `Value` lines and inside state rule blocks).

### Pitfall 2: `-> Error` is an action, not a state transition

**What goes wrong:** ntc-templates frequently use rules like `^. -> Error` (example: `alcatel_aos_show_chassis.textfsm`). TextFSM specifies a special `Error` action that aborts processing and discards all rows.

**Sources:**

- TextFSM wiki `Error Action`: https://github.com/google/textfsm/wiki/TextFSM#error-action
- Example template (ntc-templates): https://raw.githubusercontent.com/networktocode/ntc-templates/master/ntc_templates/templates/alcatel_aos_show_chassis.textfsm

**How to avoid:** Parse `Error` as a distinct action variant and implement runtime behavior: fail fast, no partial output, exit code 1.

### Pitfall 3: `Clear` vs `Clearall` semantics

**What goes wrong:** TextFSM defines `Clear` as "clear non-Filldown values" and `Clearall` as "clear all values". Treating them as the same breaks templates relying on Filldown retention.

**Source:** https://github.com/google/textfsm/wiki/TextFSM#record-actions

**How to avoid:** Represent them separately in `Action` (or separate record-action enum) and implement correct buffer clearing.

### Pitfall 4: `Continue` must not accept a state transition

**What goes wrong:** TextFSM forbids `Continue` with a state transition ("Continue action does not accept a state transition"). Allowing it can create loops and deviate from TextFSM.

**Source:** https://github.com/google/textfsm/wiki/TextFSM#new-state-transition

**How to avoid:** Reject at template load time with a human-readable error.

### Pitfall 5: Explicit `EOF` state overrides implicit record-on-EOF

**What goes wrong:** TextFSM runs an implicit `EOF` state that records the last row unless the template defines an explicit empty `EOF` state to override. Always recording at EOF breaks templates that intentionally suppress it.

**Source:** https://github.com/google/textfsm/wiki/TextFSM#reserved-states

**How to avoid:** Implement TextFSM `EOF` semantics:

- If template defines `EOF` with rules: execute them on EOF.
- If template defines empty `EOF`: do not emit implicit record.

### Pitfall 6: CSV/Table column order is nondeterministic

**What goes wrong:** Using `HashMap` key iteration for headers yields unstable ordering, and deriving headers only from the first record omits columns that appear later.

**How to avoid:** Compute the union of keys across all records and sort once (e.g., `BTreeSet<String>`).

### Pitfall 7: clap arg errors violate the 0/1 exit-code contract

**What goes wrong:** `Cli::parse()` exits internally on parse errors (clap does this by design). If you need "1 for any failure", you must intercept clap errors.

**Source:** clap `Parser::parse()` docs: "exit on error" (https://docs.rs/clap/latest/clap/trait.Parser.html)

**How to avoid:** Use `Cli::try_parse()` and call `std::process::exit(1)` yourself after printing.

## Code Examples

### TTY-aware format default (`auto`)

```rust
use std::io::{self, IsTerminal};

#[derive(Copy, Clone, Debug)]
enum OutputFormat { Auto, Json, Csv, Table }

fn choose_format(requested: OutputFormat) -> OutputFormat {
    match requested {
        OutputFormat::Auto => {
            if io::stdout().is_terminal() {
                OutputFormat::Table
            } else {
                OutputFormat::Json
            }
        }
        other => other,
    }
}
// Source: std::io::IsTerminal (https://doc.rust-lang.org/std/io/trait.IsTerminal.html)
```

### Enforce exit codes 0/1 with clap

```rust
use clap::Parser;

fn main() {
    let cli = match Cli::try_parse() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };

    if let Err(err) = run(cli) {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
// Source: clap::Parser::try_parse (https://docs.rs/clap/latest/clap/trait.Parser.html)
```

### Expand globs deterministically (and error on zero matches)

```rust
use glob::glob;
use std::path::PathBuf;

fn expand_input_glob(pattern: &str) -> anyhow::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in glob(pattern)? {
        out.push(entry?);
    }
    out.sort();
    if out.is_empty() {
        anyhow::bail!("input glob matched no files: {pattern}");
    }
    Ok(out)
}
// Source: glob crate supports /path/**/*.ext (https://docs.rs/glob/latest/glob/)
```

### Template spec resolution (path or identifier; ambiguity is error)

```rust
use std::path::{Path, PathBuf};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum TemplateFormat { Auto, Textfsm, Yaml, Toml }

fn resolve_template(spec: &str, format: TemplateFormat) -> anyhow::Result<PathBuf> {
    let p = Path::new(spec);
    if p.exists() {
        return Ok(p.to_path_buf());
    }

    let exts: &[&str] = match format {
        TemplateFormat::Auto => &["textfsm", "yaml", "yml", "toml"],
        TemplateFormat::Textfsm => &["textfsm"],
        TemplateFormat::Yaml => &["yaml", "yml"],
        TemplateFormat::Toml => &["toml"],
    };

    let mut candidates = Vec::new();
    for ext in exts {
        let cand = PathBuf::from(format!("{spec}.{ext}"));
        if cand.exists() {
            candidates.push(cand);
        }
    }

    match candidates.len() {
        0 => anyhow::bail!("template not found: {spec} (pass a path, or place {spec}.textfsm/.yaml/.toml in the current directory)"),
        1 => Ok(candidates.remove(0)),
        _ => anyhow::bail!("template identifier is ambiguous: {spec} (matches: {:?}); pass --template-format or a full path", candidates),
    }
}
```

### Structured JSON error envelope to stderr

```rust
use serde::Serialize;

#[derive(Serialize)]
struct JsonError<'a> {
    ok: bool,
    message: &'a str,
    kind: &'a str,
}

fn emit_json_error(message: &str, kind: &str) {
    let payload = JsonError { ok: false, message, kind };
    // stderr by requirement
    let _ = serde_json::to_writer(std::io::stderr(), &payload);
    let _ = std::io::Write::write_all(&mut std::io::stderr(), b"\n");
}
```

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH (all items from docs.rs/std docs)
- Architecture patterns: HIGH (aligned with established Rust CLI conventions + current repo layout)
- TextFSM semantics: HIGH (official wiki; confirmed by real ntc-templates examples)
- Repo gap analysis: MEDIUM (based on spot-checking current implementation; full corpus not tested here)

**Primary sources:**

- TextFSM specification (actions, reserved states, comments): https://github.com/google/textfsm/wiki/TextFSM
- clap `Parser::{parse,try_parse}` behavior: https://docs.rs/clap/latest/clap/trait.Parser.html
- Rust `std::io::IsTerminal`: https://doc.rust-lang.org/std/io/trait.IsTerminal.html
- glob crate semantics and `**` example: https://docs.rs/glob/latest/glob/
- csv crate writer behavior: https://docs.rs/csv/latest/csv/
- comfy-table crate: https://docs.rs/comfy-table/latest/comfy_table/
- strip ANSI escapes: https://docs.rs/strip-ansi-escapes/latest/strip_ansi_escapes/

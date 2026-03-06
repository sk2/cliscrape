# Phase 11: Documentation & Authoring Guide - Research

**Researched:** 2026-03-05
**Domain:** Rust CLI documentation + template authoring docs + executable doc examples (doc tests)
**Confidence:** HIGH (repo-grounded for CLI/template behavior; MEDIUM for doc-test tooling selection)

## Summary

This phase is primarily an information-architecture + tooling phase: write user-facing docs (README entry + `docs/` guides) that explain how to select and use templates, how to author templates (modern YAML/TOML + legacy TextFSM), how to migrate, and how to troubleshoot errors. The critical implementation detail is that docs must be *executable* in CI: examples are treated as tests, and the auto-generated template catalog (`docs/templates.md`) is regenerated and compared in CI so the committed output never drifts.

The repo already has the building blocks needed for accurate docs: `cliscrape list-templates` emits JSON for all embedded templates (including subdirectories like `modern/...`), `show-template` prints metadata + field list, `parse` supports stdin/files/globs, and error/log formatting knobs exist (`--error-format`, `--log-format`, `RUST_LOG`, `-v`). Modern templates have a strict schema with helpful path-aware errors (via `serde_path_to_error`), and the engine surfaces common failure modes with stable strings suitable for troubleshooting sections.

**Primary recommendation:** Use `trycmd` to validate command examples embedded in Markdown, and enforce `docs/templates.md` determinism by generating it from `cliscrape list-templates --format json` and comparing the generated content to the committed file in a test run by CI.

## Standard Stack

### Core
| Library/Tool | Version (observed) | Purpose | Why Standard |
|---|---:|---|---|
| Markdown (`README.md`, `docs/*.md`) | N/A | User docs | Lowest-friction for CLI users; GitHub-native rendering |
| `trycmd` | 1.0.1 (docs.rs latest) | Test CLI examples in `.md` / `.trycmd` | Purpose-built for “docs-as-tests”; supports Markdown fenced blocks and snapshot workflow |
| `assert_cmd` | 2.0 (Cargo.toml) | Targeted CLI integration tests | Already in repo; good for one-off assertions when trycmd isn’t a fit |
| `insta` / `cargo-insta` | insta 1.x (Cargo.toml) | Snapshot validation suite | Already in CI; consistent approach for output regression testing |

### Supporting
| Library/Tool | Version (observed) | Purpose | When to Use |
|---|---:|---|---|
| `clap` | 4.5.x (Cargo.toml: 4.5.58) | CLI help/version behavior | Reference for help text; examples can be validated via trycmd |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 (Cargo.toml) | Structured logs to stderr | Docs should include troubleshooting for `-v`, `RUST_LOG`, `--log-format json` |
| `xdg` | 3.0.0 (Cargo.toml) | Template lookup paths | Docs should tell users where to place user templates/overrides |
| `rust-embed` | 8.11.0 (Cargo.toml) | Embedded template library | Source of truth for what ships in the binary (used by `list-templates`) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|---|---|---|
| `trycmd` | custom Markdown parser + `assert_cmd` | High brittleness; re-invent snapshot/update workflow; hard to keep stable across OS |
| a Rust docgen binary | shell script (bash/python) | Extra runtime dependency and portability concerns; Rust keeps tooling single-stack |

**Installation (dev-dependency):**
```bash
cargo add --dev trycmd@1
```

## Architecture Patterns

### Recommended Documentation Structure
Preserve the locked decisions: README as entry; deep guides in `docs/`; troubleshooting embedded into each guide; templates catalog at `docs/templates.md`.

Suggested concrete layout:
```
README.md
docs/
  usage.md                 # DOC-01: selection + usage (pipelines-first)
  template-authoring.md     # DOC-03: modern + legacy; recipes; regex tips; migration
  templates.md              # DOC-02: generated catalog (committed)
  _fixtures/
    ...                     # tiny inputs/templates used by doc tests (optional)
tests/
  cli_docs_trycmd.rs        # DOC-05: trycmd runner for README/docs
  docs_templates_md.rs      # DOC-02: generate templates.md in-memory and compare
src/bin/
  docgen_templates_md.rs    # (optional) generator used by tests and for manual regen
```

### Pattern 1: Docs-As-Tests With `trycmd`
**What:** Put runnable CLI transcripts directly in Markdown fenced blocks (e.g. ` ```console `) and run them in CI with `trycmd`.

**When to use:** Any guide that includes copy/paste commands + expected output (especially troubleshooting sections).

**Example:**
```rust
// Source: https://docs.rs/trycmd/latest/trycmd/
#[test]
fn cli_docs() {
    trycmd::TestCases::new()
        // trycmd understands Markdown fenced blocks and will ignore non-test content
        .case("README.md")
        .case("docs/*.md");
}
```

Doc authoring convention to make examples testable:
- Use fenced blocks with `console` (or `trycmd`) info string.
- Use `$ cliscrape ...` prompt lines; include expected output lines after.
- Prefer deterministic commands: use `--format json` (stdout) and avoid `-v` unless you are explicitly testing logging; keep warnings out of “happy path” examples.

### Pattern 2: Generated Catalog Enforced By Tests
**What:** Generate `docs/templates.md` from `cliscrape list-templates --format json`, but enforce the committed output in CI by comparing generated content to the on-disk file.

**When to use:** Always. The locked decision requires committed output + CI enforcement.

**Implementation constraints from repo behavior (must document + encode):**
- Embedded templates include subdirectories (`modern/...`) (verified by `list-templates` output).
- `list-templates` JSON provides: `name`, `description`, `compatibility`, `version`, `author`, `maintainer`, `source`.
- Catalog requirements add: `format` and `vendor` (derive from template path/name).

Vendor/format derivation (deterministic, no runtime changes required):
- `format`: from extension (`.yaml`/`.toml`/`.textfsm`).
- `vendor`:
  - if `name` contains `/`, vendor = first path segment (e.g. `modern/ios_show_interfaces.yaml` -> `modern`)
  - else vendor = prefix before first `_` (e.g. `cisco_ios_show_version.yaml` -> `cisco`)
  - fallback `unknown`

### Anti-Patterns to Avoid
- **Hand-parsing Markdown for tests:** use `trycmd`’s Markdown support rather than building a parser.
- **Non-deterministic examples:** avoid including timing, host-specific paths, or warning-producing parses in “golden output” examples.
- **Relying on `list-templates` to enumerate user templates:** current implementation only enumerates embedded templates; docs should reflect this.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---|---|---|---|
| Validating CLI examples in Markdown | custom runner that scrapes code blocks and diffs output | `trycmd` | Handles Markdown, snapshots, elision, temp dirs; stable across platforms |
| Snapshot update workflow | bespoke “accept” scripts | `TRYCMD=overwrite` (trycmd) / `cargo insta` workflow | Familiar Rust snapshot ergonomics; consistent CI enforcement |
| Template catalog formatting | manual, hand-edited `docs/templates.md` | generator + test compare | Prevents drift; “validate everything” requirement |

**Key insight:** executable documentation needs a harness that is designed for snapshots and portability; otherwise examples rot quickly and become a CI burden.

## Common Pitfalls

### Pitfall 1: Examples leak logs/warnings into expected output
**What goes wrong:** Output differs between runs because logs (stderr) or warnings are emitted.
**Why it happens:** default log level is WARN; non-happy-path inputs can trigger `low_coverage` warnings; `--log-format json` changes formatting.
**How to avoid:** Use “clean” fixtures for docs; set `--format json` for stdout; avoid `-v` except in logging sections; when asserting logs, use `trycmd` elisions (`...`, `[..]`).
**Warning signs:** CI failures in doc tests with mismatched stderr; example outputs showing warnings.

### Pitfall 2: Vendor grouping becomes inconsistent
**What goes wrong:** New templates added under subdirectories or without `vendor_...` prefix break grouping.
**Why it happens:** vendor is not a first-class metadata field today.
**How to avoid:** Encode vendor derivation rules in generator + a unit test; document the rule in `docs/templates.md` header.
**Warning signs:** templates appear under “unknown” vendor unexpectedly.

### Pitfall 3: Modern template schema errors are confusing without path guidance
**What goes wrong:** Authors see errors like `fields.speed.type` but don’t know how to map them to YAML/TOML structure.
**Why it happens:** schema is strict (`deny_unknown_fields`) and uses `serde_path_to_error` path formatting.
**How to avoid:** Troubleshooting sections should include a short “how to read schema paths” explainer and examples.
**Warning signs:** repeated user confusion around “schema error” and “unknown field type”.

### Pitfall 4: TextFSM migration loses semantics
**What goes wrong:** Converted templates behave differently (notably `Clearall` / `Error` actions) or lose metadata.
**Why it happens:** Modern schema currently supports `record: clear` but not `clearall`/`error`, and converter emits `metadata: None`.
**How to avoid:** Migration guide must call out unsupported actions and require manual review in Live Lab after conversion.
**Warning signs:** doc examples show different record boundaries after migration.

## Code Examples

### 1) `trycmd` runner for README + docs
```rust
// Source: https://docs.rs/trycmd/latest/trycmd/
#[test]
fn cli_docs() {
    trycmd::TestCases::new()
        .case("README.md")
        .case("docs/*.md");
}
```

### 2) Modern template “recipe” (single-state patterns)
Repo-grounded schema (from `src/template/modern.rs`): `version`, `fields`, and exactly one of `patterns` or `states`.
```yaml
# Source: repo schema in src/template/modern.rs
metadata:
  description: "Parse 'show version'"
  compatibility: "Cisco IOS 15.x"
  version: "1.0.0"
  author: "Your Name"

version: 1
fields:
  hostname: { type: string }
patterns:
  - regex: '^(?P<hostname>\S+) uptime is'
    record: true
```

### 3) Legacy TextFSM metadata header (comment-based)
```text
# Description: Parse BGP neighbors
# Compatibility: Arista EOS 4.x
# Version: 3.0.1
# Author: NetOps

Value NEIGHBOR (\S+)
Start
  ^Neighbor ${NEIGHBOR} -> Record
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|---|---|---|---|
| Docs as prose only | Docs as executable examples (`trycmd`) | N/A (to be implemented in Phase 11) | Prevents example rot; supports “validate everything” |
| Hand-maintained template list | Generated `docs/templates.md` enforced by CI | N/A (to be implemented in Phase 11) | Catalog stays accurate as templates evolve |

**Deprecated/outdated (repo-local):**
- `README.md` roadmap section is stale relative to `.planning/ROADMAP.md` (should be replaced with a docs entrypoint).

## Open Questions

1. **Should the catalog include templates that have default metadata ("No description available")?**
   - What we know: embedded templates include `modern/...` entries without metadata; `list-templates` will show defaults.
   - What's unclear: whether to treat those as “examples/internal” vs “supported templates.”
   - Recommendation: include all embedded templates but label ones with default metadata as “(no metadata provided)” in catalog output.

2. **Exact strictness of troubleshooting expected output in docs**
   - What we know: locked decision is “validate everything”; trycmd supports elisions for variable output.
   - What's unclear: whether to snapshot full outputs or assert only key lines.
   - Recommendation: snapshot full outputs for stable commands; use elisions only for paths and platform differences.

## Sources

### Primary (HIGH confidence)
- Repo code:
  - `src/main.rs` (template resolution, list/show templates, parse options, error formatting)
  - `src/template/modern.rs` (modern template schema + validation)
  - `src/engine/fsm.rs` (FSM semantics, timeout, threshold warnings/errors)
  - `src/engine/macros.rs` (builtin + local macros)
  - `src/template/metadata.rs` (metadata extraction rules)

### Secondary (MEDIUM confidence)
- trycmd docs.rs: https://docs.rs/trycmd/latest/trycmd/
- rust-embed docs.rs: https://docs.rs/rust-embed/latest/rust_embed/
- clap docs.rs: https://docs.rs/clap/latest/clap/

### Tertiary (LOW confidence)
- GitHub UI rendering of snapbox repo (confirming relationship to trycmd): https://github.com/assert-rs/snapbox

## Metadata

**Confidence breakdown:**
- Standard stack: MEDIUM - trycmd selection verified via docs.rs, but not yet adopted in repo.
- Architecture: HIGH - derived directly from locked decisions + existing CLI/template behavior.
- Pitfalls: HIGH - derived from observed error/warning strings and schema/engine constraints.

**Research date:** 2026-03-05
**Valid until:** 2026-04-04 (30 days)

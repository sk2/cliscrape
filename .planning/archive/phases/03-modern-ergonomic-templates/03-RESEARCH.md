# Phase 03: Modern Ergonomic Templates - Research

**Researched:** 2026-02-20
**Domain:** Rust template loaders (YAML/TOML), strict schema validation, typed capture conversion, Cisco IOS prompt/echo handling
**Confidence:** MEDIUM

## Summary

Phase 03 is primarily a *template document layer* that compiles down into the existing `TemplateIR` / `Template` engine: implement a modern YAML/TOML schema (strictly validated), convert it to `TemplateIR`, then run the existing FSM engine unchanged except for adding typed conversion at record emission time.

The key planning choices are (1) a schema that supports both explicit FSM states and a simpler pattern-only mode, (2) a field model that can represent both TextFSM-style `${Value}` placeholders *and* native regex named groups, (3) a typed conversion pipeline that is explicit-first with heuristic fallback and never drops records, and (4) a conservative transcript pre-processor that detects Cisco IOS-style prompts, strips prompts + command echoes, and segments multi-command transcripts into command blocks.

**Primary recommendation:** Treat YAML/TOML templates as an input DSL that compiles to `TemplateIR`, and implement type conversion + prompt handling as separate pre/post steps around the existing engine.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `serde` | 1.x (already in repo) | Deserialize schema into Rust structs | Ecosystem standard for config/schema parsing |
| `toml` | 1.0.3+spec-1.1.0 | TOML template parsing via `toml::from_str` | Mature, spec-aligned, serde-compatible |
| `serde_yaml_ng` | 0.10.0 | YAML template parsing via `serde_yaml_ng::from_str` | Maintained serde-compatible YAML; `serde_yaml` is archived/deprecated |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `serde_path_to_error` | 0.1.20 | Better error messages with paths like `fields.speed.type` | Always for schema parse errors (both YAML and TOML) |
| `dialoguer` | 0.12.0 | Interactive TextFSM -> modern conversion prompts | For the required interactive converter CLI |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `serde_yaml_ng` | `serde_yaml` | `serde_yaml` is archived and marked “no longer maintained” (avoid) |
| `dialoguer` | `inquire` | `dialoguer` is widely used and sufficient for simple flows (keep dependency surface small) |

**Installation:**
```bash
cargo add toml@1 serde_yaml_ng@0.10 serde_path_to_error@0.1 dialoguer@0.12
```

## Architecture Patterns

### Recommended Project Structure
Keep the engine as-is; add a modern template front-end + transcript preprocessor:

```
src/
├── template/
│   ├── loader.rs              # legacy TextFSM loader (already)
│   ├── modern.rs              # YAML/TOML schema structs + loader
│   └── mod.rs
├── transcript/
│   ├── mod.rs
│   └── ios_prompt.rs          # prompt/echo detection + segmentation
└── engine/
    ├── fsm.rs                 # compile TemplateIR -> Template (already)
    ├── records.rs             # add typed conversion on emit
    └── types.rs               # extend Value metadata (type info)
```

### Pattern 1: “Schema -> TemplateIR Compiler”
**What:** Parse YAML/TOML into strict Rust structs, validate invariants, then lower into `TemplateIR` so the rest of the system remains unchanged.
**When to use:** Always for modern templates.
**Example:**
```rust
// Source: https://docs.rs/toml/latest/toml/ (toml::from_str)
// Source: https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/ (serde_yaml_ng::from_str)

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct ModernTemplate {
    version: u32,
    // ... fields/macros/states/patterns ...
}

fn load_modern(path: &std::path::Path, format: TemplateFormat) -> anyhow::Result<ModernTemplate> {
    let s = std::fs::read_to_string(path)?;
    match format {
        TemplateFormat::Toml => {
            let mut de = toml::Deserializer::new(&s);
            let doc: ModernTemplate = serde_path_to_error::deserialize(&mut de)?;
            Ok(doc)
        }
        TemplateFormat::Yaml => {
            // serde_yaml_ng has from_str; for path-aware errors, wrap a Deserializer.
            let mut de = serde_yaml_ng::Deserializer::from_str(&s);
            let doc: ModernTemplate = serde_path_to_error::deserialize(&mut de)?;
            Ok(doc)
        }
    }
}
```

### Pattern 2: “Typed Conversion at Record Emission”
**What:** Keep capture extraction as strings, but convert *only when emitting* into `serde_json::Value` using per-field type metadata + heuristics.
**When to use:** Always; it localizes conversion policy and keeps the regex engine simple.
**Example:**
```rust
// Source: https://docs.rs/regex/latest/regex/ (named capture groups)
// RecordBuffer already emits HashMap<String, serde_json::Value>.
// Extend Value to include an optional type hint; convert strings here.
```

### Pattern 3: “Transcript Pre-Processing”
**What:** Before parsing, normalize raw transcripts: detect prompt lines, strip prompts, strip command echoes, split into command blocks.
**When to use:** When CLI input is a raw transcript (not just command output).

### Anti-Patterns to Avoid
- **Parsing YAML/TOML into untyped `Value` blobs:** it makes strict validation and good errors much harder; deserialize into structs with `deny_unknown_fields`.
- **Converting types during regex capture:** capture extraction should stay string-based; conversion is a post-processing concern.
- **Aggressive prompt stripping:** false positives corrupt data; only strip prompts when confidence is high.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| YAML parsing | Custom YAML parser | `serde_yaml_ng` | YAML is large/edge-casey; serde integration is standard |
| TOML parsing | Custom TOML parser | `toml` | Spec-compliant, serde-friendly |
| Error paths | Manual breadcrumb tracking | `serde_path_to_error` | Great UX: points at the failing field precisely |
| Interactive prompts | Custom stdin parsing | `dialoguer` | Correct terminal behavior + validation helpers |

**Key insight:** This phase is glue code; leverage serde + existing crates so planning focuses on schema + lowering + invariants.

## Common Pitfalls

### Pitfall 1: YAML 1.1 implicit typing surprises
**What goes wrong:** Unquoted scalars like `on`, `off`, `yes`, `no`, `1e3`, or `01` may be interpreted as non-strings under YAML 1.1, causing deserialization errors when your schema expects `String`.
**Why it happens:** `serde_yaml_ng` documents YAML 1.1 support.
**How to avoid:** Document “quote all regex strings and command strings” in examples; prefer YAML block scalars (`|`) for regexes.
**Warning signs:** Errors like “invalid type: boolean, expected a string” at paths like `states.Start.rules[0].regex`.

### Pitfall 2: Rust `regex` feature gaps vs PCRE/Python
**What goes wrong:** Templates using look-around or backreferences fail to compile.
**Why it happens:** Rust `regex` explicitly lacks look-around and backreferences.
**How to avoid:** Validate regex compilation during template load; error early with the expanded final regex string.
**Warning signs:** Compile errors from `Regex::new`.

### Pitfall 3: Unknown-key strictness doesn’t recurse unless you apply it everywhere
**What goes wrong:** Some nested schema structs accept unknown keys silently.
**Why it happens:** `deny_unknown_fields` must be applied per-struct/enum.
**How to avoid:** Put `#[serde(deny_unknown_fields)]` on every schema type; add tests with misspelled keys at multiple nesting levels.
**Warning signs:** Typos in templates don’t error.

### Pitfall 4: Macro expansion order and recursion
**What goes wrong:** `{{macro}}` placeholders inside macro bodies don’t expand (or expand unpredictably) and can create cycles.
**Why it happens:** Current `expand_macros` is a single-pass replacement over a `HashMap` (unordered) and is non-recursive.
**How to avoid:** Define macro expansion semantics explicitly for modern templates (recommended: recursive expansion with max depth + cycle detection) or document “no macro-in-macro” as a constraint.
**Warning signs:** Expanded regex still contains `{{...}}` tokens.

### Pitfall 5: Prompt stripping false positives
**What goes wrong:** Output lines that look like `something#` get stripped.
**Why it happens:** Prompt regex too permissive.
**How to avoid:** Require prompt to appear at start-of-line, end with `#` or `>`, and (preferably) repeat consistently; keep lines when uncertain.
**Warning signs:** Missing lines in parsed output; segment boundaries in unexpected places.

## Code Examples

### Modern Template (YAML) with states + typed fields
```yaml
# Source: repo Phase 03 decisions (schema must be strict, YAML supported)
version: 1
macros:
  # shadows builtin if same name
  interface: "\\S+"

fields:
  interface:
    type: string
    required: true
  speed:
    type: int

states:
  Start:
    - regex: '^${interface} is up, line protocol is up'
      action: { record: none }
    - regex: '^  BW (?P<speed>[0-9,]+) Kbit'
      action: { record: record }
```

### Pattern-only Mode (no explicit states)
```toml
# Source: TOML crate docs (TOML parsing via toml::from_str)
version = 1

[fields]
hostname = { type = "string" }
uptime_s = { type = "duration" }

[[patterns]]
regex = '^Hostname: (?P<hostname>\\S+)$'
record = false

[[patterns]]
regex = '^Uptime: (?P<uptime_s>\\d+)s$'
record = true
```

### Cisco IOS Prompt Detection (recommended regex shape)
```text
# Goal: match these (examples)
Router#
Router>
Router(config)#
Router(config-if)#
Router(config-if-range)#

# Prompt line with command echo
Router# show interfaces
```

Recommended (conservative) prompt regex for line-start detection:

```rust
// Not from an official Cisco doc; this is a practical, conservative matcher.
// Keep it narrow: start-of-line, hostname-ish, optional mode parens, then [#>].
let prompt_re = regex::Regex::new(
    r"^(?P<prompt>[A-Za-z0-9_.:-]+(?:\([^\r\n\)]*\))?[#>])\s*(?P<cmd>.*)$"
)?;
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `serde_yaml` | `serde_yaml_ng` | `serde_yaml` archived 2024-03-25 | Avoid depending on unmaintained YAML crate |
| Untyped string captures | Typed output conversion (explicit + heuristics) | Phase 03 requirement (FORM-03) | JSON output becomes more useful downstream |
| Treat transcript as raw lines | Segment by prompt/echo into command blocks | Phase 03 requirement (CLI-02) | Multi-command transcripts become parseable |

**Deprecated/outdated:**
- `serde_yaml` (crate archived; docs label it “no longer maintained”).

## Open Questions

1. **Rich types representation in JSON**
   - What we know: numbers/bools are natural JSON primitives; IP/MAC/duration/bytes need policy.
   - What's unclear: should `ip`/`mac` stay strings (validated/normalized) or emit structured objects?
   - Recommendation: emit JSON primitives where possible (`int`, `float`, `bool`, `bytes->u64`, `duration->seconds`), otherwise emit normalized strings.

2. **Heuristic inference scope**
   - What we know: explicit-per-field type is preferred; heuristics are fallback.
   - What's unclear: whether to infer booleans from domain strings like `up/down`.
   - Recommendation: infer only numeric-ish patterns by default; keep bool inference opt-in to avoid semantic mistakes.

3. **Prompt detection confidence model**
   - What we know: must be conservative; strip prompts + command echoes only.
   - What's unclear: the minimum evidence to strip when only one prompt occurs.
   - Recommendation: strip when prompt matches at least twice (start and end) OR when a prompt+command line is followed by a prompt-only line later; otherwise keep input unchanged.

4. **Macro expansion semantics**
   - What we know: local macros override builtins.
   - What's unclear: whether to support recursive macros in Phase 03.
   - Recommendation: implement recursive expansion with a small max depth (e.g., 10) and cycle detection; error on cycles.

## Sources

### Primary (HIGH confidence)
- https://docs.rs/toml/latest/toml/ - parsing/deserializing TOML via serde
- https://docs.rs/serde_yaml_ng/latest/serde_yaml_ng/ - YAML parsing API + YAML 1.1 note
- https://docs.rs/serde_path_to_error/latest/serde_path_to_error/ - path-aware deserialization errors
- https://docs.rs/dialoguer/latest/dialoguer/ - interactive CLI prompts
- https://docs.rs/regex/latest/regex/ - named captures; unsupported look-around/backreferences

### Secondary (MEDIUM confidence)
- https://github.com/dtolnay/serde-yaml - repository archived + “no longer maintained” note (motivates YAML crate choice)

### Tertiary (LOW confidence)
- Cisco IOS prompt regex specifics: no authoritative spec sourced; validate against real transcripts during implementation.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - crate docs clearly indicate current versions and deprecation status
- Architecture: MEDIUM - aligns with current repo structure; final schema details still to be finalized in planning
- Pitfalls: MEDIUM - YAML 1.1 typing + Rust regex limitations are well-sourced; prompt heuristics need validation

**Research date:** 2026-02-20
**Valid until:** 2026-03-20

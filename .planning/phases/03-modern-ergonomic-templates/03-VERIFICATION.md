---
phase: 03-modern-ergonomic-templates
verified: 2026-02-20T22:04:27Z
status: human_needed
score: 8/9 must-haves verified
human_verification:
  - test: "Run interactive converter end-to-end"
    expected: "`cliscrape convert -i some.textfsm` prompts for format/path, writes file, and the written YAML/TOML loads via `cliscrape parse -t out.yaml`"
    why_human: "Interactive TTY prompt flows (dialoguer) and overwrite confirmation can’t be proven via static inspection"
---

# Phase 3: Modern Ergonomic Templates Verification Report

**Phase Goal:** Support YAML/TOML template formats with automatic type conversion and basic prompt handling.
**Verified:** 2026-02-20T22:04:27Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | User can define a template in YAML and load it by extension | ✓ VERIFIED | `cliscrape::FsmParser::from_file` supports `.yaml`/`.yml` in `src/lib.rs`; tests in `src/lib.rs` and `tests/modern_templates.rs` load YAML templates |
| 2 | User can define a template in TOML and load it by extension | ✓ VERIFIED | `cliscrape::FsmParser::from_file` supports `.toml` in `src/lib.rs`; tests in `src/lib.rs` and `tests/modern_templates.rs` load TOML templates |
| 3 | Numeric captures are automatically converted to JSON numbers when template declares `type: int` | ✓ VERIFIED | Conversion pipeline `src/engine/convert.rs` + emit-time application in `src/engine/records.rs`; asserted in `src/template/modern.rs` tests and `tests/modern_templates.rs` (`mtu` = `1500`) |
| 4 | Explicit `type: string` preserves captured values as JSON strings (even numeric-looking) | ✓ VERIFIED | `convert_scalar(..., Some(FieldType::String))` in `src/engine/convert.rs`; asserted in `src/engine/records.rs` and `src/template/modern.rs` tests |
| 5 | When int conversion fails, original captured value is preserved as a string | ✓ VERIFIED | Fallback behavior in `src/engine/convert.rs`; asserted in `src/engine/records.rs` (`"12x"` stays string) |
| 6 | CLI correctly handles Cisco IOS-style prompts/echo in raw transcripts (strip + segment) | ✓ VERIFIED | `src/transcript/ios_prompt.rs` provides `preprocess_ios_transcript`; `src/main.rs` preprocesses input before parsing blocks; unit tests cover multi-command segmentation + stripping |
| 7 | Prompt stripping is conservative: when confidence is low, input is left unchanged | ✓ VERIFIED | Confidence gate in `src/transcript/ios_prompt.rs` returns `vec![raw.to_string()]`; negative test `does_not_trigger_on_single_prompt_like_line_when_confidence_is_low` |
| 8 | Modern templates are strictly validated with path-aware schema errors | ✓ VERIFIED | `#[serde(deny_unknown_fields)]` on schema structs in `src/template/modern.rs` + `serde_path_to_error::deserialize(...)`; tests assert path strings like `fields.speed.type` for both TOML and YAML |
| 9 | Users can convert an existing `.textfsm` template into a best-effort modern YAML/TOML template via an interactive CLI | ? UNCERTAIN | `cliscrape convert` exists in `src/cli.rs` + `src/main.rs` and uses `dialoguer` prompts, but interactive TTY flow is not validated by tests |

**Score:** 8/9 truths verified

## Required Artifacts

| Artifact | Expected | Status | Details |
|---------|----------|--------|---------|
| `src/template/modern.rs` | Strict YAML/TOML schema + lowering to `TemplateIR` | ✓ VERIFIED | Exists (585 lines); strict serde schema + invariant validation + lowering; used by `src/lib.rs` and tests |
| `src/engine/convert.rs` | String -> typed `serde_json::Value` conversion | ✓ VERIFIED | Exists (62 lines); `pub fn convert_scalar`; explicit hint wins + numeric heuristic fallback; covered by unit tests |
| `src/engine/records.rs` | Record emission applies conversion (scalar + list) | ✓ VERIFIED | Exists (275 lines); calls `convert_scalar` at emit-time; extensive unit tests |
| `src/engine/types.rs` | Value metadata carries `type_hint` | ✓ VERIFIED | Exists (61 lines); defines `FieldType` + `Value.type_hint`; consumed by modern lowering + record emission |
| `src/transcript/ios_prompt.rs` | IOS prompt/echo detection + segmentation | ✓ VERIFIED | Exists (167 lines); conservative detection + block segmentation; unit tests |
| `src/main.rs` | CLI parse path uses transcript preprocessing + template format override | ✓ VERIFIED | Calls `transcript::preprocess_ios_transcript`; maps CLI `--template-format` to `FsmParser::from_file_with_format` |
| `templates/modern/ios_show_interfaces.yaml` | Starter YAML modern template | ✓ VERIFIED | Exists; used in `tests/modern_templates.rs` |
| `templates/modern/simple_hostname.toml` | Starter TOML modern template | ✓ VERIFIED | Exists; used in `tests/modern_templates.rs` |
| `tests/modern_templates.rs` | E2E tests: modern template load + typed output + CLI override | ✓ VERIFIED | Exists; runs `cliscrape` binary via `assert_cmd` and parses JSON output |
| `src/template/convert.rs` | TextFSM `TemplateIR` -> modern document conversion | ✓ VERIFIED | Exists (160 lines); round-trip tests through YAML/TOML loaders prove output is loadable |

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/engine/records.rs` | `src/engine/convert.rs` | `convert_scalar(...)` in `RecordBuffer::emit` | ✓ WIRED | Scalars + list elements call converter using `Value.type_hint` |
| `src/template/modern.rs` | `src/engine/types.rs` | Lowering sets `Value.type_hint` | ✓ WIRED | `FieldTypeDef` lowered to `FieldType`; defaults omitted type to `string` with `Some(FieldType::String)` |
| `src/lib.rs` | `src/template/modern.rs` | Extension-based selection (`.yaml/.yml/.toml`) | ✓ WIRED | `FsmParser::from_file` calls `modern::load_*_str` |
| `src/main.rs` | `src/transcript/mod.rs` | Preprocess input blocks before parse | ✓ WIRED | `transcript::preprocess_ios_transcript(&input_content)` then parse each block |
| `src/main.rs` | `src/lib.rs` | Template format override | ✓ WIRED | `--template-format` maps to `FsmParser::from_file_with_format(..., TemplateFormat::{Textfsm,Yaml,Toml})` |
| `src/template/modern.rs` | `src/engine/macros.rs` | `TemplateIR.macros` -> `expand_macros` | ✓ WIRED | Lowering copies `doc.macros`; `Template::from_ir` expands with local overrides (shadow builtins); recursion + cycle detection in `src/engine/macros.rs` |

## Requirements Coverage

| Requirement | Status | Blocking Issue |
|------------|--------|----------------|
| FORM-02 (Modern YAML/TOML template format) | ✓ SATISFIED | — |
| FORM-03 (Typed captures + automatic conversion) | ✓ SATISFIED | — |
| CLI-02 (Basic Cisco IOS prompt support) | ✓ SATISFIED | — |

## Anti-Patterns Found

No blocker stub patterns found in the Phase 03 implementation artifacts (no `TODO`/`FIXME` placeholders in core paths; no empty handlers).

## Human Verification Required

### 1. Interactive converter smoke test

**Test:** Run `cargo run -- convert -i path/to/template.textfsm` (omit `--output` and `--format`) and follow prompts.
**Expected:** Prompts appear for format + output path; file is written; `cargo run -- parse -t out.yaml input.txt` (or TOML) loads successfully.
**Why human:** Interactive prompt UX + overwrite confirmation is runtime/TTY-dependent.

---

_Verified: 2026-02-20T22:04:27Z_
_Verifier: Claude (gsd-verifier)_

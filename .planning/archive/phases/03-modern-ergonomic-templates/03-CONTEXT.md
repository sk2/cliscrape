# Phase 3: Modern Ergonomic Templates - Context

**Gathered:** 2026-02-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver a modern ergonomic template format (YAML/TOML) as an alternative to legacy TextFSM, with:
- automatic type conversion in output, and
- basic prompt handling for raw CLI transcripts.

This phase is about template format + parsing behavior. It is not about adding new transport layers, full TUI features, or expanding the legacy TextFSM engine.

</domain>

<decisions>
## Implementation Decisions

### Template format support
- Support BOTH YAML and TOML, with the same schema expressed in either.
- Template format selection: infer by file extension by default; allow an explicit CLI flag to override.

### Template structure (ergonomics)
- Hybrid structure:
  - allow explicit TextFSM-like state machines (named states, ordered rules) for complex cases
  - also allow a simpler "pattern-only" mode (no explicit states) for straightforward parsing.

### Macros / shared patterns
- Keep builtin macros (e.g., `{{ipv4}}`, `{{mac_address}}`).
- Allow template-local macros and allow them to override/shadow builtins.

### Captures / field definitions
- Support BOTH capture styles:
  - a value-definition section + `${Name}` placeholders, and
  - direct named captures in rule regex (e.g., `(?P<speed>\d+)`).

### Type conversion (output)
- Support rich types (beyond int/float/bool), with an explicit-per-field type declaration preferred and heuristic inference as fallback.
- If conversion fails, keep the original captured value as a string (do not error, do not drop record).
- Numeric parsing should be lenient by default (handle common CLI variants like commas and +/-).

### Prompt handling (raw transcript parsing)
- Default behavior: auto-detect and strip prompts.
- Scope of cleanup in this phase: prompts + command echoes only (no paging markers, no banners/syslog cleanup).
- Failure mode: conservative (only strip when high confidence; otherwise keep lines).
- Multi-command transcripts: segment into blocks per command and parse each block separately.

### Validation / compatibility / migration
- Modern template schema validation is STRICT: unknown keys should be treated as errors.
- Ship a small starter pack of modern templates in-repo (curated common commands) in addition to supporting user-provided template files.
- Provide an interactive conversion tool (TextFSM -> modern) as part of this phase.

### Claude's Discretion
- Exact schema details (key names) as long as it preserves the locked decisions above.
- Exact set of "rich types" supported first, as long as the system is extensible and honors explicit typing + heuristic fallback.
- Exact UX of the interactive converter (steps, prompts), as long as it's interactive and best-effort.

</decisions>

<specifics>
## Specific Ideas

- Prompt handling should work on raw transcripts and treat prompt/echo as noise, but avoid false positives.
- Multi-command inputs should be segmented rather than treated as one continuous stream.

</specifics>

<deferred>
## Deferred Ideas

- Paging marker handling (e.g., `--More--`) and deeper transcript cleanup (banners/syslog) are explicitly out of scope for Phase 3.

</deferred>

---

*Phase: 03-modern-ergonomic-templates*
*Context gathered: 2026-02-20*

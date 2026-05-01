# Phase 2: Legacy Compatibility & CLI - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable parsing of existing TextFSM templates via a standard Unix-style CLI.

In-scope for this phase:
- A `cliscrape parse` command that loads a template (legacy TextFSM and existing modern formats as applicable) and parses input from stdin and/or files.
- Default CLI behavior for output formatting, errors/warnings, exit codes, and template selection.
- Compatibility expectations aligned with ntc-templates-style TextFSM behavior.

Out of scope (separate milestones/phases): device connectivity (SSH/Telnet), fleet/batch execution orchestration, and shipping a built-in template library/discovery beyond local files.

</domain>

<decisions>
## Implementation Decisions

### CLI UX + flags
- **Output format default:** Auto
  - When stdout is a TTY: default to a human-friendly pretty table.
  - When stdout is not a TTY (piped/redirected): default to JSON.
- **Input handling:** Allow both stdin and explicit input specification; do not force a single “default” input mode.
- **Multiple inputs:** Support multiple input files via glob/pattern support (e.g., `--input-glob ...` or equivalent), not just a single file.
- **Verbosity:** Minimal status information to stderr by default on success (e.g., record count, duration). Parsed data remains on stdout.

### Error + exit behavior
- **Failure mode:** Fail fast on template load/parse errors; exit non-zero and do not emit partial record output.
- **Error presentation:** Support both human-readable errors and structured errors.
  - Default: human-readable on stderr.
  - Provide a flag/mode to emit structured JSON errors (still to stderr).
- **Exit codes:** Simple contract.
  - `0` = success
  - `1` = any failure
- **Warnings:** Non-fatal warnings go to stderr by default.

### Template discovery + selection
- **Template specification:** Support both explicit template file paths and template identifiers/names.
  - For Phase 2 docs/primary flow, path-based usage is acceptable; name-based is allowed but does not imply a bundled library.
- **Template roots/search paths:** No template search root mechanism in Phase 2 (no env var root, no `--template-root`), beyond explicitly provided paths.
- **Template format selection:** Default to auto-detect with an explicit override available (e.g., `--template-format auto|textfsm|yaml|toml`).
- **Precedence/ambiguity:** If a template identifier resolves to multiple candidates, treat as an error (do not pick first/last silently).

### Compatibility expectations
- **ntc-templates compatibility:** Strict — deviations from expected TextFSM behavior are treated as bugs.
- **Undefined/non-standard patterns:** Error (surface clearly; do not silently accept).
- **Unsupported constructs:** Warn and skip the unsupported piece where feasible, rather than hard failing the entire parse.
- **Prompt/echo/transcript cleanup:** Smart by default during `parse` (attempt common prompt stripping/echo handling automatically).

### Claude's Discretion
- Exact flag names for structured error output and for glob input support, as long as the behaviors above are preserved.
- Exact formatting of the default status line (what fields are shown), as long as it stays minimal and goes to stderr.

</decisions>

<specifics>
## Specific Ideas

- None provided beyond the decisions above.

</specifics>

<deferred>
## Deferred Ideas

- Device connectivity (SSH/Telnet) and session management.
- A shipped/built-in template library and richer template discovery.
- Fleet/batch orchestration beyond “multiple inputs” in a single command.

</deferred>

---

*Phase: 02-legacy-compatibility-cli*
*Context gathered: 2026-02-21*

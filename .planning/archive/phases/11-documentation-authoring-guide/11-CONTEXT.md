# Phase 11: Documentation & Authoring Guide - Context

**Gathered:** 2026-03-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Deliver user-facing documentation that helps users discover templates, understand CLI usage, author new templates (modern + legacy), migrate from TextFSM to modern templates, and troubleshoot common parsing problems. Include an auto-generated embedded template catalog. No new runtime features are added in this phase.

</domain>

<decisions>
## Implementation Decisions

### Doc structure and information architecture
- Use `README.md` for a short entry point and put detailed guides under `docs/`.
- Optimize primarily for CLI users (pipelines/terminal usage) while still supporting template authors.
- Keep troubleshooting content inside each relevant guide (not a standalone `TROUBLESHOOTING.md`).
- Validate documentation examples in CI: treat examples as executable/verified (not best-effort).

### Authoring guide scope and style
- Cover both modern templates and legacy TextFSM authoring.
- Use several small "recipes" rather than a single end-to-end example or an exhaustive reference.
- Include a migration guide from TextFSM to modern templates.
- Include practical regex guidance: common pitfalls and safe patterns (not a deep regex theory chapter).

### Troubleshooting approach
- Emphasize TUI Live Lab as the first troubleshooting path.
- Position warnings (coverage, partial matches, etc.) as normal and actionable.
- Include a dedicated logging/tracing section in troubleshooting (e.g. `RUST_LOG`, `--log-format`).
- Use copy-paste commands with expected outputs in troubleshooting examples.

### Template catalog
- Auto-generated catalog lives at `docs/templates.md`.
- Group templates by vendor.
- Per entry, show at least: template name, description, format, vendor.
- Catalog generation is enforced by CI and the generated output is committed.

### Claude's Discretion
- Exact filenames within `docs/` beyond the catalog path.
- Exact wording/tone and how strictly to define "expected outputs" (within the "validate everything" constraint).
- How to implement CI validation mechanics (while honoring "validate everything").

</decisions>

<specifics>
## Specific Ideas

- README as onboarding and quickstart; deep dives live in `docs/`.
- Troubleshooting examples should be runnable and copy-pastable.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 11-documentation-authoring-guide*
*Context gathered: 2026-03-05*

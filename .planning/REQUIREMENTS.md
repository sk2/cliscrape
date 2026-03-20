# Requirements: cliscrape

**Defined:** 2026-02-22
**Last updated:** 2026-03-20
**Core Value:** Extremely fast, reliable parsing of semi-structured CLI output into structured data, regardless of whether the template is legacy TextFSM or the new ergonomic format.

## Purpose

This file now distinguishes between:
- historical, shipped requirements for completed milestones
- active requirement anchors for the current roadmap

`ROADMAP.md` remains the canonical source for phase order, status, and milestone progression.

## Historical Requirements: v1.5 Production Hardening

These requirements were defined for the shipped v1.5 milestone and are retained for traceability.

### Template Library

- [x] **LIB-01**: User can parse with embedded templates without providing file paths
- [x] **LIB-02**: User can reference templates by name (e.g., `--template cisco_ios_show_version`)
- [x] **LIB-03**: User can add custom templates to XDG user directory (`~/.local/share/cliscrape/templates/`)
- [x] **LIB-04**: User can view template metadata including version, description, and compatibility
- [x] **LIB-05**: User can override embedded templates with custom versions via XDG directory
- [x] **LIB-06**: User receives security validation errors for invalid template names (path traversal protection)

### Compatibility Validation

- [x] **VAL-01**: Developer can run snapshot tests for all embedded templates
- [x] **VAL-02**: Developer can add negative test cases (malformed input, errors, truncation)
- [x] **VAL-03**: Developer can run performance benchmarks per template
- [x] **VAL-04**: Developer can verify validation suite passes in CI/CD
- [x] **VAL-05**: User receives validation warnings when template captures <80% expected fields

### TUI Integration

- [x] **TUI-01**: User can select embedded templates from Live Lab TUI mode
- [x] **TUI-02**: User can browse available templates with descriptions and metadata
- [x] **TUI-03**: User can run validation tests interactively from TUI
- [x] **TUI-04**: User can load XDG user templates in TUI mode

### Edge Case Hardening

- [x] **HARD-01**: User receives timeout errors for regex patterns with catastrophic backtracking
- [x] **HARD-02**: User receives warnings when template match threshold is not met
- [x] **HARD-03**: User receives contextual error messages showing parsing failures with line context
- [x] **HARD-04**: User can choose fail-fast mode (abort on first error) or partial-match mode (continue with warnings)
- [x] **HARD-05**: User receives graceful degradation for optional template fields

### Production Logging

- [x] **LOG-01**: User can enable structured logging via `RUST_LOG` environment variable
- [x] **LOG-02**: User can set log level filtering by module (e.g., `RUST_LOG=cliscrape::template=debug`)
- [x] **LOG-03**: User can increase verbosity with `-v`/`-vv`/`-vvv`/`-vvvv` flags
- [x] **LOG-04**: User can output logs in JSON format for production observability
- [x] **LOG-05**: Developer verifies logging overhead is <5% performance impact

### Documentation

- [x] **DOC-01**: User can read comprehensive guide covering template selection and usage
- [x] **DOC-02**: User can view auto-generated catalog of available templates
- [x] **DOC-03**: User can read template authoring guide with YAML format and FSM concepts
- [x] **DOC-04**: User can find troubleshooting guide for common parsing errors
- [x] **DOC-05**: Developer verifies all documentation examples pass CI validation

## Active Requirements: v2.0 The Network Compiler

These requirement anchors track the currently active roadmap work.

### Semantic Constraint Logic

- [x] **POLICY-01**: Template authors can declare field-level policy constraints in modern templates.
- [x] **POLICY-02**: Users can detect policy violations during parsing and optionally fail the command with `--strict-policy`.

### Universal Ledger Library

- [ ] **LEDGER-01**: Core operational states can be emitted in vendor-neutral schemas across supported templates.
- [ ] **LOG-02**: Policy and schema validation behavior integrates with structured tracing.

## Planned Requirements: v3.0 Isomorphic Ecosystem

- [ ] **MOCK-01**: User can simulate device CLI responses from structured state data.
- [ ] **MOCK-02**: Simulated device output preserves vendor-authentic formatting closely enough for tooling workflows.

## Deferred / Future Ideas

These remain potential future requirement areas but are not the active planning source for the current roadmap.

### Connectivity

- [ ] **CONN-01**: User can connect to devices via SSH/Telnet
- [ ] **CONN-02**: User can execute commands in interactive sessions
- [ ] **CONN-03**: User can run batch operations across device fleets

### Advanced Template Features

- [ ] **TMPL-01**: User can install templates from git repositories
- [ ] **TMPL-02**: User can search templates by tags and metadata
- [ ] **TMPL-03**: User can migrate `.textfsm` templates to modern YAML format

## Out of Scope

| Feature | Reason |
|---------|--------|
| Web/GUI Interface | CLI-first tool; graphical interfaces deferred to future versions |
| Configuration Management | Read-only command execution; device changes out of scope |
| SNMP/NETCONF/REST | SSH/CLI-focused; other protocols deferred to maintain scope |
| Massive File Optimization | Focus on throughput across many outputs, not gigabyte-scale files |
| Real-time Template Compilation | Templates compiled at load time; JIT compilation unnecessary complexity |
| Remote Template Registry | Embedded + XDG sufficient for current scope; registry deferred to future phases |

## Traceability

### Historical v1.5 Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| LIB-01 | Phase 6 | Complete |
| LIB-02 | Phase 6 | Complete |
| LIB-03 | Phase 6 | Complete |
| LIB-04 | Phase 6 | Complete |
| LIB-05 | Phase 6 | Complete |
| LIB-06 | Phase 6 | Complete |
| VAL-01 | Phase 7 | Complete |
| VAL-02 | Phase 7 | Complete |
| VAL-03 | Phase 7 | Complete |
| VAL-04 | Phase 7 | Complete |
| VAL-05 | Phase 7 | Complete |
| TUI-01 | Phase 8 | Complete |
| TUI-02 | Phase 8 | Complete |
| TUI-03 | Phase 8 | Complete |
| TUI-04 | Phase 8 | Complete |
| HARD-01 | Phase 9 | Complete |
| HARD-02 | Phase 9 | Complete |
| HARD-03 | Phase 9 | Complete |
| HARD-04 | Phase 9 | Complete |
| HARD-05 | Phase 9 | Complete |
| LOG-01 | Phase 10 | Complete |
| LOG-02 | Phase 10 | Complete |
| LOG-03 | Phase 10 | Complete |
| LOG-04 | Phase 10 | Complete |
| LOG-05 | Phase 10 | Complete |
| DOC-01 | Phase 11 | Mostly complete; README/user-guide workflow backfilled |
| DOC-02 | Phase 11 | Deferred reconciliation: catalog artifact not present |
| DOC-03 | Phase 11 | Mostly complete; authoring guide present |
| DOC-04 | Phase 11 | Mostly complete; troubleshooting guidance partially covered in guides |
| DOC-05 | Phase 11 | Deferred reconciliation: executable doc validation not wired |

### Active Roadmap Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| POLICY-01 | Phase 16 | Complete |
| POLICY-02 | Phase 16 | Complete |
| LEDGER-01 | Phase 17 | Planned |
| LOG-02 | Phase 17 | Planned |
| MOCK-01 | Phase 18 | Planned |
| MOCK-02 | Phase 18 | Planned |

## Notes

- Use `.planning/ROADMAP.md` as the source of truth for current phase ordering and milestone status.
- Use phase `*-CONTEXT.md`, `*-PLAN.md`, and `*-SUMMARY.md` files for execution detail.
- `v1.5-MILESTONE-AUDIT.md` is a historical pre-completion audit and not the active status source.

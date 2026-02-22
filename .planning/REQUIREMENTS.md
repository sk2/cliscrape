# Requirements: cliscrape v1.5

**Defined:** 2026-02-22
**Core Value:** Extremely fast, reliable parsing of semi-structured CLI output into structured data, regardless of whether the template is legacy TextFSM or the new ergonomic format.

## v1.5 Requirements

Requirements for Template Ecosystem & Production Hardening milestone.

### Template Library

- [x] **LIB-01**: User can parse with embedded templates without providing file paths
- [x] **LIB-02**: User can reference templates by name (e.g., `--template cisco_ios_show_version`)
- [x] **LIB-03**: User can add custom templates to XDG user directory (~/.local/share/cliscrape/templates/)
- [x] **LIB-04**: User can view template metadata including version, description, and compatibility
- [x] **LIB-05**: User can override embedded templates with custom versions via XDG directory
- [x] **LIB-06**: User receives security validation errors for invalid template names (path traversal protection)

### Compatibility Validation

- [ ] **VAL-01**: Developer can run snapshot tests for all embedded templates
- [x] **VAL-02**: Developer can add negative test cases (malformed input, errors, truncation)
- [ ] **VAL-03**: Developer can run performance benchmarks per template
- [ ] **VAL-04**: Developer can verify validation suite passes in CI/CD
- [ ] **VAL-05**: User receives validation warnings when template captures <80% expected fields

### TUI Integration

- [ ] **TUI-01**: User can select embedded templates from Live Lab TUI mode
- [ ] **TUI-02**: User can browse available templates with descriptions and metadata
- [ ] **TUI-03**: User can run validation tests interactively from TUI
- [ ] **TUI-04**: User can load XDG user templates in TUI mode

### Edge Case Hardening

- [ ] **HARD-01**: User receives timeout errors for regex patterns with catastrophic backtracking
- [ ] **HARD-02**: User receives warnings when template match threshold is not met
- [ ] **HARD-03**: User receives contextual error messages showing parsing failures with line context
- [ ] **HARD-04**: User can choose fail-fast mode (abort on first error) or partial-match mode (continue with warnings)
- [ ] **HARD-05**: User receives graceful degradation for optional template fields

### Production Logging

- [ ] **LOG-01**: User can enable structured logging via RUST_LOG environment variable
- [ ] **LOG-02**: User can set log level filtering by module (e.g., RUST_LOG=cliscrape::template=debug)
- [ ] **LOG-03**: User can increase verbosity with -v/-vv/-vvv/-vvvv flags
- [ ] **LOG-04**: User can output logs in JSON format for production observability
- [ ] **LOG-05**: Developer verifies logging overhead is <5% performance impact

### Documentation

- [ ] **DOC-01**: User can read comprehensive guide covering template selection and usage
- [ ] **DOC-02**: User can view auto-generated catalog of available templates
- [ ] **DOC-03**: User can read template authoring guide with YAML format and FSM concepts
- [ ] **DOC-04**: User can find troubleshooting guide for common parsing errors
- [ ] **DOC-05**: Developer verifies all documentation examples pass CI validation

## Future Requirements (v2.0)

Deferred to v2.0 Connectivity milestone.

### Connectivity
- **CONN-01**: User can connect to devices via SSH/Telnet
- **CONN-02**: User can execute commands in interactive sessions
- **CONN-03**: User can run batch operations across device fleets

### Advanced Template Features
- **TMPL-01**: User can install templates from git repositories
- **TMPL-02**: User can search templates by tags and metadata
- **TMPL-03**: User can migrate .textfsm templates to modern YAML format

## Out of Scope

| Feature | Reason |
|---------|--------|
| Web/GUI Interface | CLI-first tool; graphical interfaces deferred to future versions |
| Configuration Management | Read-only command execution; device changes out of scope |
| SNMP/NETCONF/REST | SSH/CLI-focused; other protocols deferred to maintain scope |
| Massive File Optimization | Focus on throughput across many outputs, not gigabyte-scale files |
| Real-time Template Compilation | Templates compiled at load time; JIT compilation unnecessary complexity |
| Remote Template Registry | Embedded + XDG sufficient for v1.5; registry deferred to v2+ |

## Traceability

Which phases cover which requirements. Updated during roadmap creation.

| Requirement | Phase | Status |
|-------------|-------|--------|
| LIB-01 | Phase 6 | Complete |
| LIB-02 | Phase 6 | Complete |
| LIB-03 | Phase 6 | Complete |
| LIB-04 | Phase 6 | Complete |
| LIB-05 | Phase 6 | Complete |
| LIB-06 | Phase 6 | Complete |
| VAL-01 | Phase 7 | Pending |
| VAL-02 | Phase 7 | Complete (07-02) |
| VAL-03 | Phase 7 | Pending |
| VAL-04 | Phase 7 | Pending |
| VAL-05 | Phase 7 | Pending |
| TUI-01 | Phase 8 | Pending |
| TUI-02 | Phase 8 | Pending |
| TUI-03 | Phase 8 | Pending |
| TUI-04 | Phase 8 | Pending |
| HARD-01 | Phase 9 | Pending |
| HARD-02 | Phase 9 | Pending |
| HARD-03 | Phase 9 | Pending |
| HARD-04 | Phase 9 | Pending |
| HARD-05 | Phase 9 | Pending |
| LOG-01 | Phase 10 | Pending |
| LOG-02 | Phase 10 | Pending |
| LOG-03 | Phase 10 | Pending |
| LOG-04 | Phase 10 | Pending |
| LOG-05 | Phase 10 | Pending |
| DOC-01 | Phase 11 | Pending |
| DOC-02 | Phase 11 | Pending |
| DOC-03 | Phase 11 | Pending |
| DOC-04 | Phase 11 | Pending |
| DOC-05 | Phase 11 | Pending |

**Coverage:**
- v1.5 requirements: 30 total
- Mapped to phases: 30/30 (100%)
- Unmapped: 0

---
*Requirements defined: 2026-02-22*
*Last updated: 2026-02-22 after roadmap creation*

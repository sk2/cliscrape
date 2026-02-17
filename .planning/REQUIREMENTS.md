# Requirements

## v1 Requirements (MVP)

### Core Engine
- [x] **CORE-01**: High-throughput Rust-based FSM engine for line-by-line parsing.
- [x] **CORE-02**: Support for full TextFSM grammar, including `Filldown`, `Required`, and all actions (`Next`, `Continue`, `Record`, `Clear`).
- [x] **CORE-03**: Shared regex pattern library (e.g., `{{ipv4}}`, `{{mac_address}}`, `{{interface_name}}`).

### Template Formats
- [x] **FORM-01**: 100% compatibility with existing `.textfsm` files (e.g., from `ntc-templates`).
- [ ] **FORM-02**: Modern hybrid YAML/TOML ergonomic format with structured state blocks.
- [ ] **FORM-03**: Typed captures (e.g., `IP`, `Integer`, `Date`) with automatic string-to-type conversion.

### TUI Debugger
- [ ] **TUI-01**: "Live Lab" split-pane view with real-time regex matching and field highlighting.
- [ ] **TUI-02**: "State Tracer" to watch FSM transitions line-by-line, including current variable values.
- [ ] **TUI-03**: Trace history buffer for debugging complex multi-state templates.

### CLI & Connectivity
- [x] **CLI-01**: Standalone CLI for parsing local files or piped input (Unix-style pipe support).
- [ ] **CLI-02**: Basic Cisco IOS/NX-OS prompt support for initial scraping workflows.

---

## v2 Requirements (Deferred)

- **CLI-03**: High-concurrency SSH scraping across a fleet of devices.
- **CONN-01**: Multi-vendor prompt handling (Juniper, Arista, etc.).
- **FORM-04**: Template inheritance and base-template "snippets" to reduce repetition.

---

## Out of Scope

- **MASS-01**: Massive single-file optimization (>200MB logs) — focus is on throughput for v1.
- **MGMT-01**: Configuration management (pushing changes) — `cliscrape` is a parser only.
- **WEB-01**: Web UI/Dashboard — focus remains on CLI/TUI.

---

## Traceability Matrix

| Requirement | Phase | Status |
|-------------|-------|--------|
| CORE-01     | Phase 1 | Pending |
| CORE-02     | Phase 2 | Pending |
| CORE-03     | Phase 1 | Pending |
| FORM-01     | Phase 2 | Pending |
| FORM-02     | Phase 3 | Pending |
| FORM-03     | Phase 3 | Pending |
| TUI-01      | Phase 4 | Pending |
| TUI-02      | Phase 5 | Pending |
| TUI-03      | Phase 5 | Pending |
| CLI-01      | Phase 2 | Pending |
| CLI-02      | Phase 3 | Pending |

---
*Last updated: 2024-05-22 after roadmap creation*

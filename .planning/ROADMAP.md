# Roadmap: cliscrape

## Milestones

- ✅ **v1.0 MVP** - Phases 1-5 (shipped 2026-02-22)
- 🚧 **v1.5 Template Ecosystem & Production Hardening** - Phases 6-11 (in progress)
- 📋 **v2.0 Connectivity** - (planned)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-5) - SHIPPED 2026-02-22</summary>

- [x] Phase 1: Core Parsing Engine (5/5 plans) — completed 2026-02-18
- [x] Phase 2: Legacy Compatibility & CLI (8/8 plans) — completed 2026-02-22
- [x] Phase 3: Modern Ergonomic Templates (6/6 plans) — completed 2026-02-20
- [x] Phase 4: TUI Debugger Foundation (Live Lab) (5/5 plans) — completed 2026-02-21
- [x] Phase 5: TUI Advanced Debugging (State Tracer) (4/4 plans) — completed 2026-02-21

**Summary:** Complete CLI parsing tool with legacy TextFSM support, modern YAML/TOML templates, and visual debugging. High-throughput engine (4.1M lines/sec), 100% TextFSM compatibility, comprehensive test coverage (77 tests).

See: `.planning/milestones/v1.0-ROADMAP.md` for full details.

</details>

### 🚧 v1.5 Template Ecosystem & Production Hardening (In Progress)

**Milestone Goal:** Transform cliscrape into production-ready tool with comprehensive template library, discovery mechanism, validation framework, and hardened edge case handling.

- [x] **Phase 6: Template Library Foundation** - Embedded templates with XDG discovery (Complete)
- [ ] **Phase 7: Compatibility Validation Suite** - Snapshot testing and negative cases
- [ ] **Phase 8: TUI Integration** - Template browser and Live Lab integration
- [ ] **Phase 9: Edge Case Hardening** - Timeout enforcement and graceful degradation
- [ ] **Phase 10: Production Logging** - Structured observability with tracing
- [ ] **Phase 11: Documentation & Authoring Guide** - User guide and template catalog

## Phase Details

### Phase 6: Template Library Foundation
**Goal**: Users can parse common network device outputs without providing template files
**Depends on**: Nothing (first phase of v1.5)
**Requirements**: LIB-01, LIB-02, LIB-03, LIB-04, LIB-05, LIB-06
**Success Criteria** (what must be TRUE):
  1. User can run `cliscrape --template cisco_ios_show_version` without providing local file
  2. User can list available embedded templates with metadata (description, version, compatibility)
  3. User can override embedded templates by placing custom versions in ~/.local/share/cliscrape/templates/
  4. User receives security validation error when attempting path traversal via template names
  5. User can view template source and metadata for any embedded template
**Plans**: 4 plans in 3 waves

Plans:
- [x] 06-01-PLAN.md — Embedded library infrastructure (rust-embed + XDG resolver + security validation)
- [x] 06-02-PLAN.md — Template metadata extraction (YAML/TOML/TextFSM)
- [x] 06-03-PLAN.md — CLI integration (list-templates and show-template subcommands)
- [x] 06-04-PLAN.md — Initial template library creation and testing

### Phase 7: Compatibility Validation Suite
**Goal**: All embedded templates verified against real device outputs with negative test coverage
**Depends on**: Phase 6
**Requirements**: VAL-01, VAL-02, VAL-03, VAL-04, VAL-05
**Success Criteria** (what must be TRUE):
  1. Developer can run `cargo test --test validation` and see snapshot tests for all embedded templates
  2. Developer can add fixture file (raw device output) and expected snapshot (parsed JSON) for any template
  3. Developer can verify negative test cases exist covering malformed input, truncation, and parser errors
  4. Developer sees CI validation suite pass before merge to main
  5. User receives actionable warning when template captures less than 80% of expected fields
**Plans**: 5 plans in 3 waves

Plans:
- [x] 07-01-PLAN.md — Snapshot testing infrastructure (insta framework + fixtures)
- [x] 07-02-PLAN.md — Negative test coverage (error cases + edge cases)
- [x] 07-03-PLAN.md — Performance benchmarking (criterion integration)
- [x] 07-04-PLAN.md — Coverage validation system (field coverage analysis)
- [ ] 07-05-PLAN.md — CI integration (GitHub Actions workflow)

### Phase 8: TUI Integration
**Goal**: Users can discover and test embedded templates interactively from TUI Live Lab
**Depends on**: Phase 6
**Requirements**: TUI-01, TUI-02, TUI-03, TUI-04
**Success Criteria** (what must be TRUE):
  1. User can press hotkey in Live Lab to open template browser showing embedded templates
  2. User can browse template list with descriptions, compatibility metadata, and selection preview
  3. User can select embedded template to load into Live Lab for editing and testing
  4. User can load custom templates from XDG user directory via TUI file picker
**Plans**: TBD

### Phase 9: Edge Case Hardening
**Goal**: Parser handles malformed input gracefully with timeouts, thresholds, and contextual errors
**Depends on**: Phase 7
**Requirements**: HARD-01, HARD-02, HARD-03, HARD-04, HARD-05
**Success Criteria** (what must be TRUE):
  1. User receives timeout error with pattern details when regex exhibits catastrophic backtracking
  2. User receives warning when template matches fewer fields than minimum threshold (default 80%)
  3. User sees contextual error messages showing line number and surrounding context for parsing failures
  4. User can choose fail-fast mode (--strict) to abort on first error or partial-match mode for warnings
  5. User receives successful parse with warnings when optional template fields are missing from input
**Plans**: TBD

### Phase 10: Production Logging
**Goal**: Production deployments have structured observability without performance degradation
**Depends on**: Phase 6
**Requirements**: LOG-01, LOG-02, LOG-03, LOG-04, LOG-05
**Success Criteria** (what must be TRUE):
  1. User can set RUST_LOG=cliscrape=debug to enable structured logging without code changes
  2. User can filter logs by module (e.g., RUST_LOG=cliscrape::template=trace,cliscrape::parser=warn)
  3. User can increase CLI verbosity with -v (info), -vv (debug), -vvv (trace), -vvvv (all modules trace)
  4. User can output logs in JSON format for production observability pipelines
  5. Developer verifies logging overhead is less than 5% performance impact via benchmarks
**Plans**: TBD

### Phase 11: Documentation & Authoring Guide
**Goal**: Users can discover templates, understand usage, author new templates, and troubleshoot errors
**Depends on**: Phases 6-10 (documents completed features)
**Requirements**: DOC-01, DOC-02, DOC-03, DOC-04, DOC-05
**Success Criteria** (what must be TRUE):
  1. User can read comprehensive user guide covering template selection, discovery, and CLI usage
  2. User can view auto-generated catalog listing all embedded templates with metadata
  3. User can follow template authoring guide explaining YAML format, FSM concepts, and regex best practices
  4. User can consult troubleshooting guide for common parsing errors with resolution steps
  5. Developer verifies all documentation code examples pass CI validation (doc tests)
**Plans**: TBD

## Progress Tracking

**Execution Order:**
Phases execute in numeric order: 6 → 7 → 8 → 9 → 10 → 11

**Dependencies:**
- Phases 7, 8, 10 depend on Phase 6 (template library infrastructure)
- Phase 9 depends on Phase 7 (validation reveals edge cases)
- Phase 11 depends on all preceding phases (documents completed features)

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Core Parsing Engine | v1.0 | 5/5 | Complete | 2026-02-18 |
| 2. Legacy Compatibility & CLI | v1.0 | 8/8 | Complete | 2026-02-22 |
| 3. Modern Ergonomic Templates | v1.0 | 6/6 | Complete | 2026-02-20 |
| 4. TUI Debugger Foundation | v1.0 | 5/5 | Complete | 2026-02-21 |
| 5. TUI Advanced Debugging | v1.0 | 4/4 | Complete | 2026-02-21 |
| 6. Template Library Foundation | v1.5 | 4/4 | Complete | 2026-02-22 |
| 7. Compatibility Validation Suite | v1.5 | 4/5 | In Progress | - |
| 8. TUI Integration | v1.5 | 0/? | Not started | - |
| 9. Edge Case Hardening | v1.5 | 0/? | Not started | - |
| 10. Production Logging | v1.5 | 0/? | Not started | - |
| 11. Documentation & Authoring Guide | v1.5 | 0/? | Not started | - |

---
*Roadmap created: 2026-02-22*
*Last updated: 2026-02-23*

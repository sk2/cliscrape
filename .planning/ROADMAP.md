# Roadmap: cliscrape

## Milestones

- ✅ **v1.0 MVP** - Phases 1-5 (shipped 2026-02-22)
- 🚧 **v1.5 Template Ecosystem & Production Hardening** - Phases 6-11 (in progress)
- 📋 **v2.0 The Network Compiler** - Phases 12-15 (planned)
- 📋 **v3.0 Connectivity** - (planned)

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

<details>
<summary>🚧 v1.5 Template Ecosystem & Production Hardening (In Progress)</summary>

**Milestone Goal:** Transform cliscrape into production-ready tool with comprehensive template library, discovery mechanism, validation framework, and hardened edge case handling.

- [x] **Phase 6: Template Library Foundation** - Embedded templates with XDG discovery (Complete)
- [x] **Phase 7: Compatibility Validation Suite** - Snapshot testing and negative cases (completed 2026-02-22)
- [x] **Phase 8: TUI Integration** - Template browser and Live Lab integration (completed 2026-03-04)
- [x] **Phase 9: Edge Case Hardening** - Timeout enforcement and graceful degradation (completed 2026-03-04)
- [x] **Phase 10: Production Logging** - Structured observability with tracing (completed 2026-03-05)
- [x] **Phase 11: Documentation & Authoring Guide** - User guide and template catalog (completed 2026-03-19)

</details>

### 📋 v2.0 The Network Compiler (Planned)

**Milestone Goal:** Move from simple parsing to mathematical state observation and vendor-neutral operational manifolds.

- [ ] **Phase 12: Semantic Drift Analysis** - Operational diffing between states
- [ ] **Phase 13: Vendor-Neutral State Manifolds** - Common schema mapping layer
- [ ] **Phase 14: Grammar Induction** - Statistical structural inference for templates
- [ ] **Phase 15: The FSM-Oracle** - Bijective verification and self-healing loop

## Phase Details

### Phase 12: Semantic Drift Analysis
**Goal**: Users can identify operational state transitions while ignoring syntactic noise
**Depends on**: Phase 3 (Modern Templates)
**Requirements**: DRIFT-01, DRIFT-02, DRIFT-03
**Success Criteria** (what must be TRUE):
  1. User can run `cliscrape diff before.txt after.txt` and see field-level transitions
  2. The system ignores "noisy" fields (timestamps, PIDs) defined in template metadata
  3. The diff engine correctly matches records based on "Identity Fields" (e.g., interface name)
**Plans**: TBD

### Phase 13: Vendor-Neutral State Manifolds
**Goal**: Abstract vendor-specific CLI into a universal data ledger
**Depends on**: Phase 3
**Requirements**: SCHEMA-01, SCHEMA-02, SCHEMA-03
**Success Criteria** (what must be TRUE):
  1. Modern templates support `common_schema` mapping for variables
  2. Users can export data in a "Common" format that is vendor-agnostic
  3. The system validates that a template satisfies its claimed schema compliance
**Plans**: TBD

### Phase 14: Grammar Induction
**Goal**: Automate template creation via statistical text analysis
**Depends on**: Phase 1
**Requirements**: INDUCT-01, INDUCT-02
**Success Criteria** (what must be TRUE):
  1. System can identify static anchors across multiple CLI samples
  2. System proposes candidate regex capture groups for variable segments
  3. System generates a boilerplate YAML template from raw input samples
**Plans**: TBD

### Phase 15: The FSM-Oracle
**Goal**: Continuous mathematical verification of template integrity
**Depends on**: Isomorphic Simulation Implementation
**Requirements**: ORACLE-01, ORACLE-02
**Success Criteria** (what must be TRUE):
  1. CI pipeline automatically verifies bijective stability (round-trip equality)
  2. System flags broken templates when round-trip verification fails
  3. CLI emits drift alerts when production input cannot be losslessly round-tripped
**Plans**: TBD

## Progress Tracking

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Core Parsing Engine | v1.0 | 5/5 | Complete | 2026-02-18 |
| 2. Legacy Compatibility & CLI | v1.0 | 8/8 | Complete | 2026-02-22 |
| 3. Modern Ergonomic Templates | v1.0 | 6/6 | Complete | 2026-02-20 |
| 4. TUI Debugger Foundation | v1.0 | 5/5 | Complete | 2026-02-21 |
| 5. TUI Advanced Debugging | v1.0 | 4/4 | Complete | 2026-02-21 |
| 6. Template Library Foundation | v1.5 | 4/4 | Complete | 2026-02-22 |
| 7. Compatibility Validation Suite | v1.5 | 5/5 | Complete | 2026-02-22 |
| 8. TUI Integration | v1.5 | 4/4 | Complete | 2026-03-04 |
| 9. Edge Case Hardening | v1.5 | 3/3 | Complete | 2026-03-04 |
| 10. Production Logging | v1.5 | 3/3 | Complete | 2026-03-05 |
| 11. Documentation & Authoring Guide | v1.5 | 3/3 | Complete | 2026-03-19 |
| 12. Semantic Drift Analysis | v2.0 | 0/? | Planned | - |
| 13. Vendor-Neutral State Manifolds | v2.0 | 0/? | Planned | - |
| 14. Grammar Induction | v2.0 | 0/? | Planned | - |
| 15. The FSM-Oracle | v2.0 | 0/? | Planned | - |

---
*Roadmap created: 2026-02-22*
*Last updated: 2026-03-18*

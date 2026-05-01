# Roadmap: cliscrape

## Milestones

- ✅ **v1.0 MVP** - Phases 1-5 (shipped 2026-02-22)
- ✅ **v1.5 Production Hardening** - Phases 6-11 (shipped 2026-03-19)
- 🚧 **v2.0 The Network Compiler** - Phases 12-17 (in progress)
- 📋 **v3.0 Isomorphic Ecosystem** - Phases 18+ (planned)

## Phases

<details>
<summary>✅ v1.0 MVP (Phases 1-5) - SHIPPED 2026-02-22</summary>

- [x] Phase 1: Core Parsing Engine
- [x] Phase 2: Legacy Compatibility & CLI
- [x] Phase 3: Modern Ergonomic Templates
- [x] Phase 4: TUI Debugger Foundation (Live Lab)
- [x] Phase 5: TUI Advanced Debugging (State Tracer)

</details>

<details>
<summary>✅ v1.5 Production Hardening - SHIPPED 2026-03-19</summary>

- [x] Phase 6: Template Library Foundation
- [x] Phase 7: Compatibility Validation Suite
- [x] Phase 8: TUI Integration
- [x] Phase 9: Edge Case Hardening
- [x] Phase 10: Production Logging
- [x] Phase 11: Documentation & Authoring Guide

> Note: user and authoring guides shipped. The original plan 03 catalog/doc-validation artifacts were deferred and are tracked as reconciliation work.

</details>

> **Operating model (2026-05-01):** v2.0 Phase 17 onward is tracked in beads
> (`br`), not as `phases/NN-*/PLAN.md` directories. This roadmap remains the
> milestone narrative; bead IDs link to executable backlog. Run
> `br dep tree <epic-id> -d up` to see children, or `br ready` for actionable
> work across the project.

### 🚧 v2.0 The Network Compiler (In Progress)

**Milestone Goal:** Move from simple parsing to structured state observation and vendor-neutral operational schemas.

- [x] **Phase 12: Semantic Drift Analysis** - Operational diffing between states (Complete)
- [x] **Phase 13: Vendor-Neutral State Mappings** - Common schema mapping primitives (Complete; embedded template coverage tracked under Phase 17 epic)
- [x] **Phase 14: Grammar Induction** - Statistical structural inference for templates (Complete)
- [x] **Phase 15: The FSM-Oracle** - Best-effort generation and advisory round-trip checks (Complete; strict selected-template verification tracked by `cliscrape-nuy`)
- [x] **Phase 16: Semantic Constraint Logic** - Policy-aware parsing with boundary assertions (implemented)
- [ ] **Phase 17: The Universal Ledger Library** - Tracked as `cliscrape-1s8` (epic) with children `cliscrape-1s8.1` / `1s8.2` / `1s8.3`

### 📋 v3.0 Isomorphic Ecosystem (Planned)

**Milestone Goal:** Use isomorphic FSM execution to power high-fidelity network simulation and generative testing.

- [ ] **Phase 18: The Semantic Mock Server** - Tracked as `cliscrape-quw` (epic). Decompose when Phase 17 closes.
- [ ] **Phase 19: State-of-the-World Manifests** - Tracked as `cliscrape-woo` (epic). Blocked by `cliscrape-quw`.
- [ ] **Phase 20: SSH/CLI Protocol Integration** - Tracked as `cliscrape-lw0` (epic). Blocked by `cliscrape-quw` + `cliscrape-woo`.

## Phase Details

### Phase 16: Semantic Constraint Logic
**Goal**: The parser identifies "Impossible States" and policy violations during the parse phase.
**Depends on**: Phase 3
**Requirements**: POLICY-01, POLICY-02
**Success Criteria**:
  1. YAML templates support `constraints` (`min`, `max`, `choices`, `regex`).
  2. The engine emits high-severity `tracing` events when a parsed value violates a constraint.
  3. CLI supports `--strict-policy` to fail the parse if constraints are not met.

### Phase 17: The Universal Ledger Library
**Tracked by**: `cliscrape-1s8` (epic). Acceptance criteria, scope, and decomposition
live in the bead. Run `br show cliscrape-1s8` for the canonical spec.
**Requirements**: LEDGER-01, LOG-02
**Children**: `cliscrape-1s8.1` (define schemas) → `cliscrape-1s8.2` (wire templates) → `cliscrape-1s8.3` (compliance tests).

### Phase 18: The Semantic Mock Server
**Tracked by**: `cliscrape-quw` (epic). Run `br show cliscrape-quw`.
**Depends on**: Phase 17 schemas (via bead dependency) and Phase 15 oracle primitives.
**Requirements**: MOCK-01, MOCK-02

## Progress Tracking

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 12. Semantic Drift Analysis | v2.0 | 3/3 | Complete | 2026-03-19 |
| 13. Vendor-Neutral State Mappings | v2.0 | 2/2 | Complete; template coverage open | 2026-03-19 |
| 14. Grammar Induction | v2.0 | 2/2 | Complete | 2026-03-19 |
| 15. The FSM-Oracle | v2.0 | 2/2 | Advisory complete; strict mode open | 2026-03-19 |
| 16. Semantic Constraint Logic | v2.0 | 1/1 | Complete | 2026-03-20 |
| 17. Universal Ledger Library | v2.0 | beads `cliscrape-1s8` (3 children) | Active | - |
| 18. Semantic Mock Server | v3.0 | beads `cliscrape-quw` | Planned | - |
| 19. State-of-the-World Manifests | v3.0 | beads `cliscrape-woo` | Planned | - |
| 20. SSH/CLI Protocol Integration | v3.0 | beads `cliscrape-lw0` | Planned | - |

---
*Roadmap created: 2026-02-22*
*Last updated: 2026-05-01 (migrated Phase 17-20 task tracking from GSD to beads)*

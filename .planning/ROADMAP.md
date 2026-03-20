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

### 🚧 v2.0 The Network Compiler (In Progress)

**Milestone Goal:** Move from simple parsing to mathematical state observation and vendor-neutral operational manifolds.

- [x] **Phase 12: Semantic Drift Analysis** - Operational diffing between states (Complete)
- [x] **Phase 13: Vendor-Neutral State Manifolds** - Common schema mapping layer (Complete)
- [x] **Phase 14: Grammar Induction** - Statistical structural inference for templates (Complete)
- [x] **Phase 15: The FSM-Oracle** - Bijective verification and self-healing loop (Complete)
- [x] **Phase 16: Semantic Constraint Logic** - Policy-aware parsing with boundary assertions (implemented)
- [ ] **Phase 17: The Universal Ledger Library** - Standardized schemas for core network operations

### 📋 v3.0 Isomorphic Ecosystem (Planned)

**Milestone Goal:** Use isomorphic FSM execution to power high-fidelity network simulation and generative testing.

- [ ] **Phase 18: The Semantic Mock Server** - Interactive CLI simulation from JSON state
- [ ] **Phase 19: State-of-the-World Manifests** - Orchestrating multi-device mock environments
- [ ] **Phase 20: SSH/CLI Protocol Integration** - High-fidelity device mocking for `deviceinteraction`

## Phase Details

### Phase 16: Semantic Constraint Logic
**Goal**: The parser identifies "Impossible States" and policy violations during the parse phase.
**Depends on**: Phase 3
**Requirements**: POLICY-01, POLICY-02
**Success Criteria**:
  1. YAML templates support `constraints` (min, max, allowed_values, regex_assertions).
  2. The engine emits high-severity `tracing` events when a parsed value violates a constraint.
  3. CLI supports `--strict-policy` to fail the parse if constraints are not met.

### Phase 17: The Universal Ledger Library
**Goal**: A "Plug-and-Play" experience where disparate vendors emit identical data structures.
**Depends on**: Phase 13
**Requirements**: LEDGER-01, LOG-02
**Success Criteria**:
  1. Standard schemas defined for: `interface`, `bgp_neighbor`, `lldp_neighbor`, `version`, `route`.
  2. All core embedded templates updated with `common_schema` mappings.
  3. Validation tool ensures a template claiming a schema actually provides required keys.

### Phase 18: The Semantic Mock Server
**Goal**: Simulate device behavior without physical hardware or VMs.
**Depends on**: Phase 15 (Isomorphic Generation)
**Requirements**: MOCK-01, MOCK-02
**Success Criteria**:
  1. `cliscrape simulate` command provides an interactive shell.
  2. User inputs a command (e.g., `show version`), and the server synthesizes the CLI output from a provided JSON state file.
  3. Response is pixel-accurate to the original vendor's formatting.

## Progress Tracking

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 12. Semantic Drift Analysis | v2.0 | 3/3 | Complete | 2026-03-19 |
| 13. Vendor-Neutral Manifolds | v2.0 | 2/2 | Complete | 2026-03-19 |
| 14. Grammar Induction | v2.0 | 2/2 | Complete | 2026-03-19 |
| 15. The FSM-Oracle | v2.0 | 2/2 | Complete | 2026-03-19 |
| 16. Semantic Constraint Logic | v2.0 | 1/1 | Complete | 2026-03-20 |
| 17. Universal Ledger Library | v2.0 | 0/3 | Planned | - |
| 18. Semantic Mock Server | v3.0 | 0/3 | Planned | - |
| 19. State-of-the-World Manifests | v3.0 | 0/3 | Planned | - |
| 20. SSH/CLI Protocol Integration | v3.0 | 0/3 | Planned | - |

---
*Roadmap created: 2026-02-22*
*Last updated: 2026-03-20*

# Roadmap: cliscrape

## Milestones

- ✅ **v1.0 MVP** — shipped 2026-02-22
- ✅ **v1.5 Production Hardening** — shipped 2026-03-19
- ✅ **v2.0 The Network Compiler** — complete 2026-05-02
- 📋 **v3.0 Isomorphic Ecosystem** — planned

> **Operating model (2026-05-01):** active work is tracked in beads (`br`), not
> as `phases/NN-*/PLAN.md` directories. This roadmap remains the milestone
> narrative; bead IDs link to executable backlog. Run `br ready` for actionable
> work across the project, or `br dep tree <epic-id> -d up` to see what an epic
> unblocks. Phase numbering survives only as historical reference for v1.0 +
> v1.5 work under `archive/phases/`.

## Completed Milestones

<details>
<summary>✅ v1.0 MVP (shipped 2026-02-22)</summary>

Phases 1-5 (historical numbering): Core Parsing Engine · Legacy Compatibility & CLI · Modern Ergonomic Templates · TUI Debugger Foundation (Live Lab) · TUI Advanced Debugging (State Tracer).

Artifacts in `archive/phases/01-*` through `archive/phases/05-*`.

</details>

<details>
<summary>✅ v1.5 Production Hardening (shipped 2026-03-19)</summary>

Phases 6-11 (historical numbering): Template Library Foundation · Compatibility Validation Suite · TUI Integration · Edge Case Hardening · Production Logging · Documentation & Authoring Guide.

Artifacts in `archive/phases/06-*` through `archive/phases/11-*`. Outstanding reconciliation work (catalog page + doc-validation harness) tracked as `cliscrape-mes`.

</details>

## ✅ v2.0 The Network Compiler (Complete 2026-05-02)

**Milestone goal (achieved):** move from simple parsing to structured state observation and vendor-neutral operational schemas.

**Shipped (legacy phase numbering, archived):**

- [x] Semantic Drift Analysis (Phase 12) — operational diffing between states.
- [x] Vendor-Neutral State Mappings (Phase 13) — common schema mapping primitives.
- [x] Grammar Induction (Phase 14) — statistical structural inference for templates.
- [x] FSM-Oracle (Phase 15) — best-effort generation and advisory round-trip checks. Strict mode tracked by `cliscrape-nuy` (debt).
- [x] Semantic Constraint Logic (Phase 16) — policy-aware parsing with boundary assertions. Artifacts under `phases/16-semantic-constraint-logic/`.

**Shipped via beads:**

- [x] **Universal Ledger Library** (`cliscrape-1s8`, closed 2026-05-02) — 5 vendor-neutral schemas (`version`, `interface`, `bgp_neighbor`, `lldp_neighbor`, `route`) with typed registry, `claims_schema:` declaration, format validators (ipv4/ipv6/ip/mac/cidr/asn), 5 embedded templates wired, CI compliance gate. Schema design surfaced 2 amendments grounded in real fixtures. 189 tests across 18 binaries enforce the contract.

## 📋 v3.0 Isomorphic Ecosystem (Planned)

**Milestone goal:** use isomorphic FSM execution to power high-fidelity network simulation and generative testing.

- [ ] **Semantic Mock Server** — `cliscrape-quw` (epic). Blocked by `cliscrape-1s8`. Decompose into children once schemas land.
- [ ] **State-of-the-World Manifests** — `cliscrape-woo` (epic). Blocked by `cliscrape-quw`.
- [ ] **SSH/CLI Protocol Integration** — `cliscrape-lw0` (epic). Blocked by `cliscrape-quw` + `cliscrape-woo`.

## Open Beads at a Glance

| Bead | Title | Milestone | Status |
|------|-------|-----------|--------|
| `cliscrape-1s8` | Universal Ledger Library (epic) | v2.0 | ✅ Closed |
| `cliscrape-quw` | Semantic Mock Server (epic) | v3.0 | Ready |
| `cliscrape-woo` | State-of-the-World Manifests (epic) | v3.0 | Blocked |
| `cliscrape-lw0` | SSH/CLI Protocol Integration (epic) | v3.0 | Blocked |
| `cliscrape-quw` | Semantic Mock Server (epic) | v3.0 | Blocked |
| `cliscrape-woo` | State-of-the-World Manifests (epic) | v3.0 | Blocked |
| `cliscrape-lw0` | SSH/CLI Protocol Integration (epic) | v3.0 | Blocked |
| `cliscrape-mes` | v1.5 reconciliation: catalog + doc validation | v1.5 (debt) | P2 |
| `cliscrape-nuy` | Strict FSM-Oracle verification | v2.0 (debt) | P2 |

For the full list, run `br list` or `br ready`.

---
*Roadmap created: 2026-02-22*
*Last updated: 2026-05-01 (migrated to beads; phase numbering retired except as historical reference)*

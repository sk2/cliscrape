# Project State - cliscrape

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-22)

**Core Value:** Extremely fast, reliable parsing of semi-structured CLI output into structured data, regardless of whether the template is legacy TextFSM or the new ergonomic format.

**Current Focus:** Milestone v2.0 - The Network Compiler

## Current Position

Active milestone: v2.0 The Network Compiler
Open epic: Universal Ledger Library (`cliscrape-1s8`) with three sequential children (`cliscrape-1s8.1` → `1s8.2` → `1s8.3`)
Status: ready to work `cliscrape-1s8.1` (define common schemas)
Last activity: 2026-05-01 — migrated active task tracking from GSD phases to beads; dropped Phase NN framing from active work.

Milestone progress: v2.0 epic-level — 5 of 6 closed (Semantic Drift, Vendor-Neutral State Mappings, Grammar Induction, FSM-Oracle, Semantic Constraint Logic). Universal Ledger remains.

## Operating Model (as of 2026-05-01)

- **Beads (`br`)** is the operational task backlog. All active and future work lives there.
- **Milestones** (v2.0, v3.0) are the human-readable grouping in `ROADMAP.md`.
- **Phase numbering is retired** for active work. Phases 1-16 keep their numbers as historical reference under `archive/phases/` (and `phases/16-*` until v2.0 closes), but new work is named after the bead it lives in, not "Phase NN".
- **`BACKLOG.md`** is a thin pointer index, not the source of truth.
- To find what's next: `br ready` (top of the list = highest-priority unblocked).
- To inspect any epic: `br show <epic-id>` and `br dep tree <epic-id> -d up`.

## Performance Metrics

**Velocity:**
- Total completed phases: 16 (5 v1.0 + 6 v1.5 + 5 v2.0)
- Total completed plans: 44
- Average duration: ~10 min (recent)
- Total execution time: Not tracked

**By Milestone:**

| Milestone | Status |
|-----------|--------|
| v1.0 MVP | Complete (2026-02-22) |
| v1.5 Production Hardening | Complete (2026-03-19); reconciliation tracked as `cliscrape-mes` |
| v2.0 The Network Compiler | 5 of 6 epics closed; Universal Ledger (`cliscrape-1s8`) remains |
| v3.0 Isomorphic Ecosystem | Planned: `cliscrape-quw` → `cliscrape-woo` → `cliscrape-lw0` |

**Recent Trend:**
- v1.0 shipped with 77 passing tests, 4.1M lines/sec throughput
- v1.5 shipped with the user and authoring guides
- v2.0 foundation work is complete; the Universal Ledger Library is the only open milestone item
- Trend: project focus has shifted from production hardening to semantic, policy-aware parsing and vendor-neutral state modeling

**Recent Plan Metrics:**

| Plan | Duration (s) | Tasks | Files |
|------|--------------|-------|-------|
| Phase 06 P01 | 502 | 4 tasks | 2 files |
| Phase 06 P02 | 244 | 2 tasks | 3 files |
| Phase 06 P03 | 544 | 2 tasks | 4 files |
| Phase 06 P04 | 413 | 3 tasks | 10 files |
| Phase 07 P01 | 864 | 3 tasks | 13 files |
| Phase 07 P03 | 296 | 2 tasks | 2 files |
| Phase 07 P04 | 160 | 2 tasks | 1 files |
| Phase 07 P05 | 42 | 1 tasks | 1 files |

## Accumulated Context

### Decisions

See PROJECT.md Key Decisions table for full history.
Recent decisions affecting current work:

- **Warning-Returning Loader API:** Library returns warnings without stderr printing (enables clean library usage)
- **TTY-Aware Format Auto:** format=auto resolves to table (TTY) or JSON (non-TTY) for Unix-style contract
- **Full Variable Snapshots in Trace:** Store complete variable state per trace event for debugging clarity
- **Embed-time Type Conversion:** Convert captured strings to typed JSON at record emission with heuristics
- **Fault-Tolerant Metadata:** Metadata extraction never blocks template usage - always returns valid struct with defaults (06-02)
- **Library Type Consistency:** Use crate::TemplateFormat from lib.rs for library API consistency (06-02)
- **Subdirectory Support in Template Names:** Template names allow forward slashes for organization while maintaining security (06-03)
- **Field Introspection API:** FsmParser exposes field_names() for template inspection without breaking encapsulation (06-03)
- **Metadata Field in Modern Templates:** ModernTemplateDoc accepts metadata field (skipped during serialization) to allow metadata sections (06-04)
- **Vendor-First Fixture Organization:** Test fixtures organized by vendor/template hierarchy for maintainability (07-01)
- **External Snapshot Storage:** Use tests/snapshots/ directory for better diff visualization vs inline snapshots (07-01)
- **Test-time Coverage Validation:** Coverage calculated at test-time only for developer feedback; 80% default threshold (07-04)
- **Modern Rust toolchain action:** Use dtolnay/rust-toolchain@stable instead of deprecated actions-rs for reliable CI (07-05)
- **Locked cargo-insta installation:** Use --locked flag for deterministic CI behavior (07-05)

### Pending Todos

Tracked in beads. To list: `br ready` or `br list --json`.

Recently converted (2026-05-01):
- `cliscrape-vwo` (P3) — Live Lab TUI interactive smoke test (was: deferred from Phase 4)
- `cliscrape-442` (P3) — Interactive converter smoke test (was: deferred from Phase 3)
- `cliscrape-mes` (P2) — Phase 11 plan 03 reconciliation: template catalog + doc validation

### Blockers/Concerns

None active. Historical entries:
- `v1.5-MILESTONE-AUDIT.md` was moved to `archive/v1.5-MILESTONE-AUDIT.md` on 2026-05-01 (no longer surfaced as active status source).
- Phase 11 reconciliation work is now tracked as bead `cliscrape-mes` rather than a free-floating concern.

## Session Continuity

Last session: 2026-05-01 00:00Z
Stopped at: Migrated Phase 17-20 task tracking from GSD `phases/NN-*/PLAN.md` directories to beads. Phase 17 epic + 3 sequential children + Phase 18-20 epics with inter-phase deps. Converted 3 carry-over TODOs to beads (`cliscrape-vwo`, `cliscrape-442`, `cliscrape-mes`). Archived completed-milestone phase directories (Phases 1-11) and the v1.5 audit to `.planning/archive/`.
Resume file: None
Next ready work: `cliscrape-1s8.1` (Define common schemas)

# Project State - cliscrape

## Project Reference

See: .planning/PROJECT.md (updated 2026-02-22)

**Core Value:** Extremely fast, reliable parsing of semi-structured CLI output into structured data, regardless of whether the template is legacy TextFSM or the new ergonomic format.

**Current Focus:** Milestone v1.5 - Template Ecosystem & Production Hardening

## Current Position

Phase: 7 of 11 (Compatibility Validation Suite)
Plan: 2 of 4 in current phase
Status: Active
Last Activity: 2026-02-23 — Completed 07-02 (negative test coverage)

Progress: [█████░░░░░] 50% (v1.0 complete: 5/11 phases, v1.5: Phase 6 complete, Phase 7: 2/4 plans)

## Performance Metrics

**Velocity:**
- Total plans completed: 32 (28 v1.0 + 4 v1.5)
- Average duration: ~6 min (recent)
- Total execution time: Not tracked

**By Milestone:**

| Milestone | Phases | Plans | Status |
|-----------|--------|-------|--------|
| v1.0 MVP | 5 | 28/28 | Complete (2026-02-22) |
| v1.5 Template Ecosystem | 6 | 4/? | Phase 6 complete |

**Recent Trend:**
- v1.0 shipped with 77 passing tests, 4.1M lines/sec throughput
- v1.5 Phase 6: 4/4 plans complete (library foundation, metadata extraction, CLI discovery, template population)
- Trend: Template ecosystem foundation complete, ready for production use

**Recent Plan Metrics:**

| Plan | Duration (s) | Tasks | Files |
|------|--------------|-------|-------|
| Phase 06 P01 | 502 | 4 tasks | 2 files |
| Phase 06 P02 | 244 | 2 tasks | 3 files |
| Phase 06 P03 | 544 | 2 tasks | 4 files |
| Phase 06 P04 | 413 | 3 tasks | 10 files |
| Phase 07 P02 | 727 | 2 tasks | 25 files |

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
- **Snapshot-Based Negative Testing:** Document parser behavior via snapshots instead of strict error assertions - parser is lenient by design (07-02)
- **Accept HashMap Field Ordering Variation:** Snapshot tests tolerate field ordering changes due to HashMap's randomized iteration (07-02)

### Pending Todos

- [ ] Run Live Lab TUI interactive verification (deferred from Phase 4 to milestone verification)
- [ ] Run interactive converter smoke test (deferred from Phase 3 to milestone verification)

### Blockers/Concerns

None currently. v1.5 starts fresh with research-informed architecture.

## Session Continuity

Last session: 2026-02-23
Stopped at: Completed 07-02-PLAN.md (Negative test coverage - Phase 7 in progress)
Resume file: None

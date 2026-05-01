# Backlog Index: The Network Compiler

> **Operating model (2026-05-01):** detailed task tracking lives in beads (`br`).
> This file is a milestone-level pointer index, not the source of truth. For
> ready-to-work items, run `br ready`. For the dependency tree of any epic,
> run `br dep tree <epic-id> -d up`.

## Active Backlog (v2.0)

### Universal Ledger Library — `cliscrape-1s8`

Closes v2.0. Seven children organized in three tiers (the original 3-child
decomposition was decomposed further once "solid" was the goal — see the
epic description for rationale).

**Tier 1 — Foundation:**

| Bead | Slice | Status | Depends on |
|------|-------|--------|------------|
| `cliscrape-1s8.1` | Define common schemas | ✅ Closed | — |
| `cliscrape-1s8.4` | Typed schema loader + template validation | Ready | — |
| `cliscrape-1s8.5` | Real fixtures for bgp/lldp/route schemas | Ready | — |

**Tier 2 — Contract:**

| Bead | Slice | Depends on |
|------|-------|------------|
| `cliscrape-1s8.6` | Top-level `claims_schema:` declaration | `cliscrape-1s8.4` |
| `cliscrape-1s8.7` | Format validators (ipv4, ipv6, mac, cidr, asn) | `cliscrape-1s8.4` |

**Tier 3 — Application:**

| Bead | Slice | Depends on |
|------|-------|------------|
| `cliscrape-1s8.2` | Wire embedded templates | `cliscrape-1s8.5`, `cliscrape-1s8.6` |
| `cliscrape-1s8.3` | Schema compliance test suite | `cliscrape-1s8.2`, `cliscrape-1s8.7` |

Related: `br-71zd` (Template Ecosystem: Core Network Commands) is the home for adding new templates beyond the current embedded set; not a blocker for this epic.

Inspect: `br show cliscrape-1s8` · `br dep tree cliscrape-1s8 -d up`

## Future Backlog (v3.0)

Each is a single epic bead. Decompose into child beads when its predecessor closes — avoids rework once upstream design is concrete.

### Semantic Mock Server — `cliscrape-quw`

Blocked by `cliscrape-1s8`. Interactive `cliscrape simulate` shell that synthesizes vendor-authentic CLI output from JSON state.

### State-of-the-World Manifests — `cliscrape-woo`

Blocked by `cliscrape-quw`. Manifest format mapping device hostnames to commands and JSON state.

### SSH/CLI Protocol Integration — `cliscrape-lw0`

Blocked by `cliscrape-quw` + `cliscrape-woo`. Run the simulator as a persistent SSH daemon for end-to-end tooling integration.

---

## Adding to the Backlog

Don't add bullet lists here. Create a bead instead:

```bash
br create "Title" -t task -p P1 --description "..." --parent <epic-id-if-applicable>
br dep add <new-id> <blocker-id>      # if it depends on something
br sync --flush-only                  # export to .beads/issues.jsonl for git
```

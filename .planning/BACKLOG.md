# Backlog Index: The Network Compiler

> **Operating model (2026-05-01):** detailed task tracking lives in beads (`br`).
> This file is a milestone-level pointer index, not the source of truth. For
> ready-to-work items, run `br ready`. For the dependency tree of any epic,
> run `br dep tree <epic-id> -d up`.

## Active Backlog (v2.0)

### Universal Ledger Library — `cliscrape-1s8`

Closes v2.0. Three sequential children:

| Bead | Slice | Depends on |
|------|-------|------------|
| `cliscrape-1s8.1` | Define common schemas (`interface`, `bgp_neighbor`, `lldp_neighbor`, `version`, `route`) | — |
| `cliscrape-1s8.2` | Wire `common_schema` mappings into embedded YAML templates | `cliscrape-1s8.1` |
| `cliscrape-1s8.3` | Schema compliance test suite (`tests/schema_compliance.rs`) | `cliscrape-1s8.2` |

Related: `br-71zd` (Template Ecosystem: Core Network Commands) consumes the schemas once defined.

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

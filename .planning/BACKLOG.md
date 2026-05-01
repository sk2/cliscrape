# GSD Backlog: The Network Compiler

> **Operating model (2026-05-01):** detailed task tracking lives in beads (`br`).
> This file remains as a milestone-level pointer index. For ready-to-work items
> across all phases, run `br ready`. For the dependency tree of any phase epic,
> run `br dep tree <epic-id> -d up`.

## Active Backlog

### Phase 17: Universal Ledger Library

Tracked as bead epic **`cliscrape-1s8`** with three sequential children:

| Bead | Slice | Depends on |
|------|-------|------------|
| `cliscrape-1s8.1` | Define common schemas (`interface`, `bgp_neighbor`, `lldp_neighbor`, `version`, `route`) | — |
| `cliscrape-1s8.2` | Wire `common_schema` mappings into embedded YAML templates | `cliscrape-1s8.1` |
| `cliscrape-1s8.3` | Schema compliance test suite (`tests/schema_compliance.rs`) | `cliscrape-1s8.2` |

Related: `br-71zd` (Template Ecosystem: Core Network Commands) consumes the schemas once defined.

Inspect: `br show cliscrape-1s8` · `br dep tree cliscrape-1s8 -d up`

## Future Backlog

### Phase 18: Semantic Mock Server (`cliscrape-quw`)

Single epic bead. Decompose into child beads when Phase 17 closes (avoids rework once schemas are concrete). Blocked by `cliscrape-1s8`.

### Phase 19: State-of-the-World Manifests (`cliscrape-woo`)

Single epic bead. Blocked by `cliscrape-quw`.

### Phase 20: SSH/CLI Protocol Integration (`cliscrape-lw0`)

Single epic bead. Blocked by `cliscrape-quw` and `cliscrape-woo`.

---

## Adding to the Backlog

Don't add bullet lists here. Create a bead instead:

```bash
br create "Title" -t task -p P1 --description "..." --parent <epic-id-if-applicable>
br dep add <new-id> <blocker-id>      # if it depends on something
br sync --flush-only                  # export to .beads/issues.jsonl for git
```

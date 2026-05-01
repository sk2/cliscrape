# Common Schemas (Universal Ledger)

This directory holds the source-of-truth specifications for cliscrape's
"Universal Ledger" — the small set of vendor-neutral record shapes that the
`--common` flag projects parsed CLI output onto.

## What lives here

One YAML file per schema. The current set:

- [`version.yaml`](version.yaml) — device version / identity (`show version`)
- [`interface.yaml`](interface.yaml) — interface state (`show interfaces`, `show ip interface brief`)
- [`bgp_neighbor.yaml`](bgp_neighbor.yaml) — BGP peers (`show ip bgp summary`, `show bgp neighbors`)
- [`lldp_neighbor.yaml`](lldp_neighbor.yaml) — LLDP adjacencies (`show lldp neighbors`)
- [`route.yaml`](route.yaml) — routing table entries (`show ip route`, `show route`)

## File format

Each file is a YAML document with this structure:

```yaml
schema: <schema_name>          # short snake_case identifier
version: 1                      # spec version; bump on breaking changes
description: |                  # one or two paragraphs
  Human-readable description.

applies_to:                     # commands typically producing this shape
  - "show version"

keys:
  <key_name>:
    type: string | int          # engine-supported types only
    required: true | false
    description: |              # what the key means + value conventions
      ...
```

## Engine type system

cliscrape's modern template format only supports two field types:

- `string` — captured as-is
- `int` — coerced from the captured string

There is no native `bool`, `ipv4_address`, or timestamp type today. The schemas
reflect that:

- Boolean-shaped values (admin/oper status) use `string` with documented
  enumerations like `"up"` / `"down"`.
- IP addresses, MACs, and prefixes are `string` with a documented format.

## Required vs optional vs missing

A key marked `required: true` means: any template that *claims* this schema
must produce this key. A key marked `required: false` may be omitted by some
templates (vendor doesn't expose it in the source command).

If a template doesn't produce an optional key, the field is **simply absent
from the record** — no explicit `null` is emitted. This matches the engine's
existing `serde_json` behavior.

## How a template "claims" a schema

For now, a template claims a schema implicitly: it has at least one field
with `common_schema: <key>` that matches a key from this directory. The
schema-compliance test (`cliscrape-1s8.3`, future work) will infer membership
this way and verify required keys are mapped.

A future extension may add an explicit top-level `claims_schema: <name>`
declaration on templates for stricter validation. The spec format above is
forward-compatible with that.

## Adding or extending a schema

1. Open a bead first — schema design is high-leverage, low-volume work.
2. Only add keys that **at least two of {Cisco IOS, Arista EOS, Juniper Junos}**
   actually expose. Vendor-unique fields stay in the raw record.
3. Bump `version:` if existing keys change shape; add a migration note in the
   description.
4. Update the user-facing docs at `docs/guides/COMMON_SCHEMAS.md`.

## See also

- [User guide: Universal Ledger section](../docs/guides/USER_GUIDE.md)
- [Authoring guide: `common_schema:` field](../docs/guides/AUTHORING.md)
- Bead epic: `cliscrape-1s8` (Universal Ledger Library)

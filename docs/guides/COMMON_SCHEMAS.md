# Common Schemas (Universal Ledger)

The Universal Ledger is cliscrape's small set of vendor-neutral record shapes.
When you pass `--common`, fields that declare `common_schema: <key>` are
renamed to the schema's canonical key, so output is the same regardless of
which vendor produced the input.

The source-of-truth specs live under [`common_schemas/`](../../common_schemas/).
This guide explains how to use them and how the projection works.

## Available schemas

| Schema | Purpose | Required keys |
|--------|---------|---------------|
| [`version`](../../common_schemas/version.yaml) | Device identity from `show version` | `hostname`, `model`, `version` |
| [`interface`](../../common_schemas/interface.yaml) | Per-interface state | `name` |
| [`bgp_neighbor`](../../common_schemas/bgp_neighbor.yaml) | BGP peers | `neighbor`, `remote_as`, `state` |
| [`lldp_neighbor`](../../common_schemas/lldp_neighbor.yaml) | LLDP adjacencies | `local_interface`, `remote_port` |
| [`route`](../../common_schemas/route.yaml) | Routing-table entries | `prefix`, `protocol` |

Each spec file lists the full key set with types, required-or-optional flags,
and value conventions (e.g. status enumerations, MAC formatting, protocol-code
normalization).

## How it works

The engine treats `common_schema:` as a per-field rename. Given a template
field:

```yaml
version: 1
claims_schema: version

fields:
  hostname:
    type: string
    common_schema: hostname
```

a parse with `--common` will emit the field under the key `hostname`. Without
`--common`, the field stays under its vendor-specific name (which may already
be `hostname` or may be something else like `device_name`).

Keys that aren't mapped to a `common_schema` value pass through unchanged
under their original names.

### Declaring schema membership: `claims_schema:`

Templates should declare a top-level `claims_schema:` so the validator can
enforce coverage and disambiguate bare references:

```yaml
claims_schema: interface             # single
claims_schema: [interface, version]  # multiple
```

When `claims_schema:` is present, the loader enforces that every required
key in each claimed schema is mapped by some field. A template claiming
`interface` without mapping `name` fails to load with a precise error.

When `claims_schema:` is absent (legacy path), bare references must resolve
unambiguously across the entire registry — bare `uptime` would fail because
both `version` and `bgp_neighbor` declare it. Use a qualified reference
(`common_schema: version.uptime`) or add `claims_schema:` to disambiguate.

## Worked example

Cisco IOS `show version` template field declarations might look like:

```yaml
fields:
  hostname:
    type: string
    common_schema: hostname
  model:
    type: string
    common_schema: model
  version:
    type: string
    common_schema: version
  serial:
    type: string
    common_schema: serial
  uptime:
    type: string
    common_schema: uptime
```

Arista EOS `show version` might capture different field names internally:

```yaml
fields:
  device_name:
    type: string
    common_schema: hostname
  hardware_model:
    type: string
    common_schema: model
  software_image_version:
    type: string
    common_schema: version
```

Both produce the same shape under `--common`:

```json
{
  "hostname": "...",
  "model": "...",
  "version": "..."
}
```

That's the Universal Ledger projection: vendor-neutral keys, identical record
structure.

## Required vs optional

A key marked `required: true` in a schema spec means: any template that claims
this schema must produce that key. The future schema-compliance test
(`cliscrape-1s8.3`) will enforce this in CI.

Optional keys may be omitted by templates whose source command doesn't expose
the value. Omitted keys are simply absent from the record — no explicit
`null` is emitted. Consumers should test for key presence (not for `null`)
when a value is optional.

## Type discipline

The engine's modern template format only supports `string` and `int` field
types. The schemas reflect that constraint:

- Boolean-shaped values (admin/oper status) use `string` with documented
  enumerations like `"up"` / `"down"` / `"administratively down"`.
- IP addresses, MACs, and prefixes are `string` with a documented format.
- Counts, ASNs, MTUs, metrics, and distances are `int`.

When a template captures a value in a different format than the schema
documents (e.g. Cisco `001c.2d3e.4f50` instead of `00:1c:2d:3e:4f:50`),
the template should normalize at parse time, not the consumer.

### Format validation

Schema keys can declare an optional `format:` (one of `ipv4`, `ipv6`, `ip`,
`mac`, `cidr`, `asn`). The engine validates the captured value at
`--common`-projection time. Mismatches emit a `tracing::warn!` event with
`event = "format_violation"`; under `--strict-policy` they fail the parse.

Today, the following keys carry format declarations:

| Schema | Key | Format |
|--------|-----|--------|
| `interface` | `mac_address` | `mac` |
| `interface` | `ipv4_address` | `ipv4` |
| `bgp_neighbor` | `neighbor` | `ip` |
| `bgp_neighbor` | `remote_as` | `asn` |
| `bgp_neighbor` | `local_as` | `asn` |
| `route` | `prefix` | `cidr` |
| `route` | `next_hop` | `ip` |

## CI gates

Two test files enforce the contract on every CI run:

- `tests/common_schema_validation.rs` — exercises template-load-time
  validation (typo suggestions, type mismatches, ambiguity, claims_schema
  required-key enforcement).
- `tests/schema_compliance.rs` — discovers every embedded template that
  declares `claims_schema:`, parses each matching fixture under
  `tests/fixtures/<vendor>/<command>/`, projects to canonical keys, and
  asserts every required key is present and no format declarations are
  violated. Includes a synthetic negative-path test that proves the format
  check catches bad data.

## Adding a new schema

Schema design is high-leverage. Open a bead first.

1. Only add a schema if it's broadly meaningful — at least two of
   {Cisco IOS, Arista EOS, Juniper Junos} should expose the source data via
   a comparable `show` command.
2. Only add keys at least two vendors actually expose. Vendor-unique fields
   stay under their original names in the raw record.
3. Write a YAML file under [`common_schemas/`](../../common_schemas/)
   following the format documented in
   [common_schemas/README.md](../../common_schemas/README.md).
4. Cross-link this guide and add the schema to the table above.
5. The compliance test (when it lands) will pick up the new file
   automatically.

## See also

- [Authoring guide: `common_schema:` field reference](AUTHORING.md)
- [User guide: Universal Ledger section](USER_GUIDE.md#universal-ledger-common-schema)
- [Source specs: `common_schemas/`](../../common_schemas/)

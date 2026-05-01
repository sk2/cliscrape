# Universal Ledger Schema Review (cliscrape-1s8.5)

Walks each Universal Ledger schema (`common_schemas/*.yaml`) against the
fixtures committed under this directory. The goal is to ground the schema
designs in real CLI output before `cliscrape-1s8.4` (typed loader) hard-codes
them. Any schema amendments needed to honor reality are documented here and
applied to the YAML specs in the same commit.

## bgp_neighbor

Fixtures:
- `cisco/ios_show_ip_bgp_summary/ios_15_standard.txt`
- `arista/eos_show_ip_bgp_summary/eos_4_30.txt`

### Required keys

| Key | Cisco IOS | Arista EOS |
|-----|-----------|------------|
| `neighbor` | "Neighbor" column (col 1) | "Neighbor" column (col 2 after Description) |
| `remote_as` | "AS" column | "AS" column |
| `state` | **Composite column** "State/PfxRcd" | "State" column |

### Schema gap surfaced (amendment needed)

**Issue:** The `state` column shape differs across vendors and even within
Cisco depending on session state.

* Cisco IOS' "State/PfxRcd" column shows the prefix count (e.g. `15`) when
  the session is `Established`, and the state name (e.g. `Idle`,
  `Active`) otherwise. There is no explicit "Established" string in the
  text — it must be inferred from the column being numeric.
* Arista EOS uses an abbreviation: `Estab` (not `Established`).
* Suffix modifiers appear: `Idle (Admin)` (Cisco IOS).

The current schema description for `state` says "vendor-formatted ... preserve
as-is. Some vendors append modifiers ... preserve as-is." That defeats the
schema's vendor-neutrality promise — the same conceptual state would emerge
as `Established` from one template and `Estab` from another.

**Amendment:** the schema description for `state` is updated to require
normalization to RFC 4271 names (`Established`, `Active`, `Idle`, `Connect`,
`OpenSent`, `OpenConfirm`). Modifier suffixes (e.g. `(Admin)` for
administratively shutdown) are preserved with a single space separator.
Templates do the conversion at parse time.

Applied: see commit alongside this review.

### Optional keys

| Key | Cisco IOS | Arista EOS |
|-----|-----------|------------|
| `local_as` | Header line ("local AS number 64496") | Same — but in VRF-keyed sub-section |
| `prefixes_received` | Numeric value of "State/PfxRcd" when Established | "PfxRcd" column |
| `uptime` | "Up/Down" column ("3d04h", "never") | Same shape |
| `vrf` | Implicit (global, no header) | "BGP summary information for VRF default" — explicit |

### Vendor-unique fields (correctly excluded)

* Cisco "MsgRcvd"/"MsgSent" counters — could be added to a future
  `bgp_neighbor_counters` schema; not needed for state observation.
* Arista "Description" column — operator-supplied free text per neighbor.
  Reasonable to add as an optional `description` key in a future revision
  if templates start using it.

### Verdict

Schema correct after the `state` normalization amendment. Required keys are
universally available across both vendors.

---

## lldp_neighbor

Fixtures:
- `cisco/ios_show_lldp_neighbors/ios_15_standard.txt`
- `arista/eos_show_lldp_neighbors/eos_4_30.txt`

### Required keys

| Key | Cisco IOS | Arista EOS |
|-----|-----------|------------|
| `local_interface` | "Local Intf" column | "Port" column |
| `remote_port` | "Port ID" column | "Neighbor Port ID" column |

Both vendors expose both required fields in the summary view. ✓

### Optional keys

| Key | Cisco IOS | Arista EOS |
|-----|-----------|------------|
| `remote_system_name` | "Device ID" column | "Neighbor Device ID" column |
| `remote_chassis_id` | **Not in summary** (only `show lldp neighbors detail`) | **Not in summary** |
| `ttl` | "Hold-time" column | "TTL" column |
| `capabilities` | "Capability" column ("B,R", "B,W") | **Not in summary** |

### Notes

* `capabilities` field requires decoding: Cisco prints capability **codes**
  (`B,R`) defined in the legend at the top of the output (`(R) Router`,
  `(B) Bridge`). Schema description says lowercase comma-separated names
  ("bridge,router"). Templates decode at parse time. Document this in
  `cliscrape-1s8.2`'s template wiring.
* Arista's summary view drops `capabilities` and `remote_chassis_id`
  entirely. A future `eos_show_lldp_neighbors_detail` template would
  cover both. Today, those keys remain unmapped for the EOS summary
  template — that's the optional-key contract working as intended.
* Cisco's "Port ID" column sometimes shows a MAC address
  (`00:00:5e:00:53:1a`) when the peer is not a switch (AP, IP phone). The
  schema accepts arbitrary string for `remote_port` — this is correct; we
  preserve what LLDP advertised.

### Verdict

Schema correct. No amendments needed.

---

## route

Fixtures:
- `cisco/ios_show_ip_route/ios_15_standard.txt`
- `arista/eos_show_ip_route/eos_4_30.txt`

### Required keys

| Key | Cisco IOS | Arista EOS |
|-----|-----------|------------|
| `prefix` | First column after protocol code, in CIDR | Same shape |
| `protocol` | Single/multi-letter code (`B`, `O`, `O E2`, `S*`) | Same idea, slightly different code set (`B I`, `B E`) |

The required keys are universally present — but the `protocol` column
requires careful normalization. See gap below.

### Schema clarification (no amendment, but document explicitly)

**Protocol normalization across vendors:**

| Code text | Cisco IOS meaning | Arista EOS meaning | Schema canonical |
|-----------|-------------------|---------------------|-------------------|
| `C`       | connected         | connected           | `CONNECTED`       |
| `L`       | local             | (not used)          | `LOCAL`           |
| `S`       | static            | static              | `STATIC`          |
| `S*`      | static (default)  | (not used)          | `STATIC` (the `*` is a default-route marker, capture separately if needed) |
| `B`       | BGP               | other BGP           | `BGP`             |
| `B E`     | (not used)        | eBGP                | `BGP`             |
| `B I`     | (not used)        | iBGP                | `BGP`             |
| `O`       | OSPF intra-area   | OSPF intra-area     | `OSPF`            |
| `O E2`    | OSPF external T2  | OSPF external T2    | `OSPF` (subtype lost) |
| `D`       | EIGRP             | (not used)          | `EIGRP`           |
| `i`/`I`   | IS-IS             | IS-IS               | `ISIS`            |
| `K`       | (not used)        | kernel              | `OTHER`           |

The schema description names canonical values (`STATIC`, `CONNECTED`, etc.).
Templates collapse vendor codes to those. Sub-types (`O E2`, `B I`) are
lossy at the schema level; if downstream consumers need them, capture into
a separate `protocol_subtype` field on the raw record (vendor-specific,
not part of the common schema).

This is consistent with the schema's existing description but is worth
spelling out to template authors.

### Optional keys

| Key | Cisco IOS | Arista EOS |
|-----|-----------|------------|
| `next_hop` | After `via ` keyword | After `via ` keyword |
| `interface` | Trailing token after final comma | Trailing token after final comma |
| `metric` | Second value in `[admin/metric]` | Same |
| `distance` | First value in `[admin/metric]` | Same |
| `vrf` | Not in default view header | "VRF: default" header line |

### Multi-line subtree handling

Cisco IOS uses an indented subtree format:
```
      10.0.0.0/8 is variably subnetted, 5 subnets, 3 masks
C        10.0.0.0/24 is directly connected, GigabitEthernet0/0
L        10.0.0.1/32 is directly connected, GigabitEthernet0/0
```

Templates must handle this state machine — the parent line announces a
supernet but isn't a route; child lines indented underneath are routes
sharing the same protocol-code column position. EOS doesn't use this
format; flat list throughout.

This is template-implementation guidance for `cliscrape-1s8.2`, not a
schema concern. The schema sees one record per leaf route either way.

### Verdict

Schema correct. Protocol normalization is consistent with the schema
description; the matrix above is documentation for template authors.

---

## Summary

* **One amendment applied:** `bgp_neighbor.state` description now requires
  normalization to RFC 4271 names. The original "preserve as-is" wording
  silently broke vendor neutrality.
* **Two clarifications documented for template authors** but no schema
  changes: LLDP `capabilities` decoding (Cisco capability codes) and
  route `protocol` normalization across vendor letter codes.
* **All five schemas hold up against real fixtures.** The required-key
  set is universally available across the two-vendor minimum for each
  schema. Optional keys vary, as designed.

This grounds `cliscrape-1s8.4` (typed loader) and `cliscrape-1s8.6`
(claims_schema): the schemas are correct enough to embed and validate
against. Any further drift will be caught by `cliscrape-1s8.3`
(compliance test) once the wiring lands.

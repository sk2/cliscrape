# Template Authoring Guide

This guide covers how to write, test, and mathematically verify modern YAML templates for `cliscrape`.

## The Modern YAML Format

While `cliscrape` is 100% compatible with legacy TextFSM templates, it introduces a strict schema-driven YAML/TOML format. This format eliminates "regex soup" and enables advanced features like native type coercion, semantic diffing, and isomorphic simulation.

### Basic Structure

```yaml
version: 1

metadata:
  description: "Parse show version"
  compatibility: "Vendor OS 1.0"
  version: "1.0.0"

fields:
  hostname:
    type: string
    pattern: '\S+'
    identity: true
  uptime:
    type: string
    pattern: '.*'
    ignore: true
  mtu:
    type: int
    pattern: '\d+'

patterns:
  - regex: '^Hostname: ${hostname}'
    record: false
  - regex: '^Uptime: ${uptime}'
    record: false
  - regex: '^MTU: ${mtu}'
    record: true
```

## Field Metadata (The Network Compiler)

The `fields` block defines the operational schema contract. By adding specific metadata to your fields, you unlock `cliscrape`'s advanced analytical capabilities.

* `type: int | string` - Enables native JSON type coercion (e.g., `"1500" -> 1500`).
* `identity: true` - Marks this field as a unique primary key for the record (e.g., an interface name). Used by the `diff` engine to track state changes.
* `ignore: true` - Marks this field as syntactic noise (e.g., uptime, timestamps). The `diff` engine will ignore changes to this field.
* `common_schema: 'key_name'` - Maps this vendor-specific variable to a Universal Ledger key (e.g., `vendor_interface -> interface`).
* `filldown: true` - Retains the captured value across multiple record emissions until overwritten.
* `list: true` - Accumulates multiple matches into a JSON array instead of overwriting.

## Grammar Induction (Auto-Generation)

If you are faced with a new, undocumented CLI output, you do not need to write the template from scratch. `cliscrape` can statistically infer the boilerplate template for you.

1. Gather 2-3 sample outputs of the command.
2. Run the induction engine:
   ```bash
   cliscrape infer sample1.txt sample2.txt > new_template.yaml
   ```
3. The engine will identify the static anchors and propose variable `(?P<data>.*)` capture groups for you to refine.

## The FSM-Oracle: Bijective Verification

A template is considered **"Bijectively Stable"** (Perfect) if it can parse raw text into JSON, and then regenerate that exact same text from the JSON without any data loss.

When authoring a template, you should aim for Bijective Stability.

1. Create a fixture file: `tests/fixtures/my_vendor/my_command.txt`
2. Run the Oracle Verification loop:
   ```bash
   cliscrape parse my_command.txt --template my_template.yaml --verify
   ```

If the verification succeeds, your template is mathematically proven to capture 100% of the relevant operational state. If it fails, `cliscrape` will emit a `verify_failed` tracing event, indicating that your regex patterns dropped syntactic information during the parse.

*(Note: Legacy TextFSM templates with complex OR `|` branching outside of capture groups are inherently non-injective and cannot be perfectly round-tripped).*
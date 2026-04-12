# Template Authoring Guide

This guide covers how to write, test, and round-trip check modern YAML/TOML templates for `cliscrape`.

## The Modern YAML Format

`cliscrape` supports legacy TextFSM templates and introduces a schema-driven YAML/TOML format. This format keeps fields explicit and enables features like native type coercion, semantic diffing, constraints, and best-effort generation.

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

The `fields` block defines the operational schema contract. By adding specific metadata to your fields, you enable `cliscrape`'s analytical features.

* `type: int | string` - Enables native JSON type coercion (e.g., `"1500" -> 1500`).
* `identity: true` - Marks this field as a unique primary key for the record (e.g., an interface name). Used by the `diff` engine to track state changes.
* `ignore: true` - Marks this field as syntactic noise (e.g., uptime, timestamps). The `diff` engine will ignore changes to this field.
* `common_schema: 'key_name'` - Maps this vendor-specific variable to a Universal Ledger key (e.g., `vendor_interface -> interface`).
* `filldown: true` - Retains the captured value across multiple record emissions until overwritten.
* `list: true` - Accumulates multiple matches into a JSON array instead of overwriting.

## Grammar Induction (Auto-Generation)

If you are faced with a new, undocumented CLI output, `cliscrape` can infer a starting template for you.

1. Gather 2-3 sample outputs of the command.
2. Run the induction engine:
   ```bash
   cliscrape infer sample1.txt sample2.txt > new_template.yaml
   ```
3. The current engine identifies lines shared across samples as static anchors and emits broad `(?P<data>.*)` capture groups for varying lines. Treat the output as a scaffold to refine.

## The FSM-Oracle: Round-Trip Verification

A template is considered round-trip stable when it can parse raw text into JSON, generate synthetic text from the JSON, and parse that synthetic text back into the same JSON.

When authoring a template, you should aim for round-trip stability where the command format makes that realistic.

1. Create a fixture file: `tests/fixtures/my_vendor/my_command.txt`
2. Run the Oracle Verification loop:
   ```bash
   cliscrape parse my_command.txt --template my_template.yaml --verify
   ```

If the verification succeeds, the template preserved the parsed semantic data for that fixture. If it fails, `cliscrape` emits a `verify_failed` tracing event and warns that the generated text did not parse back to the same JSON data.

*(Note: Legacy TextFSM templates with complex OR `|` branching outside of capture groups are inherently non-injective and cannot be perfectly round-tripped).*

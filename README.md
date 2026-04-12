# cliscrape

`cliscrape` is a high-performance parser for semi-structured network CLI output. It supports legacy TextFSM templates, a modern YAML/TOML template format, embedded template discovery, and an interactive TUI for parser development.

## What It Does

- Parse raw device output into structured JSON
- Run with embedded templates or local template files
- Author and debug templates with a Live Lab TUI
- Support policy-aware parsing with field constraints and `--strict-policy`
- Power newer v2.0 workflows such as semantic diffing, opt-in schema-key mapping, and FSM round-trip checks

## Install

Build from source:

```bash
cargo install --path .
```

Or run without installing:

```bash
cargo run -- --help
```

## Quickstart

Parse a local template against a file:

```bash
cliscrape parse tests/fixtures/inputs/hostname_file.txt --template tests/fixtures/templates/simple_hostname.toml --format json
```

List embedded templates:

```bash
cliscrape list-templates --format json
```

Inspect one template:

```bash
cliscrape show-template cisco_ios_show_version.yaml
```

Use strict policy mode:

```bash
cliscrape parse tests/fixtures/inputs/constraints/min_fail.txt --template tests/fixtures/templates/constraints.yaml --strict-policy
```

## Guides

- User guide: `docs/guides/USER_GUIDE.md`
- Authoring guide: `docs/guides/AUTHORING.md`

## Current Roadmap

- v1.0 MVP: shipped
- v1.5 Production Hardening: shipped, with some planning artifact cleanup still pending
- v2.0 The Network Compiler: active milestone
- v3.0 Isomorphic Ecosystem: planned

Canonical planning files live under `.planning/`:

- `.planning/ROADMAP.md`
- `.planning/MILESTONES.md`
- `.planning/STATE.md`

## Core Commands

```bash
cliscrape parse <input> --template <template>
cliscrape debug -t <template> -i <input>
cliscrape list-templates
cliscrape list-templates --filter 'cisco*interfaces*'
cliscrape show-template <name.yaml>
cliscrape convert --input old.textfsm --output new.yaml --defaults
cliscrape diff before.txt after.txt --template <template>
cliscrape generate --template <template> --input records.json
cliscrape infer sample1.txt sample2.txt
```

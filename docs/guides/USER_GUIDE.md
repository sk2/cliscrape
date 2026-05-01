# cliscrape User Guide

`cliscrape` is a high-performance CLI scraping and parsing tool for network devices. It converts unstructured vendor-specific CLI output into typed JSON records, enabling advanced operations like semantic differential tracking and generative testing.

## Table of Contents
1. [Core Philosophy](#core-philosophy)
2. [Basic Parsing](#basic-parsing)
3. [Template Discovery](#template-discovery)
4. [The Interactive TUI (Live Lab)](#the-interactive-tui-live-lab)
5. [Advanced Operations (The Network Compiler)](#advanced-operations-the-network-compiler)
    - [Semantic Diffs](#semantic-diffs)
    - [Universal Ledger (Common Schema)](#universal-ledger-common-schema)
    - [Isomorphic Generation](#isomorphic-generation)

---

## Core Philosophy

Network changes are not changes in text; they are **state transitions** within a distributed system. 

Legacy tools like Python's TextFSM extract unstructured string arrays. `cliscrape` uses a modern Finite State Machine (FSM) engine written in Rust to project raw device output into structured JSON records. This enables downstream systems to query the network using structured logic rather than brittle regex parsing.

## Basic Parsing

The fundamental operation is transforming raw text into JSON. You do not need to download templates; `cliscrape` ships with an embedded XDG-compliant library.

```bash
# Parse a Cisco IOS output using an embedded template
cliscrape parse output.txt --template cisco_ios_show_version
```

### Execution Modes
Network data is notoriously inconsistent. `cliscrape` provides several modes for handling firmware quirks:

* **Graceful Degradation (Default):** Will parse as much as possible and emit structured `tracing` warnings for missing fields.
* **Fail-Fast (`--strict`):** Aborts execution immediately upon the first pattern error.
* **Thresholds (`--threshold 80.0`):** Aborts if the parsed output captures less than a specified percentage of expected fields (acting as a firmware formatting drift canary).

## Template Discovery

`cliscrape` includes tools for browsing the embedded template set and inspecting template metadata.

```bash
# List all available embedded templates
cliscrape list-templates

# Search for specific vendors/commands
cliscrape list-templates --filter 'cisco*interfaces*'

# View the metadata, fields, and source of a template
cliscrape show-template cisco_ios_show_version.yaml
```

*Note: You can override any embedded template by placing a file with the identical name in `~/.local/share/cliscrape/templates/`.*

## The Interactive TUI (Live Lab)

Writing or fixing parsers using trial-and-error CLI commands is incredibly inefficient. `cliscrape` includes a fully integrated Terminal User Interface (TUI) called the **Live Lab**.

```bash
# Launch the interactive debugger
cliscrape debug -t my_template.yaml -i raw_output.txt
```

**Features:**
1. **Template Browser:** Press `t` to browse templates and load one into the lab.
2. **Evaluation Loop:** The engine re-parses when watched template/input files change, and after saving edits in the template editor.
3. **State Tracing:** Visually step through the FSM's match/action loop line-by-line to find exactly where a regex failed.

---

## Advanced Operations (The Network Compiler)

`cliscrape` v2.0 introduces the concept of the "Network Compiler", treating the CLI output as a mathematical language that can be parsed, diffed, and generated.

### Semantic Diffs

Traditional `diff` tools flag syntactic noise (e.g., changing timestamps, PIDs, or interface uptime counters). `cliscrape` performs **Operational Diffs** by isolating the delta between two parsed JSON states ($S_1 \rightarrow S_2$).

```bash
cliscrape diff before.txt after.txt --template cisco_ios_show_interfaces
```

This ignores fields marked as `ignore: true` in the template and uses fields marked as `identity: true` to match records across the two inputs.

### Universal Ledger (Common Schema)

Different vendors use different names for identical concepts. The `--common` flag renames fields that declare `common_schema` in a modern template, projecting the record onto a vendor-neutral shape.

```bash
cliscrape parse output.txt --template cisco_ios_show_version --common
```

The canonical shapes are specified under [`common_schemas/`](../../common_schemas/) and documented in [COMMON_SCHEMAS.md](COMMON_SCHEMAS.md). Five schemas ship today: `version`, `interface`, `bgp_neighbor`, `lldp_neighbor`, `route`.

Wiring the embedded templates to declare these mappings is tracked as bead `cliscrape-1s8.2`; CI enforcement of schema compliance is `cliscrape-1s8.3`.

### Isomorphic Generation (FSM-Oracle)

For templates that are written with round-trip generation in mind, `cliscrape` can run a best-effort reverse pass. It consumes structured JSON records and synthesizes CLI-like text from template patterns.

```bash
# Generate synthetic device output from JSON
cliscrape generate --template cisco_ios_show_interfaces --input mock_state.json > synthetic.txt
```

**Verification (`--verify`):** You can append `--verify` to any `parse` command. The engine parses text to JSON, generates synthetic text, and re-parses it; mismatches are reported as verification warnings/log events rather than a hard failure.

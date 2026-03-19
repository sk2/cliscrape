# cliscrape User Guide

`cliscrape` is a high-performance CLI scraping and parsing tool for network devices. It converts unstructured vendor-specific CLI output into strictly typed JSON manifolds, enabling advanced operations like semantic differential tracking and generative testing.

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

Legacy tools like Python's TextFSM extract unstructured string arrays. `cliscrape` uses a modern Finite State Machine (FSM) engine written in Rust to project raw device output into a **strongly typed JSON manifold**. This enables downstream systems (like Autonetkit and NetAssure) to query the network using structured logic rather than brittle regex parsing.

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

You do not need to manually search for templates. `cliscrape` includes discovery tools to find the correct parser for your device.

```bash
# List all available embedded templates
cliscrape list-templates

# Search for specific vendors/commands
cliscrape list-templates "cisco*interfaces*"

# View the metadata, fields, and source of a template
cliscrape show-template cisco_ios_show_version
```

*Note: You can override any embedded template by placing a file with the identical name in `~/.local/share/cliscrape/templates/`.*

## The Interactive TUI (Live Lab)

Writing or fixing parsers using trial-and-error CLI commands is incredibly inefficient. `cliscrape` includes a fully integrated Terminal User Interface (TUI) called the **Live Lab**.

```bash
# Launch the interactive debugger
cliscrape debug -t my_template.yaml -i raw_output.txt
```

**Features:**
1. **Template Browser:** Press `L` to browse the embedded template library and instantly load it into the lab.
2. **Real-time Evaluation:** The engine hot-reloads the template and re-evaluates the FSM continuously as you type.
3. **State Tracing:** Visually step through the FSM's match/action loop line-by-line to find exactly where a regex failed.

---

## Advanced Operations (The Network Compiler)

`cliscrape` v2.0 introduces the concept of the "Network Compiler", treating the CLI output as a mathematical language that can be parsed, diffed, and generated.

### Semantic Diffs

Traditional `diff` tools flag syntactic noise (e.g., changing timestamps, PIDs, or interface uptime counters). `cliscrape` performs **Operational Diffs** by isolating the delta between two parsed JSON states ($S_1 \rightarrow S_2$).

```bash
cliscrape diff before.txt after.txt --template cisco_ios_show_interfaces
```

This will ignore all fields marked as `ignore: true` in the template (like `uptime`) and only output genuine state transitions (e.g., `status: "up" -> "down"`).

### Universal Ledger (Common Schema)

Different vendors use different names for identical concepts (e.g., `vendor_interface` vs `ge-0/0/0`). By using the `--common` flag, `cliscrape` will map vendor-specific variables to a universal operational ledger.

```bash
cliscrape parse output.txt --template cisco_ios_show_version --common
```

Downstream tools can now query `"interface": "Gi0/1"` without caring whether the underlying hardware is Cisco, Juniper, or Arista.

### Isomorphic Generation (FSM-Oracle)

Because the parsing templates enforce rigorous schema contracts, `cliscrape` can be run in reverse ($FSM^{-1}$). It consumes a structured JSON object and synthesizes perfectly formatted, pixel-accurate raw CLI text.

```bash
# Generate synthetic device output from JSON
cliscrape generate --template cisco_ios_show_interfaces --input mock_state.json > synthetic.txt
```

**Verification (`--verify`):** You can append `--verify` to any `parse` command. The engine will parse the text to JSON, generate synthetic text from the JSON, and re-parse the synthetic text to guarantee 100% **Bijective Stability**.

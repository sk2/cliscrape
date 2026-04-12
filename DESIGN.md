# Technical Design: cliscrape

This document outlines the internal architecture and design decisions for `cliscrape`.

## 1. Core Component: The FSM Engine

The heart of `cliscrape` is a Finite State Machine (FSM) that processes text line-by-line.

### State Representation
Each state consists of a list of **Rules**. A Rule is triggered if its regular expression matches the current line.

### Execution Loop
1.  **Read Line:** Get the next line from the input.
2.  **Evaluate Rules:** Iterate through the rules of the *current state*.
3.  **Match:** If a rule's regex matches:
    -   **Capture:** Extract named groups into the current record's buffer.
    -   **Actions:** Execute actions like `Record` (save current buffer to results), `Clear` (wipe buffer), `Continue` (don't consume line, check next rule), or `Next` (consume line, stop checking rules for this line).
    -   **Transition:** Move to the `next_state` if specified.
4.  **Loop:** Repeat until EOF.

## 2. Template Abstraction Layer

To support both TextFSM and newer formats (YAML/TOML), `cliscrape` uses an internal intermediate representation (IR).

- **`Template` Object:** Contains `Values` and `States`.
- **`Value` Definition:** 
  - `regex`: The pattern.
  - `type`: Integer or string hint for JSON value conversion.
  - `filldown`: Carry value to subsequent records.
  - `required`: Record is only valid if this value is present.
  - `list`: Accumulate multiple matches into a list.
  - `identity`: Use the field as a semantic diff identity key.
  - `ignore`: Exclude the field from semantic diff comparisons.
  - `common_schema`: Optional output key used by `--common`.
  - `constraints`: Optional `min`, `max`, `choices`, and `regex` policy checks.

## 3. TUI Debugger Layout

The TUI is built using `ratatui` and aims for a high-density, informative display.

```
+-----------------------------------+--------------------------+
| Input Stream (Line: 42)           | FSM State: [Interface]   |
+-----------------------------------+--------------------------+
| 41: ...                           | Current Values:          |
| 42: GigabitEthernet1 is up, ...   | - interface: Gig1        |
| 43:   Internet address is ...     | - status: up             |
|                                   | - protocol: <empty>      |
+-----------------------------------+--------------------------+
| Match Trace                       | Variables Evolution      |
+-----------------------------------+--------------------------+
| Line 42 matched Rule #1 in Start  | [interface] -> "Gig1"    |
| Transition: Start -> Interface    |                          |
| Action: [None]                    |                          |
+-----------------------------------+--------------------------+
| Help: [n] Next Line [s] Step Rule [q] Quit                   |
+-----------------------------------+--------------------------+
```

## 4. Performance Considerations

- **Regex Compilation:** Each rule regex is compiled once when the template IR is lowered into the runtime `Template`.
- **Memory Management:** Captures are stored in an owned `RecordBuffer` and emitted as `serde_json::Value` records.
- **Timeout Guard:** Parse calls can set `timeout_ms`; the engine checks elapsed time while processing lines.

## 5. Modern Configuration (YAML)

The goal is to make templates more readable than the positional logic of TextFSM.

```yaml
version: 1

metadata:
  description: "Parse show version"
  compatibility: "Vendor OS 1.0"
  version: "1.0.0"

fields:
  version:
    type: string
    pattern: '\d+\.\S+'
  uptime:
    type: string
    pattern: '.+'
    ignore: true

patterns:
  - regex: '^Cisco IOS Software, .+ Version ${version},'
  - regex: '^.+ uptime is ${uptime}'
    record: true
```

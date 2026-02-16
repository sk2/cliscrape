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
  - `type`: (Integer, String, IP, etc. - extension beyond TextFSM).
  - `filldown`: Carry value to subsequent records.
  - `required`: Record is only valid if this value is present.
  - `list`: Accumulate multiple matches into a list.

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

- **Regex Compilation:** All regexes are pre-compiled into a `RegexSet` where possible for fast dispatching.
- **Memory Management:** Use a pre-allocated buffer for records to avoid frequent heap allocations during large-scale parsing.
- **Zero-Copy:** Where possible, values will be `Cow<'a, str>` referencing the original input string.

## 5. Modern Configuration (YAML)

The goal is to make templates more readable than the positional logic of TextFSM.

```yaml
# Proposed modern format
meta:
  name: cisco_show_version
  author: Simon Knight

values:
  version: (?P<version>\d+\.\S+)
  uptime: (?P<uptime>.+)

states:
  Start:
    - match: "^Cisco IOS Software, .+ Version ${version},"
      next_state: Main
  Main:
    - match: "^.+ uptime is ${uptime}"
      actions: [Record]
```

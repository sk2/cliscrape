# cliscrape

`cliscrape` is a high-performance CLI scraping and parsing tool for network devices, written in Rust. It provides a modern, ergonomic, and blazingly fast alternative to legacy tools like `TextFSM`, while maintaining first-class compatibility with existing templates.

## The Vision

The networking industry relies heavily on CLI-based management. Parsing this unstructured data is often the bottleneck in automation pipelines. `cliscrape` aims to solve this by bringing Rust's safety and performance to the parsing layer, coupled with a focus on developer experience through interactive debugging.

### Core Pillars

1.  **Performance:** Zero-cost abstractions and Rust's `regex` engine ensure that even massive `show tech-support` outputs are parsed in milliseconds.
2.  **Compatibility:** Full support for the `TextFSM` grammar. Use your existing library of hundreds of community templates (e.g., from `ntc-templates`) without modification.
3.  **Ergonomics:** A new, structured template format (YAML/TOML) that reduces the "regex soup" often found in TextFSM files.
4.  **Observability:** A built-in TUI for real-time debugging of templates. See exactly which line matched which rule and why.

---

## Architectural Overview

### 1. The Engine (Rust-FSM)
A state-machine based parser that replicates the behavior of TextFSM. It handles:
- **Value Definitions:** Typed variables with regex validation.
- **State Transitions:** `Start`, `End`, and custom user-defined states.
- **Actions:** `Next`, `Continue`, `Record`, and `Clear`.

### 2. TextFSM Compatibility Layer
A parser for `.textfsm` files that translates them into the internal `cliscrape` FSM representation. This allows seamless migration for existing workflows.

### 3. Modern Template Format (Proposal)
While TextFSM is powerful, its DSL can be hard to read. `cliscrape` proposes a structured YAML/TOML format:

```yaml
# cisco_ios_show_ip_int_brief.yaml
values:
  interface: \S+
  ip_address: \d+\.\d+\.\d+\.\d+|unassigned
  status: up|down|administratively down
  protocol: up|down

states:
  Start:
    - match: ^${interface}\s+${ip_address}\s+\S+\s+\S+\s+${status}\s+${protocol}
      action: Record
```

### 4. TUI Debugger (The "Dry Run" Mode)
The TUI provides a visual environment for template development:
- **Input Pane:** Paste or load raw CLI output.
- **State Pane:** Watch the FSM transition between states as it consumes the input.
- **Variables Pane:** See the current value of all defined variables in real-time.
- **Diff View:** Highlight which part of the line matched a specific regex.

---

## Performance Comparison (Anticipated)

| Feature | TextFSM (Python) | cliscrape (Rust) |
| :--- | :--- | :--- |
| **Parsing Speed** | Baseline | ~10-50x Faster |
| **Memory Usage** | Moderate | Low / Deterministic |
| **Startup Time** | Slow (Interpreted) | Instant (Compiled) |
| **Concurrency** | GIL-bound | Fully Parallelizable |

---

## Roadmap

- [x] Initial Vision and Project Scaffolding
- [ ] **Phase 1: Lexer/Parser for TextFSM Files**
- [ ] **Phase 2: Core FSM Execution Engine**
- [ ] **Phase 3: CLI Implementation (Basic Parse)**
- [ ] **Phase 4: TUI Debugger and Visualizer**
- [ ] **Phase 5: Modern YAML/TOML Template Support**

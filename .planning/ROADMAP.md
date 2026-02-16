# Roadmap - cliscrape

`cliscrape` is a high-performance CLI scraping and parsing tool for network devices. This roadmap outlines the path from core engine development to a full-featured TUI debugger and modern template support.

## Phase 1: Core Parsing Engine
**Goal:** Build a high-throughput, deterministic FSM engine that supports modular regex patterns.

- **Requirements:** CORE-01, CORE-03
- **Dependencies:** None
- **Success Criteria:**
    1. System processes >100,000 lines per second in benchmarks.
    2. Parser correctly handles `{{ipv4}}` and `{{mac_address}}` macros in state definitions.
    3. Unit tests verify deterministic state transitions based on regex matches.

## Phase 2: Legacy Compatibility & CLI
**Goal:** Enable parsing of existing TextFSM templates via a standard Unix-style CLI.

- **Requirements:** CORE-02, FORM-01, CLI-01
- **Dependencies:** Phase 1
- **Success Criteria:**
    1. User can run `cliscrape parse --template example.textfsm output.txt` and receive JSON output.
    2. Parser correctly handles `Filldown` and `Required` values from standard `ntc-templates`.
    3. Piped input (e.g., `cat output.txt | cliscrape parse`) produces correct structured data.

## Phase 3: Modern Ergonomic Templates
**Goal:** Support YAML/TOML template formats with automatic type conversion and basic prompt handling.

- **Requirements:** FORM-02, FORM-03, CLI-02
- **Dependencies:** Phase 2
- **Success Criteria:**
    1. User can define a template using YAML blocks that match CLI states.
    2. Numeric captures (e.g., interface speed) are automatically converted to integers in the JSON output.
    3. CLI correctly identifies and handles Cisco IOS style prompts in raw input streams.

## Phase 4: TUI Debugger Foundation (Live Lab)
**Goal:** Provide a real-time visual environment for template development and regex matching.

- **Requirements:** TUI-01
- **Dependencies:** Phase 2
- **Success Criteria:**
    1. User sees a split-screen TUI with raw text on one side and matches on the other.
    2. Changing a regex in the template file (while TUI is running) instantly updates the matching highlights.
    3. User can navigate through the CLI output and see which lines match which template rules.

## Phase 5: TUI Advanced Debugging (State Tracer)
**Goal:** Enable deep inspection of FSM state transitions and variable state during the parsing process.

- **Requirements:** TUI-02, TUI-03
- **Dependencies:** Phase 4
- **Success Criteria:**
    1. TUI shows the current FSM state (e.g., `START` -> `HEADER` -> `BODY`) for the selected line.
    2. User can step forward/backward through the parsing process to see when variables change.
    3. A trace buffer allows the user to review all transitions that led to a specific `Record` action.

## Progress Tracking

| Phase | Description | Status | Progress |
|-------|-------------|--------|----------|
| 1 | Core Parsing Engine | Pending | 0% |
| 2 | Legacy Compatibility & CLI | Pending | 0% |
| 3 | Modern Ergonomic Templates | Pending | 0% |
| 4 | TUI Debugger Foundation | Pending | 0% |
| 5 | TUI Advanced Debugging | Pending | 0% |

# Roadmap - cliscrape

`cliscrape` is a high-performance CLI scraping and parsing tool for network devices. This roadmap outlines the path from core engine development to a full-featured TUI debugger and modern template support.

## Phase 1: Core Parsing Engine
**Goal:** Build a high-throughput, deterministic FSM engine that supports modular regex patterns. [COMPLETED]

- **Requirements:** CORE-01, CORE-03
- **Dependencies:** None
- **Success Criteria:**
    1. System processes >100,000 lines per second in benchmarks.
    2. Parser correctly handles `{{ipv4}}` and `{{mac_address}}` macros in state definitions.
    3. Unit tests verify deterministic state transitions based on regex matches.

**Plans:** 5 plans
- [x] 01-01-PLAN.md — Foundation & Regex Macros
- [x] 01-02-PLAN.md — FSM Execution Engine
- [x] 01-03-PLAN.md — Record Management & Validation
- [x] 01-04-PLAN.md — Gap closure: deterministic state transitions + End termination tests
- [x] 01-05-PLAN.md — Gap closure: mac_address unit test + macro-through-compilation integration test

### Notes (Research)

- `{{mac_address}}` should match both colon-separated (`aa:bb:cc:dd:ee:ff`) and dotted (`aabb.ccdd.eeff`) formats, based on patterns used across ntc-templates (Cisco/Arista/Juniper).

## Phase 2: Legacy Compatibility & CLI [COMPLETED]
**Goal:** Enable parsing of existing TextFSM templates via a standard Unix-style CLI.

- **Requirements:** CORE-02, FORM-01, CLI-01
- **Dependencies:** Phase 1
- **Success Criteria:**
    1. User can run `cliscrape parse --template example.textfsm output.txt` and receive JSON output.
    2. Parser correctly handles `Filldown` and `Required` values from standard `ntc-templates`.
    3. Piped input (e.g., `cat output.txt | cliscrape parse`) produces correct structured data.

**Plans:** 9 plans (01-04 superseded, 05-09 executed)
- [x] 02-01-PLAN.md — TextFSM DSL Parser (Pest) [superseded]
- [x] 02-02-PLAN.md — CLI & Input Stream Handling [superseded]
- [x] 02-03-PLAN.md — Serialization & EOF Nuances [superseded]
- [x] 02-04-PLAN.md — [superseded]
- [x] 02-05-PLAN.md — Phase-2 parse CLI contract + flag wiring
- [x] 02-06-PLAN.md — TextFSM action support (ClearAll, Error) + warnings API
- [x] 02-07-PLAN.md — TextFSM EOF semantics + comment support + strict validation
- [x] 02-08-PLAN.md — Verification gap closure + unknown flag/action warnings
- [x] 02-09-PLAN.md — CLI parse e2e tests + fixtures

## Phase 3: Modern Ergonomic Templates [COMPLETED]
**Goal:** Support YAML/TOML template formats with automatic type conversion and basic prompt handling.

- **Requirements:** FORM-02, FORM-03, CLI-02
- **Dependencies:** Phase 2
- **Success Criteria:**
    1. User can define a template using YAML blocks that match CLI states.
    2. Numeric captures (e.g., interface speed) are automatically converted to integers in the JSON output.
    3. CLI correctly identifies and handles Cisco IOS style prompts in raw input streams.

**Plans:** 6 plans
- [x] 03-01-PLAN.md — Typed record emission (explicit + heuristic conversion)
- [x] 03-02-PLAN.md — IOS prompt/echo handling + transcript segmentation
- [x] 03-03-PLAN.md — Modern YAML/TOML schema + TemplateIR lowering
- [x] 03-04-PLAN.md — CLI format override + starter templates + e2e tests
- [x] 03-05-PLAN.md — Interactive TextFSM -> modern conversion tool
- [x] 03-06-PLAN.md — Modern wiring tests + recursive macros gap closure

## Phase 4: TUI Debugger Foundation (Live Lab) [COMPLETED]
**Goal:** Provide a real-time visual environment for template development and regex matching.

- **Requirements:** TUI-01
- **Dependencies:** Phase 2
- **Success Criteria:**
    1. User sees a split-screen TUI with raw text on one side and matches on the other.
    2. Changing a regex in the template file (while TUI is running) instantly updates the matching highlights.
    3. User can navigate through the CLI output and see which lines match which template rules.

**Plans:** 5 plans
- [ ] 04-01-PLAN.md — Engine debug parse report (per-line trace + spans)
- [ ] 04-02-PLAN.md — TUI scaffolding + `cliscrape debug` wiring
- [ ] 04-03-PLAN.md — Live reload (watch + worker) + error retention
- [ ] 04-04-PLAN.md — Match visualization, details pane, matches vs records toggle
- [ ] 04-05-PLAN.md — In-TUI picker + inline template editor + verification

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
| 1 | Core Parsing Engine | Complete | 100% |
| 2 | Legacy Compatibility & CLI | Complete | 100% |
| 3 | Modern Ergonomic Templates | Complete | 100% |
| 4 | TUI Debugger Foundation | Complete | 100% |
| 5 | TUI Advanced Debugging | Pending | 0% |

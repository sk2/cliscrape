# Architecture Research: v0.1 Alpha Integration

**Domain:** CLI parsing tool (IR-based compiler architecture)
**Researched:** 2026-02-17
**Confidence:** HIGH

## Executive Summary

The v0.1 Alpha architecture implements an **IR-based compiler design** where multiple frontend parsers (TextFSM, YAML, TOML) compile to a unified intermediate representation that feeds a generic FSM execution engine. This decoupling enables TextFSM compatibility while supporting modern formats, and critically allows the TUI debugger to observe engine internals through event tracing.

The architecture follows a **modular pipeline**: `Frontend → IR → Engine → Output`, with each component having clear responsibilities and well-defined interfaces. The TUI integrates via an event-driven observer pattern, receiving trace events from the engine without blocking execution.

This design draws from established compiler architecture patterns where IRs decouple language frontends from execution backends, enabling multi-language support without exponential complexity.

## High-Level Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        FRONTENDS (Parsers)                       │
├──────────────┬───────────────────┬───────────────┬──────────────┤
│  TextFSM     │   YAML Parser     │  TOML Parser  │   Future...  │
│  Parser      │   (Phase 4)       │  (Phase 4)    │              │
│  (Phase 2)   │                   │               │              │
└──────┬───────┴─────────┬─────────┴───────┬───────┴──────────────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         ↓
            ┌────────────────────────────┐
            │  INTERMEDIATE REPRESENTATION │
            │         (IR / AST)           │
            │       (Phase 1)              │
            └────────────┬─────────────────┘
                         ↓
            ┌────────────────────────────┐
            │      FSM EXECUTION ENGINE    │
            │      (State Machine)         │
            │       (Phase 1)              │
            └────┬──────────────────┬──────┘
                 │                  │
                 │ Events           │ Results
                 ↓                  ↓
        ┌────────────────┐   ┌──────────────┐
        │  TUI DEBUGGER  │   │   CLI OUTPUT │
        │   (Phase 3)    │   │   (JSON/CSV) │
        └────────────────┘   └──────────────┘
```

### Component Responsibilities

| Component | Responsibility | Key Interfaces |
|-----------|----------------|----------------|
| **TextFSM Frontend** | Parse `.textfsm` files into IR | `fn parse_textfsm(path) -> Result<Template>` |
| **YAML/TOML Frontend** | Parse modern config into IR | `fn parse_yaml(path) -> Result<Template>` |
| **IR (Template)** | Unified representation of FSM | `struct Template { values, states }` |
| **FSM Engine** | Execute IR against input text | `fn execute(&self, input) -> Results` |
| **Event System** | Emit trace events for TUI | `trait TraceListener` |
| **TUI Debugger** | Visualize FSM execution live | `struct DebuggerApp` |

## Phase 1: Core Engine & IR Design

### IR Specification (Internal Representation)

The IR is the **contract** between frontends and the engine. All frontends must compile to this exact structure.

```rust
/// The complete intermediate representation of a parsing template
pub struct Template {
    /// Named values that can be captured
    pub values: HashMap<String, ValueDef>,
    /// FSM states with their rules
    pub states: HashMap<String, State>,
    /// Initial state name (defaults to "Start")
    pub initial_state: String,
}

/// Definition of a capturable value
pub struct ValueDef {
    /// Unique identifier
    pub name: String,
    /// Compiled regex for extraction
    pub regex: Regex,
    /// Carry value forward to next records
    pub filldown: bool,
    /// Accumulate multiple matches into list
    pub is_list: bool,
    /// Record only valid if this value captured
    pub required: bool,
}

/// A state in the FSM containing ordered rules
pub struct State {
    pub name: String,
    pub rules: Vec<Rule>,
}

/// A single matching rule with actions and transitions
pub struct Rule {
    /// Pattern to match against current line
    pub regex: Regex,
    /// Actions to perform on match
    pub actions: Vec<Action>,
    /// Next state to transition to (None = stay in current)
    pub next_state: Option<String>,
    /// Captures mapping (TextFSM uses named groups)
    pub captures: Vec<String>,
}

/// Actions executed when a rule matches
#[derive(Debug, Clone)]
pub enum Action {
    /// Consume line, move to next input line
    Next,
    /// Don't consume line, try next rule
    Continue,
    /// Save current buffer to results
    Record,
    /// Clear current buffer
    Clear,
    /// Special: Clear all records (less common)
    Clearall,
}
```

**Design rationale:**
- `HashMap` for O(1) state/value lookups by name
- `Vec<Rule>` preserves rule evaluation order (critical for TextFSM)
- Pre-compiled `Regex` objects (compile once, execute many times)
- `Action` enum ensures type-safe action handling

### Engine Architecture

The engine is a **stateless executor** that processes input line-by-line using the FSM defined in the IR.

```rust
/// FSM execution engine
pub struct FsmEngine {
    template: Template,
    /// Optional event emitter for tracing
    trace_listener: Option<Arc<dyn TraceListener>>,
}

/// Execution state (ephemeral per parse)
struct ExecutionContext {
    /// Current FSM state
    current_state: String,
    /// Buffer for current record being built
    current_record: HashMap<String, RecordValue>,
    /// Completed records
    results: Vec<HashMap<String, String>>,
    /// Filldown values carried from previous records
    filldown_values: HashMap<String, String>,
    /// Current line number (for debugging)
    line_number: usize,
}

/// A value that can be a scalar or list
enum RecordValue {
    Scalar(String),
    List(Vec<String>),
}

impl FsmEngine {
    pub fn new(template: Template) -> Self {
        Self { template, trace_listener: None }
    }

    pub fn with_tracing(mut self, listener: Arc<dyn TraceListener>) -> Self {
        self.trace_listener = Some(listener);
        self
    }

    /// Execute the FSM against input text
    pub fn execute(&self, input: &str) -> Result<Vec<HashMap<String, String>>> {
        let mut ctx = ExecutionContext::new(&self.template);

        for line in input.lines() {
            ctx.line_number += 1;
            self.process_line(line, &mut ctx)?;
        }

        // Handle EOF (implicit Record if buffer has data)
        if !ctx.current_record.is_empty() {
            self.emit_trace(TraceEvent::ImplicitRecord);
            ctx.finalize_record();
        }

        Ok(ctx.results)
    }

    fn process_line(&self, line: &str, ctx: &mut ExecutionContext) -> Result<()> {
        let state = self.template.states.get(&ctx.current_state)
            .ok_or_else(|| ScraperError::InvalidState(ctx.current_state.clone()))?;

        for (rule_idx, rule) in state.rules.iter().enumerate() {
            if let Some(captures) = rule.regex.captures(line) {
                self.emit_trace(TraceEvent::RuleMatched {
                    state: &ctx.current_state,
                    rule_index: rule_idx,
                    line_number: ctx.line_number,
                    captures: captures.clone(),
                });

                // Extract captures into current record
                self.apply_captures(rule, captures, ctx)?;

                // Execute actions
                let mut should_break = false;
                for action in &rule.actions {
                    self.emit_trace(TraceEvent::ActionExecuted { action: action.clone() });
                    match action {
                        Action::Record => ctx.finalize_record(),
                        Action::Clear => ctx.current_record.clear(),
                        Action::Clearall => {
                            ctx.current_record.clear();
                            ctx.results.clear();
                        }
                        Action::Next => should_break = true,
                        Action::Continue => {} // Just continue to next rule
                    }
                }

                // Transition state if specified
                if let Some(next_state) = &rule.next_state {
                    self.emit_trace(TraceEvent::StateTransition {
                        from: &ctx.current_state,
                        to: next_state,
                    });
                    ctx.current_state = next_state.clone();
                }

                if should_break {
                    break; // Next action: stop processing rules for this line
                }
            }
        }

        Ok(())
    }

    fn emit_trace(&self, event: TraceEvent) {
        if let Some(listener) = &self.trace_listener {
            listener.on_trace(event);
        }
    }
}
```

**Design rationale:**
- Engine is **stateless**: all execution state in `ExecutionContext`
- Allows parallel parsing of multiple inputs (thread-safe with Arc<Template>)
- `TraceListener` decouples observation from execution
- Clear separation: engine logic vs tracing logic

### Event System for TUI Integration

```rust
/// Events emitted during FSM execution
#[derive(Clone, Debug)]
pub enum TraceEvent {
    /// FSM started execution
    Started { initial_state: String },

    /// Entered a new line
    LineProcessed { line_number: usize, content: String },

    /// Rule matched against current line
    RuleMatched {
        state: String,
        rule_index: usize,
        line_number: usize,
        captures: regex::Captures,
    },

    /// Action was executed
    ActionExecuted { action: Action },

    /// State transition occurred
    StateTransition { from: String, to: String },

    /// Record was finalized and added to results
    RecordCreated { record: HashMap<String, String> },

    /// FSM finished execution
    Completed { total_records: usize },
}

/// Observer trait for receiving trace events
pub trait TraceListener: Send + Sync {
    fn on_trace(&self, event: TraceEvent);
}
```

**Integration pattern:**
- TUI implements `TraceListener`
- Engine calls `listener.on_trace(event)` at key points
- Events sent via crossbeam channel to avoid blocking
- TUI processes events asynchronously in render loop

## Phase 2: TextFSM Frontend Integration

### TextFSM Parser Architecture

The TextFSM frontend is a **compiler frontend** that transforms `.textfsm` DSL into IR.

```rust
pub struct TextFsmFrontend;

impl TextFsmFrontend {
    /// Parse a .textfsm file into IR
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Template> {
        let content = std::fs::read_to_string(path)?;
        Self::parse_content(&content)
    }

    fn parse_content(content: &str) -> Result<Template> {
        // Two-pass parser
        let (values, states) = Self::split_sections(content)?;

        let value_defs = Self::parse_values_section(values)?;
        let state_defs = Self::parse_states_section(states, &value_defs)?;

        Ok(Template {
            values: value_defs,
            states: state_defs,
            initial_state: "Start".to_string(),
        })
    }

    fn parse_values_section(content: &str) -> Result<HashMap<String, ValueDef>> {
        // Parse "Value" declarations
        // Format: Value [options] name regex
        // Example: Value Filldown,Required interface (\S+)
    }

    fn parse_states_section(
        content: &str,
        values: &HashMap<String, ValueDef>,
    ) -> Result<HashMap<String, State>> {
        // Parse state definitions
        // Format:
        //   StateName
        //     ^regex -> Action NextState
    }
}
```

**TextFSM Parsing Challenges:**

1. **Named Capture Groups**: TextFSM uses `$variable` syntax, not `(?P<name>)`
   - Solution: Pre-process regex to convert `${variable}` → `(?P<variable>...)`

2. **Positional Value Matching**: Capture groups map to Values by order
   - Solution: Track value order during parsing, map captures by position

3. **Implicit Actions**: TextFSM has defaults (e.g., End state auto-Records)
   - Solution: Normalize to explicit Action::Record in IR

**Integration with textfsm-rust crate:**

```rust
use textfsm_rust::TextFsm;

impl TextFsmFrontend {
    /// Leverage textfsm-rust for parsing, translate to our IR
    pub fn parse_with_crate<P: AsRef<Path>>(path: P) -> Result<Template> {
        let textfsm = TextFsm::from_file(path)?;

        // Convert textfsm-rust's AST to our IR
        Self::convert_from_textfsm_crate(textfsm)
    }

    fn convert_from_textfsm_crate(fsm: TextFsm) -> Result<Template> {
        // Translation layer: textfsm-rust types → our IR
        // This allows us to use a battle-tested parser while maintaining our IR
    }
}
```

**Decision**: Use `textfsm-rust` crate for parsing, then translate to our IR. This gives us 99%+ compatibility immediately.

### Frontend → IR Data Flow

```
.textfsm file
    ↓ (parse)
TextFsm (textfsm-rust types)
    ↓ (convert)
Template (our IR)
    ↓ (pass to engine)
FsmEngine::execute()
```

## Phase 3: TUI Integration Architecture

### TUI Architecture Pattern: Component + Event-Driven

The TUI follows the [Component Architecture pattern](https://ratatui.rs/concepts/application-patterns/component-architecture/) recommended by Ratatui, with an event-driven core inspired by [The Elm Architecture](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/).

```
┌──────────────────────────────────────────────────┐
│                   TUI Application                 │
├─────────────┬─────────────────┬──────────────────┤
│  Input Pane │  State Pane     │  Trace Log Pane  │
│  (text)     │  (current FSM)  │  (events)        │
└─────────────┴─────────────────┴──────────────────┘
       ↑              ↑                  ↑
       └──────────────┴──────────────────┘
                      │
             ┌────────┴────────┐
             │  Event Bus      │
             │ (crossbeam)     │
             └────────┬────────┘
                      │
             ┌────────┴────────┐
             │  FSM Engine     │
             │  (worker thread)│
             └─────────────────┘
```

### Component Structure

```rust
/// Root TUI application
pub struct DebuggerApp {
    /// Current template being debugged
    template: Option<Template>,

    /// UI components
    input_viewer: InputViewer,
    state_viewer: StateViewer,
    trace_log: TraceLog,

    /// Event receiver (from engine worker thread)
    trace_rx: Receiver<TraceEvent>,

    /// Current execution state (reconstructed from events)
    execution_state: ExecutionSnapshot,
}

/// Component: Input text viewer with line highlighting
pub struct InputViewer {
    lines: Vec<String>,
    current_line: usize,
    scroll_offset: usize,
}

/// Component: FSM state visualization
pub struct StateViewer {
    current_state: String,
    current_record: HashMap<String, String>,
    filldown_values: HashMap<String, String>,
}

/// Component: Event trace log (scrollable)
pub struct TraceLog {
    events: VecDeque<TraceEvent>,
    max_events: usize,
    scroll_offset: usize,
}

/// Snapshot of execution state (for display)
pub struct ExecutionSnapshot {
    current_line: usize,
    current_state: String,
    record_buffer: HashMap<String, String>,
    last_match: Option<String>,
}
```

### Event Flow Architecture

```rust
/// TUI TraceListener implementation
pub struct TuiTraceListener {
    tx: Sender<TraceEvent>,
}

impl TraceListener for TuiTraceListener {
    fn on_trace(&self, event: TraceEvent) {
        // Non-blocking send to TUI event loop
        let _ = self.tx.try_send(event);
    }
}

impl DebuggerApp {
    pub fn run(template: Template, input: String) -> Result<()> {
        let (trace_tx, trace_rx) = crossbeam_channel::bounded(1000);
        let listener = Arc::new(TuiTraceListener { tx: trace_tx });

        // Spawn engine in worker thread
        let engine_handle = std::thread::spawn(move || {
            let engine = FsmEngine::new(template).with_tracing(listener);
            engine.execute(&input)
        });

        // Run TUI event loop
        let mut app = Self::new(trace_rx);
        app.event_loop()?;

        // Wait for engine completion
        let _results = engine_handle.join().unwrap()?;

        Ok(())
    }

    fn event_loop(&mut self) -> Result<()> {
        let mut terminal = setup_terminal()?;

        loop {
            // Process all pending trace events
            while let Ok(event) = self.trace_rx.try_recv() {
                self.handle_trace_event(event);
            }

            // Render UI
            terminal.draw(|f| self.render(f))?;

            // Handle user input (keyboard events)
            if let Event::Key(key) = crossterm::event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('n') => self.next_step(),
                    KeyCode::Up => self.scroll_up(),
                    KeyCode::Down => self.scroll_down(),
                    _ => {}
                }
            }
        }

        Ok(())
    }

    fn handle_trace_event(&mut self, event: TraceEvent) {
        // Update execution snapshot
        match event {
            TraceEvent::LineProcessed { line_number, .. } => {
                self.execution_state.current_line = line_number;
            }
            TraceEvent::StateTransition { to, .. } => {
                self.execution_state.current_state = to;
            }
            TraceEvent::RuleMatched { captures, .. } => {
                // Update record buffer with captures
            }
            _ => {}
        }

        // Add to trace log
        self.trace_log.push(event);
    }

    fn render(&self, frame: &mut Frame) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Input + State
                Constraint::Percentage(50), // Trace log
            ])
            .split(frame.area());

        // Render components
        self.input_viewer.render(frame, layout[0]);
        self.state_viewer.render(frame, layout[0]);
        self.trace_log.render(frame, layout[1]);
    }
}
```

**Key Design Decisions:**

1. **Worker Thread Pattern**: Engine runs in background thread
   - Prevents UI blocking during execution
   - Uses bounded channel (backpressure if TUI can't keep up)

2. **Event Replay**: TUI reconstructs state from events
   - Enables "step backward" functionality later
   - All state is derivable from event stream

3. **Component Isolation**: Each pane is self-contained
   - Easier testing (render each component independently)
   - Clear responsibility boundaries

### TUI Layout (Concrete)

```
╭─────────────────────────────────────────────╮
│ Input Stream (Line: 42 / 150)               │
├─────────────────────────────────────────────┤
│  40: interface Vlan1                        │
│  41:  ip address 10.0.0.1 255.255.255.0     │
│→ 42: interface GigabitEthernet1/0/1         │ ← Current line
│  43:  description Uplink to Core            │
│  44:  switchport mode trunk                 │
╰─────────────────────────────────────────────╯

╭──────────────────────────────────────────────╮
│ Current State: Interface                     │
├──────────────────────────────────────────────┤
│ Record Buffer:                               │
│  interface: "GigabitEthernet1/0/1"           │
│  description: <empty>                        │
│  mode: <empty>                               │
│                                              │
│ Filldown Values:                             │
│  vlan: "1"                                   │
╰──────────────────────────────────────────────╯

╭─────────────────────────────────────────────╮
│ Trace Log                                    │
├─────────────────────────────────────────────┤
│ [42] RuleMatched: Interface.rule[0]         │
│      Regex: ^interface (\S+)                 │
│      Captured: interface="Gig1/0/1"          │
│ [42] StateTransition: Start → Interface     │
│ [43] RuleMatched: Interface.rule[1]         │
│      Captured: description="Uplink to Core"  │
╰─────────────────────────────────────────────╯

[n] Next  [s] Step  [r] Run  [q] Quit
```

## Phase 4: Modern Frontend Integration

### YAML/TOML Frontend Architecture

Modern frontends compile the same IR, but from structured config instead of DSL.

```rust
pub struct YamlFrontend;

impl YamlFrontend {
    pub fn parse<P: AsRef<Path>>(path: P) -> Result<Template> {
        let content = std::fs::read_to_string(path)?;
        let config: YamlConfig = serde_yml::from_str(&content)?;
        Self::compile_to_ir(config)
    }

    fn compile_to_ir(config: YamlConfig) -> Result<Template> {
        // Convert structured YAML → IR
        // This is much simpler than TextFSM parsing because
        // we control the schema
    }
}

/// YAML schema for modern templates
#[derive(Deserialize)]
struct YamlConfig {
    meta: TemplateMeta,
    values: HashMap<String, ValueConfig>,
    states: HashMap<String, StateConfig>,
}

#[derive(Deserialize)]
struct ValueConfig {
    /// Regex pattern (can use ${other_value} references)
    pattern: String,
    #[serde(default)]
    filldown: bool,
    #[serde(default)]
    required: bool,
    #[serde(default)]
    is_list: bool,
}

#[derive(Deserialize)]
struct StateConfig {
    rules: Vec<RuleConfig>,
}

#[derive(Deserialize)]
struct RuleConfig {
    /// Match pattern (can use ${value} interpolation)
    pattern: String,
    /// Actions to execute
    #[serde(default)]
    actions: Vec<String>, // ["Record", "Clear"]
    /// Next state transition
    next_state: Option<String>,
}
```

**Example YAML Template:**

```yaml
meta:
  name: cisco_ios_show_interface
  author: cliscrape
  version: 1.0

values:
  interface:
    pattern: "\\S+"
    required: true
  description:
    pattern: ".+"
    filldown: false

states:
  Start:
    rules:
      - pattern: "^interface ${interface}"
        next_state: Interface
        actions: []

  Interface:
    rules:
      - pattern: "^ description ${description}"
        actions: []
      - pattern: "^interface ${interface}"
        actions: ["Record", "Clear"]
        next_state: Interface
```

**Frontend → IR Compilation:**

1. Parse YAML/TOML into typed structs (serde)
2. Compile `pattern` strings into `Regex` objects
3. Resolve `${variable}` references
4. Map action strings to `Action` enum
5. Construct `Template` IR

**Advantage**: Modern frontends are **easier to implement** than TextFSM frontend because we control the schema.

## End-to-End Data Flow

### Parse Mode (CLI)

```
┌──────────────────┐
│ User invokes:    │
│ cliscrape parse  │
│  -t template.fsm │
│  input.txt       │
└────────┬─────────┘
         ↓
┌────────────────────────┐
│ TextFsmFrontend        │
│ parses template.fsm    │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ Template (IR)          │
│ - values: {...}        │
│ - states: {...}        │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ FsmEngine::execute()   │
│ processes input.txt    │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ Vec<HashMap<>>         │
│ (structured results)   │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ Serialize to JSON/CSV  │
│ Write to stdout        │
└────────────────────────┘
```

### Debug Mode (TUI)

```
┌──────────────────┐
│ User invokes:    │
│ cliscrape debug  │
│  -t template.fsm │
└────────┬─────────┘
         ↓
┌────────────────────────┐
│ TextFsmFrontend        │
│ parses template.fsm    │
└────────┬───────────────┘
         ↓
┌────────────────────────┐
│ Template (IR)          │
└────────┬───────────────┘
         ↓
┌──────────────────────────────────────────┐
│ DebuggerApp::run(template, input)       │
│                                          │
│  ┌────────────────────────────────────┐ │
│  │ Worker Thread                      │ │
│  │ ┌────────────────────────────────┐ │ │
│  │ │ FsmEngine::execute()           │ │ │
│  │ │  with TraceListener            │ │ │
│  │ └─────────────┬──────────────────┘ │ │
│  │               ↓ (events)            │ │
│  └───────────────┼─────────────────────┘ │
│                  ↓                        │
│  ┌───────────────────────────────────┐   │
│  │ crossbeam_channel                 │   │
│  │ (bounded, non-blocking)           │   │
│  └───────────────┬───────────────────┘   │
│                  ↓                        │
│  ┌───────────────────────────────────┐   │
│  │ TUI Event Loop (main thread)     │   │
│  │ - Receive trace events            │   │
│  │ - Update execution snapshot       │   │
│  │ - Render components               │   │
│  │ - Handle keyboard input           │   │
│  └───────────────────────────────────┘   │
└──────────────────────────────────────────┘
```

## Critical Integration Points

### 1. Frontend → IR Contract

**Interface:**
```rust
pub trait TemplateFrontend {
    fn parse<P: AsRef<Path>>(path: P) -> Result<Template>;
}
```

**Requirements:**
- All frontends must produce identical IR structure
- Regex must be pre-compiled
- Actions must be normalized (no frontend-specific actions)
- State names must be valid identifiers

**Validation:**
- Engine validates IR on construction
- Check for: undefined states in transitions, undefined values in captures, invalid regex

### 2. Engine → TUI Event Contract

**Interface:**
```rust
pub trait TraceListener: Send + Sync {
    fn on_trace(&self, event: TraceEvent);
}
```

**Requirements:**
- Listener must be `Send + Sync` (called from worker thread)
- Events must be cloneable (sent across thread boundary)
- Listener must not block (use try_send, not blocking send)

**Performance:**
- Bounded channel prevents unbounded memory growth
- Events dropped if TUI can't keep up (acceptable for debugging)

### 3. IR → Engine Execution Contract

**Interface:**
```rust
impl FsmEngine {
    pub fn new(template: Template) -> Self;
    pub fn execute(&self, input: &str) -> Result<Vec<HashMap<String, String>>>;
}
```

**Requirements:**
- Engine is stateless (safe to call execute() multiple times)
- Template is immutable during execution
- All regex compiled before execution starts (no runtime compilation)

## Recommended Build Order

### Phase 1: Foundation (Core Engine & IR)
**What to build:**
1. Define IR types (`Template`, `State`, `Rule`, `Action`)
2. Implement `FsmEngine::execute()` (FSM loop)
3. Basic error handling
4. Unit tests with hand-constructed IR

**Deliverable:** Engine that executes IR, tested with hardcoded templates

**Duration:** 2-3 days

**Dependencies:** None

**Why first:** Everything else depends on IR being stable

### Phase 2: TextFSM Frontend
**What to build:**
1. Integrate `textfsm-rust` crate
2. Implement `TextFsmFrontend::parse()`
3. Translation layer: textfsm-rust types → our IR
4. Validation against `ntc-templates` test suite

**Deliverable:** `cliscrape parse -t template.fsm input.txt` works

**Duration:** 3-5 days

**Dependencies:** Phase 1 complete

**Why second:** Unlocks immediate utility, validates IR design

**Reuse from Phase 1:** Engine, IR types, execution logic

### Phase 3: TUI Debugger
**What to build:**
1. Event system (`TraceEvent`, `TraceListener`)
2. Worker thread + channel setup
3. Ratatui components (InputViewer, StateViewer, TraceLog)
4. Keyboard controls (step, scroll, quit)
5. Event → UI state mapping

**Deliverable:** `cliscrape debug -t template.fsm` launches TUI

**Duration:** 5-7 days

**Dependencies:** Phase 1 + Phase 2 complete

**Why third:** Needs working engine + TextFSM frontend to debug

**Reuse from Phase 1+2:** Engine, IR, TextFSM parser, execution logic

### Phase 4: Modern Frontends
**What to build:**
1. YAML schema design
2. `YamlFrontend::parse()`
3. TOML schema (same as YAML)
4. `TomlFrontend::parse()`
5. Documentation + examples

**Deliverable:** `cliscrape parse -t template.yaml input.txt` works

**Duration:** 2-3 days

**Dependencies:** Phase 1 complete (can be parallel with Phase 2)

**Why fourth:** Simpler than TextFSM, can wait until core features work

**Reuse from Phase 1+2+3:** Engine, IR, TUI (modern templates work in debugger too)

## Critical Dependencies (What Blocks What)

```
Phase 1 (IR + Engine)
    ↓ (provides IR contract)
    ├→ Phase 2 (TextFSM Frontend)
    │      ↓ (provides working templates)
    │      └→ Phase 3 (TUI)
    │              ↓ (TUI can debug any frontend)
    └→ Phase 4 (Modern Frontends)
            ↓
        Phase 3 (TUI now supports YAML/TOML too)
```

**Critical path:** Phase 1 → Phase 2 → Phase 3
**Parallel opportunity:** Phase 4 can start after Phase 1

## Anti-Patterns to Avoid

### Anti-Pattern 1: Tightly Coupling Frontend to Engine

**What people do:** Put TextFSM parsing logic directly in engine

**Why it's wrong:**
- Impossible to add YAML/TOML support without rewriting engine
- Engine becomes monolithic (harder to test)
- Violates separation of concerns

**Do this instead:**
- Define IR as strict contract
- All frontends compile to IR
- Engine only knows about IR

### Anti-Pattern 2: Synchronous TUI Rendering

**What people do:** Call `engine.execute()` from TUI main thread

**Why it's wrong:**
- Blocks UI during execution
- Can't render progress or step through execution
- Poor UX for large inputs

**Do this instead:**
- Worker thread pattern
- Event-driven architecture
- Non-blocking channels

### Anti-Pattern 3: Runtime Regex Compilation

**What people do:** Compile regex during `execute()` loop

**Why it's wrong:**
- Regex compilation is expensive (microseconds to milliseconds)
- Called on every line (100k lines = 100k compilations)
- Kills performance

**Do this instead:**
- Compile all regex during IR construction
- Store compiled `Regex` objects in IR
- Engine only calls `regex.captures()`, never `Regex::new()`

### Anti-Pattern 4: Mutable Global State

**What people do:** Store execution state in engine struct

**Why it's wrong:**
- Engine becomes stateful (can't execute() twice)
- Breaks thread-safety
- Hard to test (need to reset state between tests)

**Do this instead:**
- Stateless engine (template is immutable)
- All execution state in `ExecutionContext` (ephemeral per parse)
- Engine safe to call from multiple threads

## Scaling Considerations

This is a **single-user CLI tool**, not a web service. Scaling concerns are about **large inputs**, not concurrent users.

| Input Size | Performance Strategy |
|------------|----------------------|
| < 1k lines | Default (no optimization needed) |
| 1k - 100k lines | Pre-compile regex, use RegexSet for multi-pattern matching |
| 100k - 1M lines | Streaming parser (don't load entire input into memory) |
| 1M+ lines | Parallel execution (split input, merge results) |

**First bottleneck:** Regex matching (solved with pre-compilation)

**Second bottleneck:** Memory usage in TUI trace log (solved with bounded buffer)

## Sources

**Compiler Architecture & IR Design:**
- [Intermediate representation - Wikipedia](https://en.wikipedia.org/wiki/Intermediate_representation)
- [Introduction to Intermediate Representation(IR) - GeeksforGeeks](https://www.geeksforgeeks.org/compiler-design/introduction-to-intermediate-representationir/)
- [How to Build a Compiler Frontend in Rust](https://oneuptime.com/blog/post/2026-01-30-rust-compiler-frontend/view)
- [Intermediate representations (IR) in Compiler Design](https://iq.opengenus.org/intermediate-representations-in-compiler-design/)

**FSM Implementation Patterns:**
- [Generic Finite State Machines with Rust's Type State Pattern | Medium](https://medium.com/@alfred.weirich/generic-finite-state-machines-with-rusts-type-state-pattern-04593bba34a8)
- [Pretty State Machine Patterns in Rust](https://hoverbear.org/blog/rust-state-machine-pattern/)
- [Event-Based Finite State Machines in Rust — MoonBench](https://moonbench.xyz/projects/rust-event-driven-finite-state-machine/)

**Ratatui Architecture:**
- [Component Architecture | Ratatui](https://ratatui.rs/concepts/application-patterns/component-architecture/)
- [The Elm Architecture (TEA) | Ratatui](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/)
- [Flux Architecture | Ratatui](https://ratatui.rs/concepts/application-patterns/flux-architecture/)

**TextFSM & Parsing:**
- [GitHub - google/textfsm](https://github.com/google/textfsm)
- [TextFSM Wiki](https://github.com/google/textfsm/wiki/TextFSM)
- [High Performance Text Parsing Using FSM | HackerNoon](https://hackernoon.com/high-performance-text-parsing-using-finite-state-machines-fsm-6d3m33j9)

**Rust Regex Optimization:**
- [regex - Rust](https://docs.rs/regex/latest/regex/)
- [Regex engine internals as a library - Andrew Gallant's Blog](https://burntsushi.net/regex-internals/)
- [GitHub - rust-lang/regex](https://github.com/rust-lang/regex)

---
*Architecture research for: cliscrape v0.1 Alpha*
*Researched: 2026-02-17*
*Confidence: HIGH*

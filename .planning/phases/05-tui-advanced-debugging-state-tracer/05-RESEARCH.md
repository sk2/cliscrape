# Phase 05: TUI Advanced Debugging (State Tracer) - Research

**Researched:** 2026-02-21
**Domain:** Time-travel debugging + FSM state visualization + execution trace navigation (Rust TUI)
**Confidence:** MEDIUM

## Summary

Phase 5 extends the Phase 4 Live Lab with time-travel debugging capabilities: step through FSM state transitions line-by-line, inspect variable changes over time, navigate trace history with jump/search features, and filter trace events to reduce noise. This is **post-parse inspection** of the execution trace, not real-time watch-as-it-parses.

The core challenge is representing **temporal state** (FSM state + variable values at each decision point) in a way that supports efficient forward/backward stepping, filtering, and quick jumps. The standard approach is a linear trace buffer (Vec of trace events) with indexed lookups for navigation. Phase 4 already produces `DebugReport` with per-line matches; Phase 5 extends this with **variable state snapshots** at each transition and adds navigation/filtering UI.

**Primary recommendation:** extend `DebugReport` with a `Vec<TraceEvent>` that records FSM state + variable values at each line; build timeline navigation UI with configurable stepping modes (line/state/action), jump shortcuts (next Record, search state), and filtering controls (keyboard toggles + filter panel); use `StatefulWidget` pattern for timeline scrubber/slider if needed.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **State transition visualization:** Timeline view showing state sequence with line numbers (e.g., 'HEADER @L15')
- **Navigation controls:** Three stepping modes (line-by-line / state-by-state / action-by-action) + jump capabilities (next/previous Record, jump to state, jump to line) + timeline scrubbing (slider + keyboard)
- **Variable inspection:** Highlight changed variables + show old→new in tooltip/detail view (both quick feedback + detailed understanding)
- **Watch feature:** Both pin (always show at top) and filter (only show watched) options — user choice
- **Trace filtering & scope:** Default view shows everything; filtering via panel (checkboxes) + quick keyboard shortcuts

### Claude's Discretion
- Current state highlighting approach (color, marker, split panes)
- Timeline volume handling (scrolling, collapsing, pagination)
- Navigation input balance (keyboard vs mouse)
- Variable display format (table, list, JSON tree)
- Variable history access pattern (click, panel, tooltip)
- No-match line handling (show with indicator, separate, or hidden)
- Trace preset levels (whether built-in presets improve UX)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|-----------------|
| TUI-02 | TUI shows the current FSM state (e.g., `START` -> `HEADER` -> `BODY`) for the selected line | Timeline data structure (TraceEvent with state_before/state_after); timeline widget showing state sequence; state name display in UI |
| TUI-03 | User can step forward/backward through parsing process + trace buffer allows reviewing all transitions that led to a Record action | Vec<TraceEvent> navigation (index-based stepping); filtering by action type (Record); jump-to-event shortcuts; trace retention in DebugReport |

</phase_requirements>

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ratatui` | 0.30.0 (already in repo) | TUI widgets + layout + styling | Phase 4 foundation; supports `StatefulWidget` for interactive timeline scrubber (docs.rs) |
| `crossterm` | 0.29.0 (already in repo) | Keyboard events for step/jump/filter shortcuts | Already integrated in Phase 4 event loop (docs.rs) |
| `serde_json` | (already in repo) | Display typed variable values | Engine already emits typed captures as `serde_json::Value` |
| `regex` | 1.12.3 (already in repo) | Byte offsets for highlight spans | Phase 4 already uses for capture span rendering |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `tui-textarea` | ~0.7.x | Text input for search/jump-to-line forms | If inline search input is needed for "jump to state" feature; supports search patterns via regex (GitHub) |
| `rat-widget` | ~2.5.x | Slider widget for timeline scrubber | Optional: if visual timeline slider improves navigation UX (supports both horizontal and vertical orientations) (crates.io) |
| `tui-slider` | (ecosystem widget) | Alternative slider for timeline navigation | If `rat-widget` compatibility is unclear; highly customizable (awesome-ratatui) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `Vec<TraceEvent>` (linear buffer) | Sparse interval tree (Expositor-style) | Interval trees support efficient range queries but add complexity; linear Vec is O(1) indexed access and sufficient for traces <100k events |
| Custom timeline widget | Third-party slider widget | Custom widget gives full control over rendering; third-party widgets may need compatibility verification with Ratatui 0.30 |
| Filtering via UI toggles | Save filter presets to config file | Presets add persistence but increase scope; in-session toggles are simpler and cover primary use case |

**Installation (new deps for this phase — if slider widget chosen):**
```bash
# Optional: only if visual slider improves UX
cargo add rat-widget  # or tui-slider if compatibility verified
```

## Architecture Patterns

### Recommended Project Structure
Extend Phase 4's `src/tui/` module with trace navigation logic.

```
src/
├── tui/
│   ├── app.rs            # Add: trace_index, stepping_mode, filter_state, watch_list
│   ├── ui.rs             # Add: render_timeline_pane, render_variable_history_pane
│   ├── event.rs          # Add: key mappings for step/jump/filter shortcuts
│   └── trace.rs          # NEW: TraceEvent, TraceNavigator (stepping logic)
└── engine/
    └── debug.rs          # Extend: DebugReport with Vec<TraceEvent>
```

### Pattern 1: Trace Event Buffer with State Snapshots
**What:** a linear `Vec<TraceEvent>` where each event captures FSM state + variable values at a specific line index.

**When to use:** required for time-travel debugging; supports forward/backward stepping and filtering.

**Example:**
```rust
// Extension of src/engine/debug.rs (Phase 4 already has LineMatch/EmittedRecord)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    pub line_idx: usize,
    pub state_before: String,
    pub state_after: String,
    pub variables: HashMap<String, serde_json::Value>,  // Current variable state
    pub event_type: TraceEventType,  // Line, StateChange, Record, Clear
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TraceEventType {
    LineProcessed,    // Every line
    StateChange,      // FSM transitioned
    RecordEmitted,    // Record action
    VariableChanged,  // Variable updated
}

// In DebugReport:
pub struct DebugReport {
    pub lines: Vec<String>,
    pub matches_by_line: Vec<Vec<LineMatch>>,
    pub records: Vec<EmittedRecord>,
    pub trace: Vec<TraceEvent>,  // NEW: temporal trace
}
```

**Notes for planning:**
- Record one `TraceEvent` per line processed (captures state + variables at each step).
- For efficiency, consider storing only changed variables per event (delta encoding) if traces become large.
- Filtering operates on `event_type` to show subset (e.g., only `StateChange` or `RecordEmitted`).

### Pattern 2: Stepping Modes via Index Navigation
**What:** navigation state tracks current `trace_index` and stepping mode determines how index advances.

**When to use:** always; implements user's requested stepping granularity.

**Example:**
```rust
// In src/tui/app.rs (extension of AppState)
pub enum SteppingMode {
    LineByLine,       // Every trace event
    StateByState,     // Only events where state_before != state_after
    ActionByAction,   // Only Record/Clear events
}

impl AppState {
    pub fn step_forward(&mut self) {
        let Some(report) = &self.last_good else { return };
        let next_idx = self.find_next_event(report, self.trace_index, self.stepping_mode);
        self.trace_index = next_idx;
        self.sync_cursor_to_trace();
    }

    fn find_next_event(&self, report: &DebugReport, current: usize, mode: SteppingMode) -> usize {
        match mode {
            SteppingMode::LineByLine => (current + 1).min(report.trace.len().saturating_sub(1)),
            SteppingMode::StateByState => {
                report.trace.iter().enumerate()
                    .skip(current + 1)
                    .find(|(_, e)| e.state_before != e.state_after)
                    .map(|(i, _)| i)
                    .unwrap_or(current)
            },
            SteppingMode::ActionByAction => {
                report.trace.iter().enumerate()
                    .skip(current + 1)
                    .find(|(_, e)| matches!(e.event_type, TraceEventType::RecordEmitted))
                    .map(|(i, _)| i)
                    .unwrap_or(current)
            }
        }
    }
}
```

### Pattern 3: Variable Change Highlighting
**What:** compare current trace event's variables with previous event; highlight variables that changed.

**When to use:** required for variable inspection feature (highlight + old→new display).

**Example (UI rendering logic):**
```rust
// In src/tui/ui.rs
fn render_variables_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let Some(report) = &app.last_good else { return };
    let Some(current_event) = report.trace.get(app.trace_index) else { return };
    let prev_event = if app.trace_index > 0 {
        report.trace.get(app.trace_index - 1)
    } else {
        None
    };

    let mut lines: Vec<Line> = Vec::new();
    for (var_name, current_val) in &current_event.variables {
        let changed = prev_event
            .and_then(|p| p.variables.get(var_name))
            .map(|prev_val| prev_val != current_val)
            .unwrap_or(true);  // New variable

        let style = if changed {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        let display = format!("{} = {}", var_name, display_value_compact(current_val));
        lines.push(Line::from(Span::styled(display, style)));
    }

    // Render list...
}
```

**Notes for planning:**
- Use color (yellow/red foreground) to indicate changed variables (industry standard from Visual Studio/IntelliJ).
- Tooltip/detail pane can show "old→new" by fetching previous event's value.

### Pattern 4: Jump-to-Event Shortcuts
**What:** keyboard shortcuts to jump to next/previous event matching criteria (e.g., next Record, find state).

**When to use:** required for efficient navigation in large traces.

**Example:**
```rust
// In src/tui/event.rs (extend key handler)
use crossterm::event::{KeyCode, KeyModifiers};

match key.code {
    KeyCode::Char('n') if key.modifiers.contains(KeyModifiers::CONTROL) => {
        // Jump to next Record action
        app.jump_to_next_record();
    }
    KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
        // Jump to previous Record action
        app.jump_to_previous_record();
    }
    KeyCode::Char('/') => {
        // Enter search mode: prompt for state name to jump to
        app.enter_search_mode();
    }
    // ...
}

// In src/tui/app.rs
impl AppState {
    pub fn jump_to_next_record(&mut self) {
        let Some(report) = &self.last_good else { return };
        if let Some((idx, _)) = report.trace.iter().enumerate()
            .skip(self.trace_index + 1)
            .find(|(_, e)| matches!(e.event_type, TraceEventType::RecordEmitted))
        {
            self.trace_index = idx;
            self.sync_cursor_to_trace();
        }
    }
}
```

### Pattern 5: Filter State Management
**What:** track active filters (show/hide event types) and apply during rendering.

**When to use:** required for trace filtering feature (panel + keyboard toggles).

**Example:**
```rust
// In src/tui/app.rs
#[derive(Debug, Clone)]
pub struct FilterState {
    pub show_line_events: bool,
    pub show_state_changes: bool,
    pub show_record_actions: bool,
    pub show_no_match_lines: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            show_line_events: true,
            show_state_changes: true,
            show_record_actions: true,
            show_no_match_lines: true,
        }
    }
}

impl AppState {
    pub fn toggle_filter_line_events(&mut self) {
        self.filter_state.show_line_events = !self.filter_state.show_line_events;
    }

    pub fn filtered_trace<'a>(&'a self, report: &'a DebugReport) -> Vec<(usize, &TraceEvent)> {
        report.trace.iter().enumerate()
            .filter(|(_, e)| {
                match e.event_type {
                    TraceEventType::LineProcessed => self.filter_state.show_line_events,
                    TraceEventType::StateChange => self.filter_state.show_state_changes,
                    TraceEventType::RecordEmitted => self.filter_state.show_record_actions,
                    _ => true,
                }
            })
            .collect()
    }
}
```

**Notes for planning:**
- Keyboard shortcuts (e.g., `f1` toggle line events, `f2` toggle state changes) for quick filtering.
- Filter panel (checkbox UI) for visual feedback of active filters.

### Anti-Patterns to Avoid
- **Storing full variable snapshots without delta encoding:** traces can grow large (10k+ events); consider storing only changed variables per event.
- **Linear search for jumps:** cache indices of specific event types (e.g., all Record events) for O(1) jump navigation.
- **Tight coupling of timeline UI to trace data:** keep trace navigation logic (stepping, filtering, jumping) separate from rendering for testability.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Text input for search | Custom key-by-key input handling | `tui-textarea` or `rat-input` | Handles cursor movement, editing, backspace, UTF-8; regex search built-in (tui-textarea) |
| Timeline slider widget | Manual drag/scroll rendering | `rat-widget` slider or `tui-slider` | Cross-platform mouse/keyboard interaction; visual feedback |
| Variable diff logic | Manual old/new value comparison | Direct HashMap comparison (`prev.get(k) != current.get(k)`) | Serde's `PartialEq` handles JSON value equality |

**Key insight:** the "hard" part of State Tracer is not the UI widgets; it is **designing the trace data model** (what to capture per event) and **efficient navigation logic** (stepping modes, filtering, jumping).

## Common Pitfalls

### Pitfall 1: Trace buffer memory explosion
**What goes wrong:** large input files (1000+ lines) produce traces with 10k+ events; storing full variable state per event consumes excessive memory.
**Why it happens:** naive snapshot approach duplicates unchanged variables.
**How to avoid:** store only changed variables per event (delta encoding); reconstruct full state on-demand by replaying deltas from last full snapshot.
**Warning signs:** TUI becomes sluggish with large inputs; high memory usage.

### Pitfall 2: Cursor position out-of-sync with trace index
**What goes wrong:** stepping through trace moves trace index, but cursor remains on old line; UI shows wrong line as "current".
**Why it happens:** forgetting to sync `cursor_line_idx` when `trace_index` changes.
**How to avoid:** always call `sync_cursor_to_trace()` after updating trace index; extract sync logic to method called by all navigation functions.
**Warning signs:** stepping forward shows wrong line highlighted; line numbers don't match trace event.

### Pitfall 3: Filtering breaks jump-to shortcuts
**What goes wrong:** "jump to next Record" finds an event that is currently hidden by filter; UI shows no change.
**Why it happens:** jump logic operates on full trace, not filtered view.
**How to avoid:** apply filtering before jump search, or display "event hidden by filter" message if jump target is filtered out.
**Warning signs:** jump shortcuts appear broken when filters are active.

### Pitfall 4: Variable watch list not persisting
**What goes wrong:** user pins variables, then restarts TUI; watch list is lost.
**Why it happens:** watch list only stored in `AppState`, not persisted.
**How to avoid:** if persistence is desired, save watch list to config file (e.g., `.cliscrape/watch.json`); otherwise document that watch is session-only.
**Warning signs:** user complaints about re-adding watches every session.

### Pitfall 5: Timeline scrubber performance with large traces
**What goes wrong:** rendering 10k+ trace events in a scrollable timeline causes lag.
**Why it happens:** rendering all events every frame; no virtualization.
**How to avoid:** render only visible window of events (viewport slicing); use `ratatui::widgets::List` with `start_corner` to show subset.
**Warning signs:** frame rate drops below 30 FPS; scrolling feels sluggish.

## Code Examples

### Build trace event from FSM execution
```rust
// In src/engine/fsm.rs (extend parse_internal)
// Source: project codebase + standard trace buffer pattern

fn parse_internal(
    &self,
    input: &str,
    mut debug: Option<&mut DebugReport>,
) -> Result<Vec<HashMap<String, serde_json::Value>>, ScraperError> {
    let mut current_state = "Start".to_string();
    let mut record_buffer = RecordBuffer::new();

    let lines: Vec<&str> = input.lines().collect();
    let mut line_idx = 0;

    while line_idx < lines.len() {
        let line = lines[line_idx];

        // Capture state before processing line
        let state_before = current_state.clone();

        // ... rule matching logic ...

        // After match, record trace event
        if let Some(ref mut debug_report) = debug {
            let trace_event = TraceEvent {
                line_idx,
                state_before: state_before.clone(),
                state_after: current_state.clone(),
                variables: record_buffer.current_values(),  // Snapshot
                event_type: if matched {
                    if state_before != current_state {
                        TraceEventType::StateChange
                    } else {
                        TraceEventType::LineProcessed
                    }
                } else {
                    TraceEventType::LineProcessed
                },
            };
            debug_report.trace.push(trace_event);
        }

        line_idx += 1;
    }

    Ok(results)
}
```

### Render timeline with state transitions
```rust
// In src/tui/ui.rs (new timeline pane)
// Source: Ratatui List widget + custom styling

use ratatui::widgets::{List, ListItem};

fn render_timeline_pane(frame: &mut Frame, area: Rect, app: &AppState) {
    let Some(report) = &app.last_good else {
        let block = Block::default().borders(Borders::ALL).title("Timeline");
        frame.render_widget(Paragraph::new("(no trace)").block(block), area);
        return;
    };

    let filtered = app.filtered_trace(report);

    let items: Vec<ListItem> = filtered.iter()
        .map(|(idx, event)| {
            let is_current = *idx == app.trace_index;
            let prefix = if is_current { ">" } else { " " };

            let text = format!(
                "{} L{:>4} | {} -> {} | {:?}",
                prefix,
                event.line_idx + 1,
                event.state_before,
                event.state_after,
                event.event_type
            );

            let style = if is_current {
                Style::default().bg(Color::Rgb(50, 50, 50)).fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let block = Block::default().borders(Borders::ALL)
        .title(format!("Timeline ({}/{})", app.trace_index + 1, report.trace.len()));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
```

### Variable diff with old→new display
```rust
// In src/tui/ui.rs (variable detail tooltip)

fn format_variable_change(
    var_name: &str,
    current_val: &serde_json::Value,
    prev_val: Option<&serde_json::Value>,
) -> String {
    if let Some(prev) = prev_val {
        if prev != current_val {
            format!("{}: {} -> {}", var_name,
                display_value_compact(prev),
                display_value_compact(current_val))
        } else {
            format!("{}: {}", var_name, display_value_compact(current_val))
        }
    } else {
        format!("{}: {} (new)", var_name, display_value_compact(current_val))
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Full variable snapshots per event | Delta encoding + periodic full snapshots | Expositor research (2012) | Reduces trace memory by 60-80%; O(k) reconstruction where k = events since last snapshot |
| Linear scan for filtered events | Pre-indexed event type lookups | Modern debuggers (Visual Studio IntelliTrace 2019+) | O(1) jump to next Record instead of O(n) scan |
| Single stepping mode | Multi-mode stepping (line/statement/instruction) | GDB multi-level stepping (2010s) | Matches user's debugging mental model (zoom in/out) |
| Text-only variable display | Color-coded change highlighting | Visual Studio 2015+ (red for changed) | Immediate visual feedback on variable mutations |

**Deprecated/outdated:**
- Storing trace as linked list for bidirectional stepping: Vec with index is faster (O(1) vs O(n) for jumps).
- Embedding full stack traces in each event: FSM is single-threaded; only state name needed.

## Open Questions

1. **Trace buffer size limits**
   - What we know: Large inputs (10k lines) could produce 50k+ trace events; full snapshots would consume significant memory.
   - What's unclear: Whether delta encoding adds enough implementation complexity to outweigh benefits for typical cliscrape use (templates parse <1000 lines).
   - Recommendation: Start with full snapshots per event (simple); add delta encoding only if memory profiling shows issue with real-world templates.

2. **Timeline scrubber widget compatibility**
   - What we know: `rat-widget` and `tui-slider` exist in ecosystem; Ratatui 0.30 is current.
   - What's unclear: Whether these widgets have verified Ratatui 0.30 compatibility (crates.io pages not loaded successfully).
   - Recommendation: Verify compatibility via cargo check before committing to slider widget; fallback is custom `StatefulWidget` using List + keyboard navigation (no visual slider).

3. **Search input UX**
   - What we know: `tui-textarea` supports regex search patterns; useful for "jump to state name" feature.
   - What's unclear: Whether inline search input (modal prompt) vs dedicated search pane improves UX.
   - Recommendation: Start with keyboard shortcut that prompts for state name (simple modal); evaluate if dedicated search pane is needed based on user feedback.

## Sources

### Primary (HIGH confidence)
- [Ratatui StatefulWidget documentation](https://docs.rs/ratatui/latest/ratatui/) - widget state management
- [Crossterm key event handling](https://docs.rs/crossterm/latest/crossterm/) - keyboard shortcuts
- [Microsoft Visual Studio debugger variable highlighting](https://github.com/microsoft/vscode/issues/866) - changed variable UI pattern
- [GDB TUI keyboard navigation](https://ftp.gnu.org/old-gnu/Manuals/gdb/html_chapter/gdb_19.html) - debugger navigation shortcuts
- [Time Travel Debugging (TTD) Microsoft docs](https://learn.microsoft.com/en-us/windows-hardware/drivers/debuggercmds/time-travel-debugging-replay) - trace buffer patterns

### Secondary (MEDIUM confidence)
- [Expositor: Scriptable time-travel debugging](https://api.drum.lib.umd.edu/server/api/core/bitstreams/2d79238c-c752-44b2-878b-3ed3742a84f1/content) - sparse interval tree for traces
- [Visual Studio IntelliTrace step-back](https://learn.microsoft.com/en-us/visualstudio/debugger/view-historical-application-state?view=vs-2022) - execution history patterns
- [tui-textarea search patterns](https://github.com/rhysd/tui-textarea) - regex search in TUI text widgets
- [rat-widget crates.io](https://crates.io/crates/rat-widget) - slider widget availability
- [Ratatui awesome list](https://github.com/ratatui/awesome-ratatui) - ecosystem widgets (tui-slider)

### Tertiary (LOW confidence - flagged for validation)
- tui-slider compatibility with Ratatui 0.30 (crates.io page not verified)
- rat-widget v2.5.0 ratatui dependency version (not confirmed in search)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - Ratatui/Crossterm already in Phase 4; serde_json/regex in engine
- Architecture: MEDIUM - trace buffer pattern verified via TTD docs; stepping modes verified via GDB; variable change UI verified via VS Code
- Pitfalls: MEDIUM - trace memory issue is known (Expositor); cursor sync is general TUI state management; filtering/jumping interaction is logical inference

**Research date:** 2026-02-21
**Valid until:** 2026-03-14 (Ratatui ecosystem relatively stable; widget compatibility may shift)

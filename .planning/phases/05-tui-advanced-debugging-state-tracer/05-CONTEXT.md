# Phase 5: TUI Advanced Debugging (State Tracer) - Context

**Gathered:** 2026-02-21
**Status:** Ready for planning

<domain>
## Phase Boundary

Enable deep inspection of FSM state transitions and variable state during the parsing process. This phase adds time-travel debugging capabilities to the TUI, allowing users to step through the parser's decision-making process, view variable changes over time, and understand why specific records were emitted. Template editing and live visualization are Phase 4 - this focuses on execution tracing and state inspection.

</domain>

<decisions>
## Implementation Decisions

### State transition visualization
- **Primary display:** Timeline view showing state sequence
- **Line number display:** Always show line numbers with each transition (e.g., 'HEADER @L15')
- Helps correlate FSM decisions with specific input lines

### Navigation controls
- **Step granularity:** Configurable - user can toggle between three stepping modes:
  - Line-by-line (every input line processed)
  - State-by-state (only FSM state changes)
  - Action-by-action (only Record/Clear/etc events)
- **Jump capabilities:** Support multiple quick-jump features:
  - Jump to next/previous Record action (find where records were emitted)
  - Jump to specific state name (search for state occurrences)
  - Jump to line number (when you know where the issue is)
- **Timeline scrubbing:** Both visual scrubber (slider) and keyboard shortcuts for jumping through trace
- Maximum navigation flexibility for different debugging workflows

### Variable inspection
- **Change visualization:** Combination approach:
  - Highlight changed variables (immediate visual feedback)
  - Show old→new values in tooltip or detail view
  - Best of both worlds - quick to spot + detailed to understand
- **Watch feature:** Both pin and filter options:
  - Pin specific variables to always show at top
  - OR filter variable list to show only watched vars
  - User choice for their preferred workflow

### Trace filtering & scope
- **Default view:** Show everything (every line processed, every state check, every decision)
  - Maximum detail available by default
- **Filtering controls:** Both filter panel and quick keyboard toggles:
  - UI panel with checkboxes for granular control (show/hide state changes, actions, no-match lines, etc.)
  - Quick keyboard shortcuts for common filters (fast workflow)
  - Supports both careful configuration and rapid filtering

### Claude's Discretion
- Current state highlighting approach (color, marker, split panes - choose most effective)
- Timeline volume handling (scrolling, collapsing, pagination - optimize for large traces)
- Navigation input method balance (keyboard vs mouse - support both effectively)
- Variable display format (table, list, JSON tree - choose clearest)
- Variable history access pattern (click, panel, tooltip - best UX)
- No-match line handling (show with indicator, separate section, or hidden - reduce noise while maintaining visibility)
- Trace preset levels (whether built-in presets improve debugging experience)

</decisions>

<specifics>
## Specific Ideas

None mentioned - implementation should follow standard debugging tool patterns and leverage Phase 4's TUI foundation.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 05-tui-advanced-debugging-state-tracer*
*Context gathered: 2026-02-21*

# Phase 4: TUI Debugger Foundation (Live Lab) - Context

**Gathered:** 2026-02-20
**Status:** Ready for planning

<domain>
## Phase Boundary

Provide a real-time "Live Lab" TUI debugger for template development and regex matching:
- split-screen UI with raw CLI output and match information,
- live updates when the template file changes, and
- navigation through the CLI output to see which lines match which template rules.

This phase is about the TUI foundation and live-update loop. Deep step-through state tracing and variable timelines belong to Phase 5.

</domain>

<decisions>
## Implementation Decisions

### Screen layout + navigation
- Layout: 3-pane design.
- Primary cursor: line cursor over raw text (up/down by line); other panes follow.
- Keyboard: support BOTH arrow keys and vim-style navigation keys.
- Pane sync: default to lockstep (selecting a line updates the match/details panes automatically).

### Match visualization
- Raw text highlighting: BOTH full-line shading for matched lines and capture-span accents for field matches.
- Details pane: show BOTH captured fields (typed values) AND rule context (state, which rule matched, action).
- Views: support BOTH per-line matches and emitted records (toggle between them).
- Multiple matches per line (Continue): show stacked match entries by default.

### Live reload + error surfacing
- Live reload: auto-reload immediately when files change.
- Watch BOTH template file and input text file; reparse on either change.
- Error surfacing: dedicated error panel.
- Invalid template behavior: keep last good results visible while showing the new error (avoid blanking the UI on transient edits).

### Workflow + states
- Entrypoint: ship as a `cliscrape debug` subcommand (single distribution).
- Startup: if template/input paths are missing, show an interactive picker inside the TUI.
- Editing: inline template editing is in-scope for Phase 4 (not external-editor-only).
- Empty state: emphasize guidance (why no matches/records and what to try next).

### Claude's Discretion
- Exact pane contents and keybinding map, as long as the above navigation and view decisions are preserved.
- Visual styling (colors, legends, status bar), as long as match visualization remains readable.

</decisions>

<specifics>
## Specific Ideas

- The UI should make it obvious which line is selected and what rule/fields are active for that line.
- Reload feedback should be visible but subtle (status indicator + error panel when needed).

</specifics>

<deferred>
## Deferred Ideas

- Step-forward/backward FSM tracing, variable timeline, and a trace buffer for transitions are Phase 5.

</deferred>

---

*Phase: 04-tui-debugger-foundation-(live-lab)*
*Context gathered: 2026-02-20*

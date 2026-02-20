# Phase 04: TUI Debugger Foundation (Live Lab) - Research

**Researched:** 2026-02-21
**Domain:** Rust TUI (Ratatui) + live file reload + regex/FSM match visualization
**Confidence:** MEDIUM

## Summary

This phase is a real-time TUI “workbench” for template development: show raw CLI output, show which rules match which lines (including `Continue` stacking), and show captured fields + rule context for the currently selected line. The differentiator vs a normal `parse` run is that the UI must stay responsive while continuously re-parsing on template/input changes and while the user navigates.

The standard approach with Ratatui is an immediate-mode render loop driven by an event pump (keyboard + periodic tick) plus background producers (filesystem watcher, parse worker) sending messages into the main loop. Parsing must produce a **debug report** (per-line match trace + emitted records) rather than only final records, so the engine needs a “debug parse” API that records match events (state/rule/actions + capture spans).

**Primary recommendation:** implement a single UI event loop (Ratatui + Crossterm) that consumes a unified `Message` channel (keys/ticks/watch/parse results) and drives a cached `DebugReport` model; use `notify-debouncer-mini` to coalesce editor save storms; keep “last good parse” visible when edits are invalid.

## Standard Stack

### Core
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `ratatui` | 0.30.0 (already in repo) | TUI layout, widgets, styling | Current Ratatui release; provides `run()`/`init()` helpers and `Text`/`Line`/`Span` primitives for highlighting (docs.rs) |
| `crossterm` | 0.29.0 (already in repo) | Terminal backend + input events | Canonical backend used by Ratatui; key events include `KeyEventKind::Press` filtering (docs.rs) |
| `notify` | 8.2.x | Cross-platform file watch | De-facto Rust FS watcher; supports `recommended_watcher()` and documents editor save edge cases (docs.rs) |
| `notify-debouncer-mini` | 0.7.x | Debounce/coalesce FS events | Recommended by `notify` docs to avoid event storms and editor behavior differences (docs.rs) |
| `regex` | 1.12.3 (already in repo) | Rule matching + capture spans | Provides capture group byte offsets for highlight spans (already in engine) |

### Supporting
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `anyhow` / `thiserror` | (already in repo) | Error propagation + typed errors | TUI should display errors without crashing; keep last-good results |
| `serde_json` | (already in repo) | Display typed captures | Details pane can show typed values (engine already emits `serde_json::Value`) |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| sync event loop | `tokio` + `crossterm::EventStream` | More plumbing + new runtime dependency; only worth it if you want async select over many sources (Ratatui recipe exists) |
| external editor only | inline editor widget crate | Inline editing is explicitly in-scope; however, popular textarea crates may lag Ratatui 0.30 compatibility, so plan for a minimal in-app editor if needed |

**Installation (new deps for this phase):**
```bash
cargo add notify notify-debouncer-mini
```

## Architecture Patterns

### Recommended Project Structure
Keep TUI code isolated from parsing/engine logic.

```
src/
├── tui/
│   ├── mod.rs
│   ├── app.rs            # AppState, view model, selections
│   ├── ui.rs             # draw(frame, &AppState)
│   ├── event.rs          # Key mapping -> Action/Message
│   ├── watch.rs          # file watch + debouncer -> Message
│   └── worker.rs         # parse worker -> Message(DebugReport)
└── main.rs               # `cliscrape debug` entrypoint
```

### Pattern 1: Immediate-Mode UI + Message Bus
**What:** a single main loop that: (1) draws the frame from current state, (2) handles one incoming message (key/tick/watch/parse result), (3) updates state.

**When to use:** always for Ratatui apps that also need background inputs.

**Example:**
```rust
// Source: https://docs.rs/ratatui/latest/ratatui/ (run()/init()/restore patterns)
fn main() -> std::io::Result<()> {
    ratatui::run(|terminal| {
        // loop { draw; handle messages; }
        Ok(())
    })
}
```

**Notes for planning:**
- Use a single enum `Message` (e.g., `Key(KeyEvent)`, `Tick`, `FsChanged(PathBuf)`, `ParseDone(DebugReport)`, `ParseError(ParseError)`).
- Keep the render step pure: `ui::draw(frame, &app_state)`.
- Filter keys by `KeyEventKind::Press` to avoid double-handling repeats.

### Pattern 2: Debounced File Watch -> “Parse Request”
**What:** watch template + input paths; debounce events; on change send a single “reparse” request.

**When to use:** always; real editors often write temp files and generate multiple events.

**Example:**
```rust
// Source: https://docs.rs/notify/latest/notify/ and
//         https://docs.rs/notify-debouncer-mini/latest/notify_debouncer_mini/
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use notify::{RecursiveMode, Watcher};
use std::{path::Path, time::Duration};

let mut debouncer = new_debouncer(Duration::from_millis(150), move |res: DebounceEventResult| {
    // send Message::FsChanged for relevant paths
    let _ = res;
})?;
debouncer.watcher().watch(Path::new("/path/to/template"), RecursiveMode::NonRecursive)?;
debouncer.watcher().watch(Path::new("/path/to/input"), RecursiveMode::NonRecursive)?;
```

**Notes for planning:**
- Watch the parent directory when you need to handle replace-on-save (atomic rename) reliably (notify docs: “Editor Behaviour”).
- Debounce window should be short (100–300ms) to feel “instant” but avoid storms.

### Pattern 3: “Debug Parse” API That Produces Match Trace
**What:** extend the engine with an API that records per-line match events while running the same FSM logic as `parse()`.

**When to use:** required to implement per-line highlights and “rule context” pane.

**Minimum data model (Phase 4 scope):**
- `LineMatch`: `line_idx`, `state_before`, `state_after`, `rule_idx`, `line_action`, `record_action`, `next_state`, `captures[]`.
- `CaptureSpan`: `name`, `start_byte`, `end_byte`, `raw`, `typed` (optional), `is_list`.
- `EmittedRecord`: `at_line_idx`, `record: HashMap<String, Value>`.

**Key insight:** `regex::Match` gives byte offsets; Ratatui highlights should be built by slicing the line string on UTF-8 boundaries and wrapping slices in styled `Span`s.

### Anti-Patterns to Avoid
- **Parsing on the UI thread:** a template edit that recompiles many regexes will freeze the TUI; use a worker thread and send `ParseDone/ParseError`.
- **Clearing results on parse error:** transient invalid edits are common; keep last-good `DebugReport` visible and show error panel with “new parse failed”.
- **Assuming file watchers deliver one event per save:** editors can write temp files, truncate, rename; rely on debouncing and consider watching parent directory.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| FS watch + debounce | polling loops / custom debouncer | `notify` + `notify-debouncer-mini` | Cross-platform edge cases; editor save storms; documented known problems |
| Terminal init/restore | manual raw mode + alt screen + panic restore | `ratatui::run()` (or `init()`/`restore()`) | Correct teardown on panic/early return; fewer footguns |
| Styling/highlight rendering | manual ANSI + cursor movement | `Text`/`Line`/`Span` + widgets (`Paragraph`, `List`, `Table`) | Ratatui diff renderer avoids flicker; easy composition |

**Key insight:** the “hard” part of Live Lab is not drawing panes; it is **consistent, debounced, non-blocking state updates** plus **stable debug trace output**.

## Common Pitfalls

### Pitfall 1: Terminal left in raw mode
**What goes wrong:** after crash/early exit, terminal input behaves strangely.
**Why it happens:** missing `restore()` on error/panic.
**How to avoid:** run via `ratatui::run()` which sets up restoration hooks (docs.rs).
**Warning signs:** keyboard input not echoed after `cliscrape debug` exits.

### Pitfall 2: File saves not detected (or detected too often)
**What goes wrong:** reload doesn’t trigger, or triggers multiple times per save.
**Why it happens:** editor uses atomic rename/temp files; watcher emits multiple events.
**How to avoid:** use `notify-debouncer-mini`; consider watching the parent directory; treat any event as “dirty” and re-read the file.
**Warning signs:** status bar shows rapid reload flicker; missed updates when saving from certain editors.

### Pitfall 3: Incorrect highlight spans
**What goes wrong:** capture highlight is shifted or panics due to invalid slicing.
**Why it happens:** mixing byte offsets with char indices; tabs/unicode width issues.
**How to avoid:** slice the original `&str` using regex-provided byte offsets (UTF-8 boundaries); avoid converting to “column indices” until necessary.
**Warning signs:** panic from `str::get(..)` returning `None`; highlights misaligned on non-ASCII.

### Pitfall 4: `Continue` semantics not represented
**What goes wrong:** user sees only one match per line though multiple rules matched.
**Why it happens:** trace model overwrites prior match for the same `line_idx`.
**How to avoid:** store `Vec<LineMatch>` per line (stacked entries) in match order.
**Warning signs:** templates using `Continue` appear to “skip” intermediate rules.

## Code Examples

### Build a highlighted line from capture spans
```rust
// Source: Ratatui text primitives docs
// https://docs.rs/ratatui/latest/ratatui/ (Text/Line/Span overview)
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

fn highlight_line(line: &str, spans: &[(usize, usize, Style)]) -> Line {
    // spans: (start_byte, end_byte, style), non-overlapping and sorted
    let mut out: Vec<Span> = Vec::new();
    let mut cursor = 0;
    for (s, e, st) in spans {
        if cursor < *s {
            out.push(Span::raw(&line[cursor..*s]));
        }
        out.push(Span::styled(&line[*s..*e], *st));
        cursor = *e;
    }
    if cursor < line.len() {
        out.push(Span::raw(&line[cursor..]));
    }
    Line::from(out)
}

let _example_style = Style::default().fg(Color::Black).bg(Color::Yellow);
```

### `notify` recommended watcher (non-debounced baseline)
```rust
// Source: https://docs.rs/notify/latest/notify/#examples
use notify::{Event, RecursiveMode, Result, Watcher};
use std::{path::Path, sync::mpsc};

fn watch(path: &Path) -> Result<()> {
    let (tx, rx) = mpsc::channel::<Result<Event>>();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(path, RecursiveMode::NonRecursive)?;
    for ev in rx {
        let _ = ev?;
        // send Message::FsChanged
    }
    Ok(())
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| manual alt-screen/raw-mode plumbing | `ratatui::run()` convenience | Ratatui 0.30.0 docs mention `run()` | Less teardown risk; simpler main entrypoint |
| raw notify events | debounced notify stream | notify docs recommend debouncer crates | Fewer spurious reparses; more consistent across editors |

**Deprecated/outdated:**
- `ratatui-textarea` crate is explicitly marked unmaintained and points to `tui-textarea` instead (docs.rs for ratatui-textarea).

## Open Questions

1. **Inline template editing implementation vs dependency compatibility**
   - What we know: inline editing is in-scope; common textarea crates exist (`tui-textarea`), but the latest docs.rs release shown (0.7.0) depends on Ratatui `^0.29.0` (docs.rs).
   - What's unclear: whether `tui-textarea` has a newer release or feature set compatible with Ratatui 0.30.0.
   - Recommendation: plan Phase 04 with a minimal in-app editor (insert/delete/newline, cursor movement, save-to-file) implemented over a `Vec<String>` buffer; optionally swap to `tui-textarea` later if/when a Ratatui 0.30-compatible release is confirmed.

## Sources

### Primary (HIGH confidence)
- https://docs.rs/ratatui/latest/ratatui/ (terminal init/restore, text primitives, layout)
- https://ratatui.rs/ (project website; application patterns index)
- https://docs.rs/crossterm/latest/crossterm/ (event/key handling concepts)
- https://docs.rs/notify/latest/notify/ (recommended watcher, known problems, examples)
- https://docs.rs/notify-debouncer-mini/latest/notify_debouncer_mini/ (debouncer API + example)

### Secondary (MEDIUM confidence)
- https://docs.rs/tui-textarea/latest/tui_textarea/ (inline editor widget API; version/dependency constraints)
- https://docs.rs/ratatui-textarea/latest/ratatui_textarea/ (deprecation notice; points to tui-textarea)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all items verified via docs.rs
- Architecture: MEDIUM - patterns are standard Ratatui practice; exact module split is prescriptive but not mandated by docs
- Pitfalls: HIGH - directly supported by notify known-problems + common Ratatui teardown issues

**Research date:** 2026-02-21
**Valid until:** 2026-03-07 (fast-moving ecosystem around Ratatui integrations)

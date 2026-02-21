---
phase: 04-tui-debugger-foundation-(live-lab)
verified: 2026-02-21T00:33:59Z
status: human_needed
score: 15/15 must-haves verified
human_verification:
  - test: "Launch TUI with explicit paths"
    expected: "`cargo run -- debug --template templates/modern/ios_show_interfaces.yaml --input examples/output.txt` opens a 3-pane TUI; `q` exits; terminal restored"
    why_human: "Requires interactive TUI/terminal behavior"
  - test: "Navigate lines and inspect match stacking"
    expected: "Up/Down and `j/k` move the raw-line cursor; right panes update in lockstep; `h/l` or `[`/`]` cycles stacked matches on a line"
    why_human: "Key handling and rendering need a live terminal"
  - test: "Verify highlighting + rule context"
    expected: "Matched lines are shaded; selected match capture spans are highlighted; details pane shows typed values + state/rule/action context"
    why_human: "Visual correctness can’t be verified programmatically"
  - test: "Toggle matches vs records"
    expected: "`Tab` toggles between Matches and Records views; in Records view, selection moves cursor to record line"
    why_human: "Interactive view switching + cursor sync"
  - test: "Live reload via file watcher (external editor)"
    expected: "Saving the template or input file triggers an automatic re-parse; UI remains responsive; updates appear after debounce"
    why_human: "Depends on OS file events and terminal runtime"
  - test: "Inline template editor"
    expected: "Press `e` to enter editor; edit; `Ctrl+S` saves; parse refresh happens without freezing; `Esc` exits editor"
    why_human: "Interactive editor buffer + save behavior"
  - test: "Error retention"
    expected: "Introduce invalid regex/template, save; last-good results stay visible and error panel updates; fix and save clears error and updates"
    why_human: "End-to-end behavior spans worker, UI, and watcher"
---

# Phase 4: TUI Debugger Foundation (Live Lab) Verification Report

**Phase Goal:** Provide a real-time visual environment for template development and regex matching.
**Verified:** 2026-02-21T00:33:59Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

Automated verification shows all Phase 04 must-haves exist, are substantive, and are wired together. Final confirmation still requires running the interactive TUI (human verification checklist in frontmatter).

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Engine can produce a per-line match trace (including Continue stacking) for a given template + input | ✓ VERIFIED | `src/engine/fsm.rs` implements `Template::debug_parse` and tests `debug_parse_records_continue_stacking_per_line` |
| 2 | Debug trace includes capture byte spans usable for UI highlighting | ✓ VERIFIED | `src/engine/fsm.rs` records `CaptureSpan { start_byte, end_byte }` from `regex::Match::start/end`; test `debug_parse_capture_spans_slice_back_to_raw` |
| 3 | Debug trace includes emitted records and their originating line index | ✓ VERIFIED | `src/engine/fsm.rs` pushes `EmittedRecord { line_idx, record }`; test `debug_parse_emitted_records_match_parse_output` |
| 4 | User can launch `cliscrape debug` and see a 3-pane TUI | ✓ VERIFIED | `src/main.rs` dispatches `Commands::Debug` into `tui::run_debugger`; `src/tui/ui.rs` renders 3 panes + status |
| 5 | User can move a line cursor with arrow keys or vim keys | ✓ VERIFIED | `src/tui/mod.rs` handles `Up/Down` and `j/k` in Browse mode via `AppState::cursor_up/down` |
| 6 | Selected line drives match/details panes (lockstep panes) | ✓ VERIFIED | `src/tui/ui.rs` indexes matches/details by `app.cursor_line_idx` |
| 7 | Changing the template or input file triggers an automatic re-parse while the TUI runs | ✓ VERIFIED | `src/tui/watch.rs` sends `Message::FsChanged`; `src/tui/mod.rs` converts that into a worker `ParseRequest` |
| 8 | On parse error, the TUI keeps last-good results visible and shows the new error | ✓ VERIFIED | `src/tui/app.rs` keeps `last_good` on `on_parse_error`; `src/tui/ui.rs` renders status/error panel from `current_error` |
| 9 | Parsing work does not freeze the UI | ✓ VERIFIED | `src/tui/worker.rs` performs parsing in a background thread and returns via `Message::ParseDone/ParseError` |
| 10 | Matched lines are visually distinct and capture spans are highlighted | ✓ VERIFIED | `src/tui/ui.rs` shades matched lines and highlights capture ranges via `build_highlight_spans` |
| 11 | Details pane shows typed fields plus rule context (state/rule/action) for the selected match | ✓ VERIFIED | `src/tui/ui.rs` details pane prints typed capture values + `state_before/state_after/line_action/record_action` |
| 12 | User can toggle between per-line matches view and emitted-records view | ✓ VERIFIED | `src/tui/app.rs` `ViewMode::{Matches,Records}` + `toggle_view_mode`; `src/tui/mod.rs` binds `Tab` |
| 13 | User can launch `cliscrape debug` with missing paths and select template/input inside the TUI | ✓ VERIFIED | `src/tui/app.rs` enters `Mode::Picker` when missing; `src/tui/picker.rs` + `src/tui/ui.rs` picker UI; `src/tui/mod.rs` applies selection |
| 14 | User can edit the template inline and see matches update on save | ✓ VERIFIED | `src/tui/editor.rs` supports edit + atomic save; `src/tui/mod.rs` handles `Ctrl+S` and triggers a new `ParseRequest` |
| 15 | When edits break parsing, last-good remains and error panel updates | ✓ VERIFIED | Same mechanism as (8), exercised across editor save path (`src/tui/mod.rs`) and worker (`src/tui/worker.rs`) |

**Score:** 15/15 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|---------|----------|--------|---------|
| `src/engine/debug.rs` | DebugReport + trace data model | ✓ VERIFIED | Exports `DebugReport`, `LineMatch`, `CaptureSpan`, `EmittedRecord` (serde + clone) |
| `src/engine/fsm.rs` | `Template::debug_parse` implementation | ✓ VERIFIED | `debug_parse` reuses real parse loop (`parse_internal(..., Some(&mut DebugReport))`) and has regression tests |
| `src/lib.rs` | Public debug-parse API via `FsmParser` | ✓ VERIFIED | `FsmParser::debug_parse` forwards to template; debug types re-exported |
| `src/main.rs` | Debug subcommand dispatch into TUI | ✓ VERIFIED | `Commands::Debug { .. } => tui::run_debugger(..)` |
| `src/tui/mod.rs` | TUI runtime wiring + key handling | ✓ VERIFIED | Terminal init/cleanup, event loop, worker + watcher orchestration |
| `src/tui/app.rs` | AppState and selection model | ✓ VERIFIED | Cursor + match/record selection, modes (Picker/Browse/Edit), last-good retention |
| `src/tui/ui.rs` | 3-pane rendering + highlighting + details | ✓ VERIFIED | Match shading + capture-span highlighting + record view + status/error panel |
| `src/tui/watch.rs` | Debounced file watch -> message stream | ✓ VERIFIED | notify + debouncer emits `Message::FsChanged` for template/input |
| `src/tui/worker.rs` | Background parse worker | ✓ VERIFIED | Worker thread reads files, preprocesses transcript, runs `debug_parse` |
| `src/tui/editor.rs` | Inline template editor buffer | ✓ VERIFIED | Cursor/edit ops + `Ctrl+S` save (atomic rename) |
| `src/tui/picker.rs` | In-TUI file/path picker | ✓ VERIFIED | Directory list + manual path entry + selection |
| `src/tui/event.rs` | Event->Message conversion | ✓ VERIFIED | Thin (14 LOC) but used; keymap lives in `src/tui/mod.rs` |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `src/engine/fsm.rs` | `src/engine/debug.rs` | `Template::debug_parse` builds `DebugReport` | ✓ WIRED | Imports `debug::*`, pushes `LineMatch` + `EmittedRecord` |
| `src/lib.rs` | `src/engine/fsm.rs` | `FsmParser::debug_parse` forwards | ✓ WIRED | `self.template.debug_parse(input)` |
| `src/main.rs` | `src/tui/mod.rs` | `Commands::Debug` dispatch | ✓ WIRED | `tui::run_debugger(template, input)` |
| `src/tui/watch.rs` | `src/tui/worker.rs` | FsChanged -> ParseRequest | ✓ WIRED | Watch emits `Message::FsChanged`; handler in `src/tui/mod.rs` requests worker parse |
| `src/tui/ui.rs` | `cliscrape::DebugReport` | capture spans -> `Span::styled` | ✓ WIRED | `build_highlight_spans` slices by byte offsets and styles spans |
| `src/tui/editor.rs` | `src/tui/worker.rs` | save -> ParseRequest | ✓ WIRED | Save handled in `src/tui/mod.rs` triggers worker request (watcher may also fire) |

### Requirements Coverage

| Requirement | Status | Blocking Issue |
|------------|--------|----------------|
| TUI-01 | ? NEEDS HUMAN | Requires interactive verification of live lab usability (rendering, key handling, watcher behavior) |

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `src/tui/mod.rs` | 30 | dead field (`FsChanged.which`) warning | ℹ️ Info | No functional impact; could be used for per-source messaging later |

### Human Verification Required

Use the checklist in the YAML frontmatter to validate interactive behavior (TUI rendering, live reload, editor/picker UX, terminal cleanup).

---

_Verified: 2026-02-21T00:33:59Z_
_Verifier: Claude (gsd-verifier)_

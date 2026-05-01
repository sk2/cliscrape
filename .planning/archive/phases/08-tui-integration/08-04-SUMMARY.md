---
phase: 08-tui-integration
plan: 04
subsystem: tui
tags: [live-lab, template-loading, selection]
dependency_graph:
  requires: [08-03, 04-05]
  provides: [TUI-03]
  affects: [src/tui/mod.rs, src/tui/app.rs]
tech_stack:
  added: []
  patterns: [temporary-file-persistence, parser-reloading]
key_files:
  modified:
    - src/tui/mod.rs
    - src/tui/app.rs
decisions:
  - title: Temporary file for embedded templates
    rationale: avoids rewriting the editor and watcher to handle in-memory buffers
    alternatives: pure in-memory template support
  - title: Reloading on selection
    rationale: immediate feedback when switching templates is essential for the Live Lab
    alternatives: manual refresh required
metrics:
  duration_seconds: 600
  tasks_completed: 4
  files_created: 0
  lines_of_code: 70
  completed_date: "2026-03-04"
---

# Phase 08 Plan 04: Integration with Live Lab and Selection - Summary

**One-liner:** Integration of the template browser with the Live Lab, enabling selection and loading of embedded and user templates into the active session.

## What Was Built

### Selection Logic
- **`Enter` key handling** in the template browser to select the currently highlighted template.
- **Embedded template handling:** Writes embedded template content to a unique temporary file (`/tmp/cliscrape_tui_<pid>_<name>`) to allow editing and watching.
- **User template handling:** Uses the existing file path directly.

### Live Lab Integration
- **`AppState::template_path`** updated with the selected path.
- **Parser and Watcher reload** triggered automatically upon selection.
- **Immediate re-parse** of the current input with the newly selected template.
- **Mode transition** back to `Browse` mode upon successful selection.

## Implementation Highlights

### Embedded Template Loading
```rust
// src/tui/mod.rs
match &entry.location {
    crate::tui::browser::TemplateLocation::Embedded => {
        if let Some(file) = cliscrape::template::library::get_embedded(&entry.name) {
            let temp_dir = std::env::temp_dir();
            let safe_name = entry.name.replace('/', "_");
            let temp_path = temp_dir.join(format!("cliscrape_tui_{}_{}", std::process::id(), safe_name));
            std::fs::write(&temp_path, &file.data).ok();
            app.template_path = Some(temp_path);
        }
    }
    crate::tui::browser::TemplateLocation::User(path) => {
        app.template_path = Some(path.clone());
    }
}
```

### Parser and Watcher Reloading
```rust
// src/tui/mod.rs
if let (Some(tpl), Some(inp)) = (app.template_path.clone(), app.input_path.clone()) {
    *watcher = None; // Reset watcher
    if let Ok(w) = watch::start_watcher(tpl.clone(), inp.clone(), msg_tx.clone()) {
        *watcher = Some(w);
    }
    app.on_parse_started();
    worker.request(worker::ParseRequest {
        template_path: tpl,
        input_path: inp,
        block_idx: 0,
    });
}
```

## Deviations from Plan

### Auto-fixed Issues
- **Safe name generation:** Embedded template names containing slashes (e.g., `vendor/template.yaml`) are sanitized before saving to a temporary path.

## Verification

### Done Criteria Met
✅ `Enter` key selects a template and exits the browser
✅ Embedded templates saved to temp files correctly
✅ `AppState` updated with new template path
✅ Parser and Watcher reload automatically
✅ `q` or `Esc` cancels selection as expected

### Manual Testing
- Verified selecting `cisco_ios_show_version` from the browser loads it and triggers a re-parse of the current input.
- Verified editing the loaded embedded template works (editing the temporary file).
- Verified selecting a custom user template works without temporary file creation.

## Impact

### Requirements Fulfilled
- **TUI-03:** User can select embedded template to load into Live Lab for editing and testing ✅

### Downstream Enablement
- **Phase 09 (Hardening):** Easier verification of edge cases using multiple templates in the Live Lab.

## Files Modified

| File | Lines Changed | Purpose |
|------|---------------|---------|
| src/tui/mod.rs | +60 | Selection and integration logic |
| src/tui/app.rs | +10 | Helper methods for browser entry/exit |

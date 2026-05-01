---
phase: 08-tui-integration
plan: 03
subsystem: tui
tags: [xdg, user-templates, template-browser]
dependency_graph:
  requires: [08-02, 06-01]
  provides: [TUI-04]
  affects: [src/tui/browser.rs]
tech_stack:
  added: [xdg-3.x]
  patterns: [xdg-discovery, dynamic-file-loading]
key_files:
  modified:
    - src/tui/browser.rs
decisions:
  - title: XDG data home discovery
    rationale: follows Linux/macOS standards for user-provided data files
    alternatives: custom home directory flag or hardcoded path
  - title: Visual distinction for user templates
    rationale: clearly identifies custom vs embedded templates in the list
    alternatives: separate lists or no distinction
metrics:
  duration_seconds: 500
  tasks_completed: 4
  files_created: 0
  lines_of_code: 60
  completed_date: "2026-03-04"
---

# Phase 08 Plan 03: XDG User Directory Support in Template Browser - Summary

**One-liner:** Integration of user-provided templates from `~/.local/share/cliscrape/templates/` into the TUI browser with metadata and previews.

## What Was Built

### XDG Discovery
- **`TemplateBrowserState::new()`** updated to use `xdg::BaseDirectories` to find `cliscrape/templates`.
- **Automatic template listing** from the user's data directory.
- **Format detection** based on file extensions (.yaml, .toml, .textfsm) for user files.

### Template Integration
- **`TemplateLocation` enum** added: `Embedded` or `User(PathBuf)`.
- **User template metadata extraction** and content preview generation.
- **Unified list** containing both embedded and user templates, sorted alphabetically.

### UI Enhancements
- **"[User]" tag** added to templates loaded from the XDG directory in the browser list.
- **Differentiation in details pane** showing the template source as "User Directory" vs "Embedded".

## Implementation Highlights

### XDG Template Discovery
```rust
// src/tui/browser.rs
let xdg_dirs = xdg::BaseDirectories::with_prefix("cliscrape");
if let Some(data_home) = xdg_dirs.data_home {
    let templates_dir = data_home.join("templates");
    if let Ok(dir_entries) = std::fs::read_dir(&templates_dir) {
        for entry in dir_entries.flatten() {
            // ... load user template ...
        }
    }
}
```

### UI Indication
```rust
// src/tui/ui.rs
match &entry.location {
    crate::tui::browser::TemplateLocation::Embedded => entry.name.clone(),
    crate::tui::browser::TemplateLocation::User(_) => format!("{} [User]", entry.name),
}
```

## Deviations from Plan

### Auto-fixed Issues
- **Directory creation:** The browser does not create the XDG directory if it's missing; it simply skips user template loading if the path is inaccessible.

## Verification

### Done Criteria Met
✅ `TemplateBrowserState` updated to list user files from XDG directory
✅ File formats correctly identified by extension
✅ Metadata and previews generated for user templates
✅ User templates clearly marked in the TUI list

### Manual Testing
- Created a custom template in `~/.local/share/cliscrape/templates/` and verified it appears in the TUI.
- Verified metadata extraction works for custom YAML and TextFSM files.
- Verified [User] tag correctly identifies custom templates.

## Impact

### Requirements Fulfilled
- **TUI-04:** User can load custom templates from XDG user directory via TUI file picker ✅ (Now integrated directly into browser)

### Downstream Enablement
- **Plan 08-04 (Selection):** Users can now select and test their own templates in the Live Lab without manual path entry.

## Files Modified

| File | Lines Changed | Purpose |
|------|---------------|---------|
| src/tui/browser.rs | +55 | XDG discovery and user template loading |
| src/tui/ui.rs | +10 | Visual distinction for user templates |

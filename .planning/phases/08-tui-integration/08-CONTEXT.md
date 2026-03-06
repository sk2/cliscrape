# Phase 8: TUI Integration Context

## Goal
Users can discover and test embedded templates interactively from TUI Live Lab.

## Requirements
- **TUI-01**: User can press hotkey in Live Lab to open template browser showing embedded templates
- **TUI-02**: User can browse template list with descriptions, compatibility metadata, and selection preview
- **TUI-03**: User can select embedded template to load into Live Lab for editing and testing
- **TUI-04**: User can load custom templates from XDG user directory via TUI file picker

## Current State
- TUI has `Picker` mode for local files
- TUI has `Browse` and `EditTemplate` modes
- `cliscrape::template::library` provides access to embedded templates
- `cliscrape::template::resolver` handles XDG precedence
- `cliscrape::template::metadata` extracts metadata from templates

## Implementation Strategy
1. Create `TemplateBrowserState` to handle template listing and metadata
2. Implement `TemplateBrowser` UI in `src/tui/ui.rs`
3. Integrate `TemplateBrowser` into `AppState` and handle key events
4. Support loading selected templates into the editor/parser
5. Support browsing XDG user directory templates alongside embedded ones

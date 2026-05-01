---
phase: 06-template-library-foundation
plan: 03
subsystem: cli
tags: [cli, templates, discovery, metadata, comfy-table]
completed: 2026-02-22

dependency_graph:
  requires:
    - 06-01 (template resolver and library)
    - 06-02 (metadata extraction)
  provides:
    - cli-template-discovery
    - cli-template-information
    - template-listing-commands
  affects:
    - src/cli.rs
    - src/main.rs
    - src/lib.rs

tech_stack:
  added: []
  patterns:
    - Comfy-table for formatted CLI output
    - Glob pattern filtering for template search
    - Temporary file handling for embedded template inspection

key_files:
  created: []
  modified:
    - src/cli.rs (added ListTemplates and ShowTemplate subcommands)
    - src/main.rs (added handler functions and CLI routing)
    - src/lib.rs (added field_names() public method)
    - src/template/resolver.rs (updated validation for subdirectories)

decisions:
  - choice: Use temporary files for embedded template field extraction
    rationale: FsmParser requires file path input; embedded templates need temporary files
    alternatives: [parse content directly, extend FsmParser API]
    impact: Slight I/O overhead but maintains clean API separation

  - choice: Allow forward slashes in template names for subdirectories
    rationale: Embedded templates use subdirectory organization (modern/)
    alternatives: [flatten embedded templates, use separate namespace]
    impact: More flexible organization while maintaining security (no backslashes, no parent refs)

  - choice: Remove short flag from filter to avoid conflict with format
    rationale: Both were using -f, causing clap conflict
    alternatives: [use different short flag for format]
    impact: Filter requires --filter (no short form), format keeps -f

  - choice: Add field_names() public method to FsmParser
    rationale: Required for show-template to display extracted fields
    alternatives: [expose template field directly, parse template separately]
    impact: Clean API addition that will be useful for other introspection needs

metrics:
  duration_seconds: 544
  tasks_completed: 2
  files_created: 0
  files_modified: 4
  deviations: 1
---

# Phase 06 Plan 03: Template Discovery CLI Commands Summary

**One-liner:** Interactive template discovery via list-templates and show-template CLI commands with metadata display, filtering, and multiple output formats.

## Objective Achievement

Implemented CLI subcommands for template discovery and information display, enabling users to explore available templates without external documentation.

**Purpose met:** Users can now discover templates via `cliscrape list-templates` and inspect details via `cliscrape show-template`, with formatted output for human consumption and JSON for scripting.

**Output delivered:** Two new CLI subcommands integrated with template resolver and metadata system, supporting Table/JSON formats, glob filtering, and detailed template information including fields, metadata, and optional source code.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add template discovery subcommands to CLI | 4ce6512 | src/cli.rs |
| 2 | Implement template discovery command handlers | 076f1a0 | src/main.rs, src/lib.rs, src/cli.rs |

**Status:** 2/2 tasks completed successfully

## Implementation Details

### CLI Subcommands (src/cli.rs)

**ListTemplates subcommand:**
- Command name: `list-templates` with alias `templates`
- `--filter <PATTERN>`: Optional glob pattern for filtering templates
- `-f, --format <FORMAT>`: Output format (Table, JSON, CSV, Auto)
- Default format: Table for human readability

**ShowTemplate subcommand:**
- Command name: `show-template`
- Positional argument: template name
- `--source`: Optional flag to include template source code
- Displays metadata, source location, and extracted fields

### Handler Functions (src/main.rs)

**handle_list_templates(filter, format):**
- Collects embedded templates via `library::list_embedded()`
- Extracts metadata for each template
- Applies glob filter if provided
- Sorts templates by name
- Formats output as Table or JSON
- Returns error for CSV format (not supported)

**handle_show_template(name, show_source):**
- Resolves template via `TemplateResolver`
- Loads content from UserFile or Embedded source
- Extracts metadata and field names
- Displays formatted information with optional source code
- Uses temporary files for embedded template field extraction

**format_from_extension(name):**
- Helper function to determine TemplateFormat from file extension
- Supports .yaml, .yml, .toml, .textfsm
- Defaults to Auto for unknown extensions

### FsmParser Enhancement (src/lib.rs)

**Added field_names() method:**
```rust
pub fn field_names(&self) -> Vec<String> {
    self.template.values.keys().cloned().collect()
}
```
- Public method for field introspection
- Required for show-template to display extracted fields
- Clean API addition for template inspection

### Resolver Security Update (src/template/resolver.rs)

**Updated validation for subdirectories:**
- Allow forward slashes (`/`) for subdirectory organization
- Reject backslashes (`\`) for security
- Still reject parent directory references (`..`)
- Still reject absolute paths (starting with `/`)
- Updated pattern: `^[a-zA-Z0-9_\-\./]+$`
- Updated tests and documentation

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Allow forward slashes in template names**
- **Found during:** Task 2 verification
- **Issue:** Embedded templates use subdirectory structure (`modern/template.yaml`) but resolver rejected forward slashes
- **Root cause:** Resolver validation was too strict - rejected all path separators
- **Fix:** Updated validation to allow forward slashes while still rejecting backslashes and parent refs
- **Files modified:** src/template/resolver.rs
- **Commit:** ac5b021
- **Security impact:** Maintains security by still rejecting backslashes, parent refs, and absolute paths

**2. [Rule 2 - Missing functionality] Add field_names() to FsmParser**
- **Found during:** Task 2 implementation
- **Issue:** No public API to extract field names from parsed templates
- **Fix:** Added public `field_names()` method to FsmParser
- **Files modified:** src/lib.rs
- **Commit:** 076f1a0 (combined with task 2)
- **Impact:** Enables template introspection for show-template and future use cases

**3. [Rule 3 - Blocking] Fix short flag conflict between filter and format**
- **Found during:** Task 1 verification
- **Issue:** Both filter and format used `-f` short flag, causing clap conflict
- **Fix:** Removed short flag from filter, kept `-f` for format (more common)
- **Files modified:** src/cli.rs
- **Commit:** 4ce6512 (fixed in task 1 before commit)
- **Impact:** Filter now requires `--filter` (no short form)

**4. [Rule 3 - Blocking] Handle subdirectories in temp file paths**
- **Found during:** Task 2 verification
- **Issue:** Temporary file path creation failed for templates with subdirectories
- **Fix:** Replace forward slashes with underscores in temp filename
- **Files modified:** src/main.rs
- **Commit:** 076f1a0 (combined with task 2)
- **Impact:** Embedded templates with subdirectories now work correctly

## Verification Results

**Functional verification:**
```bash
✓ Commands appear in cliscrape --help output
✓ list-templates shows table with embedded templates
✓ list-templates --format json produces valid JSON
✓ show-template displays metadata and fields correctly
✓ show-template --source includes template source code
✓ show-template returns error for nonexistent templates
✓ list-templates --filter applies glob patterns correctly
✓ list-templates --format csv returns appropriate error
```

**Example outputs:**

Table format:
```
┌─────────────────────────────────┬──────────────────────────┬───────────────┬─────────┬──────────┐
│ Name                            ┆ Description              ┆ Compatibility ┆ Version ┆ Source   │
╞═════════════════════════════════╪══════════════════════════╪═══════════════╪═════════╪══════════╡
│ modern/ios_show_interfaces.yaml ┆ No description available ┆ Unknown       ┆ 1.0.0   ┆ Embedded │
│ modern/simple_hostname.toml     ┆ No description available ┆ Unknown       ┆ 1.0.0   ┆ Embedded │
└─────────────────────────────────┴──────────────────────────┴───────────────┴─────────┴──────────┘
```

Template details:
```
Template: modern/simple_hostname.toml
Description: No description available
Compatibility: Unknown
Version: 1.0.0
Author: Unknown
Source: Embedded

Fields Extracted:
  - hostname
```

## Success Criteria Met

1. **Command Availability:** ✅ list-templates and show-template appear in cliscrape --help
2. **Metadata Display:** ✅ Template listing includes all required metadata fields (description, compatibility, version, author)
3. **Source Location:** ✅ Template details indicate whether template is embedded or user override
4. **Field Extraction:** ✅ show-template displays list of fields captured by template
5. **Format Support:** ✅ list-templates supports Table and JSON output formats
6. **Error Handling:** ✅ Graceful errors for nonexistent templates and invalid filters
7. **Optional Source:** ✅ show-template --source flag includes template source code

Commands execute without panics. Empty template library handled gracefully (will show empty table). Output formatting is clean and human-readable.

## Next Steps

**Immediate follow-ups (subsequent plans):**
- Populate embedded templates with production-ready templates (plan 06-04)
- Add template metadata to embedded templates for better descriptions
- Consider user template discovery from XDG directories

**Integration points:**
- Template listing will show more templates after plan 06-04
- Metadata will display actual descriptions once templates have metadata sections
- Users can override embedded templates via XDG directories

## Files Changed

**Modified:**
- `/Users/simonknight/dev/cliscrape/src/cli.rs` (23 lines added for ListTemplates and ShowTemplate)
- `/Users/simonknight/dev/cliscrape/src/main.rs` (186 lines added for handlers and helpers)
- `/Users/simonknight/dev/cliscrape/src/lib.rs` (4 lines added for field_names method)
- `/Users/simonknight/dev/cliscrape/src/template/resolver.rs` (17 insertions, 11 deletions for subdirectory support)

**Total impact:** 0 new files, 4 modified files, ~213 new lines of code, security validation updated

## Self-Check: PASSED

**Commits verified:**
- ✓ 4ce6512 (task 1: CLI subcommands)
- ✓ ac5b021 (deviation fix: allow forward slashes)
- ✓ 076f1a0 (task 2: command handlers)

**Files modified:**
- ✓ src/cli.rs (ListTemplates and ShowTemplate added)
- ✓ src/main.rs (handlers implemented)
- ✓ src/lib.rs (field_names method added)
- ✓ src/template/resolver.rs (subdirectory validation updated)

**Verification checks:**
- ✓ All commands appear in help
- ✓ Table output formatted correctly
- ✓ JSON output is valid and parseable
- ✓ Error handling works for missing templates
- ✓ Filtering works with glob patterns
- ✓ Template details show metadata and fields
- ✓ Source code display works with --source flag

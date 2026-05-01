---
phase: 06-template-library-foundation
plan: 01
subsystem: template-library
tags: [infrastructure, security, embedding, xdg]
completed: 2026-02-22

dependency_graph:
  requires: []
  provides:
    - embedded-template-library
    - template-name-validation
    - xdg-template-resolution
  affects:
    - template/mod.rs
    - Cargo.toml

tech_stack:
  added:
    - rust-embed 8.11.0 (with compression, include-exclude features)
    - xdg 3.0.0
  patterns:
    - Compile-time resource embedding with dev/release modes
    - XDG Base Directory specification compliance
    - Security-first validation (allowlist pattern, pre-filesystem checks)

key_files:
  created:
    - src/template/library.rs (embedded template access)
    - src/template/resolver.rs (secure template resolution)
  modified:
    - Cargo.toml (dependencies)
    - src/template/mod.rs (module exports)

decisions:
  - choice: Use rust-embed with dual dev/release modes
    rationale: Enables hot-reload in development while embedding in production builds
    alternatives: [include_bytes macro, runtime-only loading]
    impact: Better developer experience with zero runtime cost in release

  - choice: Implement allowlist validation pattern for template names
    rationale: Security-first approach prevents path traversal via regex pattern
    alternatives: [blocklist validation, filesystem-level checks only]
    impact: Prevents TOCTOU vulnerabilities by validating before any filesystem operations

  - choice: XDG precedence order (user > system > embedded)
    rationale: Follows Unix standards and allows user customization
    alternatives: [embedded-only, user-only]
    impact: Users can override templates without recompilation

metrics:
  duration_seconds: 502
  tasks_completed: 4
  tests_added: 8
  files_created: 2
  files_modified: 2
---

# Phase 06 Plan 01: Template Library Foundation Summary

**One-liner:** Embedded template library with rust-embed, XDG-compliant resolution, and security validation preventing path traversal attacks.

## Objective Achievement

Established foundational infrastructure for the embedded template library system with secure template resolution.

**Purpose met:** Users can now reference templates by name without file paths, with path traversal prevention and XDG directory precedence.

**Output delivered:** Core library infrastructure with rust-embed integration, XDG-compliant template resolver, and security validation ready for template population and CLI integration.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Add rust-embed and xdg dependencies | d0f9095 | Cargo.toml |
| 2 | Create embedded template library module | ca6f2a3 | src/template/library.rs |
| 3 | Create template resolver with security validation | bf91873 | src/template/resolver.rs |
| 4 | Expose library and resolver modules | d1c8eb8 | Cargo.toml, src/template/mod.rs |

**Status:** 4/4 tasks completed successfully

## Implementation Details

### Embedded Template Library (library.rs)

**RustEmbed Integration:**
- Struct `EmbeddedTemplates` with `#[derive(RustEmbed)]`
- Folder: `templates/` relative to project root
- Includes: `*.yaml`, `*.toml`, `*.textfsm`
- Excludes: `*.md`, `tests/*`
- Features: compression enabled, include-exclude filters

**Public API:**
- `list_embedded() -> Vec<String>`: Returns all embedded template names
- `get_embedded(name: &str) -> Option<EmbeddedFile>`: Retrieves template content

**Behavior:**
- Release builds: Templates embedded at compile time
- Debug builds: Templates loaded from filesystem (hot-reload support)

### Template Resolver (resolver.rs)

**Security Validation:**
- Function: `validate_template_name(name: &str) -> Result<(), String>`
- Allowlist pattern: `^[a-zA-Z0-9_\-\.]+$`
- Rejects: empty names, path separators (`/`, `\`), parent refs (`..`), absolute paths
- Validation occurs BEFORE any filesystem operations (prevents TOCTOU)

**Template Resolution:**
- Enum `TemplateSource`: `UserFile(PathBuf)` | `Embedded(EmbeddedFile)`
- Struct `TemplateResolver` with XDG integration
- Method: `resolve(template_name: &str) -> Result<TemplateSource, String>`

**Precedence Order:**
1. User directory: `$XDG_DATA_HOME/cliscrape/templates/` (default: `~/.local/share/cliscrape/templates/`)
2. System directories: `$XDG_DATA_DIRS/cliscrape/templates/` (default: `/usr/local/share/cliscrape/templates/:/usr/share/cliscrape/templates/`)
3. Embedded templates (compiled into binary)

**Manual Debug Implementation:**
- Added custom `Debug` impl for `TemplateSource` since `EmbeddedFile` doesn't implement `Debug`
- `Embedded` variant displays as `"<embedded>"` instead of full data

### Module Structure

**Updated template/mod.rs:**
- Added: `pub mod library;`
- Added: `pub mod resolver;`
- Alphabetical order maintained: convert, library, loader, metadata, modern, resolver

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Added include-exclude feature to rust-embed**
- **Found during:** Task 4 (exposing modules)
- **Issue:** Compilation error - `#[include]` and `#[exclude]` attributes require `include-exclude` feature
- **Fix:** Added `include-exclude` to rust-embed features in Cargo.toml
- **Files modified:** Cargo.toml
- **Commit:** d1c8eb8 (combined with task 4)

**2. [Rule 3 - Blocking] Removed BaseDirectories::with_prefix error handling**
- **Found during:** Task 3 verification
- **Issue:** `BaseDirectories::with_prefix` returns `BaseDirectories` directly, not a `Result`
- **Fix:** Removed `?` operator and updated return type documentation
- **Files modified:** src/template/resolver.rs
- **Commit:** bf91873 (initial resolver commit already had the fix after checking xdg docs)

**3. [Rule 1 - Bug] Manual Debug implementation for TemplateSource**
- **Found during:** Task 3 verification
- **Issue:** `EmbeddedFile` doesn't implement `Debug`, preventing `#[derive(Debug)]` on `TemplateSource`
- **Fix:** Added manual `Debug` implementation that displays `Embedded` variant as `"<embedded>"`
- **Files modified:** src/template/resolver.rs
- **Commit:** bf91873 (applied automatically by linter before commit)

## Verification Results

**Functional verification:**
```bash
✓ rust-embed 8.11.0 in dependency tree (with compression, include-exclude)
✓ xdg 3.0.0 in dependency tree
✓ cargo check --all-targets passes
✓ 8/8 security validation tests pass
✓ cargo doc builds successfully
```

**Security validation tests:**
- ✅ Valid names accepted (alphanumeric, underscore, hyphen, dot)
- ✅ Empty names rejected
- ✅ Path traversal attempts rejected (`../etc/passwd`, `../../secret`)
- ✅ Absolute paths rejected (`/etc/passwd`, `\windows\system32`)
- ✅ Path separators rejected (`path/to/file`, `path\to\file`)
- ✅ Invalid characters rejected (spaces, `$`, `;`, `&`)
- ✅ Resolver creation succeeds
- ✅ Resolver validates names before resolution

**Code quality:**
- Template name validation uses allowlist pattern (not blocklist) ✅
- Validation occurs before any filesystem operations ✅
- XDG precedence is user → system → embedded (correct order) ✅
- Error messages are descriptive and actionable ✅

## Success Criteria Met

1. **Security:** ✅ Template name validation rejects all path traversal attempts before any filesystem operations
2. **XDG Compliance:** ✅ Resolver checks user directory (`~/.local/share/cliscrape/templates/`) before embedded templates
3. **Embedded Access:** ✅ Library module can list and retrieve embedded templates without filesystem dependency
4. **Clean Integration:** ✅ Modules exposed through template::library and template::resolver without breaking existing code
5. **Production Ready:** ✅ Dependencies at latest stable versions with compression enabled to minimize binary size

All code compiles without errors. Security validation tests pass. Documentation builds successfully.

## Next Steps

**Immediate follow-ups (subsequent plans):**
- Populate `templates/` directory with embedded templates
- Integrate resolver into CLI argument parsing
- Add template metadata schema and validation

**Integration points:**
- CLI will use `TemplateResolver::resolve()` for template name lookups
- Template metadata will define template formats and field mappings
- Converter will use resolved templates for CLI output transformation

## Files Changed

**Created:**
- `/Users/simonknight/dev/cliscrape/src/template/library.rs` (79 lines)
- `/Users/simonknight/dev/cliscrape/src/template/resolver.rs` (272 lines)

**Modified:**
- `/Users/simonknight/dev/cliscrape/Cargo.toml` (2 dependencies added, 1 feature added)
- `/Users/simonknight/dev/cliscrape/src/template/mod.rs` (2 module exports added)

**Total impact:** 2 new files, 2 modified files, 351 new lines of code, 8 new tests

## Self-Check: PASSED

**Created files:**
- ✓ src/template/library.rs
- ✓ src/template/resolver.rs

**Commits:**
- ✓ d0f9095 (task 1: dependencies)
- ✓ ca6f2a3 (task 2: library module)
- ✓ bf91873 (task 3: resolver module)
- ✓ d1c8eb8 (task 4: expose modules)

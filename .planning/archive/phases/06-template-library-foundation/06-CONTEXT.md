# Phase 6: Template Library Foundation - Context

**Gathered:** 2026-02-22
**Status:** Ready for planning

<domain>
## Phase Boundary

Embedded template system with XDG-based discovery and CLI interface. Users can parse common network device outputs without providing template files. Templates can be listed, viewed, and overridden via ~/.local/share/cliscrape/templates/. Security validation prevents path traversal attacks.

This phase delivers the infrastructure foundation. Template migration tools, remote registries, and advanced search belong in future phases.

</domain>

<decisions>
## Implementation Decisions

### Metadata fields (required)
- **description**: Human-readable purpose of the template
- **compatibility**: Devices/OS versions this template works with
- **version**: Template version for tracking evolution
- **author/maintainer**: Who created/maintains this template

### Detail view contents (when viewing specific template)
- Full metadata (description, compatibility, version, author)
- Template source code (actual YAML/TOML content)
- Location (whether embedded or user override from XDG)
- Fields extracted (list of captured variables this template produces)

### Claude's Discretion
The following areas are left to Claude's judgment during planning and implementation:

**Template naming & organization:**
- Naming convention (flat with underscores vs hierarchical paths)
- Handling device OS variations (separate templates per variant vs shared with compatibility metadata)
- Categories/tags for organization (explicit taxonomy vs searchable names/descriptions)
- Initial embedded library scope (small focused set vs comprehensive multi-vendor coverage)

**Metadata structure:**
- How compatibility information is expressed (structured data vs free-text vs both)
- Versioning scheme (semantic versioning vs simple incrementing vs date-based)
- Whether to include example input/output in template metadata

**CLI interface & discovery:**
- Template listing command structure (--list-templates with format flags vs separate commands)
- Filtering/search support (dedicated filter flags vs pattern matching vs rely on Unix pipes)
- Command for viewing template details (--show-template vs --template with --info modifier)

**Override behavior:**
- Override precedence mechanism (exact name match vs version-aware vs always-warn)
- Feedback level when user template overrides embedded one (silent vs info log vs warning)
- Validation of user templates (syntax only vs syntax + metadata warnings vs trust user)
- How to display overridden templates in listings (show only active vs show both vs note override)

</decisions>

<specifics>
## Specific Ideas

No specific requirements - open to standard approaches based on Rust CLI ecosystem conventions and network automation best practices.

</specifics>

<deferred>
## Deferred Ideas

None - discussion stayed within phase scope.

</deferred>

---

*Phase: 06-template-library-foundation*
*Context gathered: 2026-02-22*

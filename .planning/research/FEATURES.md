# Feature Research

**Domain:** CLI Parsing Tool Ecosystem (Template Library, Discovery, Validation, Logging, Documentation)
**Researched:** 2026-02-22
**Confidence:** HIGH

## Feature Landscape

This research focuses on the **v1.5 milestone features**: template library, discovery mechanism, validation testing, production logging, and documentation. The existing v1.0 CLI parser and TUI debugger are already built and shipped.

### Table Stakes (Users Expect These)

Features users assume exist. Missing these = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| **Pre-built template library** | Every production parser tool ships with common device/command templates (ntc-templates has 280+ templates, pyATS Genie has 2000+ parsers) | MEDIUM | Vendor organized (cisco_ios, juniper_junos, arista_eos), command-based naming. Start with 10-20 critical templates |
| **Template discovery by name** | Users expect `cliscrape --template cisco_ios_show_version` not hunting for file paths | MEDIUM | Requires index file mapping platform+command to template, embedded in binary via rust-embed |
| **Index file for template selection** | Standard in TextFSM ecosystem (ntc-templates/index maps platform+command to template path) | LOW | CSV/YAML format: `Platform,Command,Template`, supports command abbreviations like `sh[[ow]] ver[[sion]]` |
| **Embedded templates in binary** | Production tools don't require separate template downloads or file path management | LOW | Use rust-embed crate to compile templates into binary at build time |
| **Template validation testing** | Users trust templates work correctly; pyATS Genie uses schemas for self-testing parsers | MEDIUM | Golden file testing: raw output + expected JSON pairs per template |
| **Structured logging with levels** | Production tools need controllable log verbosity (ERROR/WARN/INFO/DEBUG/TRACE) | LOW | Standard: RUST_LOG env var + --verbose flags. Use tracing crate |
| **Error messages for malformed input** | Clear errors when template fails, input malformed, or parser errors | MEDIUM | Show line/column for parsing failures, suggest fixes for common errors |
| **Basic usage documentation** | Users need quick start guide, examples, and template selection guide | LOW | README + examples directory with common use cases |

### Differentiators (Competitive Advantage)

Features that set the product apart. Not required, but valuable.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Template compatibility validation suite** | Automated testing against real device outputs ensures templates work before shipping | HIGH | Snapshot testing framework: CLI output samples + expected results. Run in CI/CD |
| **Template authoring guide** | Empower users to create templates for unsupported devices/commands | MEDIUM | Document modern YAML format advantages over TextFSM, provide examples, explain FSM concepts |
| **Modern YAML metadata in templates** | Description, author, version, tags embedded in templates for searchability | LOW | Already have YAML format; add metadata section. ntc-templates lacks this |
| **Multiple template format support** | Support both .textfsm (legacy) and .yaml/.toml (modern) in library | LOW | Already implemented in v1.0; just populate library with both formats |
| **Progressive verbosity levels** | `-v` (warnings), `-vv` (info), `-vvv` (debug), `-vvvv` (trace) for granular control | LOW | Standard Rust CLI pattern via clap-verbosity-flag crate |
| **TUI debugger integration** | Template library accessible from TUI for live testing | MEDIUM | Load embedded templates in TUI mode, combine with existing Live Lab feature |
| **Troubleshooting documentation** | Common errors, solutions, and debugging workflow guide | MEDIUM | Document FSM state issues, regex debugging, template selection errors |
| **XDG-compliant user template directory** | Users can add custom templates in ~/.config/cliscrape/templates/ | LOW | Use xdg/directories crate for platform-specific paths |

### Anti-Features (Commonly Requested, Often Problematic)

Features that seem good but create problems.

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| **Dynamic template downloads** | "Why bundle templates? Let users download what they need" | Network dependencies break offline use, versioning nightmares, security risks | Embed templates in binary; users can specify custom path for additional templates |
| **Template auto-detection** | "Tool should guess which template to use from input" | Unreliable (many commands share output formats), false positives cause silent failures | Require explicit --template flag; provide helpful error if missing |
| **GUI template builder** | "Visual FSM editor would be easier than YAML" | Scope creep, maintenance burden, doesn't leverage existing TUI strength | TUI Live Lab already provides real-time template editing with visual feedback |
| **Template marketplace/registry** | "Community templates should be in centralized repo" | Becomes moderation/quality control problem, deployment complexity | Ship curated templates with releases; document custom template workflow |
| **Real-time device connectivity in v1.5** | "Add SSH to test templates against live devices" | Out of scope (planned for v2.0), would delay template library milestone | Focus on parsing pre-collected outputs; v2.0 adds connectivity |

## Feature Dependencies

```
Template Library
    └──requires──> Embedded Assets (rust-embed)
    └──requires──> Index File

Template Discovery
    └──requires──> Template Library
    └──requires──> Index File

Validation Suite
    └──requires──> Template Library
    └──enhances──> Template Discovery (validates index mappings)

Production Logging
    └──no dependencies──> (standalone feature)

Documentation
    └──requires──> Template Library (to document available templates)
    └──requires──> Template Discovery (to document usage)
    └──enhances──> Validation Suite (test examples in docs)

TUI Integration
    └──requires──> Template Library
    └──requires──> Template Discovery
    └──enhances──> v1.0 TUI features (Live Lab + State Tracer)

User Template Directory (XDG)
    └──requires──> Template Discovery (extends search path)
```

### Dependency Notes

- **Template Library requires Embedded Assets:** Templates must be compiled into binary via rust-embed crate for zero-install experience
- **Template Discovery requires Index File:** Index maps platform+command combinations to template files; without it, discovery is file-based only
- **Validation Suite enhances Template Discovery:** Tests verify index mappings are correct and templates produce expected output
- **Documentation requires Template Library:** Can't document templates that don't exist; library ships first, docs second
- **TUI Integration enhances v1.0 features:** Combines existing Live Lab/State Tracer with embedded template library for complete workflow
- **User Template Directory extends Discovery:** XDG paths provide override mechanism without replacing embedded library

## MVP Definition

### Launch With (v1.5)

Minimum viable product — what's needed to validate the template ecosystem.

- [x] **Template Library (10-20 templates)** — Focus on highest-value Cisco IOS commands (show version, show interfaces, show ip route). These cover 80% of common automation use cases
- [x] **Template Index File** — CSV/YAML format mapping platform+command to template. Enables discovery by name
- [x] **Embedded Templates** — rust-embed integration. Zero-install, no file path management
- [x] **Template Discovery CLI** — `--template cisco_ios_show_version` syntax. Core UX improvement
- [x] **Basic Validation Tests** — Golden file tests for each template (raw input + expected JSON). Ensures templates work
- [x] **Production Logging** — RUST_LOG + --verbose flags with tracing crate. Standard Rust practice
- [x] **Usage Documentation** — README updates, examples, template selection guide. Users need to know templates exist

### Add After Validation (v1.5.x)

Features to add once core is working.

- [ ] **Expanded Template Library** — Add Juniper, Arista, other vendors. Triggered by user requests for specific platforms
- [ ] **Template Authoring Guide** — Detailed YAML format documentation, FSM concept explanations, examples. Add when users ask "how do I create templates?"
- [ ] **Troubleshooting Guide** — Common errors, solutions, debugging workflows. Add based on support requests
- [ ] **TUI Template Browser** — Interactive template selection in TUI mode. Add when users want better discovery UX
- [ ] **User Template Directory (XDG)** — ~/.config/cliscrape/templates/ for custom templates. Add when users ask "where do I put my templates?"
- [ ] **Template Metadata** — Description, author, version, tags in YAML frontmatter. Add when library grows large enough to need searchability

### Future Consideration (v2.0+)

Features to defer until product-market fit is established.

- [ ] **Template Versioning** — Semantic versioning for templates, backward compatibility tracking. Defer until breaking changes become problem
- [ ] **Multi-format Output Validation** — Test templates against multiple output variations per command. Defer until variation issues reported
- [ ] **Performance Benchmarks** — Templates/sec throughput testing. Defer until performance complaints
- [ ] **Template Migration Tools** — .textfsm to .yaml converter. Defer until users request bulk migration
- [ ] **Device Connectivity Integration** — SSH templates directly to devices (v2.0 milestone). Deferred by design

## Feature Prioritization Matrix

| Feature | User Value | Implementation Cost | Priority |
|---------|------------|---------------------|----------|
| Template Library (10-20 templates) | HIGH | MEDIUM | P1 |
| Template Discovery by name | HIGH | MEDIUM | P1 |
| Embedded Templates (rust-embed) | HIGH | LOW | P1 |
| Index File | HIGH | LOW | P1 |
| Basic Validation Tests | HIGH | MEDIUM | P1 |
| Production Logging (RUST_LOG) | MEDIUM | LOW | P1 |
| Usage Documentation | MEDIUM | LOW | P1 |
| Template Authoring Guide | MEDIUM | MEDIUM | P2 |
| Troubleshooting Guide | MEDIUM | MEDIUM | P2 |
| TUI Template Browser | MEDIUM | MEDIUM | P2 |
| User Template Directory (XDG) | MEDIUM | LOW | P2 |
| Template Metadata | LOW | LOW | P2 |
| Expanded Library (Juniper, Arista) | MEDIUM | MEDIUM | P2 |
| Template Versioning | LOW | HIGH | P3 |
| Performance Benchmarks | LOW | MEDIUM | P3 |
| Template Migration Tools | LOW | MEDIUM | P3 |

**Priority key:**
- P1: Must have for v1.5 launch (template ecosystem viable)
- P2: Should have, add in v1.5.x based on user feedback
- P3: Nice to have, defer to v2.0+

## Competitor Feature Analysis

| Feature | ntc-templates (TextFSM) | pyATS Genie | cliscrape Approach |
|---------|-------------------------|-------------|-------------------|
| **Template Library** | 280+ templates, community-driven | 2000+ parsers, Cisco official | Start small (10-20), curated for quality over quantity |
| **Discovery Mechanism** | parse_output(platform, command, data) | device.parse(command) | --template flag with index file, embedded in binary |
| **Template Format** | .textfsm only | Python parser classes | Both .textfsm (compat) and .yaml/.toml (modern) |
| **Validation** | Unit tests with raw+parsed YAML pairs | Schema-driven self-testing | Golden file snapshot tests in CI/CD |
| **Logging** | Python logging module | Python logging | Rust tracing crate with RUST_LOG + --verbose |
| **Documentation** | Basic README, community wiki | Comprehensive DevNet docs | Focus on examples and troubleshooting |
| **Embedded vs External** | Separate pip package | Separate pip package | Embedded in binary (zero-install) |
| **User Templates** | Fork repo or local PATH | Custom parser registration | XDG directory + --template-path override |
| **TUI Integration** | None | None | Unique: Live Lab + State Tracer with template library |
| **Metadata** | Filename convention only | Python docstrings | YAML frontmatter (description, author, version, tags) |

## Implementation Notes by Category

### 1. Template Library

**Structure:**
```
templates/
  cisco_ios/
    show_version.textfsm
    show_version.yaml
    show_interfaces.yaml
    show_ip_route.yaml
  juniper_junos/
    show_version.yaml
  arista_eos/
    show_version.yaml
```

**Naming Convention:** `{vendor_os}_{command_with_underscores}.{format}`

**Initial Scope:** Focus on Cisco IOS (most common). 10-20 templates covering:
- show version (device info)
- show interfaces (link status)
- show ip interface brief (quick status)
- show ip route (routing table)
- show running-config (config export)
- show cdp neighbors (topology)
- show vlan (switching)
- show mac address-table (layer 2)
- show inventory (hardware)
- show logging (syslog)

**Dependencies:** Must exist before discovery, validation, or documentation.

### 2. Template Discovery

**Index File Format (CSV):**
```csv
Platform,Command,Template
cisco_ios,sh[[ow]] ver[[sion]],cisco_ios_show_version.yaml
cisco_ios,sh[[ow]] int[[erfaces]],cisco_ios_show_interfaces.yaml
juniper_junos,show version,juniper_junos_show_version.yaml
```

**CLI Usage:**
```bash
cliscrape --template cisco_ios_show_version input.txt
cliscrape --platform cisco_ios --command "show version" input.txt
```

**Implementation:**
- Use rust-embed to compile index + templates into binary
- Parser: given platform+command, lookup template in index
- Fallback: if template has path separator, treat as file path (backward compat)

**Dependencies:** Requires template library and index file.

### 3. Validation Testing

**Golden File Structure:**
```
tests/golden/
  cisco_ios_show_version/
    input_01.txt         # raw device output
    expected_01.json     # expected parsed result
    input_02.txt         # variation
    expected_02.json
```

**Test Approach:**
- Load template + input, run parser, compare to expected JSON
- Use insta crate for snapshot testing (Rust standard)
- Run in CI/CD on every commit
- Fail build if templates produce unexpected output

**Complexity:** HIGH — requires collecting real device outputs, validating correctness.

**Dependencies:** Requires template library. Enhances template discovery by validating index mappings.

### 4. Production Logging

**Standard Rust Pattern:**
```rust
use tracing::{error, warn, info, debug, trace};
use tracing_subscriber::EnvFilter;

// Initialize
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();

// Usage
error!("Template not found: {}", name);
warn!("Template produced no matches");
info!("Parsed {} records from {}", count, template);
debug!("FSM transition: {} -> {}", from_state, to_state);
trace!("Regex match: {:?}", captures);
```

**CLI Flags:**
```bash
cliscrape --verbose           # -v: WARN level
cliscrape -vv                 # INFO level
cliscrape -vvv                # DEBUG level
cliscrape -vvvv               # TRACE level
RUST_LOG=debug cliscrape      # env var override
```

**Implementation:**
- Replace println!/eprintln! with tracing macros
- Add clap-verbosity-flag for CLI integration
- File logging via tracing_appender (optional, P2)

**Complexity:** LOW — standard Rust patterns, well-documented.

**Dependencies:** None (standalone).

### 5. Documentation

**Required Sections:**

1. **README Updates:**
   - Template library existence announcement
   - Discovery syntax examples
   - Available templates list

2. **Template Selection Guide:**
   - How to list available templates
   - Platform + command syntax
   - Custom template path usage

3. **Examples:**
   - Common use cases per vendor
   - Piped input workflows
   - Batch processing multiple files

4. **Template Authoring Guide (P2):**
   - YAML format specification
   - FSM concepts (states, rules, values)
   - Regex tips and macro library usage
   - Testing workflow

5. **Troubleshooting (P2):**
   - Template not found errors
   - No matches produced
   - FSM state debugging with TUI
   - Regex debugging tips

**Complexity:** LOW (basic), MEDIUM (authoring guide + troubleshooting).

**Dependencies:** Requires template library and discovery mechanism.

## Sources

**Template Libraries and Organization:**
- [ntc-templates GitHub](https://github.com/networktocode/ntc-templates) — 280+ TextFSM templates, community-driven
- [ntc-templates index file](https://github.com/networktocode/ntc-templates/blob/master/ntc_templates/templates/index) — CSV format for platform+command mapping
- [pyATS Genie parser library](https://github.com/CiscoTestAutomation/genieparser) — 2000+ parsers with schema validation
- [Network to Code: NTC Templates Best Practices](https://networktocode.com/blog/leveraging-ntc-templates-for-network-automation-2025-08-08/)

**Template Discovery and Selection:**
- [TextFSM CLI Table](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_clitable.html) — Index file format and template selection
- [ntc-templates Getting Started](https://ntc-templates.readthedocs.io/en/latest/user/lib_getting_started/) — parse_output() API

**Validation and Testing:**
- [Snapshot Testing in Rust](https://blog.anp.lol/rust/2017/08/18/golden-master-regression-in-rust/) — Golden master testing approach
- [Golden Testing Helm Charts](https://developerzen.com/golden-testing-helm-charts/) — Template validation patterns
- [Network Validation with pyATS](https://netcraftsmen.com/network-validation-with-pyats/) — Parser testing frameworks

**Production Logging:**
- [Logging in Rust (2025)](https://www.shuttle.dev/blog/2023/09/20/logging-in-rust) — tracing vs log crate comparison
- [Structured JSON Logs with tracing](https://oneuptime.com/blog/post/2026-01-25-structured-json-logs-tracing-rust/view) — Production logging best practices
- [tracing EnvFilter documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html) — RUST_LOG usage
- [clap-verbosity-flag](https://docs.rs/clap-verbosity-flag) — CLI verbosity pattern
- [Rust CLI: Communicating with Humans](https://rust-cli.github.io/book/in-depth/human-communication.html) — Logging best practices

**Documentation Patterns:**
- [Command Line Interface Guidelines](https://clig.dev/) — CLI UX best practices, help text, examples
- [CLI Documentation Checklist](https://www.infrasity.com/blog/cli-docs-checklist) — Developer-first CLI docs
- [Template Text Parser Guide](https://www.packetcoders.io/a-beginners-guide-to-textfsm-for-network-automation/) — Template authoring examples

**Embedded Assets:**
- [rust-embed crate](https://docs.rs/crate/rust-embed) — Compile-time asset embedding
- [CLI Progress Indicators](https://evilmartians.com/chronicles/cli-ux-best-practices-3-patterns-for-improving-progress-displays) — UX patterns

**Configuration and Directories:**
- [xdg crate](https://docs.rs/xdg) — XDG Base Directory specification for Rust
- [directories crate](https://lib.rs/crates/directories) — Platform-specific config/cache paths

**Versioning and Compatibility:**
- [Backward Compatibility: Versioning, Migrations, and Testing](https://medium.com/@QuarkAndCode/backward-compatibility-versioning-migrations-and-testing-b69637ca5e3d) — Migration strategies
- [Semantic Versioning 2.0.0](https://semver.org/) — Version numbering standards

---
*Feature research for: CLI Parsing Tool Ecosystem (v1.5 Template Library)*
*Researched: 2026-02-22*

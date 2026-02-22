# Domain Pitfalls: CLI Parsing Tools

**Domain:** CLI Parsing / FSM-based Text Processing
**Last Updated:** 2026-02-22
**Confidence:** HIGH

This document covers pitfalls across two milestone contexts:
- **v0.1 Alpha** (Phases 1-4): Engine, TextFSM compatibility, TUI, Modern formats
- **v1.5 Template Ecosystem & Production Hardening** (Phases 6-11): Template libraries, discovery, validation, logging, documentation

---

## v1.5 Milestone Pitfalls: Template Ecosystem & Production Hardening

### Pitfall 1: Template Library Versioning Without Breaking Change Protection

**What goes wrong:**
Template library updates silently break existing parsing scripts when vendors change CLI output formats or templates get refactored. Users upgrade the template library (via package manager or git pull) and suddenly their production automation fails because template field names changed, regex patterns tightened, or state transitions were reorganized.

**Why it happens:**
Template authors improve templates based on new device output samples without recognizing that existing users depend on current field names and structure. The ntc-templates library has gone through 9 major versions (v9.0.0 as of Feb 2026) with breaking changes requiring migration guides, demonstrating this is an endemic problem in the network automation space.

**How to avoid:**
1. Implement semantic versioning at the template level, not just the library level
2. Provide template metadata with `min_version` and `deprecated_fields` markers
3. Create compatibility shims that map old field names to new ones
4. Run automated regression tests comparing new template versions against historical device outputs
5. Document breaking changes in CHANGELOG with migration paths
6. Consider a "lock file" mechanism (like package-lock.json) to pin template versions

**Warning signs:**
- Templates lack version metadata
- No automated testing against historical output samples
- Missing CHANGELOG or migration documentation
- Users cannot pin specific template versions independently from library version
- Field names are inconsistent across similar templates

**Phase to address:**
Phase 6 (Template Library) - Build versioning directly into template metadata format and discovery mechanism.

---

### Pitfall 2: Template Discovery Security: Path Traversal via Template Names

**What goes wrong:**
Template discovery mechanisms that resolve names to filesystem paths without proper sanitization allow path traversal attacks. User provides template name like `../../../etc/passwd` or `..\Windows\System32\config\SAM`, and the parser attempts to load arbitrary files from the system.

**Why it happens:**
Developers implement convenience features like `--template cisco_ios_show_version` mapping to `templates/cisco/ios/show_version.textfsm` without considering malicious input. The ntc-templates integration with Netmiko uses an index file and `NET_TEXTFSM` environment variable, but without proper path validation, this creates attack surface.

**How to avoid:**
1. **Whitelist-only discovery**: Template names MUST match `^[a-z0-9_-]+$` regex (alphanumeric + underscore/dash only)
2. **Canonical path validation**: Resolve template path, then verify it starts with expected template directory
3. **No user-controlled path components**: Never concatenate user input directly into file paths
4. **Explicit index file**: Use manifest/index file listing valid template names → paths mapping
5. **Reject absolute paths**: Template names starting with `/` or containing `:` (Windows drive letters) must fail
6. **Embedded templates only in untrusted contexts**: When running with elevated privileges or in shared environments, disable filesystem template loading entirely

**Warning signs:**
- Template name validation uses blacklist (blocking `..` or `/`) rather than whitelist
- Direct string concatenation: `format!("templates/{}.textfsm", user_template_name)`
- No canonical path verification after resolution
- Error messages reveal filesystem structure
- Template discovery works with absolute paths

**Phase to address:**
Phase 6 (Template Library) - Discovery mechanism must implement whitelist validation and canonical path checks before ANY filesystem access.

---

### Pitfall 3: Incomplete Validation Coverage: Templates Pass Syntax But Fail Semantically

**What goes wrong:**
Templates pass validation suite with perfect syntax scores but fail in production because validation only tested happy-path inputs. Real device output contains edge cases: blank lines, inconsistent spacing, vendor-specific prompts, pagination markers (`--More--`), error messages intermixed with data, truncated output, Unicode characters, or ANSI escape sequences. Parser returns partial data silently rather than failing explicitly.

**Why it happens:**
Validation suites use sanitized "textbook" device outputs from lab environments or documentation. Real production output is messy: devices might inject timestamp banners, include warning messages, use locale-specific date formats, or produce truncated output under load. As noted in TextFSM validation research: "Ensuring that the textfsm template accounts for every line is the only method to ensure data was not accidentally missed."

**How to avoid:**
1. **Require negative test cases**: Every template MUST include tests for malformed input, incomplete output, error conditions
2. **Field-level validation**: Validate captured data types and ranges (e.g., VLAN IDs 1-4094, valid IP addresses)
3. **Line coverage tracking**: Flag when template doesn't consume all input lines (unless explicitly allowed)
4. **Real-world corpus testing**: Collect production device outputs (anonymized) and run entire template library against them
5. **Fail-fast mode**: Option to error on partial matches rather than returning incomplete data
6. **Mutation testing**: Apply field-level and structural mutations to test inputs to verify parser robustness

**Warning signs:**
- Validation suite only includes successful parsing examples
- No tests for error messages, empty output, or truncated data
- Templates don't handle optional sections (device might not display certain data)
- Parser returns empty arrays without warnings when input doesn't match
- No validation that captured values make semantic sense (e.g., MAC address format)

**Phase to address:**
Phase 7 (Compatibility Validation) - Build validation suite with negative tests, edge cases, mutation testing, and real-world corpus validation. Phase 8 (TUI Verification) - Use TUI to manually verify edge case handling.

---

### Pitfall 4: Vendor Output Format Changes Break Templates Without Detection

**What goes wrong:**
Network vendors change CLI output format in firmware updates (new columns, reordered fields, renamed headings, different spacing) and existing templates silently fail. Parsers return empty results or partial data, but automation scripts don't detect the failure because they assume successful parsing means correct data.

**Why it happens:**
Template authors test against specific firmware versions, but vendors release updates that modify output formatting. The networking industry lacks output format stability guarantees. As documented in network automation research: "Configuration files saved on SR Linux release 24.10 cannot be loaded on SR Linux release 25.3 if they use changed data model parameters." This applies equally to CLI output formats.

**How to avoid:**
1. **Template metadata includes tested firmware versions**: Document which device OS versions template was validated against
2. **Minimum match thresholds**: Template fails if it matches less than N% of expected fields (configurable, default 80%)
3. **Community feedback loop**: Collect failure reports with device firmware versions automatically
4. **Regression detection**: CI/CD runs templates against archived outputs from multiple firmware versions
5. **Graceful degradation markers**: Templates specify which fields are required vs. optional
6. **Change detection warnings**: Parser warns when input structure looks unfamiliar (e.g., unexpected line count, column headers missing)

**Warning signs:**
- Templates lack firmware version metadata
- Parser returns success with empty result array (should be suspicious)
- No CI testing against multiple device firmware versions
- Missing required fields treated same as truly optional fields
- No community-driven template update mechanism

**Phase to address:**
Phase 6 (Template Library) - Add firmware version metadata to templates. Phase 7 (Compatibility Validation) - Test templates against multiple firmware versions and flag coverage gaps.

---

### Pitfall 5: Production Logging Performance Degradation Through Over-Instrumentation

**What goes wrong:**
Adding comprehensive structured logging and tracing causes 30%+ performance degradation in production. High-throughput parsing jobs (processing thousands of device outputs) slow from 4.1M lines/sec to 2.8M lines/sec because every regex match, state transition, and record emission generates trace events with full variable snapshots. Performance regresses below Python TextFSM levels, destroying core value proposition.

**Why it happens:**
Developers add `#[instrument]` to all functions during development for debugging visibility, then ship to production with trace/debug logging still compiled in. As documented in Rust tracing research: "JSON serialization has overhead" and "excessive logging can cause slowdowns and increase data transfer times, with unnecessary logging impacting performance by at least 30% in large deployments."

**How to avoid:**
1. **Compile-time log level filtering**: Use `max_level_debug` feature flags to remove trace/debug logs from release builds
2. **Selective instrumentation**: Only instrument high-level operations (parser initialization, record emission) not inner loops
3. **Skip large fields**: Use `#[instrument(skip(input_text, captured_vars))]` to avoid serializing large data structures
4. **Sampling**: Log 1/1000 parses at TRACE level, rest at INFO/WARN/ERROR only
5. **Conditional tracing**: Enable detailed tracing only when TUI debugger is active, not in CLI batch mode
6. **Lazy formatting**: Use closures for expensive log messages: `debug!(msg = ?expensive_closure)` so formatting only happens if level enabled
7. **Benchmark with logging enabled**: Include logging overhead in performance benchmarks, not just raw parsing

**Warning signs:**
- `#[instrument]` on functions called thousands of times per parse (state transitions, regex matching)
- Logging full input text or captured variable state in loops
- JSON serialization in hot paths
- Performance benchmarks run with logging disabled
- No max_level compile-time feature configuration
- Tracing subscriber configured the same in dev and production

**Phase to address:**
Phase 10 (Production Logging) - Implement with compile-time filtering, selective instrumentation, and production-tuned configuration. Verify no regression in benchmarks from Phase 1.

---

### Pitfall 6: Documentation Drift: Examples Reference Non-Existent Features

**What goes wrong:**
Documentation examples show CLI commands, template syntax, or API usage that doesn't match shipped code. Users copy examples from docs and get errors. Common manifestations: docs reference deprecated flags, show old template syntax, demonstrate features not yet implemented, or use example templates that don't exist in shipped library.

**Why it happens:**
Documentation lives in separate files from code (or separate repository), updated manually after code changes. No automated validation that examples in docs actually work. As research shows: "Documentation drift happens because no one has time to update docs manually after every small change" and "no automated validation that examples in docs actually execute successfully."

**How to avoid:**
1. **Executable documentation**: Documentation examples are actual runnable tests (doc tests in Rust, doctest in Python)
2. **CI validates examples**: Extract code blocks from markdown, execute them, verify they succeed
3. **Generated examples**: Auto-generate CLI help examples from actual argument parser definitions
4. **Version-aware docs**: Docs explicitly state which version examples apply to, with legacy version archives
5. **Integration tests as docs**: Link to integration tests demonstrating features rather than duplicating examples
6. **Docs-as-code pipeline**: Docs build fails if examples fail, forcing synchronization
7. **Community validation**: Public examples run in CI against each release candidate

**Warning signs:**
- Documentation examples not tested in CI
- Examples hardcode paths that won't exist on user systems
- Docs show features not present in code (aspirational docs)
- No version tags on examples
- Copy-paste between docs creates inconsistent examples
- Error messages in docs don't match actual error messages

**Phase to address:**
Phase 11 (Documentation) - Write as doc tests where possible, create CI validation for markdown examples, generate CLI help from parser definition. Requires coordination with Phase 6 (templates must exist) and Phase 2 (CLI must be stable).

---

### Pitfall 7: Template Distribution: Embedded vs. Filesystem Tradeoffs

**What goes wrong:**
**Embedded approach**: Templates compiled into binary via `rust-embed` or `include_str!` create inflexible distribution. Users cannot add custom templates without recompiling. Binary size bloats (hundreds of templates × kilobytes each). Template updates require full binary release.

**Filesystem approach**: Templates distributed separately create version skew (binary v1.5 + templates v2.0 incompatible), installation fragility (`NET_TEXTFSM` environment variable not set, templates not found), and security exposure (template discovery path traversal).

**Why it happens:**
Each approach optimizes for different use case. Embedded suits standalone CLI distribution (single binary, no dependencies). Filesystem suits library ecosystem (shared templates, user customization). Network automation tools need BOTH, but implementing hybrid distribution is complex.

**How to avoid:**
1. **Hybrid distribution model**:
   - Ship with embedded template library (fallback, always available)
   - Support filesystem override (search `./templates/`, `~/.cliscrape/templates/`, `$CLISCRAPE_TEMPLATES`)
   - Filesystem templates take precedence over embedded
2. **Explicit template source flags**: `--template-source=embedded|filesystem|auto`
3. **Template list command**: `cliscrape templates list` shows all available templates with sources (embedded vs. filesystem)
4. **Metadata validation**: Both embedded and filesystem templates have metadata with required engine version
5. **Clear load order documentation**: Document template search path and precedence
6. **Size budget**: Embedded templates limited to "core" set (<100), full library available via filesystem/download

**Warning signs:**
- Only one distribution model supported
- No way to list available templates
- Template source unclear in error messages
- Cannot determine which template version loaded
- Binary size unexpectedly large (>50MB for CLI parser)
- No template override mechanism

**Phase to address:**
Phase 6 (Template Library) - Design hybrid distribution from start. Embedded core templates + filesystem discovery with clear precedence model.

---

### Pitfall 8: Environment Variable Path Expansion Inconsistencies

**What goes wrong:**
Template discovery via environment variables (following `NET_TEXTFSM` pattern) fails when users specify paths with tilde (`~`) for home directory. The tilde doesn't expand, parser looks for literal directory named `~`, templates not found. This is particularly problematic on Windows where path handling differs from Unix.

**Why it happens:**
Tilde expansion is a shell feature, not an OS-level feature. Environment variables contain literal strings. As documented in ntc-templates issues: "Using the tilde (~) symbol to indicate the home directory in the NET_TEXTFSM environment variable causes exceptions, even though absolute paths work as expected."

**How to avoid:**
1. **Explicit tilde expansion**: Manually expand `~` to home directory before using env var paths
2. **Cross-platform path handling**: Use `dirs::home_dir()` or equivalent, test on Windows + Unix
3. **Clear error messages**: If template directory not found, show attempted paths and suggest fixes
4. **Documentation**: Explicitly state env vars need absolute paths, show examples for each OS
5. **Validation on startup**: Check env var paths exist, warn if misconfigured
6. **Alternative config**: Support config file (TOML/YAML) with proper path expansion in addition to env vars

**Warning signs:**
- Template discovery works with absolute paths but not `~/templates`
- Different behavior on Windows vs Unix for same env var
- Error messages don't show which path was attempted
- Users report "templates not found" despite correct installation

**Phase to address:**
Phase 6 (Template Library) - Discovery mechanism must handle tilde expansion explicitly and provide helpful error messages.

---

### Pitfall 9: Template Index File Format Without Schema Validation

**What goes wrong:**
Template index file (mapping template names to files) uses ad-hoc format without schema validation. Typos in index file cause silent failures (templates exist but aren't discoverable), malformed entries crash parser, or incompatible metadata causes runtime errors. Users manually edit index and break things.

**Why it happens:**
Developers create index file format organically without formal schema. The ntc-templates uses index.yml but schema validation isn't enforced. Manual edits introduce errors that aren't caught until runtime.

**How to avoid:**
1. **Formal schema definition**: JSON Schema or similar for index file format
2. **Validation on load**: Parse index with schema validator, fail fast on invalid entries
3. **Helpful error messages**: "Index file line 42: 'platform' field missing for template 'cisco_ios_show_version'"
4. **Auto-generation**: Provide tool to scan template directory and generate valid index file
5. **Version compatibility**: Index schema versioned, parser checks compatibility
6. **Required fields**: template_name, file_path, platform, command, min_engine_version

**Warning signs:**
- Index file has no documented format
- Typos in index cause silent failures
- No validation when index is loaded
- Users must manually maintain index
- Index format changes break existing installations

**Phase to address:**
Phase 6 (Template Library) - Define index schema, implement validation, provide generation tool.

---

### Pitfall 10: Regex Complexity DoS in User-Provided Templates

**What goes wrong:**
User loads custom template with regex containing catastrophic backtracking patterns (e.g., `(a+)+b` or `(a|a)*b`). Parser hangs indefinitely or consumes gigabytes of memory when processing malicious or unexpected input. Production parsing pipeline becomes denial-of-service vector.

**Why it happens:**
Rust's standard `regex` crate prevents this with linear-time guarantees, but `fancy-regex` (needed for lookahead/backreferences) uses backtracking and is vulnerable. Users can craft templates with exponential-time regex patterns.

**How to avoid:**
1. **Regex compilation timeout**: Abort compilation if it takes >1 second
2. **Regex execution timeout**: Abort matching if it takes >100ms per line
3. **Complexity analysis**: Analyze regex AST for pathological patterns before compilation
4. **Prefer standard regex**: Use Rust `regex` crate (linear time) where possible, `fancy-regex` only when required
5. **Template sandboxing**: Filesystem templates have stricter limits than embedded trusted templates
6. **Pattern length limits**: Reject regexes longer than 1000 characters
7. **Documentation**: Warn template authors about backtracking risks

**Warning signs:**
- Parser hangs on specific input combinations
- Memory usage grows exponentially during regex matching
- Certain templates cause timeout errors
- No limits on regex compilation/execution time

**Phase to address:**
Phase 6 (Template Library) - Implement regex complexity checks and timeouts. Phase 9 (Edge Case Handling) - Verify timeout mechanisms work correctly.

---

## Integration Pitfalls: Adding Features to Existing System

### Pitfall 11: Template Library Integration Breaking Existing CLI Contracts

**What goes wrong:**
Adding template discovery (`--template cisco_ios_show_version`) changes behavior of existing `--template <file>` flag. Scripts that worked with file paths now interpret them as template names. Backward compatibility breaks.

**Why it happens:**
Overloading single flag for both filesystem paths and logical template names. Ambiguity: is `cisco_ios` a file named `cisco_ios` or a template name?

**How to avoid:**
1. **Separate flags**: `--template-file <path>` vs `--template-name <name>` (explicit)
2. **Smart detection with escape hatch**: If argument contains `/` or `\`, treat as path; otherwise template name. Provide `--template-file` to force path interpretation
3. **Backward compatibility**: Existing `--template` continues to work with file paths
4. **Clear error messages**: "Ambiguous template argument. Use --template-file for paths or --template-name for library templates"

**Warning signs:**
- Existing scripts break after template library feature added
- Cannot load template files with names that match library template names
- No way to force path interpretation

**Phase to address:**
Phase 6 (Template Library) - Design CLI integration preserving backward compatibility.

---

### Pitfall 12: Validation Suite Without Performance Regression Detection

**What goes wrong:**
Comprehensive validation suite tests correctness but not performance. Template library update changes regex patterns, adding complexity. Templates still pass validation (correct results) but parsing slows 10x. Production deployments regress.

**Why it happens:**
Validation focuses on functional correctness (do results match expected?), ignoring performance. Performance testing requires benchmark infrastructure, often added as afterthought.

**How to avoid:**
1. **Benchmark suite alongside validation**: Each template has performance benchmark (lines/sec)
2. **Regression detection**: CI fails if template performance drops >20%
3. **Complexity metrics**: Track regex complexity, state count, average transitions per parse
4. **Performance budgets**: Template library has aggregate performance target (e.g., avg 1M lines/sec across all templates)
5. **Profiling in CI**: Flamegraphs generated for slow templates, archived for comparison

**Warning signs:**
- Validation suite has no performance tests
- Template updates don't track performance impact
- Production deployments slower after template library updates
- No way to identify which template is slow

**Phase to address:**
Phase 7 (Compatibility Validation) - Add performance benchmarks to validation suite.

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| Skip template versioning metadata | Faster initial implementation | Cannot track breaking changes, no compatibility guarantees, migration nightmares | Never - versioning is foundational |
| Regex-only template name validation (block `..` and `/`) | Quick security "fix" | Bypasses exist (`....//`, URL encoding, Unicode normalization), false security | Never - use whitelist |
| Validation suite with only happy-path tests | Appears feature-complete quickly | Silent failures in production, data corruption, false confidence | Never - negative tests are mandatory |
| Global logging level configuration only | Simple implementation | Cannot reduce logging in hot paths without disabling everywhere | Only in MVP - production needs granular control |
| Templates embedded in binary only | Easy distribution, no path issues | Users cannot customize, updates require binary release, bloated binary | Only if library is tiny (<10 templates) |
| Documentation examples written by hand | Full creative control | Guaranteed to drift, examples break, user frustration | Never - automate or extract from tests |
| Template discovery via direct filesystem path | Works immediately | Security vulnerability, unclear failures, no validation | Only in explicitly local-only mode with warnings |
| Warning-level logs for parsing failures | Doesn't break existing scripts | Silent data loss, hard-to-debug automation failures | Never in production - fail fast |
| No regex timeout enforcement | Simpler implementation | DoS vulnerability with malicious templates | Only for embedded trusted templates - never user templates |
| Manual template index maintenance | Skip building index generator | Typos break discovery, doesn't scale, error-prone | Only in early MVP with <10 templates |

## Integration Gotchas

Common mistakes when integrating template library with existing parser engine.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| Environment variables (NET_TEXTFSM style) | Assume `~` expands in env var paths | Manually expand `~` to home directory before use. Tilde expansion is shell-specific, not OS-level |
| Template index file format | Use custom format without schema validation | Define JSON/YAML schema, validate at load time. Follow ntc-templates index.yml pattern for compatibility |
| Template loading errors | Return generic "file not found" | Distinguish: template name invalid, template file missing, template syntax error, wrong format, version incompatible |
| Cross-platform paths | Hardcode `/` or `\` separators | Use `std::path::PathBuf` and platform-agnostic APIs. Test on Windows + Unix |
| Template caching | Cache templates forever without invalidation | Cache with mtime/hash checks, or provide `--no-cache` for development. TUI mode should disable caching |
| Embedded resource access | Assume `include_str!` works for user-provided paths | Embedded only for compile-time known paths. Filesystem for runtime paths |
| Unicode in template names | Assume ASCII-only template names | Support Unicode but normalize (NFC) and validate against homograph attacks |
| Error messages expose paths | Show full filesystem paths in errors | Sanitize paths: show template name + relative path only, not absolute system paths |
| CLI flag overloading | Single `--template` flag for both paths and names | Separate flags or smart detection with explicit override flags |
| Logging overhead in library mode | Always configure global logger | Library returns structured data (warnings, errors), caller configures logging |

## Performance Traps

Patterns that work at small scale but fail as usage grows.

| Trap | Symptoms | Prevention | When It Breaks |
|------|----------|------------|----------------|
| Logging every state transition | Parsing slows 30-40% with debug logging enabled | Compile-time log filtering, sampling, skip large fields | >100K lines of input, high-frequency parsing |
| Full variable snapshots in traces | Memory usage grows unbounded in TUI tracer | Configurable history depth, snapshot deltas instead of full state | >1000 state transitions per parse |
| Template re-parsing on every use | Parser startup latency increases linearly with template count | Cache parsed templates, lazy load, index file | >50 templates in library |
| JSON serialization in hot path | Throughput drops below 1M lines/sec | Use structured logging only at record boundaries, not per-line | Logging in regex match loops |
| Regex compilation per parse | Parse time increases 10x for complex templates | Compile regex once during template load, reuse across parses | Templates with >20 regex patterns |
| Unbounded template library scans | `--list-templates` takes seconds | Pre-build index file, don't scan filesystem recursively at runtime | >200 templates in nested directories |
| Per-line string allocations | Parser allocates GBs for large inputs | Use string slices, arena allocation, or mmap for large files | >1M lines in single parse |
| No regex timeout | Parser hangs indefinitely | Implement compilation and execution timeouts | Catastrophic backtracking patterns in fancy-regex |
| Validation suite without benchmarks | Performance regressions go unnoticed | Benchmark each template, track performance over time | Template library grows >50 templates |

## Security Mistakes

Domain-specific security issues beyond general web security.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Template names with path separators | Path traversal, arbitrary file read, information disclosure | Whitelist: `^[a-z0-9_-]+$` only. Reject if contains `/`, `\`, `.`, or `:` |
| Unrestricted template filesystem access | Users load malicious templates that exploit parser bugs (regex DoS, state explosion) | Sandbox: filesystem templates only from whitelisted directories, validate file size <1MB, regex complexity limits |
| Regex DoS in user templates | Catastrophic backtracking crashes parser (e.g., `(a+)+b` on "aaaa...") | Regex complexity analysis, timeout on compilation, linear-time regex engine, limit pattern length |
| Template metadata injection | Malicious template metadata executes code via format string, YAML deserialization | Parse metadata with safe parser (no `!!python` tags), validate all fields against schema |
| Environment variable injection | `NET_TEXTFSM=/etc/passwd` causes parser to leak system files | Validate env var points to directory (not file), canonicalize, verify prefix |
| Unvalidated template downloads | `--install-template https://evil.com/template` installs malicious code | HTTPS only, checksum validation, signed templates, or curated repository only |
| Verbose error messages | Errors reveal system paths, template internals, input data | Sanitize errors: show logical names only, hide filesystem paths, rate-limit errors |
| No schema validation on index file | Malicious index injects code via YAML tags or crafted paths | Validate index with JSON schema, safe YAML parser, reject unknown fields |

## UX Pitfalls

Common user experience mistakes in this domain.

| Pitfall | User Impact | Better Approach |
|---------|-------------|-----------------|
| Silent partial parsing | Parser returns incomplete data without warning, automation uses bad data | Fail loudly on incomplete match, or emit warnings with `--strict` mode for errors |
| Unclear template names | User doesn't know which template to use for "show ip route" | Follow naming: `{vendor}_{os}_{command_underscored}` (e.g., `cisco_ios_show_ip_route`) |
| No template discovery | User must know exact filename/path | `cliscrape templates list [--vendor cisco]`, search by command, fuzzy matching |
| Cryptic parsing errors | "State machine failed" - user doesn't know why | Show which line failed, expected pattern, captured variables so far, suggest fixes |
| Template validation failure without context | "Template invalid" - user doesn't know what to fix | Line number, specific validation error, link to template syntax docs |
| Output format mismatch in pipeline | TUI opens when user pipes output: `cliscrape | jq` | TTY-aware: JSON when piped, table when interactive (already implemented) |
| No template debugging workflow | User edits template, runs parser, sees "failed", repeats | TUI live lab mode (already implemented) - but need docs showing this workflow |
| Version mismatches silently fail | User's template works locally, fails in CI (different cliscrape version) | Show template metadata min_version, error if incompatible, document version policy |
| Template list shows no metadata | User sees names but doesn't know what each does | List shows: platform, command, description, tested firmware versions, last updated |
| No feedback on template library updates | User doesn't know templates were updated | Show "N templates updated" after library refresh, changelog of template changes |

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **Template Library:** Often missing version metadata - verify each template has `version`, `min_engine_version`, `tested_firmware_versions` fields
- [ ] **Template Discovery:** Often missing security validation - verify whitelist-only names, canonical path checks, no user-controlled path components
- [ ] **Validation Suite:** Often missing negative tests - verify tests for malformed input, incomplete output, vendor variations, error conditions
- [ ] **Production Logging:** Often missing compile-time filtering - verify release builds use `max_level_*` features, benchmarks include logging overhead
- [ ] **Documentation Examples:** Often missing CI validation - verify examples extracted and run in CI, or are actual doc tests
- [ ] **Template Distribution:** Often missing hybrid model - verify both embedded defaults and filesystem override work with clear precedence
- [ ] **Error Handling:** Often missing path sanitization - verify error messages don't expose absolute filesystem paths
- [ ] **Breaking Changes:** Often missing migration guide - verify CHANGELOG documents field renames, new required metadata, API changes
- [ ] **Performance:** Often missing real-world benchmarks - verify benchmarks use production-sized inputs and logging configuration
- [ ] **Security:** Often missing threat model - verify template loading, discovery, and execution all have documented security boundaries
- [ ] **Template Index:** Often missing schema validation - verify index file validated against schema, helpful errors on malformed entries
- [ ] **Regex Timeouts:** Often missing - verify compilation and execution timeouts prevent DoS
- [ ] **Tilde Expansion:** Often missing - verify `~/templates` paths work correctly on all platforms
- [ ] **Performance Regression:** Often missing from validation - verify benchmarks detect template performance degradation

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Template library breaking change shipped | HIGH | 1. Immediate patch release with compatibility shim. 2. Publish migration guide. 3. Support parallel old+new field names for 2 major versions. 4. Add regression tests. |
| Path traversal vulnerability discovered | MEDIUM | 1. Emergency security release with whitelist validation. 2. Security advisory. 3. Audit all path handling code. 4. Add fuzzing tests. |
| Documentation examples broken | LOW | 1. Fix examples in docs. 2. Add CI validation so it doesn't repeat. 3. Patch release if examples in shipped --help text. |
| Performance regression from logging | MEDIUM | 1. Profile to identify hot paths. 2. Add compile-time filtering. 3. Patch release. 4. Update benchmarks to include logging. |
| Vendor output format breaks template | LOW | 1. Community submits updated template. 2. Add new test case with new firmware version. 3. Minor version bump. 4. Document supported firmware versions. |
| Template validation false confidence | HIGH | 1. Audit entire validation suite for negative tests. 2. Add mutation testing. 3. Collect real-world corpus. 4. Re-validate all templates. Document gaps. |
| Embedded templates bloat binary | MEDIUM | 1. Split into minimal embedded set + optional download. 2. Document migration. 3. Provide `--download-templates` command. 4. Keep compatibility with old template paths. |
| Environment variable path issues | LOW | 1. Add explicit tilde expansion. 2. Better error messages showing attempted paths. 3. Docs clarify env var behavior. 4. Add validation on startup. |
| Regex DoS attack | HIGH | 1. Emergency release with timeout enforcement. 2. Add complexity analysis. 3. Audit all templates for pathological patterns. 4. Document safe regex practices. |
| Template index corruption | LOW | 1. Regenerate index with tool. 2. Add schema validation if missing. 3. Document index format. 4. Provide index validator command. |
| CLI backward compatibility break | MEDIUM | 1. Restore old behavior under legacy flag. 2. Document migration path. 3. Deprecation warning for 2 versions before removal. |
| Performance regression in validation | MEDIUM | 1. Identify slow template. 2. Optimize regex or reduce state transitions. 3. Add performance budget. 4. Document complexity guidelines. |

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| Template Library Versioning (Pitfall 1) | Phase 6: Template Library | Metadata schema includes version fields, CI tests version compatibility, migration guide template exists |
| Template Discovery Security (Pitfall 2) | Phase 6: Template Library | Security tests: path traversal attempts fail, whitelist validation works, canonical path checks pass |
| Incomplete Validation Coverage (Pitfall 3) | Phase 7: Compatibility Validation | Validation suite includes negative tests, mutation testing, real-world corpus, line coverage tracking |
| Vendor Output Format Changes (Pitfall 4) | Phase 6 + Phase 7 | Template metadata has firmware versions, CI tests multiple versions, minimum match threshold enforced |
| Logging Performance Degradation (Pitfall 5) | Phase 10: Production Logging | Benchmarks show <5% regression, compile-time filtering verified, selective instrumentation documented |
| Documentation Drift (Pitfall 6) | Phase 11: Documentation | Examples are doc tests or CI-validated, generated from code where possible, version-tagged |
| Template Distribution (Pitfall 7) | Phase 6: Template Library | Hybrid model works (embedded + filesystem), precedence documented, list command shows sources |
| Env Var Path Expansion (Pitfall 8) | Phase 6: Template Library | Tilde expansion works, cross-platform tests pass, helpful error messages verified |
| Template Index Format (Pitfall 9) | Phase 6: Template Library | Schema validation enforced, auto-generation tool exists, version compatibility checked |
| Regex Complexity DoS (Pitfall 10) | Phase 6 + Phase 9 | Timeout enforcement tested, complexity analysis works, pathological patterns rejected |
| CLI Integration Breaking (Pitfall 11) | Phase 6: Template Library | Backward compatibility tests pass, existing scripts work unchanged, clear error messages |
| Validation Performance Blind Spot (Pitfall 12) | Phase 7: Compatibility Validation | Benchmarks integrated in validation suite, regression detection in CI, performance budgets set |

---

## v0.1 Alpha Milestone Pitfalls: Core Engine & Formats

### Phase 1 Pitfalls: FSM Engine & IR Design

### Critical: Incomplete TextFSM Action Semantics

**What goes wrong:**
The FSM engine implements Record, Continue, and Next actions but misses subtle interactions between them. For example, `Continue.Record` requires continuing to process the current line after saving a record, but naive implementations consume the line first.

**Why it happens:**
TextFSM documentation doesn't exhaustively enumerate all action combinations. Developers implement actions independently without testing interaction semantics.

**How to avoid:**
- Study ntc-templates corpus for actual usage patterns of action combinations
- Test matrix: every action × every other action × state transitions
- Reference implementation: TextFSM Python source handles Continue.Record by NOT advancing line pointer after Record

**Warning signs:**
- Templates with Continue.Record produce different results than Python TextFSM
- Multi-line records get duplicated or skipped

**Phase to address:** Phase 1 (Engine design)

**Sources:**
- [TextFSM Wiki - Actions](https://github.com/google/textfsm/wiki/TextFSM)
- [Continue.Record examples](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_examples.html)

---

### Critical: Filldown/Fillup State Persistence Edge Cases

**What goes wrong:**
Filldown values persist across Record operations until explicitly cleared with Clearall, but the FSM terminates with a non-empty row containing only Filldown values, creating a spurious final record.

**Why it happens:**
Quote from research: "When the last valid row is saved, the FSM creates a new empty row ready to have values filled in. Normally the FSM will discard empty rows when it terminates, but in this case the 'Filldown' option has populated the 'Chassis' column, so the FSM keeps this non-empty row and saves it when the FSM terminates."

**How to avoid:**
- Implement "empty row" detection: row is empty if it contains ONLY Filldown values that were carried forward, no newly matched values
- Track which values were "freshly captured" vs "filled down" this iteration
- On FSM termination, discard rows with zero fresh captures

**Warning signs:**
- Output has extra trailing record with values from previous record
- Record count is off-by-one compared to Python TextFSM
- Last record is duplicate of second-to-last

**Phase to address:** Phase 1 (Engine design)

**Sources:**
- [TextFSM Filldown behavior](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_syntax.html)
- [TextFSM Wiki](https://github.com/google/textfsm/wiki/TextFSM)

---

### Critical: Required Values and Partial Records

**What goes wrong:**
Required values prevent record emission until populated, but if template has bugs or input is malformed, the engine accumulates values silently without ever emitting records.

**Why it happens:**
Quote from research: "No record will be output on first pass as 'Slot' is 'Required' but empty." FSM doesn't warn when Required fields remain unpopulated.

**How to avoid:**
- Track "pending record" state: if values are captured but Record never fires due to Required constraint, warn user
- Add diagnostic mode: "N lines processed, M values captured, but 0 records emitted — check Required fields"
- TUI should highlight Required fields and show whether they've been populated

**Warning signs:**
- Parser runs successfully but returns empty results array
- Values pane in TUI shows captures but Results pane is empty
- No error message despite obvious parsing failure

**Phase to address:** Phase 1 (Engine) + Phase 3 (TUI diagnostics)

**Sources:**
- [TextFSM Required field validation](https://github.com/google/textfsm/wiki/TextFSM)

---

### High: IR Design - Regex Engine Abstraction Leakage

**What goes wrong:**
IR stores regex strings directly without specifying engine flavor. Python's `re` module supports backreferences and lookahead. Rust's `regex` crate doesn't. Using `fancy-regex` for everything incurs catastrophic backtracking risk.

**Why it happens:**
Quote from research: "Rust's regex library guarantees linear time searching using finite automata, but in exchange must give up some common regex features such as backreferences and arbitrary lookaround."

**How to avoid:**
- IR must tag each regex with required features: `{pattern: "...", features: ["backreference", "lookahead"]}`
- Engine selects appropriate backend: standard `regex` crate for simple patterns, `fancy-regex` for advanced features
- Fail fast at template load time if required features unavailable
- Document which TextFSM features require fancy-regex (and accept performance tradeoff)

**Warning signs:**
- Template loads but regex never matches despite working in Python
- Regex with `(?<=...)` or `\1` backreference silently fails
- Performance degrades catastrophically on certain inputs (fancy-regex backtracking)

**Phase to address:** Phase 1 (IR design) + Phase 2 (TextFSM translation)

**Sources:**
- [Rust regex crate limitations](https://docs.rs/regex/latest/regex/)
- [fancy-regex documentation](https://docs.rs/fancy-regex/)
- [Python vs Rust regex compatibility](https://github.com/rust-lang/regex/discussions/910)

---

[Additional v0.1 pitfalls continue as in original file...]

---

## Performance Benchmarks (Target)

To validate that pitfalls are avoided:

| Scenario | Target | Critical Threshold |
|----------|--------|-------------------|
| Parse 10K line CLI output | < 10ms | > 100ms indicates problem |
| Load & compile template | < 5ms | > 50ms indicates regex recompilation |
| TUI responsiveness (60fps) | < 16ms per frame | > 32ms = visible lag |
| Memory: 100K line file | < 50MB | > 500MB = unbounded trace storage |
| Concurrency: 10 threads | ~10x speedup | < 5x = locking contention |
| Template library scan | < 100ms | > 1s = needs index optimization |
| Logging overhead (production) | < 5% slowdown | > 10% = instrumentation too verbose |

---

## Sources

**v1.5 Template Ecosystem & Production Hardening:**
- [ntc-templates GitHub Repository](https://github.com/networktocode/ntc-templates) - Template library architecture, 9 major versions, migration guides
- [Netmiko TextFSM Integration](https://pynet.twb-tech.com/blog/netmiko-and-textfsm.html) - Template discovery patterns
- [Netmiko Environment Variable Issues](https://github.com/ktbyers/netmiko/issues/756) - Tilde expansion problems
- [OWASP Path Traversal](https://owasp.org/www-community/attacks/Path_Traversal) - Security vulnerability patterns
- [Network Device API Breaking Changes](https://blog.ipspace.net/2025/04/api-data-model-contract/) - Vendor firmware compatibility
- [Large-Scale Log Parsing Performance](https://zbchern.github.io/papers/issta24.pdf) - 30% logging overhead impact
- [Rust Tracing Framework](https://docs.rs/tracing) - Performance optimization
- [Testing Device Configuration Templates](https://blog.ipspace.net/2024/05/netlab-integration-tests/) - Validation approaches
- [Parser Validation Edge Cases](https://arxiv.org/html/2504.18050v1) - Mutation testing strategies
- [Documentation Drift Prevention](https://medium.com/@timawang/stop-documentation-drift-with-kiro-keep-code-and-docs-in-sync-to-ship-faster-79e0a644e1bc) - Docs-as-code
- [Rust Embedded Resources](https://docs.rs/crate/rust-embed/3.0.2) - Binary embedding strategies
- [Arduino CLI Versioning](https://arduino.github.io/arduino-cli/0.35/versioning/) - Breaking change management
- [Semantic Versioning](https://semver.org/) - Version compatibility standards

**v0.1 Alpha Core Engine:**
- [TextFSM Wiki](https://github.com/google/textfsm/wiki/TextFSM)
- [TextFSM Examples](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_examples.html)
- [TextFSM Syntax](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_syntax.html)
- [ntc-templates repository](https://github.com/networktocode/ntc-templates)
- [Rust regex crate documentation](https://docs.rs/regex/latest/regex/)
- [fancy-regex documentation](https://docs.rs/fancy-regex/)
- [Ratatui documentation](https://ratatui.rs/)
- [YAML Norway problem](https://hitchdev.com/strictyaml/why/implicit-typing-is-evil/)
- [Zero-copy parsing strategies](https://medium.com/@chopra.kanta.73/zero-copy-parsers-rust-pipelines-that-outrun-json-7db2a5644db3)
- [Rust performance pitfalls](https://llogiq.github.io/2017/06/01/perf-pitfalls.html)

---

*Pitfalls research for: cliscrape*
*Last Updated: 2026-02-22*
*Confidence: HIGH (extensive web research + domain expertise)*

# Project Research Summary

**Project:** cliscrape v1.5 Template Ecosystem & Production Hardening
**Domain:** CLI Parser with Template Library Management
**Researched:** 2026-02-22
**Confidence:** HIGH

## Executive Summary

The v1.5 milestone transforms cliscrape from a capable parser engine (v1.0 shipped) into a production-ready ecosystem with a curated template library, discovery mechanism, validation framework, and observability. Research reveals this is the critical maturation step for network automation tools: ntc-templates ships 280+ templates, pyATS Genie has 2000+ parsers, but both suffer from versioning chaos and validation gaps that we must avoid.

The recommended approach is **hybrid distribution**: embed a curated core library (10-20 high-value Cisco IOS templates) using rust-embed for zero-install convenience, while supporting XDG user templates for customization. Combine this with snapshot-based validation testing (insta crate), structured logging (tracing ecosystem), and comprehensive documentation (mdbook). The architecture extends existing modules minimally—add two new modules (template::library, template::discovery) without touching the proven FSM engine.

Key risks are template versioning without semantic compatibility guarantees (ntc-templates has 9 breaking major versions), validation blind spots (happy-path tests missing edge cases), and logging performance overhead (30%+ slowdown if done naively). Mitigation: build versioning into template metadata from day one, require negative test cases in validation suite, and use compile-time log filtering with selective instrumentation.

## Key Findings

### Recommended Stack

The v1.5 stack builds on the solid v1.0 foundation (Rust + pest parser + ratatui TUI) by adding production ecosystem capabilities. Core additions: rust-embed 8.11.0 for compile-time template embedding, directories-next 2.0.0 for XDG-compliant user paths, insta 1.46.0 for snapshot testing, tracing 0.27 + tracing-subscriber 0.3.20 for structured logging, and mdbook 0.4.x for user documentation.

**Core technologies:**
- **rust-embed 8.11.0**: Compile templates into binary — zero-install experience, metadata-only mode for listing, zero runtime overhead
- **insta 1.46.0**: Snapshot testing for validation — most popular Rust snapshot framework, VS Code integration, perfect for parser output verification
- **tracing + tracing-subscriber**: Production logging — current Rust standard, structured output, async context support, supersedes env_logger
- **directories-next 2.0.0**: XDG user template discovery — cross-platform config/cache paths, more ergonomic than low-level dirs-next
- **mdbook 0.4.x**: User guide generation — official Rust docs tool, rich plugin ecosystem, complements cargo doc for reference material

**Critical version notes:** tracing ecosystem is current standard as of 2026. rust-embed chosen over include_dir for optimization (include_dir has compile-time issues at scale). insta has largest community and best DX for snapshot testing.

### Expected Features

Research into ntc-templates and pyATS Genie ecosystems reveals clear feature tiers.

**Must have (table stakes):**
- **Pre-built template library** — every production parser ships with common device templates; start with 10-20 Cisco IOS covering 80% use cases
- **Template discovery by name** — users expect `--template cisco_ios_show_version` not file path hunting
- **Embedded templates in binary** — zero-install, no separate downloads
- **Template validation testing** — users trust templates work; golden file tests per template mandatory
- **Structured logging with levels** — production tools need RUST_LOG env var + --verbose flags
- **Basic usage documentation** — README + examples for template selection

**Should have (competitive):**
- **Template compatibility validation suite** — automated testing against real device outputs in CI/CD
- **Template authoring guide** — empower users to create templates for unsupported devices
- **Modern YAML metadata** — description, author, version, tags for searchability (ntc-templates lacks this)
- **Multiple format support** — both .textfsm (legacy compat) and .yaml/.toml (modern) already implemented in v1.0
- **Progressive verbosity** — -v/-vv/-vvv/-vvvv for granular control
- **XDG-compliant user template directory** — ~/.local/share/cliscrape/templates/ for custom templates

**Defer (v2+):**
- **Template versioning** — semantic versioning with compatibility tracking (defer until breaking changes become problem)
- **Multi-format output validation** — test against multiple output variations (defer until variation issues reported)
- **Performance benchmarks** — templates/sec throughput (defer until performance complaints)
- **Template migration tools** — .textfsm to .yaml converter (defer until users request bulk migration)
- **Device connectivity integration** — SSH to test templates on live devices (v2.0 milestone by design)

### Architecture Approach

The architecture extends v1.0's proven layered design with minimal invasiveness. Core principle: add template discovery layer between CLI and existing template loaders without modifying the FSM engine or TUI. Template resolution follows priority: explicit path → XDG user directory → embedded library → CWD fallback.

**Major components:**
1. **Template Resolver (new)** — maps template spec (path or name) to TemplateSource enum, implements security whitelist validation, canonical path checks
2. **Embedded Library (new)** — rust-embed wraps templates/ directory, provides lookup_template() API, parses index file for metadata
3. **Template Loaders (existing)** — TextFSM/YAML/TOML parsers unchanged, extended with from_str_with_format() for embedded content
4. **FsmParser Engine (existing)** — zero changes, proven v1.0 implementation preserved
5. **Validation Suite (new)** — independent test harness using insta for snapshot testing, fixtures from real device outputs
6. **Tracing Integration (new)** — structured logging layer instrumenting main operations, not hot paths, compile-time filtering for production

**Key pattern: Layered Discovery** — multi-source resolution (CLI → XDG → embedded → CWD) with explicit precedence prevents both embedded inflexibility and filesystem fragility. Security critical: whitelist-only template names (^[a-z0-9_-]+$), canonical path verification after resolution.

**Key pattern: Compile-Time Embedding** — templates bundled at build time guarantee availability, fast lookup, zero runtime dependencies. Trade-off: binary size +~40KB for 20 templates (acceptable), updates require recompile (mitigated by XDG override).

**Key pattern: Snapshot Testing** — insta provides golden master testing for parser correctness. Each template has fixture (raw device output) + expected snapshot (parsed JSON). CI runs cargo insta test, developers review with cargo insta review.

### Critical Pitfalls

Research identified 12 major pitfalls specific to v1.5, plus integration risks. Top 5 by severity:

1. **Template Library Versioning Without Breaking Change Protection** — ntc-templates has 9 major versions with breaking changes, users upgrade and automation breaks silently. Mitigation: build semantic versioning into template metadata, compatibility shims for field renames, automated regression testing, lock file mechanism to pin template versions.

2. **Template Discovery Security: Path Traversal** — naive discovery allowing `--template ../../../etc/passwd` creates arbitrary file read. Mitigation: whitelist-only validation (^[a-z0-9_-]+$), canonical path checks, explicit index file, reject absolute paths, embedded-only in untrusted contexts.

3. **Incomplete Validation Coverage** — validation passes with happy-path inputs but fails in production with edge cases (pagination markers, ANSI escapes, vendor prompts, truncated output). Mitigation: require negative test cases, field-level validation, line coverage tracking, real-world corpus testing, fail-fast mode option.

4. **Production Logging Performance Degradation** — comprehensive tracing causes 30%+ slowdown if naive instrumentation applied. Mitigation: compile-time log filtering (max_level_debug feature), skip large fields in spans, selective instrumentation (not in hot paths), sampling, benchmark with logging enabled.

5. **Vendor Output Format Changes Break Templates Without Detection** — firmware updates change CLI output format, templates silently fail. Mitigation: template metadata includes tested firmware versions, minimum match thresholds (fail if <80% fields captured), community feedback loop, regression testing against archived outputs.

**Security pitfalls** require immediate attention: path traversal (Pitfall 2), regex DoS via catastrophic backtracking in user templates (Pitfall 10), environment variable path injection (Pitfall 8). All must be addressed in Phase 6 (Template Library) design.

**Performance pitfalls** threaten core value proposition: logging overhead (Pitfall 5), validation without performance regression detection (Pitfall 12). Address via compile-time filtering and benchmark integration in validation suite.

## Implications for Roadmap

Based on combined research, the v1.5 milestone should be structured into 6 phases with clear dependencies and risk mitigation.

### Phase 6: Template Library Foundation
**Rationale:** Core infrastructure must be solid before building on it. Template library is foundation for discovery, validation, and docs. Security must be baked in from start (path traversal prevention, versioning metadata).

**Delivers:**
- 10-20 curated Cisco IOS templates (show version, interfaces, ip route, etc.)
- Embedded library via rust-embed with index file
- Template discovery mechanism with security whitelist validation
- XDG user template directory support
- Template metadata schema with versioning

**Addresses:**
- Table stakes features: pre-built library, embedded templates, discovery by name
- Pitfall 2 (path traversal security) via whitelist validation
- Pitfall 7 (distribution trade-offs) via hybrid embedded + XDG model
- Pitfall 9 (index format) via schema validation

**Avoids:**
- Breaking CLI contracts (Pitfall 11) — preserve backward compatibility for `--template <path>`
- Environment variable issues (Pitfall 8) — explicit tilde expansion
- Regex DoS (Pitfall 10) — timeout enforcement for user templates

**Research flag:** SKIP — architecture well-defined, patterns proven (ntc-templates, rust-embed docs)

### Phase 7: Compatibility Validation Suite
**Rationale:** Templates cannot be trusted without validation. Must test against real device outputs, not sanitized examples. Validation suite provides confidence templates work before shipping.

**Delivers:**
- Snapshot testing framework with insta
- Golden file fixtures for each template (real device outputs)
- Negative test cases (malformed input, truncated output, error conditions)
- Real-world corpus testing
- Performance benchmarks per template

**Addresses:**
- Table stakes: template validation testing
- Competitive: compatibility validation suite
- Pitfall 3 (incomplete validation) via negative tests, mutation testing, corpus
- Pitfall 4 (vendor format changes) via multi-firmware testing
- Pitfall 12 (validation performance blind spot) via integrated benchmarks

**Avoids:**
- False confidence from happy-path-only tests
- Performance regression going unnoticed

**Research flag:** SKIP — insta well-documented, snapshot testing patterns established

### Phase 8: TUI Verification Integration
**Rationale:** TUI Live Lab is existing v1.0 differentiator. Integrating template library into TUI provides complete authoring workflow (edit template, test live, verify state transitions). Leverages existing TUI capabilities.

**Delivers:**
- Template library accessible from TUI
- Live Lab template selection
- State Tracer with library templates
- TUI template browser (interactive discovery)

**Addresses:**
- Competitive: TUI debugger integration differentiator
- Enhances validation by providing manual verification tool
- Addresses Pitfall 3 (edge case validation) via interactive debugging

**Avoids:**
- Template authoring without feedback loop

**Research flag:** SKIP — extends existing TUI, no new patterns needed

### Phase 9: Edge Case Hardening
**Rationale:** Production readiness requires handling edge cases that validation might miss. Focus on robustness: timeout enforcement, error handling, graceful degradation.

**Delivers:**
- Regex compilation/execution timeouts
- Minimum match threshold enforcement (fail if <80% fields captured)
- Graceful degradation for optional fields
- Enhanced error messages showing context
- Fail-fast vs. partial-match modes

**Addresses:**
- Pitfall 10 (regex DoS) via timeout enforcement
- Pitfall 4 (vendor format changes) via match thresholds
- Pitfall 3 (validation gaps) via fail-fast mode

**Avoids:**
- Silent partial parsing returning incomplete data
- Parser hangs on pathological input

**Research flag:** MODERATE — timeout implementation patterns need validation, testing edge cases requires real-world corpus

### Phase 10: Production Logging
**Rationale:** Observability is production requirement but must not degrade performance. After core functionality proven, add instrumentation carefully. Compile-time filtering critical.

**Delivers:**
- tracing + tracing-subscriber integration
- RUST_LOG env var support
- Progressive verbosity (-v/-vv/-vvv/-vvvv)
- JSON log format for production
- Compile-time log filtering via max_level features
- Selective instrumentation (high-level ops only)

**Addresses:**
- Table stakes: structured logging with levels
- Competitive: progressive verbosity
- Pitfall 5 (logging performance) via compile-time filtering, selective instrumentation
- Pitfall 5 verification via benchmarks showing <5% regression

**Avoids:**
- Performance degradation from over-instrumentation
- Logging in hot paths (regex matching, state transitions)

**Research flag:** SKIP — tracing well-documented, Rust ecosystem standard

### Phase 11: Documentation & Template Authoring Guide
**Rationale:** Final phase ensures users can discover, use, and extend template library. Documentation as code prevents drift. Must exist after features complete so examples work.

**Delivers:**
- mdbook user guide (template selection, authoring, troubleshooting)
- Template catalog auto-generated from index
- Template authoring guide (YAML format, FSM concepts, regex tips)
- Troubleshooting guide (common errors, debugging workflow)
- README updates with v1.5 features
- Doc tests / CI-validated examples

**Addresses:**
- Table stakes: basic usage documentation
- Competitive: template authoring guide, troubleshooting guide
- Pitfall 6 (documentation drift) via doc tests, CI validation, generated examples

**Avoids:**
- Examples that don't match shipped code
- Documentation referencing non-existent features

**Research flag:** SKIP — mdbook standard, doc test patterns established

### Phase Ordering Rationale

**Dependencies dictate order:**
- Phase 6 (Library) must precede all others — provides templates for validation, TUI integration, docs
- Phase 7 (Validation) should precede Phase 8 (TUI) — validates templates before user-facing integration
- Phase 9 (Edge Cases) builds on Phase 7 validation findings
- Phase 10 (Logging) independent, can run parallel to Phases 8-9
- Phase 11 (Docs) must be last — documents completed features, examples must work

**Risk mitigation order:**
- Security (Phase 6) addressed immediately — path traversal, versioning foundation
- Validation (Phase 7) before user exposure (Phase 8) — confidence before scale
- Performance (Phase 10) after functionality proven — avoid premature optimization
- Documentation (Phase 11) after stabilization — prevent drift

**Parallel opportunities:**
- Phase 10 (Logging) can overlap with Phases 8-9 — independent systems
- Phase 7 fixture collection can start during Phase 6 development

### Research Flags

Phases likely needing deeper research during planning:
- **Phase 9 (Edge Case Hardening):** Timeout implementation patterns need verification. Testing catastrophic backtracking requires real-world corpus. Moderate research need.

Phases with standard patterns (skip research-phase):
- **Phase 6 (Template Library):** Architecture fully defined, rust-embed well-documented, ntc-templates provides proven patterns
- **Phase 7 (Validation):** insta crate well-documented, snapshot testing patterns established
- **Phase 8 (TUI Integration):** Extends existing v1.0 TUI, no new patterns
- **Phase 10 (Logging):** tracing ecosystem is Rust standard, extensive documentation
- **Phase 11 (Documentation):** mdbook official tool, doc test patterns established

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All crates verified via docs.rs, versions confirmed as of Jan-Feb 2026, rust-embed/insta/tracing well-established |
| Features | HIGH | ntc-templates and pyATS Genie provide clear feature benchmarks, table stakes vs. differentiators validated |
| Architecture | HIGH | Layered discovery pattern proven in ntc-templates, rust-embed architecture documented, minimal engine changes reduce risk |
| Pitfalls | HIGH | 12 domain-specific pitfalls identified with ntc-templates version history as evidence, security patterns from OWASP |

**Overall confidence:** HIGH

Research sources are authoritative (official docs, established libraries, production tools with multi-year history). Stack choices aligned with 2026 Rust ecosystem standards. Architecture extends proven v1.0 design conservatively.

### Gaps to Address

While overall confidence is high, several areas need validation during implementation:

- **Template corpus availability:** Need to collect real device outputs for validation fixtures. May require community contributions or test lab access. Mitigation: start with Cisco IOS (most common), expand based on user requests.

- **Performance benchmarks baseline:** Current v1.0 throughput establishes baseline, but need to verify no regression from template library overhead. Mitigation: benchmark in Phase 6, continuously monitor in Phase 7 validation.

- **Regex timeout tuning:** Timeout values (100ms per line suggested) need empirical validation. Too short = false positives on slow hardware, too long = DoS risk. Mitigation: make configurable, document tuning during Phase 9.

- **XDG behavior on Windows:** directories-next claims cross-platform support but Windows path handling differs. Mitigation: test on Windows in Phase 6, document platform-specific behavior.

- **Template versioning policy:** Semantic versioning clear in principle, but need to define compatibility guarantees (can field names change? can required become optional?). Mitigation: document versioning policy in Phase 6, enforce via CI in Phase 7.

- **Community template contributions:** Post-v1.5, need process for accepting contributed templates. Quality bar, validation requirements, maintenance ownership unclear. Mitigation: defer to v1.6+, focus on curated core library for v1.5.

## Sources

### Primary (HIGH confidence)
- [rust-embed 8.11.0 documentation](https://docs.rs/crate/rust-embed/latest) — compile-time embedding architecture
- [insta 1.46.0 GitHub releases](https://github.com/mitsuhiko/insta/releases) — snapshot testing framework
- [tracing documentation](https://docs.rs/tracing) — structured logging standard
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) — log formatting
- [directories-next documentation](https://docs.rs/directories-next) — XDG paths
- [mdBook documentation](https://rust-lang.github.io/mdBook/) — user guide generation
- [ntc-templates GitHub](https://github.com/networktocode/ntc-templates) — 280+ template library, version history
- [ntc-templates index file](https://github.com/networktocode/ntc-templates/blob/master/ntc_templates/templates/index) — discovery pattern
- [pyATS Genie parser library](https://github.com/CiscoTestAutomation/genieparser) — 2000+ parsers, validation approach

### Secondary (MEDIUM confidence)
- [Structured JSON logs with tracing](https://oneuptime.com/blog/post/2026-01-25-structured-json-logs-tracing-rust/view) — production logging patterns
- [Rust structured logs with OpenTelemetry](https://oneuptime.com/blog/post/2026-01-07-rust-tracing-structured-logs/view) — observability best practices
- [Network to Code: NTC Templates Best Practices](https://networktocode.com/blog/leveraging-ntc-templates-for-network-automation-2025-08-08/) — template organization
- [OWASP Path Traversal](https://owasp.org/www-community/attacks/Path_Traversal) — security vulnerability patterns
- [Network Device API Breaking Changes](https://blog.ipspace.net/2025/04/api-data-model-contract/) — vendor compatibility issues
- [Large-Scale Log Parsing Performance](https://zbchern.github.io/papers/issta24.pdf) — 30% logging overhead measurement

### Tertiary (LOW confidence)
- Various blog posts on Rust CLI best practices — informative but not authoritative
- Community discussions on template validation strategies — anecdotal

---
*Research completed: 2026-02-22*
*Ready for roadmap: yes*

# Feature Landscape: cliscrape v0.1 Alpha

**Project:** cliscrape - CLI parsing tool for network devices
**Milestone:** v0.1 Alpha (FSM Engine, TextFSM Compat, TUI Debugger, Modern Formats)
**Researched:** 2026-02-17
**Overall Confidence:** HIGH

## Executive Summary

v0.1 Alpha focuses on four core capabilities:
1. **High-performance FSM Engine** - Zero-copy parsing, thread-safe, sub-millisecond execution
2. **Full TextFSM Compatibility** - Support all ntc-templates (1000+ existing templates)
3. **TUI Debugger** - Template development workflow (biggest user pain point)
4. **Modern YAML/TOML Format** - Ergonomic alternative to TextFSM DSL

This document maps features by phase with complexity estimates, dependencies, and user workflow justification.

---

## Phase 1: FSM Engine Features

The engine must be **faster than Python TextFSM** and **correct** for all TextFSM edge cases.

### Table Stakes (Must Have)

| Feature | Complexity | Dependencies | User Workflow |
|---------|-----------|--------------|---------------|
| **Line-by-line state machine execution** | Medium | None | Core parsing loop: process CLI output incrementally |
| **Regex compilation cache** | Low | None | Performance: compile once, use many times (10-100x speedup) |
| **Value capture with named groups** | Low | Regex cache | Extract variables from matched lines |
| **State transition logic** | Medium | None | Navigate between parsing states based on matches |
| **Record accumulation** | Low | None | Collect parsed records in memory-efficient structure |
| **Zero-copy string references** | Medium | None | Performance: use `Cow<'a, str>` to avoid heap allocations |

**Rationale:** These are the minimum viable features for a functioning FSM parser. Without these, the tool cannot parse anything.

### Differentiators (Competitive Advantage)

| Feature | Complexity | Dependencies | Why Better Than Python TextFSM |
|---------|-----------|--------------|----------------------------------|
| **Pre-compiled regex bundles** | Medium | Regex cache | Rust's `RegexSet` allows parallel matching of multiple patterns |
| **Thread-safe engine instances** | Low | None | Parse multiple outputs concurrently without GIL contention |
| **Streaming mode for large files** | High | Zero-copy | Handle 50MB+ BGP tables without OOM (circular buffer) |
| **Instrumentation hooks** | Medium | None | Pluggable tracing for TUI/metrics without performance cost |

**Performance targets:**
- Parse 100k line output in <100ms (vs 1-2s in Python)
- Support 100+ concurrent parsing jobs
- Handle files up to 100MB without memory issues

### Anti-Features (Deliberately Excluded)

| Anti-Feature | Why NOT Build | What Instead |
|--------------|---------------|--------------|
| Dynamic regex compilation at runtime | Kills performance, unsafe | Pre-compile all templates at load time |
| Global state / singleton engine | Not thread-safe, hard to test | Instance-based design with `Context` |
| Automatic regex optimization | Complex, unpredictable, marginal gains | Rely on Rust `regex` crate's optimization |

---

## Phase 2: TextFSM Compatibility Features

Must support **99%+ of ntc-templates** without modification. Compatibility = adoption.

### Table Stakes (Must Have)

| Feature | Complexity | Dependencies | ntc-templates Usage |
|---------|-----------|--------------|---------------------|
| **Value options: Filldown** | Low | Record accumulation | ~60% of templates use this (carry values forward) |
| **Value options: Required** | Low | Record validation | ~40% of templates (filter incomplete records) |
| **Value options: Key** | Low | Record uniqueness | ~20% of templates (deduplicate entries) |
| **Value options: List** | Medium | Record accumulation | ~15% of templates (multiple matches per record) |
| **Value options: Fillup** | Medium | Record accumulation | Rare but critical (populate upwards in table) |
| **Actions: Record** | Low | None | Every template uses this |
| **Actions: Next** | Low | State transitions | Default action, ~90% of rules |
| **Actions: Continue** | Medium | Rule processing | ~20% of templates (multi-rule line matching) |
| **Actions: Clear** | Low | Record reset | ~30% of templates (reset non-Filldown values) |
| **Actions: Clearall** | Low | Record reset | ~10% of templates (full state reset) |
| **Actions: Error** | Low | Validation | ~5% of templates (explicit failure on malformed input) |
| **Reserved states: Start, End, EOF** | Low | State transitions | Every template requires Start |

**Critical edge cases:**
- **Filldown + List interaction:** List appends, Filldown carries only last value
- **Continue + state transitions:** Continue cannot transition (loop-free guarantee)
- **Required + Fillup interaction:** Not allowed (conflict between directions)
- **Error action:** Discards all records and raises exception (not partial failure)

**Sources:**
- [TextFSM Wiki](https://github.com/google/textfsm/wiki/TextFSM) (HIGH confidence)
- [TextFSM template syntax - Python for network engineers](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_syntax.html) (HIGH confidence)

### Differentiators (Better Than Python)

| Feature | Complexity | Dependencies | Why Better |
|---------|-----------|--------------|------------|
| **Validation at compile-time** | Medium | Template parser | Catch incompatible options (Required + Fillup) before parsing |
| **Helpful error messages** | Medium | Template parser | Point to exact line/column in template with suggestions |
| **Regex syntax validation** | Low | `regex` crate | Rust regex is stricter, catch errors early |
| **Performance profiling hooks** | Low | Instrumentation | Show which rules are slow in TUI debugger |

### Anti-Features

| Anti-Feature | Why NOT Build | What Instead |
|--------------|---------------|--------------|
| Automatic template repair | Magic = unpredictable, breaks debugging workflow | Show errors, suggest fixes |
| Regex syntax translation | Python `re` != Rust `regex`, unsafe | Document differences, validate at load |
| Template versioning | Scope creep, Git exists | Use Git for template version control |

---

## Phase 3: TUI Debugger Features

The **killer feature**. Template development is trial-and-error hell. TUI makes it visual and interactive.

### Table Stakes (Must Have)

| Feature | Complexity | Dependencies | User Workflow |
|---------|-----------|--------------|---------------|
| **Live template editing** | High | Ratatui textarea | Edit template and see results immediately |
| **Input text display** | Low | Ratatui paragraph | Show CLI output being parsed |
| **Current state indicator** | Low | Engine instrumentation | Know which state FSM is in |
| **Matched lines highlighting** | Medium | Engine instrumentation | See which lines matched which rules |
| **Current record values** | Medium | Engine context | Inspect variables as they're captured |
| **Results table** | Medium | Ratatui table | View all records produced so far |
| **Step-through execution** | High | Engine control | Execute line-by-line with pause/resume |

**Layout (4-pane TUI):**
```
┌─────────────────────────┬─────────────────────────┐
│ Input Stream (scrolling)│ FSM State: [Interface]  │
│ Line 42: GigabitEth...  │ Current Values:         │
│ > Line 43: Internet ... │ - interface: Gig1       │
│                         │ - status: up            │
├─────────────────────────┼─────────────────────────┤
│ Match Trace (live log)  │ Results Table           │
│ Line 42: Rule #1 matched│ Record 1: ...           │
│ Transition: Start->Intf │ Record 2: ...           │
├─────────────────────────┴─────────────────────────┤
│ Help: [n] Next [s] Step [q] Quit [e] Edit         │
└───────────────────────────────────────────────────┘
```

### Differentiators (Unique Value)

| Feature | Complexity | Dependencies | Why Valuable |
|---------|-----------|--------------|--------------|
| **Hot-reload on save** | Medium | File watching | Zero-friction edit loop (no manual re-run) |
| **Regex match highlighting** | High | Syntax highlighting | See capture groups visually in text |
| **Rule performance metrics** | Medium | Instrumentation | Identify slow regexes causing bottlenecks |
| **Coverage warnings** | Medium | Engine tracing | Flag unmatched lines (potential missing data) |
| **History/time-travel** | High | Circular buffer | Rewind execution to previous state |
| **Breakpoints on states/rules** | High | Engine control | Pause when entering specific state |
| **Export trace logs** | Low | File I/O | Share debugging session with teammates |

**Critical UX patterns:**
- **Async rendering:** Parse in background thread, update UI via channels (avoid blocking)
- **Circular buffer for traces:** Last 10k events only (prevent OOM on large files)
- **Keyboard focus clarity:** Clear visual indicator of which pane is active
- **Responsive layouts:** Adapt to terminal size (Ratatui constraint-based layout)

**Sources:**
- [Ratatui official site](https://ratatui.rs) (HIGH confidence)
- [TUI mode debugging patterns](https://sourceware.org/gdb/current/onlinedocs/gdb.html/TUI.html) (MEDIUM confidence)
- [Debug TUI blog post](https://www.dantleech.com/blog/2025/05/11/debug-tui/) (MEDIUM confidence)

### Anti-Features

| Anti-Feature | Why NOT Build | What Instead |
|--------------|---------------|--------------|
| Vim keybindings | Scope creep, not needed for MVP | Standard arrow keys, simple shortcuts |
| Graphical visualizations | Terminal-only tool | Use ASCII art for state diagrams |
| Multi-file editing | Complexity, unclear value | Edit one template at a time |
| Remote debugging | Network complexity, security | Run TUI locally, debug local files |

---

## Phase 4: Modern YAML/TOML Format Features

Make template authoring **ergonomic** for new users. TextFSM DSL is cryptic.

### Table Stakes (Must Have)

| Feature | Complexity | Dependencies | User Experience Win |
|---------|-----------|--------------|---------------------|
| **YAML template support** | Medium | serde_yml | Familiar format, readable structure |
| **TOML template support** | Medium | toml | Preferred by Rust community |
| **Named states with clear syntax** | Low | Template IR | `states.Start.rules[0]` vs positional |
| **Inline regex patterns** | Low | Template IR | Patterns next to variable names |
| **Comments in templates** | Low | YAML/TOML parsers | Document complex logic |
| **Compile to FSM IR** | Medium | Core IR | Same engine for all formats |

**Example YAML template:**
```yaml
meta:
  name: cisco_show_interface
  author: cliscrape

values:
  interface:
    regex: '^(?P<interface>\S+) is'
    filldown: true
    required: true

  status:
    regex: 'is (?P<status>up|down)'

  ip_address:
    regex: 'Internet address is (?P<ip_address>\S+)'

states:
  Start:
    - match: "${interface}"
      actions: [Clear]
      next_state: Interface

  Interface:
    - match: "${status}"
    - match: "${ip_address}"
      actions: [Record]
      next_state: Start
```

### Differentiators (Better Than TextFSM)

| Feature | Complexity | Dependencies | Why Better |
|---------|-----------|--------------|------------|
| **Schema validation** | High | JSON Schema | Catch errors before parsing (IDE integration) |
| **Helpful validation errors** | Medium | Parser | "Line 42: Unknown state 'Interfce' (did you mean 'Interface'?)" |
| **Variable interpolation** | Medium | Template compiler | `"${interface}"` instead of `^Value interface (.+)` |
| **Type annotations** | Medium | Template IR | `type: ipv4` validates format, converts to structured type |
| **Progressive disclosure** | Low | Documentation | Start simple, add complexity as needed |

**Type system (future extensibility):**
- `string` (default)
- `integer`
- `ipv4` / `ipv6`
- `mac_address`
- `timestamp` (with format specifier)

**Sources:**
- [JSON vs YAML vs TOML comparison 2026](https://devtoolbox.dedyn.io/blog/json-vs-yaml-vs-toml) (MEDIUM confidence)
- [YAML Schema Validation](https://json-schema-everywhere.github.io/yaml) (HIGH confidence)
- [TOML Schema Validation](https://json-schema-everywhere.github.io/toml) (HIGH confidence)

### Anti-Features

| Anti-Feature | Why NOT Build | What Instead |
|--------------|---------------|--------------|
| Custom DSL | TextFSM mistake, don't repeat | Use existing YAML/TOML standards |
| Template macros/includes | Complexity, hard to debug | Keep templates self-contained |
| Template inheritance | OOP patterns don't fit FSMs | Composition via clear states |
| Turing-complete templating | Security risk, complexity | Keep templates declarative |

---

## Cross-Cutting Features

Features that span multiple phases or support all capabilities.

### Development Tooling

| Feature | Complexity | Phase | User Need |
|---------|-----------|-------|-----------|
| **CLI interface** | Low | All | Run parser from command line |
| **JSON output** | Low | 1 | Structured data for downstream tools |
| **CSV output** | Low | 1 | Excel compatibility, human-readable |
| **Error reporting** | Medium | All | Helpful messages with context |
| **Template validation** | Medium | 2,4 | Check syntax before parsing |
| **Example templates** | Low | All | Quick start, learning material |

### Testing Infrastructure

| Feature | Complexity | Phase | Purpose |
|---------|-----------|-------|---------|
| **Unit tests for engine** | Medium | 1 | Verify FSM correctness |
| **ntc-templates test suite** | High | 2 | Ensure compatibility |
| **Property-based tests** | High | 1 | Find edge cases automatically |
| **Benchmarks** | Medium | 1 | Track performance regressions |
| **Example corpus** | Low | All | Real-world test data |

### Documentation

| Feature | Complexity | Phase | Audience |
|---------|-----------|-------|----------|
| **README with quick start** | Low | All | First-time users |
| **YAML/TOML format guide** | Medium | 4 | Template authors |
| **TUI keyboard shortcuts** | Low | 3 | TUI users |
| **Migration guide from TextFSM** | Medium | 2 | Python TextFSM users |
| **Performance tuning guide** | Medium | 1 | Power users |

---

## Feature Dependencies

Critical path for v0.1 Alpha:

```
Phase 1: FSM Engine
  ├─ Basic execution loop
  ├─ Regex compilation cache
  ├─ Value capture
  └─ State transitions
      └─ Phase 2: TextFSM Compatibility
          ├─ All Value options (Filldown, Required, etc.)
          ├─ All Actions (Record, Continue, etc.)
          └─ Edge case handling
              └─ Phase 3: TUI Debugger
                  ├─ Instrumentation hooks
                  ├─ Live template editing
                  ├─ Step-through execution
                  └─ Results visualization
                      └─ Phase 4: Modern Formats
                          ├─ YAML parser
                          ├─ TOML parser
                          ├─ Compile to IR
                          └─ Schema validation
```

**Critical dependencies:**
- TUI requires instrumentation hooks from Phase 1
- Modern formats require full TextFSM compatibility to prove IR design
- All phases depend on rock-solid Phase 1 engine

---

## MVP Recommendation

For v0.1 Alpha launch, prioritize in order:

### Must Have (Launch Blockers)
1. **Phase 1 complete** - Engine with all core features
2. **Phase 2 core** - Filldown, Required, List, Record, Next, Continue
3. **Phase 3 basic** - 4-pane TUI with live editing and step-through
4. **Phase 4 YAML only** - YAML format support (defer TOML to v0.2)

### Should Have (Strong User Value)
5. **Phase 2 edge cases** - Fillup, Clearall, Error action
6. **Phase 3 advanced** - Hot-reload, coverage warnings, history
7. **Phase 4 validation** - Schema validation with helpful errors

### Could Defer (Post-v0.1)
8. Type system for YAML (string/int/ip types)
9. Template performance profiling in TUI
10. Export trace logs from TUI
11. Breakpoints in TUI debugger
12. TOML format support

**Rationale for deferral:**
- Type system adds complexity, can iterate post-launch
- Advanced TUI features are nice-to-have, not blockers
- TOML is lower priority than YAML (less common in network automation)
- Focus on "complete + excellent" for core workflows vs "feature-complete + rough"

---

## Complexity Analysis

### Low Complexity (1-2 days)
- Regex compilation cache
- Value capture
- Basic actions (Record, Next, Clear)
- JSON/CSV output
- CLI interface

### Medium Complexity (3-5 days)
- State machine execution loop
- Zero-copy string handling
- TextFSM value options (Filldown, Required, Key, List)
- YAML/TOML parsing
- 4-pane TUI layout
- Schema validation

### High Complexity (1-2 weeks)
- Streaming mode for large files
- Continue action (multi-rule matching)
- Fillup action (upward propagation)
- Live template editing with hot-reload
- Step-through execution with breakpoints
- Time-travel debugging
- Regex match highlighting

**Total estimated effort for v0.1 MVP:** 6-8 weeks
- Phase 1: 2 weeks
- Phase 2: 1.5 weeks
- Phase 3: 2.5 weeks
- Phase 4: 1 week
- Testing/polish: 1 week

---

## Success Criteria

v0.1 Alpha is successful when:

### Functional Success
- [ ] Parses 95%+ of ntc-templates without modification
- [ ] TUI allows template development without leaving terminal
- [ ] YAML format is easier to read/write than TextFSM for new users
- [ ] Zero crashes on valid input (Rust safety guarantees)

### Performance Success
- [ ] 10x faster than Python TextFSM on 10k+ line outputs
- [ ] Handles 100MB files without OOM
- [ ] TUI remains responsive during parsing (async boundaries)
- [ ] Step-through execution feels instant (<50ms per step)

### User Experience Success
- [ ] First-time user can parse CLI output in <5 minutes
- [ ] Template author can debug failing template in TUI in <10 minutes
- [ ] Error messages point to exact problem with suggested fix
- [ ] Documentation covers 90% of common use cases

### Adoption Success
- [ ] Python TextFSM users can migrate without rewriting templates
- [ ] New users prefer YAML format over TextFSM DSL
- [ ] Community contributes new templates in YAML format
- [ ] GitHub stars > 100 within 2 months of launch

---

## Open Questions for Phase-Specific Research

These questions may require deeper investigation during implementation:

### Phase 1: Engine
- What's the optimal circular buffer size for streaming mode?
- Should we use `RegexSet` or individual `Regex` instances?
- How to make instrumentation zero-cost when disabled?

### Phase 2: TextFSM
- Are there ntc-templates using undocumented TextFSM features?
- What's the migration path for templates using Python-specific regex?
- How to handle templates with implicit behaviors vs explicit?

### Phase 3: TUI
- What's the right balance between history depth and memory usage?
- How to make regex highlighting fast for 100k+ line outputs?
- Should we support multiple input files in one TUI session?

### Phase 4: Modern Formats
- Should variable interpolation support nested expressions?
- What schema format: JSON Schema, custom DSL, or Rust types?
- How to prevent "Norway problem" (YAML type coercion bugs)?

---

## Sources

**TextFSM Documentation (HIGH confidence):**
- [TextFSM Wiki](https://github.com/google/textfsm/wiki/TextFSM)
- [TextFSM template syntax - Python for network engineers](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_syntax.html)
- [TextFSM Continue/Next/Error actions](https://anirudhkamath.github.io/network-automation-blog/notes/textfsm.html)

**NTC Templates (HIGH confidence):**
- [ntc-templates GitHub](https://github.com/networktocode/ntc-templates)
- [Parsing Strategies - NTC Templates](https://networktocode.com/blog/parsing-strategies-ntc-templates/)
- [Leveraging NTC-Templates 2025](https://networktocode.com/blog/leveraging-ntc-templates-for-network-automation-2025-08-08/)

**Ratatui Documentation (HIGH confidence):**
- [Ratatui official site](https://ratatui.rs)
- [Ratatui Architecture Patterns](https://ratatui.rs/concepts/architecture/)

**Performance & Profiling (HIGH confidence):**
- [Rust Performance Book - Profiling](https://nnethercote.github.io/perf-book/profiling.html)
- [Flamegraph for Rust](https://github.com/flamegraph-rs/flamegraph)
- [How to Profile Rust Applications 2026](https://oneuptime.com/blog/post/2026-02-03-rust-profiling/view)

**Configuration Formats (MEDIUM confidence):**
- [JSON vs YAML vs TOML 2026](https://devtoolbox.dedyn.io/blog/json-vs-yaml-vs-toml)
- [YAML Schema Validation](https://json-schema-everywhere.github.io/yaml)
- [TOML Schema Validation](https://json-schema-everywhere.github.io/toml)

**TUI Best Practices (MEDIUM confidence):**
- [TUI Debugging with GDB](https://sourceware.org/gdb/current/onlinedocs/gdb.html/TUI.html)
- [Debug TUI Blog](https://www.dantleech.com/blog/2025/05/11/debug-tui/)
- [Dev Process Tracker TUI](https://www.techedubyte.com/dev-process-tracker-cli-tui-service-management-debugging/)

**UX Design Patterns 2026 (LOW confidence - general principles):**
- [UI/UX Design Trends 2026](https://www.index.dev/blog/ui-ux-design-trends)
- [Progressive Disclosure Patterns](https://www.onething.design/post/b2b-saas-ux-design)

# Domain Pitfalls: CLI Parsing Tools (v0.1 Alpha Focus)

**Domain:** CLI Parsing / FSM-based Text Processing
**Researched:** 2026-02-17
**Confidence:** MEDIUM-HIGH
**Scope:** v0.1 Alpha milestone (4 phases: Engine, TextFSM, TUI, Modern Frontends)

---

## Phase 1 Pitfalls: FSM Engine & IR Design

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
- [Filldown edge case discussion](https://github.com/google/textfsm/wiki/TextFSM)

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

### High: Zero-Copy IR with Lifetime Complications

**What goes wrong:**
Attempting zero-copy design with `Cow<'a, str>` for performance, but Filldown requires values to outlive the current line's lifetime, breaking the borrow checker.

**Why it happens:**
Filldown means "value from line 10 must persist when processing line 50." With zero-copy, line 10's buffer may have been recycled, invalidating the reference.

**How to avoid:**
- Use `Cow<'static, str>` or owned `String` for Filldown values specifically
- Zero-copy only for non-Filldown values that are recorded immediately
- Alternative: arena allocator for all captures within a single parse run, cleared at end

**Warning signs:**
- Borrow checker errors around Filldown value storage
- Intermittent corruption of Filldown values (use-after-free in unsafe code)
- Performance gains from zero-copy are negligible if most values are Filldown

**Phase to address:** Phase 1 (IR + Engine design)

**Sources:**
- [Zero-copy parsing in Rust](https://medium.com/@chopra.kanta.73/zero-copy-parsers-rust-pipelines-that-outrun-json-7db2a5644db3)
- [Rust performance optimization](https://llogiq.github.io/2017/06/01/perf-pitfalls.html)

---

## Phase 2 Pitfalls: TextFSM Frontend Compatibility

### Critical: Regex Syntax Translation Errors

**What goes wrong:**
Python regex uses different syntax/semantics than Rust. Named groups have different syntax. Character classes differ. Unicode handling differs.

**Why it happens:**
Quote from research: "The Rust engine operates on byte offsets in the given search text, while Python operates on Unicode code points."

**How to avoid:**
- Build a comprehensive translation layer that converts Python regex to Rust regex syntax
- Known translations:
  - Python `(?P<name>...)` → Rust `(?P<name>...)` (same, but verify)
  - Python `\d` in ASCII mode vs Unicode mode
  - Python backreferences `\1` → requires fancy-regex `\1`
- Test suite: ntc-templates corpus against both Python TextFSM and cliscrape
- Fail with clear error message if untranslatable regex detected

**Warning signs:**
- Template loads but matches fail immediately
- Regex compilation errors mentioning syntax issues
- Character offsets don't align (Python uses code points, Rust uses bytes)

**Phase to address:** Phase 2 (TextFSM frontend)

**Sources:**
- [Python regex vs Rust compatibility](https://users.rust-lang.org/t/regex-slower-than-in-python-im-probably-doing-it-wrong/94408)
- [Offset handling differences](https://github.com/spyoungtech/regexrs)

---

### Critical: TextFSM Version Drift (escaped parentheses)

**What goes wrong:**
Quote from research: "TextFSM released an update that may affect some of the templates. Specifically, an error was raised indicating that escaped parentheses in regex patterns must be contained within a '()' pair."

Older ntc-templates may use `\(` and `\)` bare, which newer TextFSM versions reject.

**How to avoid:**
- Test against multiple TextFSM versions' behavior (0.4.x, 1.1.x)
- Support both old and new escaping styles
- Emit warning if template uses deprecated syntax, suggest fix
- Document which TextFSM version semantics cliscrape targets (recommend: match latest 1.1.x)

**Warning signs:**
- Specific ntc-templates fail with "invalid escape" errors
- Templates work in older TextFSM Python but not cliscrape
- Parentheses in regexes cause parse failures

**Phase to address:** Phase 2 (TextFSM frontend)

**Sources:**
- [TextFSM 1.1.2 changes](https://github.com/networktocode/ntc-templates/issues/958)
- [ntc-templates issues](https://github.com/networktocode/ntc-templates/issues)

---

### High: Implicit vs Explicit Type Coercion in Values

**What goes wrong:**
TextFSM treats all captures as strings. Attempting to add type inference (Integer, IP address) in the IR can cause compatibility breaks if the regex doesn't actually match the type.

**Example:** Value definition says type is Integer, regex is `\S+`, input contains "N/A" → type coercion fails.

**How to avoid:**
- Phase 2 (TextFSM): NO type coercion, everything is a string (100% compatibility)
- Phase 4 (Modern YAML/TOML): Explicit types with validation, fail fast on mismatch
- Document the difference clearly

**Warning signs:**
- Templates that work in Python TextFSM fail in cliscrape with type errors
- Unexpected `None` or parse failures on valid input

**Phase to address:** Phase 2 (TextFSM compatibility) + Phase 4 (Modern format types)

---

### High: ntc-templates Corpus Edge Cases

**What goes wrong:**
Quote from research: "Aruba Interface Parsing Failure: executing aruba_aoscx_show_interface.textfsm against devices which contain at least one interface assigned to a VRF, resulting in parsing failures."

Real-world templates in ntc-templates have device-specific quirks and edge cases.

**How to avoid:**
- Clone ntc-templates repository as test suite
- Run cliscrape against every template with sample data
- Track compatibility: "N of M templates pass"
- File GitHub issues for templates that don't work, document known failures
- Target: 95%+ ntc-templates compatibility

**Warning signs:**
- Templates from ntc-templates work in Python but not cliscrape
- Specific vendor formats (Aruba, Juniper, etc.) consistently fail

**Phase to address:** Phase 2 (TextFSM frontend + comprehensive testing)

**Sources:**
- [ntc-templates repository](https://github.com/networktocode/ntc-templates)
- [Aruba VRF parsing failure](https://github.com/networktocode/ntc-templates/issues/2252)

---

## Phase 3 Pitfalls: TUI Architecture & Performance

### Critical: Blocking UI Thread with Synchronous Parsing

**What goes wrong:**
Running the FSM engine on the same thread as the Ratatui render loop. Large files cause UI to freeze for seconds/minutes.

**Why it happens:**
Ratatui uses immediate-mode rendering, expecting frequent re-draws. Long-running parse blocks `terminal.draw()` calls.

**How to avoid:**
- Parse in dedicated worker thread: `std::thread::spawn` or `tokio::task::spawn_blocking`
- Send progress updates to UI via channel: `crossbeam::channel` or `tokio::sync::mpsc`
- UI polls channel and updates display without blocking on parse completion
- Show progress: "Line 1234 / 5000 (24%)"

**Warning signs:**
- TUI unresponsive to keyboard input during parsing
- No way to cancel long-running parse
- Terminal doesn't update until parse completes

**Phase to address:** Phase 3 (TUI architecture)

**Sources:**
- [Ratatui async patterns](https://ratatui.rs/concepts/async/)
- [Rust TUI performance](https://blog.logrocket.com/rust-and-tui-building-a-command-line-interface-in-rust/)

---

### Critical: Unbounded Memory in State Trace History

**What goes wrong:**
Quote from earlier research: "Storing the entire history of the state machine transitions in a `Vec<State>` without limits. Tool crashes on large outputs, precisely when the debugger is most needed for complex parsing."

**Why it happens:**
TUI "time-travel" debugging stores every state transition, match, and value capture. 50MB BGP table = millions of events = OOM.

**How to avoid:**
- Implement ring buffer with configurable size (default: last 10,000 events)
- Streaming mode: only keep current state + last N lines of context
- "Pin" feature: user can mark specific lines to keep in history
- Compression: store diffs, not full state snapshots

**Warning signs:**
- Memory usage grows linearly with input size
- TUI crashes on files > 10MB
- Profiler shows most memory in trace storage

**Phase to address:** Phase 3 (TUI architecture)

**Sources:**
- Identified in previous domain research

---

### High: TUI Rendering Performance with High-Frequency Updates

**What goes wrong:**
Sending state updates to UI for every line processed causes excessive render calls. Ratatui render is cheap but not free; 100K updates/sec thrashes the terminal.

**How to avoid:**
- Batch updates: worker thread accumulates 100 state changes, sends one UI update
- Rate limiting: max 60 UI updates/sec (16ms debounce)
- Progress bar for bulk parsing, detailed trace only when user pauses/steps through

**Warning signs:**
- Terminal flickers or shows corruption during fast parsing
- CPU usage of TUI thread rivals parser thread
- Terminal emulator struggles to keep up

**Phase to address:** Phase 3 (TUI architecture)

**Sources:**
- [Ratatui performance considerations](https://docs.rs/tui/latest/tui/)
- [TUI rendering optimization](https://www.w3resource.com/rust-tutorial/rust-terminal-ui-tutorial.php)

---

### Medium: TUI State Diagram Visualization Clutter

**What goes wrong:**
Complex templates have dozens of states. Rendering full state graph in TUI is unreadable.

**How to avoid:**
- Show only current state + immediate neighbors
- Highlight active state, dim others
- Minimap view for full graph, detail view for current context
- Option to collapse unused states

**Warning signs:**
- TUI state diagram is incomprehensible for real templates
- Users can't understand current execution context

**Phase to address:** Phase 3 (TUI design)

---

## Phase 4 Pitfalls: Modern Frontends (YAML/TOML)

### Critical: The "Norway Problem" (YAML Implicit Typing)

**What goes wrong:**
Quote from research: "YAML's data model has strong, implicit typing where the parser guesses the type tag in a process called 'tag resolution,' which leads to edge cases like the 'Norway problem' present in YAML 1.1 and earlier."

Unquoted values get coerced: `NO` → `false`, `08` → octal error, `1e10` → float `10000000000.0`

**Why it happens:**
Standard YAML parsers (like `serde_yaml`) apply implicit type resolution to unquoted scalars.

**How to avoid:**
- Force all template fields to be explicitly typed or quoted strings
- Use schema validation: `values.interface` must be type `string`, reject if parser sees bool/float
- Emit warning: "Value 'NO' interpreted as boolean false — did you mean string 'NO'?"
- Consider TOML instead (less implicit typing), or StrictYAML approach

**Warning signs:**
- Template loads but values have wrong types
- Serial numbers become floats
- Boolean values appear where strings expected

**Phase to address:** Phase 4 (YAML/TOML frontend)

**Sources:**
- [YAML Norway problem](https://hitchdev.com/strictyaml/why/implicit-typing-is-evil/)
- [YAML implicit typing issues](https://www.yamllint.com/)

---

### High: TOML Schema Validation Limitations

**What goes wrong:**
Quote from research: "When validating TOML with JSON Schema, floating point NaN and infinity values cannot be validated as standard float fields and need to be encoded as dual type ["float", "string"] fields instead."

TOML has no representation for null values in arrays: "TOML has no representation that allows arrays to contain null values, with the closest equivalent being 'omit that key.'"

**How to avoid:**
- Don't use TOML if schema requires nullable array elements
- For cliscrape: probably not an issue (FSM templates don't use complex nullability)
- If using JSON Schema validation, document TOML limitations upfront

**Warning signs:**
- Schema validation fails on valid TOML
- Edge cases with special float values (NaN, Infinity)

**Phase to address:** Phase 4 (if TOML is chosen)

**Sources:**
- [TOML JSON Schema validation](https://github.com/toml-lang/toml/discussions/1038)

---

### High: YAML/TOML Ergonomics vs TextFSM Compatibility

**What goes wrong:**
Modern YAML format is more readable but can't express certain TextFSM idioms cleanly. Developers create "better" syntax that lacks feature parity.

**Example:** TextFSM allows regex with embedded value references: `^Interface ${interface} is ${status}`. YAML version needs different syntax, possibly less intuitive.

**How to avoid:**
- Design YAML format to be SUPERSET of TextFSM capabilities
- Provide migration tool: convert TextFSM → YAML
- Document feature mapping: TextFSM concept X = YAML syntax Y
- Don't compromise on features for aesthetics

**Warning signs:**
- Users can't express complex templates in YAML that work in TextFSM
- Migration tool fails on real templates
- YAML format requires multiple files where TextFSM used one

**Phase to address:** Phase 4 (Modern format design)

---

## Cross-Cutting Pitfalls

### Critical: Testing Strategy - Insufficient Real-World Corpus

**What goes wrong:**
Testing against synthetic examples or a few hand-written samples. Real TextFSM templates from ntc-templates have bizarre edge cases and quirks.

**How to avoid:**
Quote from research: "Ensuring that the TextFSM template can account for every line is the only method to ensure that data was not accidentally missed."

- CI integration: Run full ntc-templates corpus on every commit
- Track coverage: "N of M templates pass all tests"
- Regression tests: When fixing a bug, add that template to permanent test suite
- Test data: include real device outputs (sanitized), not just synthetic examples

**Warning signs:**
- Tests pass but ntc-templates fail
- User reports template works in Python TextFSM but not cliscrape
- No quantitative measure of TextFSM compatibility

**Phase to address:** All phases (continuous integration)

**Sources:**
- [ntc-templates testing](https://ntc-templates.readthedocs.io/en/latest/dev/dev_parser/)
- [Parser testing strategies](https://pythontest.com/testing-argparse-apps/)

---

### Critical: Performance - Regex Compilation in Hot Loop

**What goes wrong:**
Quote from research: "Compiling `Regex::new()` inside a loop for every line of CLI output... Significant performance degradation (10x-100x slower) on large files."

**How to avoid:**
- Compile all regexes once at template load time
- Store compiled regexes in template IR: `struct Rule { compiled_regex: Regex, ... }`
- Use `RegexSet` for fast multi-pattern dispatch if possible
- Profile early: flamegraph should show zero time in `regex::compile` during parse

**Warning signs:**
- Parsing is 10x+ slower than expected
- Flamegraph shows significant time in regex compilation
- Performance degrades linearly with number of rules

**Phase to address:** Phase 1 (Engine) + all phases (profiling)

**Sources:**
- [Rust regex performance](https://docs.rs/regex/latest/regex/#performance)
- [Rust performance pitfalls](https://llogiq.github.io/2017/06/01/perf-pitfalls.html)

---

### High: Concurrent Parsing - Arc/Mutex Deadlock Risk

**What goes wrong:**
Making FSM engine thread-safe with `Arc<Mutex<State>>`, but complex locking order leads to deadlocks under concurrent load.

**Why it happens:**
Quote from research: "Prevent deadlocks by acquiring locks in consistent order."

**How to avoid:**
- Design FSM engine to be immutable after template load: no shared mutable state
- Each parse run gets its own mutable state (not shared): no locks needed
- If sharing is required (e.g., template cache), use `Arc` for immutable template, no `Mutex`
- Rule: locks only at coarse boundaries (e.g., template registry), never within parse loop

**Warning signs:**
- Concurrent parse tests hang
- Deadlock detection tools fire
- Performance doesn't scale with threads

**Phase to address:** Phase 1 (Engine architecture) + continuous testing

**Sources:**
- [Rust thread safety](https://oneuptime.com/blog/post/2026-02-01-rust-safe-multithreading/view)
- [Rust concurrency pitfalls](https://codezup.com/mastering-rust-concurrency-thread-safe-data-structures/)

---

### High: Allocation Performance - String Copying Overhead

**What goes wrong:**
Quote from research: "Every time your parser creates a new string, every duplicate of data already sitting in your buffer, every unnecessary copy operation compounds into a performance tax."

Naive implementation allocates new String for every captured value.

**How to avoid:**
- Use `Cow<'a, str>` where possible (but see Filldown lifetime issue above)
- Arena allocator for single parse run: all captures allocated from arena, freed in bulk at end
- Benchmark: measure allocations with `dhat` or similar tool
- Target: zero allocations per line in steady state (amortized)

**Warning signs:**
- Profiler shows significant time in allocator
- Memory usage spikes during parse
- Performance degrades on large files despite linear algorithm

**Phase to address:** Phase 1 (Engine optimization) + continuous profiling

**Sources:**
- [Zero-copy parsing strategies](https://medium.com/@chopra.kanta.73/zero-copy-parsers-rust-pipelines-that-outrun-json-7db2a5644db3)
- [Rust allocation performance](https://www.rapidinnovation.io/post/performance-optimization-techniques-in-rust)

---

### Medium: API Design - Overly Generic IR Complicates Engine

**What goes wrong:**
Attempting to design IR that supports TextFSM + YAML + future formats leads to abstraction bloat. Engine code is full of `match format_type` branches.

**How to avoid:**
- Design IR for TextFSM semantics specifically (Phase 1-2)
- YAML/TOML frontends (Phase 4) translate to same IR — no engine changes needed
- If future format needs different semantics, extend IR minimally or create separate engine
- Prefer "compilation" model: YAML → TextFSM IR → parse, not "interpretation" model

**Warning signs:**
- IR has fields that only one format uses
- Engine has format-specific branches
- Adding new format requires engine changes (should only require new frontend)

**Phase to address:** Phase 1 (IR design) + Phase 4 (format frontends)

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

---

## Pitfall-to-Phase Mapping

| Pitfall | Priority | Phase | Verification |
|---------|----------|-------|--------------|
| TextFSM action semantics | Critical | Phase 1 | Test suite: all action combinations |
| Filldown state persistence | Critical | Phase 1 | ntc-templates corpus passes |
| Required values silent failure | Critical | Phase 1+3 | TUI shows diagnostic warnings |
| Regex engine abstraction | High | Phase 1+2 | Feature tagging in IR |
| Zero-copy lifetimes | High | Phase 1 | Benchmark shows <10 allocations/line |
| Python regex translation | Critical | Phase 2 | ntc-templates 95%+ pass rate |
| TextFSM version drift | Critical | Phase 2 | Test against v0.4 and v1.1 |
| ntc-templates edge cases | High | Phase 2 | Comprehensive corpus testing |
| Blocking UI thread | Critical | Phase 3 | TUI responsive during 10s parse |
| Unbounded trace memory | Critical | Phase 3 | Ring buffer implementation |
| TUI render performance | High | Phase 3 | 60fps maintained during parse |
| YAML implicit typing | Critical | Phase 4 | Schema validation catches type errors |
| TOML limitations | High | Phase 4 | Document unsupported features |
| YAML/TextFSM parity | High | Phase 4 | Migration tool converts all templates |
| Regex compilation hot loop | Critical | All | Flamegraph shows zero compile time |
| Arc/Mutex deadlocks | High | Phase 1 | Concurrent parse stress tests |
| String allocation overhead | High | Phase 1 | < 10 allocations/line measured |

---

## Recovery Strategies

When pitfalls occur despite prevention:

| Pitfall | Detection | Recovery Cost | Recovery Steps |
|---------|-----------|---------------|----------------|
| Action semantics bug | Test failures | MEDIUM | Add unit test for specific combination, fix engine logic |
| Filldown spurious record | Off-by-one record count | LOW | Add empty-row detection logic |
| Required field silent failure | Empty results | LOW | Add diagnostic mode, emit warning |
| Regex incompatibility | Match failures | MEDIUM | Add to translation layer, document unsupported patterns |
| UI thread blocking | Frozen TUI | HIGH | Refactor to worker thread (architectural change) |
| Unbounded memory | OOM crash | MEDIUM | Implement ring buffer (should be done in Phase 3) |
| YAML type coercion | Wrong output types | MEDIUM | Add schema validation, reject ambiguous inputs |
| Regex hot loop compilation | 10x+ slowdown | LOW | Move compilation to template load (easy fix) |
| Deadlock | Hang under concurrency | HIGH | Redesign locking strategy (architectural) |
| Excessive allocation | Slow performance | MEDIUM | Profile, identify hot paths, optimize with arena/Cow |

---

## "Looks Done But Isn't" Checklist

- [ ] **TextFSM compatibility:** Passes test suite with synthetic examples — verify against ntc-templates corpus (200+ templates)
- [ ] **Regex handling:** Simple patterns work — verify backreferences, lookahead, Unicode edge cases
- [ ] **TUI responsiveness:** Works on small files — verify 50MB+ file doesn't freeze UI
- [ ] **Memory usage:** No leaks in short runs — verify long-running TUI session doesn't grow unbounded
- [ ] **Concurrent parsing:** Basic threading works — verify no deadlocks under stress test (1000+ concurrent parses)
- [ ] **YAML format:** Loads and displays — verify schema validation catches type coercion bugs
- [ ] **Performance:** Faster than Python on benchmarks — verify no regex recompilation in hot loop (flamegraph)
- [ ] **Filldown semantics:** Basic filldown works — verify doesn't emit spurious final record

---

## Sources

**TextFSM Semantics & Edge Cases:**
- [TextFSM Wiki](https://github.com/google/textfsm/wiki/TextFSM)
- [TextFSM Examples](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_examples.html)
- [TextFSM Syntax](https://pyneng.readthedocs.io/en/latest/book/21_textfsm/textfsm_syntax.html)
- [ntc-templates repository](https://github.com/networktocode/ntc-templates)
- [ntc-templates issues](https://github.com/networktocode/ntc-templates/issues)
- [Aruba VRF parsing failure](https://github.com/networktocode/ntc-templates/issues/2252)
- [TextFSM 1.1.2 changes](https://github.com/networktocode/ntc-templates/issues/958)

**Regex Engine Compatibility:**
- [Rust regex crate documentation](https://docs.rs/regex/latest/regex/)
- [fancy-regex documentation](https://docs.rs/fancy-regex/)
- [Rust regex lookaround discussion](https://github.com/rust-lang/regex/discussions/910)
- [Python vs Rust regex compatibility](https://users.rust-lang.org/t/regex-slower-than-in-python-im-probably-doing-it-wrong/94408)
- [regexrs Python wrapper](https://github.com/spyoungtech/regexrs)

**TUI Architecture & Performance:**
- [Ratatui documentation](https://ratatui.rs/)
- [Ratatui async patterns](https://ratatui.rs/concepts/async/)
- [Rust TUI guide](https://blog.logrocket.com/rust-and-tui-building-a-command-line-interface-in-rust/)
- [TUI performance](https://www.w3resource.com/rust-tutorial/rust-terminal-ui-tutorial.php)

**YAML/TOML Format Issues:**
- [YAML Norway problem](https://hitchdev.com/strictyaml/why/implicit-typing-is-evil/)
- [YAML schema validation](https://json-schema-everywhere.github.io/yaml)
- [TOML JSON Schema validation](https://github.com/toml-lang/toml/discussions/1038)
- [YAML linting and validation](https://www.mavjs.org/post/yaml-linting-schema-validation/)

**Performance & Optimization:**
- [Rust performance pitfalls](https://llogiq.github.io/2017/06/01/perf-pitfalls.html)
- [Zero-copy parsing strategies](https://medium.com/@chopra.kanta.73/zero-copy-parsers-rust-pipelines-that-outrun-json-7db2a5644db3)
- [Rust performance optimization guide](https://www.rapidinnovation.io/post/performance-optimization-techniques-in-rust)
- [Rust profiling guide](https://oneuptime.com/blog/post/2026-02-03-rust-profiling/view)

**Concurrency & Thread Safety:**
- [Rust thread safety guide](https://oneuptime.com/blog/post/2026-02-01-rust-safe-multithreading/view)
- [Rust concurrency patterns](https://codezup.com/mastering-rust-concurrency-thread-safe-data-structures/)
- [Rust concurrency chapter](https://doc.rust-lang.org/book/ch16-00-concurrency.html)

**Testing Strategies:**
- [Parser testing](https://pythontest.com/testing-argparse-apps/)
- [ntc-templates testing guide](https://ntc-templates.readthedocs.io/en/latest/dev/dev_parser/)

---

*Pitfalls research for: cliscrape v0.1 Alpha*
*Researched: 2026-02-17*
*Confidence: MEDIUM-HIGH (verified with web research + domain knowledge)*

# Technology Stack

**Project:** cliscrape
**Researched:** 2024-11-21
**Overall Confidence:** HIGH

## Recommended Stack

### Core Framework & Runtime
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| [Rust](https://www.rust-lang.org/) | 1.82+ | Language | Performance, safety, and excellent ecosystem for CLI/TUI tools. |
| [Tokio](https://tokio.rs/) | 1.41 | Async Runtime | Industry standard for high-performance async I/O, required for scaling SSH connections. |

### Connectivity
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| [russh](https://docs.rs/russh/latest/russh/) | 0.45 | SSH Client | Pure Rust, async-native, avoids C-FFI issues of `libssh2`. Better for high-concurrency scraping. |

### Parsing
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| [textfsm-rust](https://docs.rs/textfsm-rust/latest/textfsm_rust/) | 0.3 | Parsing | 99%+ compatibility with `ntc-templates`. Supports Python regex quirks and Serde integration. |
| [Serde](https://serde.rs/) | 1.0 | Serialization | De-facto standard for mapping parsed CLI data to typed Rust structs. |
| [serde_yml](https://docs.rs/serde_yml/latest/serde_yml/) | 0.0.10 | YAML Format | Modern, maintained fork of `serde_yaml` for hybrid configuration. |
| [toml](https://docs.rs/toml/latest/toml/) | 0.8 | TOML Format | Standard Rust configuration format, part of the hybrid YAML/TOML requirement. |

### TUI & Debugging
| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| [Ratatui](https://ratatui.rs/) | 0.29 | TUI Framework | Modern, actively maintained fork of `tui-rs`. Best-in-class for live-tracing interfaces. |
| [ratatui-textarea](https://docs.rs/ratatui-textarea/latest/ratatui_textarea/) | 0.6 | Editor Widget | Lightweight multi-line editor for the live-editing requirement in the TUI debugger. |
| [tui-logger](https://docs.rs/tui-logger/latest/tui_logger/) | 0.13 | State Tracing | Native support for `tracing` subscriber to display live logs/state-traces in a TUI widget. |
| [Tracing](https://tokio.rs/blog/2019-08-tracing) | 0.1 | Instrumentation | Better for structured diagnostic data than the standard `log` crate. |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| SSH | `russh` | `ssh2-rs` | `ssh2` relies on `libssh2` (C). Friction with async/tokio and harder to cross-compile. |
| Parsing | `textfsm-rust` | `nom` | `nom` is great for new protocols but the requirement is TextFSM compatibility. |
| TUI Editor | `ratatui-textarea` | `edtui` | `edtui` is more powerful (Vim-like) but might be overkill for a simple debugger. |

## Installation

```bash
# Core dependencies
cargo add tokio --features full
cargo add russh textfsm-rust serde --features derive
cargo add serde_yml toml ratatui ratatui-textarea tui-logger tracing tracing-subscriber
```

## Sources
- [TextFSM-Rust GitHub & Crates.io](https://crates.io/crates/textfsm-rust) (High Confidence)
- [Russh Documentation](https://docs.rs/russh/latest/russh/) (High Confidence)
- [Ratatui Ecosystem Gallery](https://ratatui.rs/ecosystem/) (High Confidence)
- Web Research: "Rust SSH network automation 2025" (Medium Confidence)

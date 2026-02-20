use crate::tui::app::AppState;
use anyhow::Context;
use cliscrape::FsmParser;
use crossterm::{
    cursor, event as crossterm_event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub mod app;
pub mod event;
pub mod watch;
pub mod ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsWhich {
    Template,
    Input,
}

#[derive(Debug, Clone)]
pub enum Message {
    FsChanged { which: FsWhich },
}

pub fn run_debugger(template: Option<PathBuf>, input: Option<PathBuf>) -> anyhow::Result<()> {
    let mut app = AppState::new(template.clone(), input.clone());

    match (template, input) {
        (Some(template_path), Some(input_path)) => {
            let parser = FsmParser::from_file(&template_path)
                .with_context(|| format!("Failed to load template from {:?}", template_path))?;

            let input_content = std::fs::read_to_string(&input_path)
                .with_context(|| format!("Failed to read input from {:?}", input_path))?;
            let blocks = crate::transcript::preprocess_ios_transcript(&input_content);
            let block = blocks.first().map(|s| s.as_str()).unwrap_or(&input_content);

            let report = parser
                .debug_parse(block)
                .with_context(|| "Failed to debug-parse input")?;
            app.set_debug_report(report);
        }
        _ => {
            let mut msg: Vec<String> = Vec::new();
            msg.push("cliscrape debug".to_string());
            msg.push("".to_string());
            msg.push("Missing required paths (picker will be added in a later plan).".to_string());
            msg.push(format!(
                "template: {}",
                app.template_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "<missing>".to_string())
            ));
            msg.push(format!(
                "input:    {}",
                app.input_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "<missing>".to_string())
            ));
            msg.push("".to_string());
            msg.push("Usage:".to_string());
            msg.push("  cliscrape debug --template <PATH> --input <PATH>".to_string());
            app.lines = msg;
        }
    }

    run(app)
}

pub fn run(mut app: AppState) -> anyhow::Result<()> {
    let _cleanup = TerminalCleanup;

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let tick_rate = Duration::from_millis(100);
    let exit_after = std::env::var("CLISCRAPE_TUI_EXIT_AFTER_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_millis);
    let started = Instant::now();

    let res = (|| -> anyhow::Result<()> {
        loop {
            terminal.draw(|f| ui::draw(f, &app))?;

            if let Some(d) = exit_after {
                if started.elapsed() >= d {
                    break;
                }
            }

            if crossterm_event::poll(tick_rate)? {
                let ev = crossterm_event::read()?;
                if let Some(action) = crate::tui::event::action_from_event(ev) {
                    match action {
                        crate::tui::event::Action::Quit => break,
                        crate::tui::event::Action::CursorUp => app.cursor_up(),
                        crate::tui::event::Action::CursorDown => app.cursor_down(),
                    }
                }
            }
        }

        Ok(())
    })();

    let _ = terminal.show_cursor();
    res
}

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
    }
}

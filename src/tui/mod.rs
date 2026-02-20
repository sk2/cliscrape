use crate::tui::app::AppState;
use crossterm::{
    cursor, event as crossterm_event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

pub mod app;
pub mod event;
pub mod ui;

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

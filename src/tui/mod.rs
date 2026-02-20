use crate::tui::app::AppState;
use crossterm::{
    cursor, event as crossterm_event, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, Instant};

pub mod app;
pub mod event;
pub mod ui;
pub mod watch;
pub mod worker;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FsWhich {
    Template,
    Input,
}

#[derive(Debug)]
pub enum Message {
    Quit,
    CursorUp,
    CursorDown,
    ToggleView,
    MatchPrev,
    MatchNext,
    FsChanged { which: FsWhich },
    ParseDone(cliscrape::DebugReport),
    ParseError(String),
}

pub fn run_debugger(template: Option<PathBuf>, input: Option<PathBuf>) -> anyhow::Result<()> {
    let app = AppState::new(template, input);
    run(app)
}

pub fn run(mut app: AppState) -> anyhow::Result<()> {
    let _cleanup = TerminalCleanup;

    let (msg_tx, msg_rx) = mpsc::channel::<Message>();
    let worker = worker::ParseWorker::start(msg_tx.clone());

    let mut watcher: Option<watch::WatcherHandle> = None;
    if let (Some(tpl), Some(inp)) = (app.template_path.clone(), app.input_path.clone()) {
        watcher = Some(watch::start_watcher(
            tpl.clone(),
            inp.clone(),
            msg_tx.clone(),
        )?);
        app.on_parse_started();
        worker.request(worker::ParseRequest {
            template_path: tpl,
            input_path: inp,
            block_idx: 0,
        });
    }

    enable_raw_mode()?;
    execute!(io::stdout(), EnterAlternateScreen, cursor::Hide)?;

    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let tick_rate = Duration::from_millis(50);
    let exit_after = std::env::var("CLISCRAPE_TUI_EXIT_AFTER_MS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_millis);
    let started = Instant::now();

    let res = (|| -> anyhow::Result<()> {
        loop {
            while let Ok(msg) = msg_rx.try_recv() {
                if handle_message(&mut app, msg, &worker) {
                    return Ok(());
                }
            }

            terminal.draw(|f| ui::draw(f, &app))?;

            if let Some(d) = exit_after {
                if started.elapsed() >= d {
                    break;
                }
            }

            if crossterm_event::poll(tick_rate)? {
                let ev = crossterm_event::read()?;
                if let Some(msg) = crate::tui::event::message_from_event(ev) {
                    if handle_message(&mut app, msg, &worker) {
                        break;
                    }
                }
            }
        }

        Ok(())
    })();

    let _ = terminal.show_cursor();

    drop(watcher);
    drop(worker);
    res
}

fn handle_message(app: &mut AppState, msg: Message, worker: &worker::ParseWorker) -> bool {
    match msg {
        Message::Quit => true,
        Message::CursorUp => {
            app.cursor_up();
            false
        }
        Message::CursorDown => {
            app.cursor_down();
            false
        }
        Message::ToggleView => {
            app.toggle_view_mode();
            false
        }
        Message::MatchPrev => {
            app.selected_match_prev();
            false
        }
        Message::MatchNext => {
            app.selected_match_next();
            false
        }
        Message::FsChanged { .. } => {
            let (Some(tpl), Some(inp)) = (app.template_path.clone(), app.input_path.clone()) else {
                return false;
            };
            app.on_parse_started();
            worker.request(worker::ParseRequest {
                template_path: tpl,
                input_path: inp,
                block_idx: 0,
            });
            false
        }
        Message::ParseDone(report) => {
            app.on_parse_done(report);
            false
        }
        Message::ParseError(err) => {
            app.on_parse_error(err);
            false
        }
    }
}

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
    }
}

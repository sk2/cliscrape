use crate::tui::app::{AppState, Mode};
use crossterm::event::{KeyCode, KeyModifiers};
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
pub mod editor;
pub mod event;
pub mod picker;
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
    Key(crossterm::event::KeyEvent),
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
                if handle_message(&mut app, msg, &worker, &msg_tx, &mut watcher) {
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
                    if handle_message(&mut app, msg, &worker, &msg_tx, &mut watcher) {
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

fn handle_message(
    app: &mut AppState,
    msg: Message,
    worker: &worker::ParseWorker,
    msg_tx: &mpsc::Sender<Message>,
    watcher: &mut Option<watch::WatcherHandle>,
) -> bool {
    match msg {
        Message::Key(key) => handle_key(app, key, worker, msg_tx, watcher),
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

fn handle_key(
    app: &mut AppState,
    key: crossterm::event::KeyEvent,
    worker: &worker::ParseWorker,
    msg_tx: &mpsc::Sender<Message>,
    watcher: &mut Option<watch::WatcherHandle>,
) -> bool {
    // Safety exits.
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && matches!(key.code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return true;
    }

    match app.mode {
        Mode::Picker => handle_key_picker(app, key, worker, msg_tx, watcher),
        Mode::Browse => handle_key_browse(app, key),
        Mode::EditTemplate => handle_key_editor(app, key, worker),
    }
}

fn handle_key_picker(
    app: &mut AppState,
    key: crossterm::event::KeyEvent,
    worker: &worker::ParseWorker,
    msg_tx: &mpsc::Sender<Message>,
    watcher: &mut Option<watch::WatcherHandle>,
) -> bool {
    if matches!(key.code, KeyCode::Char('q')) {
        return true;
    }

    let Some(picker) = app.picker.as_mut() else {
        return false;
    };

    if matches!(key.code, KeyCode::Tab) {
        picker.set_target(match picker.target {
            crate::tui::picker::PickTarget::Template => crate::tui::picker::PickTarget::Input,
            crate::tui::picker::PickTarget::Input => crate::tui::picker::PickTarget::Template,
        });
        return false;
    }

    match picker.apply_key(key) {
        crate::tui::picker::PickerKeyResult::Noop => false,
        crate::tui::picker::PickerKeyResult::Selected(path) => {
            if path.is_dir() {
                picker.cwd = path;
                if let Err(e) = picker.refresh() {
                    picker.last_error = Some(format!("{:#}", e));
                }
                return false;
            }

            if !path.exists() {
                picker.last_error = Some(format!("path does not exist: {}", path.display()));
                return false;
            }
            if !path.is_file() {
                picker.last_error = Some(format!("not a file: {}", path.display()));
                return false;
            }

            match picker.target {
                crate::tui::picker::PickTarget::Template => {
                    app.template_path = Some(path);
                }
                crate::tui::picker::PickTarget::Input => {
                    app.input_path = Some(path);
                }
            }

            if app.template_path.is_none() {
                picker.set_target(crate::tui::picker::PickTarget::Template);
                return false;
            }
            if app.input_path.is_none() {
                picker.set_target(crate::tui::picker::PickTarget::Input);
                return false;
            }

            app.mode = Mode::Browse;
            app.picker = None;
            app.current_error = None;

            if let (Some(tpl), Some(inp)) = (app.template_path.clone(), app.input_path.clone()) {
                // (Re)start watcher.
                *watcher = None;
                match watch::start_watcher(tpl.clone(), inp.clone(), msg_tx.clone()) {
                    Ok(w) => *watcher = Some(w),
                    Err(err) => {
                        app.current_error = Some(format!("{:#}", err));
                    }
                }

                app.on_parse_started();
                worker.request(worker::ParseRequest {
                    template_path: tpl,
                    input_path: inp,
                    block_idx: 0,
                });
            }

            false
        }
    }
}

fn handle_key_browse(app: &mut AppState, key: crossterm::event::KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') => true,
        KeyCode::Up | KeyCode::Char('k') => {
            app.cursor_up();
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.cursor_down();
            false
        }
        KeyCode::Tab => {
            app.toggle_view_mode();
            false
        }
        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('[') => {
            app.selected_match_prev();
            false
        }
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(']') => {
            app.selected_match_next();
            false
        }
        KeyCode::Char('e') => {
            app.enter_edit_template();
            false
        }
        _ => false,
    }
}

fn handle_key_editor(
    app: &mut AppState,
    key: crossterm::event::KeyEvent,
    worker: &worker::ParseWorker,
) -> bool {
    let Some(ed) = app.editor.as_mut() else {
        app.exit_edit_template();
        return false;
    };

    match ed.apply_key(key) {
        crate::tui::editor::EditorKeyResult::Noop => false,
        crate::tui::editor::EditorKeyResult::Exit => {
            app.exit_edit_template();
            false
        }
        crate::tui::editor::EditorKeyResult::Save => {
            match ed.save() {
                Ok(()) => {
                    if let (Some(tpl), Some(inp)) =
                        (app.template_path.clone(), app.input_path.clone())
                    {
                        app.on_parse_started();
                        worker.request(worker::ParseRequest {
                            template_path: tpl,
                            input_path: inp,
                            block_idx: 0,
                        });
                    }
                }
                Err(err) => {
                    ed.last_save_error = Some(format!("{:#}", err));
                }
            }
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

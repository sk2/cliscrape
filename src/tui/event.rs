use crossterm::event::{Event, KeyCode, KeyEventKind};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    CursorUp,
    CursorDown,
}

pub fn action_from_event(ev: Event) -> Option<Action> {
    match ev {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return None;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
                KeyCode::Up | KeyCode::Char('k') => Some(Action::CursorUp),
                KeyCode::Down | KeyCode::Char('j') => Some(Action::CursorDown),
                _ => None,
            }
        }
        _ => None,
    }
}

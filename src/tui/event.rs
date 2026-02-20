use crossterm::event::{Event, KeyCode, KeyEventKind};

pub fn message_from_event(ev: Event) -> Option<crate::tui::Message> {
    match ev {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return None;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => Some(crate::tui::Message::Quit),
                KeyCode::Up | KeyCode::Char('k') => Some(crate::tui::Message::CursorUp),
                KeyCode::Down | KeyCode::Char('j') => Some(crate::tui::Message::CursorDown),
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('[') => {
                    Some(crate::tui::Message::MatchPrev)
                }
                KeyCode::Right | KeyCode::Char('l') | KeyCode::Char(']') => {
                    Some(crate::tui::Message::MatchNext)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

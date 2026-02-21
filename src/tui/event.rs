use crossterm::event::{Event, KeyEventKind};

pub fn message_from_event(ev: Event) -> Option<crate::tui::Message> {
    match ev {
        Event::Key(key) => {
            if key.kind != KeyEventKind::Press {
                return None;
            }

            Some(crate::tui::Message::Key(key))
        }
        _ => None,
    }
}

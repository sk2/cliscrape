use cliscrape::engine::debug::{DebugReport, TraceEvent, TraceEventType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SteppingMode {
    LineByLine,       // Every trace event
    StateByState,     // Only state transitions
    ActionByAction,   // Only Record/Clear actions
}

#[derive(Debug, Clone)]
pub struct FilterState {
    pub show_line_events: bool,
    pub show_state_changes: bool,
    pub show_record_actions: bool,
    pub show_clear_actions: bool,
}

impl Default for FilterState {
    fn default() -> Self {
        Self {
            show_line_events: true,
            show_state_changes: true,
            show_record_actions: true,
            show_clear_actions: true,
        }
    }
}

impl FilterState {
    pub fn matches(&self, event_type: &TraceEventType) -> bool {
        match event_type {
            TraceEventType::LineProcessed => self.show_line_events,
            TraceEventType::StateChange => self.show_state_changes,
            TraceEventType::RecordEmitted => self.show_record_actions,
            TraceEventType::RecordCleared => self.show_clear_actions,
        }
    }
}

use cliscrape::DebugReport;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    Parsing,
    Ok,
    Error,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    Matches,
    Records,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Picker,
    Browse,
    EditTemplate,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub template_path: Option<PathBuf>,
    pub input_path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor_line_idx: usize,
    pub selected_match_idx: usize,
    pub view_mode: ViewMode,
    pub selected_record_idx: usize,
    pub last_good: Option<DebugReport>,
    pub current_error: Option<String>,
    pub status: ParseStatus,
    pub mode: Mode,
    pub editor: Option<crate::tui::editor::EditorState>,
    pub picker: Option<crate::tui::picker::PickerState>,
    pub trace_index: usize,
    pub stepping_mode: crate::tui::trace::SteppingMode,
    pub filter_state: crate::tui::trace::FilterState,
    pub watch_list: HashSet<String>,
}

impl AppState {
    pub fn new(template_path: Option<PathBuf>, input_path: Option<PathBuf>) -> Self {
        let mut lines: Vec<String> = Vec::new();
        lines.push("cliscrape debug".to_string());
        lines.push("".to_string());
        lines.push("Usage:".to_string());
        lines.push("  cliscrape debug --template <PATH> --input <PATH>".to_string());

        let mut mode = Mode::Browse;
        let mut picker: Option<crate::tui::picker::PickerState> = None;
        let mut current_error: Option<String> = None;

        if template_path.is_none() || input_path.is_none() {
            mode = Mode::Picker;
            let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let target = if template_path.is_none() {
                crate::tui::picker::PickTarget::Template
            } else {
                crate::tui::picker::PickTarget::Input
            };

            match crate::tui::picker::PickerState::new(cwd, target) {
                Ok(p) => picker = Some(p),
                Err(err) => {
                    current_error = Some(format!("{:#}", err));
                    mode = Mode::Browse;
                }
            }
        }

        Self {
            template_path,
            input_path,
            lines,
            cursor_line_idx: 0,
            selected_match_idx: 0,
            view_mode: ViewMode::Matches,
            selected_record_idx: 0,
            last_good: None,
            current_error,
            status: ParseStatus::Idle,
            mode,
            editor: None,
            picker,
            trace_index: 0,
            stepping_mode: crate::tui::trace::SteppingMode::LineByLine,
            filter_state: crate::tui::trace::FilterState::default(),
            watch_list: HashSet::new(),
        }
    }

    pub fn enter_edit_template(&mut self) {
        let Some(path) = self.template_path.clone() else {
            self.current_error = Some("no template path set".to_string());
            return;
        };

        match crate::tui::editor::EditorState::open(path) {
            Ok(ed) => {
                self.editor = Some(ed);
                self.mode = Mode::EditTemplate;
                self.current_error = None;
            }
            Err(err) => {
                self.current_error = Some(format!("{:#}", err));
            }
        }
    }

    pub fn exit_edit_template(&mut self) {
        self.mode = Mode::Browse;
    }

    pub fn on_parse_started(&mut self) {
        self.status = ParseStatus::Parsing;
    }

    pub fn on_parse_done(&mut self, report: DebugReport) {
        self.lines = report.lines.clone();
        self.last_good = Some(report);
        self.current_error = None;
        self.status = ParseStatus::Ok;
        self.clamp_cursor();
        self.clamp_selected_match();
        self.clamp_selected_record();
        self.sync_selections();
    }

    pub fn on_parse_error(&mut self, error: String) {
        self.current_error = Some(error);
        self.status = ParseStatus::Error;
        self.clamp_cursor();
        self.clamp_selected_match();
        self.clamp_selected_record();
    }

    pub fn cursor_up(&mut self) {
        self.cursor_line_idx = self.cursor_line_idx.saturating_sub(1);
        self.selected_match_idx = 0;
        if self.view_mode == ViewMode::Records {
            self.sync_record_selection_to_cursor();
        }
    }

    pub fn cursor_down(&mut self) {
        let max = self.lines.len().saturating_sub(1);
        self.cursor_line_idx = (self.cursor_line_idx + 1).min(max);
        self.selected_match_idx = 0;
        if self.view_mode == ViewMode::Records {
            self.sync_record_selection_to_cursor();
        }
    }

    pub fn toggle_view_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Matches => ViewMode::Records,
            ViewMode::Records => ViewMode::Matches,
        };
        self.sync_selections();
    }

    pub fn selected_match_prev(&mut self) {
        match self.view_mode {
            ViewMode::Matches => {
                self.selected_match_idx = self.selected_match_idx.saturating_sub(1);
            }
            ViewMode::Records => {
                self.selected_record_idx = self.selected_record_idx.saturating_sub(1);
                self.sync_cursor_to_selected_record();
            }
        }
    }

    pub fn selected_match_next(&mut self) {
        match self.view_mode {
            ViewMode::Matches => {
                self.selected_match_idx = self.selected_match_idx.saturating_add(1);
                self.clamp_selected_match();
            }
            ViewMode::Records => {
                self.selected_record_idx = self.selected_record_idx.saturating_add(1);
                self.clamp_selected_record();
                self.sync_cursor_to_selected_record();
            }
        }
    }

    fn clamp_cursor(&mut self) {
        if self.lines.is_empty() {
            self.cursor_line_idx = 0;
            return;
        }
        let max = self.lines.len() - 1;
        self.cursor_line_idx = self.cursor_line_idx.min(max);
    }

    fn clamp_selected_match(&mut self) {
        let Some(report) = &self.last_good else {
            self.selected_match_idx = 0;
            return;
        };

        let Some(matches) = report.matches_by_line.get(self.cursor_line_idx) else {
            self.selected_match_idx = 0;
            return;
        };
        if matches.is_empty() {
            self.selected_match_idx = 0;
            return;
        }

        let max = matches.len() - 1;
        self.selected_match_idx = self.selected_match_idx.min(max);
    }

    fn clamp_selected_record(&mut self) {
        let Some(report) = &self.last_good else {
            self.selected_record_idx = 0;
            return;
        };
        if report.records.is_empty() {
            self.selected_record_idx = 0;
            return;
        }
        let max = report.records.len() - 1;
        self.selected_record_idx = self.selected_record_idx.min(max);
    }

    fn sync_selections(&mut self) {
        match self.view_mode {
            ViewMode::Matches => {
                self.clamp_cursor();
                self.clamp_selected_match();
            }
            ViewMode::Records => {
                self.clamp_cursor();
                self.clamp_selected_record();
                self.sync_record_selection_to_cursor();
                self.sync_cursor_to_selected_record();
            }
        }
    }

    fn sync_record_selection_to_cursor(&mut self) {
        let Some(report) = &self.last_good else {
            self.selected_record_idx = 0;
            return;
        };
        if report.records.is_empty() {
            self.selected_record_idx = 0;
            return;
        }

        let cursor = self.cursor_line_idx;
        let mut best_idx = 0usize;
        let mut best_dist = usize::MAX;
        for (i, r) in report.records.iter().enumerate() {
            let dist = if r.line_idx > cursor {
                r.line_idx - cursor
            } else {
                cursor - r.line_idx
            };
            if dist < best_dist {
                best_dist = dist;
                best_idx = i;
                if dist == 0 {
                    break;
                }
            }
        }
        self.selected_record_idx = best_idx;
    }

    fn sync_cursor_to_selected_record(&mut self) {
        let Some(report) = &self.last_good else {
            return;
        };
        let Some(rec) = report.records.get(self.selected_record_idx) else {
            return;
        };
        self.cursor_line_idx = rec.line_idx;
        self.clamp_cursor();
    }

    fn sync_cursor_to_trace(&mut self) {
        let Some(report) = &self.last_good else {
            return;
        };
        let Some(event) = report.trace.get(self.trace_index) else {
            return;
        };
        self.cursor_line_idx = event.line_idx;
        self.clamp_cursor();
    }

    // Trace navigation methods

    pub fn step_forward(&mut self) {
        let Some(report) = &self.last_good else {
            return;
        };
        let next = self.find_next_event(report, self.trace_index, self.stepping_mode);
        if next != self.trace_index {
            self.trace_index = next;
            self.sync_cursor_to_trace();
        }
    }

    pub fn step_backward(&mut self) {
        let Some(report) = &self.last_good else {
            return;
        };
        let prev = self.find_prev_event(report, self.trace_index, self.stepping_mode);
        if prev != self.trace_index {
            self.trace_index = prev;
            self.sync_cursor_to_trace();
        }
    }

    fn find_next_event(
        &self,
        report: &DebugReport,
        current: usize,
        mode: crate::tui::trace::SteppingMode,
    ) -> usize {
        use crate::tui::trace::SteppingMode;
        use cliscrape::engine::debug::TraceEventType;

        match mode {
            SteppingMode::LineByLine => (current + 1).min(report.trace.len().saturating_sub(1)),
            SteppingMode::StateByState => report
                .trace
                .iter()
                .enumerate()
                .skip(current + 1)
                .find(|(_, e)| e.state_before != e.state_after)
                .map(|(i, _)| i)
                .unwrap_or(current),
            SteppingMode::ActionByAction => report
                .trace
                .iter()
                .enumerate()
                .skip(current + 1)
                .find(|(_, e)| {
                    matches!(
                        e.event_type,
                        TraceEventType::RecordEmitted | TraceEventType::RecordCleared
                    )
                })
                .map(|(i, _)| i)
                .unwrap_or(current),
        }
    }

    fn find_prev_event(
        &self,
        report: &DebugReport,
        current: usize,
        mode: crate::tui::trace::SteppingMode,
    ) -> usize {
        use crate::tui::trace::SteppingMode;
        use cliscrape::engine::debug::TraceEventType;

        if current == 0 {
            return 0;
        }

        match mode {
            SteppingMode::LineByLine => current.saturating_sub(1),
            SteppingMode::StateByState => report
                .trace
                .iter()
                .enumerate()
                .take(current)
                .rev()
                .find(|(_, e)| e.state_before != e.state_after)
                .map(|(i, _)| i)
                .unwrap_or(current),
            SteppingMode::ActionByAction => report
                .trace
                .iter()
                .enumerate()
                .take(current)
                .rev()
                .find(|(_, e)| {
                    matches!(
                        e.event_type,
                        TraceEventType::RecordEmitted | TraceEventType::RecordCleared
                    )
                })
                .map(|(i, _)| i)
                .unwrap_or(current),
        }
    }

    // Jump methods

    pub fn jump_to_next_record(&mut self) {
        use cliscrape::engine::debug::TraceEventType;

        let Some(report) = &self.last_good else {
            return;
        };
        if let Some((idx, _)) = report
            .trace
            .iter()
            .enumerate()
            .skip(self.trace_index + 1)
            .find(|(_, e)| matches!(e.event_type, TraceEventType::RecordEmitted))
        {
            self.trace_index = idx;
            self.sync_cursor_to_trace();
        }
    }

    pub fn jump_to_previous_record(&mut self) {
        use cliscrape::engine::debug::TraceEventType;

        let Some(report) = &self.last_good else {
            return;
        };
        if let Some((idx, _)) = report
            .trace
            .iter()
            .enumerate()
            .take(self.trace_index)
            .rev()
            .find(|(_, e)| matches!(e.event_type, TraceEventType::RecordEmitted))
        {
            self.trace_index = idx;
            self.sync_cursor_to_trace();
        }
    }

    pub fn jump_to_line(&mut self, line_idx: usize) {
        let Some(report) = &self.last_good else {
            return;
        };
        if let Some((idx, _)) = report
            .trace
            .iter()
            .enumerate()
            .find(|(_, e)| e.line_idx == line_idx)
        {
            self.trace_index = idx;
            self.sync_cursor_to_trace();
        }
    }

    // Filter and mode toggles

    pub fn toggle_stepping_mode(&mut self) {
        use crate::tui::trace::SteppingMode;
        self.stepping_mode = match self.stepping_mode {
            SteppingMode::LineByLine => SteppingMode::StateByState,
            SteppingMode::StateByState => SteppingMode::ActionByAction,
            SteppingMode::ActionByAction => SteppingMode::LineByLine,
        };
    }

    pub fn toggle_filter_line_events(&mut self) {
        self.filter_state.show_line_events = !self.filter_state.show_line_events;
    }

    pub fn toggle_filter_state_changes(&mut self) {
        self.filter_state.show_state_changes = !self.filter_state.show_state_changes;
    }

    pub fn toggle_filter_record_actions(&mut self) {
        self.filter_state.show_record_actions = !self.filter_state.show_record_actions;
    }

    pub fn toggle_filter_clear_actions(&mut self) {
        self.filter_state.show_clear_actions = !self.filter_state.show_clear_actions;
    }

    // Watch list methods

    pub fn toggle_watch(&mut self, var_name: String) {
        if self.watch_list.contains(&var_name) {
            self.watch_list.remove(&var_name);
        } else {
            self.watch_list.insert(var_name);
        }
    }
}

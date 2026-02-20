use cliscrape::DebugReport;
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
}

impl AppState {
    pub fn new(template_path: Option<PathBuf>, input_path: Option<PathBuf>) -> Self {
        let mut lines: Vec<String> = Vec::new();
        lines.push("cliscrape debug".to_string());
        lines.push("".to_string());
        lines.push("Usage:".to_string());
        lines.push("  cliscrape debug --template <PATH> --input <PATH>".to_string());

        Self {
            template_path,
            input_path,
            lines,
            cursor_line_idx: 0,
            selected_match_idx: 0,
            view_mode: ViewMode::Matches,
            selected_record_idx: 0,
            last_good: None,
            current_error: None,
            status: ParseStatus::Idle,
        }
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
}

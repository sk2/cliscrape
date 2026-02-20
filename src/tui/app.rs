use cliscrape::DebugReport;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStatus {
    Parsing,
    Ok,
    Error,
    Idle,
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub template_path: Option<PathBuf>,
    pub input_path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor_line_idx: usize,
    pub selected_match_idx: usize,
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
    }

    pub fn on_parse_error(&mut self, error: String) {
        self.current_error = Some(error);
        self.status = ParseStatus::Error;
        self.clamp_cursor();
        self.clamp_selected_match();
    }

    pub fn cursor_up(&mut self) {
        self.cursor_line_idx = self.cursor_line_idx.saturating_sub(1);
        self.selected_match_idx = 0;
    }

    pub fn cursor_down(&mut self) {
        let max = self.lines.len().saturating_sub(1);
        self.cursor_line_idx = (self.cursor_line_idx + 1).min(max);
        self.selected_match_idx = 0;
    }

    pub fn selected_match_prev(&mut self) {
        self.selected_match_idx = self.selected_match_idx.saturating_sub(1);
    }

    pub fn selected_match_next(&mut self) {
        self.selected_match_idx = self.selected_match_idx.saturating_add(1);
        self.clamp_selected_match();
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
}

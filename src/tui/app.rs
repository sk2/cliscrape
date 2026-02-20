use cliscrape::DebugReport;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppState {
    pub template_path: Option<PathBuf>,
    pub input_path: Option<PathBuf>,
    pub lines: Vec<String>,
    pub cursor_line_idx: usize,
    pub debug_report: Option<DebugReport>,
}

impl AppState {
    pub fn new(template_path: Option<PathBuf>, input_path: Option<PathBuf>) -> Self {
        Self {
            template_path,
            input_path,
            lines: Vec::new(),
            cursor_line_idx: 0,
            debug_report: None,
        }
    }

    pub fn set_debug_report(&mut self, report: DebugReport) {
        self.lines = report.lines.clone();
        self.debug_report = Some(report);
        self.clamp_cursor();
    }

    pub fn cursor_up(&mut self) {
        self.cursor_line_idx = self.cursor_line_idx.saturating_sub(1);
    }

    pub fn cursor_down(&mut self) {
        let max = self.lines.len().saturating_sub(1);
        self.cursor_line_idx = (self.cursor_line_idx + 1).min(max);
    }

    fn clamp_cursor(&mut self) {
        if self.lines.is_empty() {
            self.cursor_line_idx = 0;
            return;
        }
        let max = self.lines.len() - 1;
        self.cursor_line_idx = self.cursor_line_idx.min(max);
    }
}

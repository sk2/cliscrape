use anyhow::Context;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EditorBuffer {
    pub lines: Vec<String>,
    pub cursor_row: usize,
    pub cursor_col: usize,
    pub scroll_row: usize,
}

impl EditorBuffer {
    pub fn new(lines: Vec<String>) -> Self {
        let mut lines = lines;
        if lines.is_empty() {
            lines.push(String::new());
        }

        Self {
            lines,
            cursor_row: 0,
            cursor_col: 0,
            scroll_row: 0,
        }
    }

    pub fn content_string(&self) -> String {
        let mut out = self.lines.join("\n");
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out
    }

    pub fn move_left(&mut self) {
        if self.cursor_col > 0 {
            self.cursor_col -= 1;
            return;
        }
        if self.cursor_row > 0 {
            self.cursor_row -= 1;
            self.cursor_col = self.line_len_chars(self.cursor_row);
        }
    }

    pub fn move_right(&mut self) {
        let len = self.line_len_chars(self.cursor_row);
        if self.cursor_col < len {
            self.cursor_col += 1;
            return;
        }
        if self.cursor_row + 1 < self.lines.len() {
            self.cursor_row += 1;
            self.cursor_col = 0;
        }
    }

    pub fn move_up(&mut self) {
        self.cursor_row = self.cursor_row.saturating_sub(1);
        self.cursor_col = self.cursor_col.min(self.line_len_chars(self.cursor_row));
    }

    pub fn move_down(&mut self) {
        let max = self.lines.len().saturating_sub(1);
        self.cursor_row = (self.cursor_row + 1).min(max);
        self.cursor_col = self.cursor_col.min(self.line_len_chars(self.cursor_row));
    }

    pub fn insert_char(&mut self, ch: char) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.line_len_chars(row));
        let byte = char_col_to_byte(&self.lines[row], col);
        self.lines[row].insert(byte, ch);
        self.cursor_col += 1;
    }

    pub fn insert_newline(&mut self) {
        let row = self.cursor_row.min(self.lines.len().saturating_sub(1));
        let col = self.cursor_col.min(self.line_len_chars(row));
        let byte = char_col_to_byte(&self.lines[row], col);

        let rest = self.lines[row].split_off(byte);
        self.lines.insert(row + 1, rest);
        self.cursor_row += 1;
        self.cursor_col = 0;
    }

    pub fn backspace(&mut self) {
        if self.cursor_col > 0 {
            let row = self.cursor_row;
            let col = self.cursor_col;
            let start = char_col_to_byte(&self.lines[row], col - 1);
            let end = char_col_to_byte(&self.lines[row], col);
            self.lines[row].replace_range(start..end, "");
            self.cursor_col -= 1;
            return;
        }

        if self.cursor_row == 0 {
            return;
        }

        let row = self.cursor_row;
        let prev = row - 1;
        let prev_len = self.line_len_chars(prev);
        let cur_line = self.lines.remove(row);
        self.lines[prev].push_str(&cur_line);
        self.cursor_row = prev;
        self.cursor_col = prev_len;
    }

    pub fn delete(&mut self) {
        let row = self.cursor_row;
        let col = self.cursor_col;
        let len = self.line_len_chars(row);

        if col < len {
            let start = char_col_to_byte(&self.lines[row], col);
            let end = char_col_to_byte(&self.lines[row], col + 1);
            self.lines[row].replace_range(start..end, "");
            return;
        }

        if row + 1 >= self.lines.len() {
            return;
        }
        let next = self.lines.remove(row + 1);
        self.lines[row].push_str(&next);
    }

    pub fn ensure_visible(&mut self, view_height: usize) {
        if view_height == 0 {
            return;
        }

        if self.cursor_row < self.scroll_row {
            self.scroll_row = self.cursor_row;
            return;
        }

        let bottom = self
            .scroll_row
            .saturating_add(view_height.saturating_sub(1));
        if self.cursor_row > bottom {
            self.scroll_row = self
                .cursor_row
                .saturating_sub(view_height.saturating_sub(1));
        }
    }

    fn line_len_chars(&self, row: usize) -> usize {
        self.lines.get(row).map(|s| s.chars().count()).unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
pub struct EditorState {
    pub path: PathBuf,
    pub buf: EditorBuffer,
    pub dirty: bool,
    pub last_save_error: Option<String>,
    pub viewport_height: usize,
}

impl EditorState {
    pub fn open(path: PathBuf) -> anyhow::Result<Self> {
        let s = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read template: {}", path.display()))?;
        let lines = split_lines_preserve_empty(&s);

        Ok(Self {
            path,
            buf: EditorBuffer::new(lines),
            dirty: false,
            last_save_error: None,
            viewport_height: 20,
        })
    }

    pub fn apply_key(&mut self, key: KeyEvent) -> EditorKeyResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('s') | KeyCode::Char('S') => return EditorKeyResult::Save,
                _ => return EditorKeyResult::Noop,
            }
        }

        match key.code {
            KeyCode::Esc => EditorKeyResult::Exit,
            KeyCode::Up => {
                self.buf.move_up();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Down => {
                self.buf.move_down();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Left => {
                self.buf.move_left();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Right => {
                self.buf.move_right();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Char('h') => {
                self.buf.move_left();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Char('l') => {
                self.buf.move_right();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Char('k') => {
                self.buf.move_up();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Char('j') => {
                self.buf.move_down();
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Enter => {
                self.buf.insert_newline();
                self.dirty = true;
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Backspace => {
                self.buf.backspace();
                self.dirty = true;
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Delete => {
                self.buf.delete();
                self.dirty = true;
                self.buf.ensure_visible(self.viewport_height);
                EditorKeyResult::Noop
            }
            KeyCode::Tab => {
                self.buf.insert_char('\t');
                self.dirty = true;
                EditorKeyResult::Noop
            }
            KeyCode::Char(ch) => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    return EditorKeyResult::Noop;
                }
                self.buf.insert_char(ch);
                self.dirty = true;
                EditorKeyResult::Noop
            }
            _ => EditorKeyResult::Noop,
        }
    }

    pub fn save(&mut self) -> anyhow::Result<()> {
        let content = self.buf.content_string();
        write_file_atomic(&self.path, content.as_bytes())
            .with_context(|| format!("failed to save template: {}", self.path.display()))?;
        self.dirty = false;
        self.last_save_error = None;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorKeyResult {
    Noop,
    Save,
    Exit,
}

fn split_lines_preserve_empty(s: &str) -> Vec<String> {
    let mut out: Vec<String> = s.lines().map(|l| l.to_string()).collect();
    if s.ends_with('\n') {
        out.push(String::new());
    }
    out
}

fn char_col_to_byte(s: &str, col: usize) -> usize {
    if col == 0 {
        return 0;
    }
    let mut count = 0usize;
    for (idx, _ch) in s.char_indices() {
        if count == col {
            return idx;
        }
        count += 1;
    }
    s.len()
}

fn write_file_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("template");

    let tmp = parent.join(format!(".{}.cliscrape.tmp", filename));
    std::fs::write(&tmp, bytes)?;

    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            let _ = std::fs::remove_file(path);
            std::fs::rename(&tmp, path).or(Err(err))
        }
    }
}

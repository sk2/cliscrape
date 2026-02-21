use anyhow::Context;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::cmp::Ordering;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickTarget {
    Template,
    Input,
}

#[derive(Debug, Clone)]
pub struct PickerEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
}

#[derive(Debug, Clone)]
pub struct PickerState {
    pub cwd: PathBuf,
    pub entries: Vec<PickerEntry>,
    pub selected_idx: usize,
    pub target: PickTarget,

    pub manual: bool,
    pub manual_input: String,
    pub manual_cursor: usize,

    pub last_error: Option<String>,
}

impl PickerState {
    pub fn new(cwd: PathBuf, target: PickTarget) -> anyhow::Result<Self> {
        let mut s = Self {
            cwd,
            entries: Vec::new(),
            selected_idx: 0,
            target,
            manual: false,
            manual_input: String::new(),
            manual_cursor: 0,
            last_error: None,
        };
        s.refresh()?;
        Ok(s)
    }

    pub fn refresh(&mut self) -> anyhow::Result<()> {
        let mut entries: Vec<PickerEntry> = Vec::new();

        for e in std::fs::read_dir(&self.cwd)
            .with_context(|| format!("failed to read dir: {}", self.cwd.display()))?
        {
            let e = e?;
            let path = e.path();
            let md = e.metadata()?;
            let is_dir = md.is_dir();
            let is_file = md.is_file();
            if !is_dir && !is_file {
                continue;
            }

            let name = e
                .file_name()
                .to_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| path.display().to_string());

            entries.push(PickerEntry { name, path, is_dir });
        }

        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.entries = entries;
        if self.entries.is_empty() {
            self.selected_idx = 0;
        } else {
            self.selected_idx = self.selected_idx.min(self.entries.len() - 1);
        }
        Ok(())
    }

    pub fn set_target(&mut self, target: PickTarget) {
        self.target = target;
    }

    pub fn apply_key(&mut self, key: KeyEvent) -> PickerKeyResult {
        if self.manual {
            return self.apply_key_manual(key);
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_idx = self.selected_idx.saturating_sub(1);
                PickerKeyResult::Noop
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !self.entries.is_empty() {
                    self.selected_idx = (self.selected_idx + 1).min(self.entries.len() - 1);
                }
                PickerKeyResult::Noop
            }
            KeyCode::Enter => self.select_current(),
            KeyCode::Backspace => {
                if let Some(parent) = self.cwd.parent().map(ToOwned::to_owned) {
                    self.cwd = parent;
                    if let Err(e) = self.refresh() {
                        self.last_error = Some(format!("{:#}", e));
                    }
                }
                PickerKeyResult::Noop
            }
            KeyCode::Char('i') => {
                self.manual = true;
                self.manual_input.clear();
                self.manual_cursor = 0;
                PickerKeyResult::Noop
            }
            _ => PickerKeyResult::Noop,
        }
    }

    fn apply_key_manual(&mut self, key: KeyEvent) -> PickerKeyResult {
        if key.modifiers.contains(KeyModifiers::CONTROL) {
            return PickerKeyResult::Noop;
        }

        match key.code {
            KeyCode::Esc => {
                self.manual = false;
                self.last_error = None;
                PickerKeyResult::Noop
            }
            KeyCode::Enter => {
                let p = parse_path(&self.cwd, self.manual_input.as_str());
                self.manual = false;
                PickerKeyResult::Selected(p)
            }
            KeyCode::Left => {
                self.manual_cursor = self.manual_cursor.saturating_sub(1);
                PickerKeyResult::Noop
            }
            KeyCode::Right => {
                self.manual_cursor =
                    (self.manual_cursor + 1).min(self.manual_input.chars().count());
                PickerKeyResult::Noop
            }
            KeyCode::Backspace => {
                if self.manual_cursor == 0 {
                    return PickerKeyResult::Noop;
                }
                let start = char_col_to_byte(self.manual_input.as_str(), self.manual_cursor - 1);
                let end = char_col_to_byte(self.manual_input.as_str(), self.manual_cursor);
                self.manual_input.replace_range(start..end, "");
                self.manual_cursor -= 1;
                PickerKeyResult::Noop
            }
            KeyCode::Delete => {
                let len = self.manual_input.chars().count();
                if self.manual_cursor >= len {
                    return PickerKeyResult::Noop;
                }
                let start = char_col_to_byte(self.manual_input.as_str(), self.manual_cursor);
                let end = char_col_to_byte(self.manual_input.as_str(), self.manual_cursor + 1);
                self.manual_input.replace_range(start..end, "");
                PickerKeyResult::Noop
            }
            KeyCode::Char(ch) => {
                if key.modifiers.contains(KeyModifiers::ALT) {
                    return PickerKeyResult::Noop;
                }
                let byte = char_col_to_byte(self.manual_input.as_str(), self.manual_cursor);
                self.manual_input.insert(byte, ch);
                self.manual_cursor += 1;
                PickerKeyResult::Noop
            }
            _ => PickerKeyResult::Noop,
        }
    }

    fn select_current(&mut self) -> PickerKeyResult {
        let Some(ent) = self.entries.get(self.selected_idx).cloned() else {
            return PickerKeyResult::Noop;
        };

        if ent.is_dir {
            self.cwd = ent.path;
            if let Err(e) = self.refresh() {
                self.last_error = Some(format!("{:#}", e));
            }
            return PickerKeyResult::Noop;
        }

        PickerKeyResult::Selected(ent.path)
    }
}

#[derive(Debug, Clone)]
pub enum PickerKeyResult {
    Noop,
    Selected(PathBuf),
}

fn parse_path(cwd: &Path, s: &str) -> PathBuf {
    let p = PathBuf::from(s);
    if p.is_absolute() {
        return p;
    }
    cwd.join(p)
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

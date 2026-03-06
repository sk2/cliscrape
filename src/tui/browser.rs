use cliscrape::TemplateFormat;
use cliscrape::template::library;
use cliscrape::template::metadata::{self, TemplateMetadata};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum TemplateLocation {
    Embedded,
    User(PathBuf),
}

#[derive(Debug, Clone)]
pub struct TemplateBrowserEntry {
    pub name: String,
    pub location: TemplateLocation,
    pub metadata: TemplateMetadata,
    pub preview: String,
}

#[derive(Debug, Clone)]
pub struct TemplateBrowserState {
    pub entries: Vec<TemplateBrowserEntry>,
    pub selected_idx: usize,
}

impl TemplateBrowserState {
    pub fn new() -> Self {
        let mut entries = Vec::new();

        // Add embedded templates
        for name in library::list_embedded() {
            if let Some(file) = library::get_embedded(&name) {
                let content = std::str::from_utf8(&file.data).unwrap_or("");
                let format = if name.ends_with(".yaml") || name.ends_with(".yml") {
                    TemplateFormat::Yaml
                } else if name.ends_with(".toml") {
                    TemplateFormat::Toml
                } else {
                    TemplateFormat::Textfsm
                };
                let metadata = metadata::extract_metadata(content, format);

                // Get first 20 lines for preview
                let preview: String = content.lines().take(20).collect::<Vec<_>>().join("\n");

                entries.push(TemplateBrowserEntry {
                    name,
                    location: TemplateLocation::Embedded,
                    metadata,
                    preview,
                });
            }
        }

        // Add user templates from XDG directory
        let xdg_dirs = xdg::BaseDirectories::with_prefix("cliscrape");
        if let Some(data_home) = xdg_dirs.data_home {
            let templates_dir = data_home.join("templates");
            if let Ok(dir_entries) = std::fs::read_dir(&templates_dir) {
                for entry in dir_entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                            let format = if name.ends_with(".yaml") || name.ends_with(".yml") {
                                TemplateFormat::Yaml
                            } else if name.ends_with(".toml") {
                                TemplateFormat::Toml
                            } else if name.ends_with(".textfsm") {
                                TemplateFormat::Textfsm
                            } else {
                                continue;
                            };

                            if let Ok(content) = std::fs::read_to_string(&path) {
                                let metadata = metadata::extract_metadata(&content, format);
                                let preview: String =
                                    content.lines().take(20).collect::<Vec<_>>().join("\n");

                                entries.push(TemplateBrowserEntry {
                                    name: name.to_string(),
                                    location: TemplateLocation::User(path),
                                    metadata,
                                    preview,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Sort alphabetically by name
        entries.sort_by(|a, b| a.name.cmp(&b.name));

        Self {
            entries,
            selected_idx: 0,
        }
    }

    pub fn selected_entry(&self) -> Option<&TemplateBrowserEntry> {
        self.entries.get(self.selected_idx)
    }

    pub fn next(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        self.selected_idx = (self.selected_idx + 1) % self.entries.len();
    }

    pub fn previous(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        if self.selected_idx == 0 {
            self.selected_idx = self.entries.len() - 1;
        } else {
            self.selected_idx -= 1;
        }
    }
}

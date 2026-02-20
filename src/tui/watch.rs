use crate::tui::{FsWhich, Message};
use anyhow::Context;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::Duration;

pub struct WatcherHandle {
    // Dropping the debouncer stops its internal thread.
    _debouncer: notify_debouncer_mini::Debouncer<notify::RecommendedWatcher>,
}

pub fn start_watcher(
    template_path: PathBuf,
    input_path: PathBuf,
    sender: Sender<Message>,
) -> anyhow::Result<WatcherHandle> {
    let template_name = file_name(&template_path)?;
    let input_name = file_name(&input_path)?;

    let template_dir = parent_dir(&template_path)?;
    let input_dir = parent_dir(&input_path)?;

    let mut debouncer = new_debouncer(
        Duration::from_millis(150),
        move |res: DebounceEventResult| {
            let Ok(events) = res else {
                return;
            };

            let mut template_changed = false;
            let mut input_changed = false;

            for ev in events {
                let Some(name) = ev.path.file_name() else {
                    continue;
                };
                if name == template_name {
                    template_changed = true;
                }
                if name == input_name {
                    input_changed = true;
                }
            }

            if template_changed {
                let _ = sender.send(Message::FsChanged {
                    which: FsWhich::Template,
                });
            }
            if input_changed {
                let _ = sender.send(Message::FsChanged {
                    which: FsWhich::Input,
                });
            }
        },
    )
    .with_context(|| "failed to start debounced filesystem watcher")?;

    let mut watched: HashSet<PathBuf> = HashSet::new();
    for dir in [template_dir, input_dir] {
        if watched.insert(dir.clone()) {
            debouncer
                .watcher()
                .watch(&dir, RecursiveMode::NonRecursive)
                .with_context(|| format!("failed to watch directory: {}", dir.display()))?;
        }
    }

    Ok(WatcherHandle {
        _debouncer: debouncer,
    })
}

fn file_name(path: &Path) -> anyhow::Result<OsString> {
    path.file_name()
        .map(ToOwned::to_owned)
        .with_context(|| format!("path has no filename: {}", path.display()))
}

fn parent_dir(path: &Path) -> anyhow::Result<PathBuf> {
    path.parent()
        .map(ToOwned::to_owned)
        .with_context(|| format!("path has no parent directory: {}", path.display()))
}
